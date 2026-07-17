use asr::{settings::{Gui, gui::Title}, watcher::Pair};

#[derive(Gui, Clone, Copy, PartialEq)]
pub enum TimingMethod {
    // Real-time with out-of-race time subtracted
    #[default]
    LoadRemovedTime,
    // Defer to the in-game race time for each segment
    InGameTime
}

#[derive(Gui)]
pub struct Settings {
    /// Timing Method
    pub timing_method: Pair<TimingMethod>,

    /// Start Options
    _title_start: Title,

    /// Start on level entry (starts immediately after exiting main menu to a track)
    #[default = true]
    pub start_on_entry: bool,

    /// Start on level restart
    #[default = true]
    pub start_on_restart: bool,

    /// Reset Options
    _title_reset: Title,

    /// Reset on level entry (starts immediately after exiting main menu to a track)
    #[default = true]
    pub reset_on_entry: bool,

    /// Reset on level restart (on the first race only)
    #[default = true]
    pub reset_on_restart: bool,

    /// Split Options
    _title_split: Title,

    /// Split on crossing the finish line on the last lap
    #[default = true]
    pub split_on_finish: bool,

    /// Split on crossing the finish line for each lap
    #[default = false]
    pub split_on_lap: bool

}