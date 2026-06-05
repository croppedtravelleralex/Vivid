use std::path::{Path, PathBuf};

use chrono::Utc;
use redb::{ReadableDatabase, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::models::world::WorldState;

const CHECKPOINTS_TABLE: TableDefinition<&str, &[u8]> =
    TableDefinition::new("checkpoints");

// ---------------------------------------------------------------------------
// Checkpoint metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointMeta {
    pub tag: String,
    pub tick: u64,
    pub datetime: String,
    pub saved_at: String,
    pub char_count: usize,
}

// ---------------------------------------------------------------------------
// WriteAheadLog entry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub seq: u64,
    pub tick: u64,
    pub action: String,
    pub data: Vec<u8>,
}

// ---------------------------------------------------------------------------
// CheckpointManager
// ---------------------------------------------------------------------------

pub struct CheckpointManager {
    db: redb::Database,
    base_dir: PathBuf,
    current_version: u64,
    wal_entries: u64,
    max_auto: usize,
}

impl CheckpointManager {
    /// Open or create the checkpoint database.
    pub fn new(base_dir: impl AsRef<Path>, max_auto: usize) -> Result<Self, String> {
        let base_dir = base_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_dir).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(base_dir.join("auto")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(base_dir.join("user")).map_err(|e| e.to_string())?;

        let db_path = base_dir.join("metadata.redb");
        let db = redb::Database::open(&db_path).map_err(|e| e.to_string())?;

        // Ensure table exists
        {
            let tx = db.begin_write().map_err(|e| e.to_string())?;
            tx.open_table(CHECKPOINTS_TABLE).map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
        }

        Ok(Self {
            db,
            base_dir,
            current_version: 0,
            wal_entries: 0,
            max_auto,
        })
    }

    /// Save a full world snapshot with the given tag.
    pub fn save_snapshot(&self, world: &WorldState, tag: &str) -> Result<CheckpointMeta, String> {
        let data = world.to_bincode()?;
        let tick = world.timeline.tick;

        let filename = if tag.starts_with("auto_") {
            format!("auto/tick_{:06}_{}.bin", tick, tag)
        } else {
            format!("user/{}.snapshot", tag)
        };

        let path = self.base_dir.join(&filename);
        std::fs::write(&path, &data).map_err(|e| e.to_string())?;

        let meta = CheckpointMeta {
            tag: tag.to_string(),
            tick,
            datetime: world.timeline.time.to_string(),
            saved_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            char_count: world.character_count(),
        };

        // Index in redb
        let tx = self.db.begin_write().map_err(|e| e.to_string())?;
        {
            let mut table = tx.open_table(CHECKPOINTS_TABLE).map_err(|e| e.to_string())?;
            let encoded = bincode::serialize(&meta).map_err(|e| e.to_string())?;
            table
                .insert(tag, encoded.as_slice())
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;

        info!(tag, tick, "检查点已保存");
        Ok(meta)
    }

    /// Load a world snapshot by tag.
    pub fn load_snapshot(&self, tag: &str, seed: u64) -> Result<WorldState, String> {
        let tx = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = tx
            .open_table(CHECKPOINTS_TABLE)
            .map_err(|e| e.to_string())?;

        let stored = table
            .get(tag)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Checkpoint '{}' not found", tag))?;

        let meta: CheckpointMeta =
            bincode::deserialize(stored.value()).map_err(|e| e.to_string())?;

        let filename = if tag.starts_with("auto_") {
            format!("auto/tick_{:06}_{}.bin", meta.tick, tag)
        } else {
            format!("user/{}.snapshot", tag)
        };

        let path = self.base_dir.join(&filename);
        let data = std::fs::read(&path).map_err(|e| e.to_string())?;
        let world = WorldState::from_bincode(&data, seed)?;

        info!(tag, tick = meta.tick, "检查点已加载");
        Ok(world)
    }

    /// List all saved checkpoints.
    pub fn list_checkpoints(&self) -> Result<Vec<CheckpointMeta>, String> {
        let tx = self.db.begin_read().map_err(|e| e.to_string())?;
        let table = tx
            .open_table(CHECKPOINTS_TABLE)
            .map_err(|e| e.to_string())?;

        let mut list = vec![];
        for result in table.iter().map_err(|e| e.to_string())? {
            let (_, value) = result.map_err(|e| e.to_string())?;
            if let Ok(meta) = bincode::deserialize::<CheckpointMeta>(value.value()) {
                list.push(meta);
            }
        }

        list.sort_by(|a, b| b.tick.cmp(&a.tick));
        Ok(list)
    }

    /// Write-ahead log: append an entry.
    pub fn wal_append(&mut self, entry: WalEntry) -> Result<(), String> {
        self.wal_entries += 1;
        let log_path = self.base_dir.join("wal.log");
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .map_err(|e| e.to_string())?;

        let buf = bincode::serialize(&entry).map_err(|e| e.to_string())?;
        use std::io::Write;
        file.write_all(&buf).map_err(|e| e.to_string())?;
        file.write_all(b"\n").map_err(|e| e.to_string())?;
        file.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Replay WAL to recover uncommitted ticks.
    pub fn wal_replay(&self) -> Result<Vec<WalEntry>, String> {
        let log_path = self.base_dir.join("wal.log");
        if !log_path.exists() {
            return Ok(vec![]);
        }

        let data = std::fs::read(&log_path).map_err(|e| e.to_string())?;
        let mut entries = vec![];
        for chunk in data.split(|&b| b == b'\n').filter(|c| !c.is_empty()) {
            if let Ok(entry) = bincode::deserialize::<WalEntry>(chunk) {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    /// Clear WAL after successful checkpoint save.
    pub fn wal_clear(&self) -> Result<(), String> {
        let log_path = self.base_dir.join("wal.log");
        if log_path.exists() {
            std::fs::write(&log_path, b"").map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Check for crash recovery.
    pub fn check_crash_recovery(&self) -> bool {
        let log_path = self.base_dir.join("wal.log");
        if !log_path.exists() {
            return false;
        }
        if let Ok(metadata) = std::fs::metadata(&log_path) {
            return metadata.len() > 0;
        }
        false
    }
}

// ---------------------------------------------------------------------------
// CrashRecovery helper (for main.rs)
// ---------------------------------------------------------------------------

pub struct CrashRecovery;

impl CrashRecovery {
    pub async fn check(base_dir: impl AsRef<Path>) -> bool {
        let manager = CheckpointManager::new(base_dir, 50);
        match manager {
            Ok(mgr) => mgr.check_crash_recovery(),
            Err(_) => false,
        }
    }
}
