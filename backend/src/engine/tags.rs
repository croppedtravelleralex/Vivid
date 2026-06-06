//! Tag system -- global metadata layer for events, characters, and plot threads.
//! Tracks freshness lifecycle, auto-tagging, and thread management.
//! See doc 13 section 2.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::event::EventType;

// ============================================================
// Tag types (doc 13 2.1)
// ============================================================

/// All tag variants -- the union type for the tag index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Tag {
    Event(EventTag),
    Tone(EmotionalTone),
    NarrativeFunction(NarrativeFunction),
    ArcStage(String),
    CoreConflict(String),
    Focus(String),
    Thread(ThreadStatus),
    ThreadType(ThreadType),
    Freshness(Freshness),
    Confidence(f64),
}

/// Event-type tags derived from EventType.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventTag {
    CharacterDevelopment,
    RelationshipChange,
    ResourceCrisis,
    ExternalThreat,
    Discovery,
    Conflict,
    Resolution,
    WorldBuilding,
    Foreshadowing,
    Callback,
    Pivot,
    Climax,
}

/// Emotional tone of an event or thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EmotionalTone {
    Hope, Fear, Tension, Relief,
    Grief, Joy, Anger, Melancholy,
    Neutral,
}

/// Narrative function the event serves in the broader story.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NarrativeFunction {
    Setup, Payoff, Escalation, Breathing,
    Transition, Reveal, Twist,
}

/// Lifespan status of a plot thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreadStatus {
    Active, Dormant, Resolved, Abandoned, ForeshadowPing,
}

/// High-level classification of a plot thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ThreadType {
    Mystery, Promise, Threat, Relationship, CharacterArc, WorldSecret,
}

/// Freshness level for a thread or piece of narrative metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Freshness {
    Current, Recent, Stale, Archived,
}

// ============================================================
// ThreadInfo (doc 13 2.3)
// ============================================================

/// Metadata for a single narrative thread.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreadInfo {
    pub id: Uuid,
    pub seed_event_id: Uuid,
    pub thread_type: ThreadType,
    pub description: String,
    pub status: ThreadStatus,
    pub freshness: Freshness,
    pub last_updated: u64,
    pub expiry_tick: Option<u64>,
    pub reactivation_condition: Option<String>,
    pub emotional_tone: EmotionalTone,
    pub participating_characters: Vec<Uuid>,
}

// ============================================================
// TagIndex (doc 13 2.3)
// ============================================================

/// Global tag index -- the central metadata store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TagIndex {
    pub event_tags: HashMap<Uuid, Vec<Tag>>,
    pub character_tags: HashMap<Uuid, Vec<Tag>>,
    pub active_threads: HashMap<ThreadType, Vec<ThreadInfo>>,
    pub archived_threads: Vec<ThreadInfo>,
}

impl Default for TagIndex {
    fn default() -> Self {
        Self {
            event_tags: HashMap::new(),
            character_tags: HashMap::new(),
            active_threads: HashMap::new(),
            archived_threads: Vec::new(),
        }
    }
}

