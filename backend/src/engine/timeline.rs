use crate::models::timeline::SimTime;
use crate::models::world::WorldState;

impl WorldState {
    /// Advance simulation time by `minutes`.
    pub fn advance_minutes(&mut self, minutes: u64) {
        self.timeline.advance_minutes(minutes);
        self.environment
            .update(self.timeline.tick, &self.timeline.time);
    }

    /// Advance simulation time by `hours`.
    pub fn advance_hours(&mut self, hours: u64) {
        self.timeline.advance_hours(hours);
        self.environment
            .update(self.timeline.tick, &self.timeline.time);
    }

    /// Current simulation time.
    pub fn sim_time(&self) -> &SimTime {
        &self.timeline.time
    }
}
