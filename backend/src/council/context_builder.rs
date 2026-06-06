use serde::{Deserialize, Serialize};

use super::CouncilInput;

/// Construct a `CouncilInput` from raw fields extracted from `WorldState`.
///
/// Keeps the council module decoupled from the full `WorldState` type.
/// The caller (e.g. the engine) is responsible for gathering the required
/// data from `WorldState` / `TagIndex` and passing it here.
pub fn build_context(
    tick: u64,
    character_count: usize,
    recent_events: Vec<String>,
    active_threads: Vec<String>,
) -> CouncilInput {
    tracing::info!(
        tick = tick,
        character_count = character_count,
        event_count = recent_events.len(),
        thread_count = active_threads.len(),
        "Building council input context"
    );

    CouncilInput {
        tick,
        character_count,
        recent_events,
        active_threads,
    }
}

/// Convenience wrapper that accepts string slices for callers that already
/// own the data in slice form.
pub fn build_context_from_str(
    tick: u64,
    character_count: usize,
    recent_events: &[&str],
    active_threads: &[&str],
) -> CouncilInput {
    build_context(
        tick,
        character_count,
        recent_events.iter().map(|s| s.to_string()).collect(),
        active_threads.iter().map(|s| s.to_string()).collect(),
    )
}

/// Empty/zero-value `CouncilInput` — useful as a placeholder during
/// simulation phases where the council has not yet been wired up.
pub fn empty_input(tick: u64) -> CouncilInput {
    CouncilInput {
        tick,
        character_count: 0,
        recent_events: Vec::new(),
        active_threads: Vec::new(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub tick: u64,
    pub character_count: usize,
    pub recent_event_count: usize,
    pub active_thread_count: usize,
}

impl From<&CouncilInput> for ContextSnapshot {
    fn from(input: &CouncilInput) -> Self {
        Self {
            tick: input.tick,
            character_count: input.character_count,
            recent_event_count: input.recent_events.len(),
            active_thread_count: input.active_threads.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_context() {
        let input = build_context(42, 5, vec!["event1".into()], vec!["thread1".into()]);
        assert_eq!(input.tick, 42);
        assert_eq!(input.character_count, 5);
        assert_eq!(input.recent_events.len(), 1);
        assert_eq!(input.active_threads.len(), 1);
    }

    #[test]
    fn test_build_context_from_str() {
        let input = build_context_from_str(10, 3, &["a", "b"], &["x"]);
        assert_eq!(input.tick, 10);
        assert_eq!(input.recent_events, vec!["a", "b"]);
    }

    #[test]
    fn test_empty_input() {
        let input = empty_input(99);
        assert_eq!(input.tick, 99);
        assert!(input.recent_events.is_empty());
        assert!(input.active_threads.is_empty());
    }

    #[test]
    fn test_context_snapshot() {
        let input = build_context(7, 2, vec!["e1".into(), "e2".into()], vec!["t1".into()]);
        let snap: ContextSnapshot = ContextSnapshot::from(&input);
        assert_eq!(snap.tick, 7);
        assert_eq!(snap.recent_event_count, 2);
        assert_eq!(snap.active_thread_count, 1);
    }
}