impl TagIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tag_event(&mut self, event_id: Uuid, tags: Vec<Tag>) {
        self.event_tags.entry(event_id).or_default().extend(tags);
    }

    pub fn tag_character(&mut self, char_id: Uuid, tags: Vec<Tag>) {
        self.character_tags.entry(char_id).or_default().extend(tags);
    }

    pub fn get_event_tags(&self, event_id: &Uuid) -> &[Tag] {
        self.event_tags.get(event_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn get_character_tags(&self, char_id: &Uuid) -> &[Tag] {
        self.character_tags.get(char_id).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn start_thread(&mut self, info: ThreadInfo) {
        self.active_threads.entry(info.thread_type).or_default().push(info);
    }

    pub fn update_thread(&mut self, thread_id: Uuid, f: impl FnOnce(&mut ThreadInfo)) -> bool {
        for threads in self.active_threads.values_mut() {
            if let Some(thread) = threads.iter_mut().find(|t| t.id == thread_id) {
                f(thread);
                return true;
            }
        }
        false
    }

    pub fn archive_thread(&mut self, thread_id: Uuid, current_tick: u64) {
        for threads in self.active_threads.values_mut() {
            if let Some(pos) = threads.iter().position(|t| t.id == thread_id) {
                let mut thread = threads.remove(pos);
                thread.freshness = Freshness::Archived;
                self.archived_threads.push(thread);
                return;
            }
        }
    }

    pub fn resurrect_thread(&mut self, thread_id: Uuid, current_tick: u64) -> bool {
        if let Some(pos) = self.archived_threads.iter().position(|t| t.id == thread_id) {
            let mut thread = self.archived_threads.remove(pos);
            thread.freshness = Freshness::Current;
            thread.last_updated = current_tick;
            self.active_threads.entry(thread.thread_type).or_default().push(thread);
            true
        } else {
            false
        }
    }

    pub fn tick_freshness(&mut self, current_tick: u64) {
        let mut to_archive: Vec<(ThreadType, Uuid)> = Vec::new();
        for (tt, threads) in &mut self.active_threads {
            for thread in threads.iter_mut() {
                let age = current_tick.saturating_sub(thread.last_updated);
                match thread.freshness {
                    Freshness::Current if age > 10 => thread.freshness = Freshness::Recent,
                    Freshness::Recent if age > 30 => thread.freshness = Freshness::Stale,
                    Freshness::Stale if age > 60 => to_archive.push((*tt, thread.id)),
                    _ => {}
                }
            }
        }
        for (_tt, id) in to_archive {
            self.archive_thread(id, current_tick);
        }
    }
}

// ============================================================
// Auto-tagger (doc 13 2.1 / 2.2)
// ============================================================

/// Derive a set of tags from an EventType based on heuristics.
pub fn auto_tag_event(event_type: &EventType, _tick: u64, _participants: &[Uuid]) -> Vec<Tag> {
    match event_type {
        EventType::TemperatureDrop(_) => vec![
            Tag::Event(EventTag::WorldBuilding),
            Tag::Tone(EmotionalTone::Melancholy),
            Tag::NarrativeFunction(NarrativeFunction::Setup),
        ],
        EventType::Blizzard => vec![
            Tag::Event(EventTag::ExternalThreat),
            Tag::Tone(EmotionalTone::Tension),
            Tag::NarrativeFunction(NarrativeFunction::Escalation),
        ],
        EventType::WaterPipeFreeze => vec![
            Tag::Event(EventTag::ResourceCrisis),
            Tag::Tone(EmotionalTone::Tension),
            Tag::NarrativeFunction(NarrativeFunction::Setup),
        ],
        EventType::CharacterArrival(_) => vec![
            Tag::Event(EventTag::CharacterDevelopment),
            Tag::Tone(EmotionalTone::Hope),
            Tag::NarrativeFunction(NarrativeFunction::Transition),
        ],
        EventType::CharacterDeparture(_) => vec![
            Tag::Event(EventTag::CharacterDevelopment),
            Tag::Tone(EmotionalTone::Melancholy),
            Tag::NarrativeFunction(NarrativeFunction::Transition),
        ],
        EventType::CharacterInjury(_) => vec![
            Tag::Event(EventTag::ExternalThreat),
            Tag::Tone(EmotionalTone::Fear),
            Tag::NarrativeFunction(NarrativeFunction::Escalation),
        ],
        EventType::CharacterDiscovery(_, _) => vec![
            Tag::Event(EventTag::Discovery),
            Tag::Tone(EmotionalTone::Hope),
            Tag::NarrativeFunction(NarrativeFunction::Reveal),
        ],
        EventType::Conflict(_, _) => vec![
            Tag::Event(EventTag::Conflict),
            Tag::Tone(EmotionalTone::Tension),
            Tag::NarrativeFunction(NarrativeFunction::Escalation),
        ],
        EventType::Alliance(_, _) => vec![
            Tag::Event(EventTag::RelationshipChange),
            Tag::Tone(EmotionalTone::Hope),
            Tag::NarrativeFunction(NarrativeFunction::Setup),
        ],
        EventType::Betrayal(_, _) => vec![
            Tag::Event(EventTag::Conflict),
            Tag::Tone(EmotionalTone::Anger),
            Tag::NarrativeFunction(NarrativeFunction::Twist),
        ],
        EventType::ResourceFound(_, _) => vec![
            Tag::Event(EventTag::Discovery),
            Tag::Event(EventTag::ResourceCrisis),
            Tag::Tone(EmotionalTone::Relief),
            Tag::NarrativeFunction(NarrativeFunction::Payoff),
        ],
        EventType::ResourceDepleted(_) => vec![
            Tag::Event(EventTag::ResourceCrisis),
            Tag::Tone(EmotionalTone::Fear),
            Tag::NarrativeFunction(NarrativeFunction::Setup),
        ],
        EventType::PlotAdvance(_) => vec![
            Tag::Event(EventTag::WorldBuilding),
            Tag::Tone(EmotionalTone::Neutral),
            Tag::NarrativeFunction(NarrativeFunction::Transition),
        ],
        EventType::PromiseBroken(_, _, _) => vec![
            Tag::Event(EventTag::Conflict),
            Tag::Tone(EmotionalTone::Anger),
            Tag::NarrativeFunction(NarrativeFunction::Twist),
        ],
        EventType::Reminder(_, _) => vec![
            Tag::Event(EventTag::Foreshadowing),
            Tag::Tone(EmotionalTone::Melancholy),
            Tag::NarrativeFunction(NarrativeFunction::Setup),
        ],
        EventType::ExternalStimulus(_) => vec![
            Tag::Event(EventTag::ExternalThreat),
            Tag::Tone(EmotionalTone::Tension),
            Tag::NarrativeFunction(NarrativeFunction::Escalation),
        ],
        EventType::StressCascade(_, _) => vec![
            Tag::Event(EventTag::Conflict),
            Tag::Tone(EmotionalTone::Fear),
            Tag::NarrativeFunction(NarrativeFunction::Escalation),
        ],
        EventType::ResourceConflict(_, _, _) => vec![
            Tag::Event(EventTag::Conflict),
            Tag::Event(EventTag::ResourceCrisis),
            Tag::Tone(EmotionalTone::Anger),
            Tag::NarrativeFunction(NarrativeFunction::Escalation),
        ],
        EventType::Custom(_) => vec![
            Tag::Event(EventTag::WorldBuilding),
            Tag::Tone(EmotionalTone::Neutral),
            Tag::NarrativeFunction(NarrativeFunction::Transition),
        ],
    }
}
