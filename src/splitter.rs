use asr::timer::TimerState;

use crate::Memory;
use crate::TICK_RATE;
use crate::settings::Settings;

#[derive(Default)]
pub struct Splitter {
    loading_flag: bool,
    pause_flag: bool,
    menu_flag: bool,
    race_started_flag: bool,
    post_race_flag: bool,
    race_finished_this_run_flag: bool,
    restart_tracked_flag: bool,
    buffered_restart_detected_time: f64,
    buffered_restart_new_game_time: f64,

    menu_reset_signal: bool,
    menu_start_signal: bool,
    menu_restart_signal: bool,
    restart_start_signal: bool,
    buffered_restart_signal: bool,
    set_game_time_after_buffered_restart_signal: bool,
    race_finish_split_signal: bool,
    lap_split_signal: bool
}

impl Splitter {
    pub fn update(&mut self, ticks: u64, memory: &Memory, settings: &Settings) {
        self.update_vars(ticks, &memory);
        self.update_timer(ticks, &settings);
    }

    fn update_vars(&mut self, ticks: u64, memory: &Memory) {
        // Check for stopped timer to clear race finish flag
        if asr::timer::state() != TimerState::Running && asr::timer::state() != TimerState::Paused {
            self.race_finished_this_run_flag = false;
        }

        // Check for loading
        self.loading_flag = Memory::current(memory.loading_flag);
        
        // Check for pauses
        self.pause_flag = Memory::current(memory.pause_flag);

        // Check for menu
        self.menu_flag = Memory::current(memory.location_id) == 0;

        // Check for menu->level entry
        if Memory::current(memory.location_id) != 0 && Memory::old(memory.location_id) == 0 {
            self.menu_start_signal = true;
            self.menu_reset_signal = true;
        }

        // Check for race status
        if !self.race_started_flag && !self.loading_flag && !self.menu_flag && Memory::current(memory.laps_completed) == 0 {
            self.race_started_flag = true;
        }
        if self.race_started_flag && !self.loading_flag && Memory::current(memory.laps_completed) == Memory::current(memory.total_laps)  {
            self.race_started_flag = false;
            self.race_finish_split_signal = true;
            self.race_finished_this_run_flag = true;
            self.post_race_flag = true;
        }
        else if self.race_started_flag && !self.loading_flag && Memory::current(memory.laps_completed) > Memory::old(memory.laps_completed) {
            self.lap_split_signal = true;
        }

        // Check post race flag
        if self.race_started_flag {
            self.post_race_flag = false;
        }
        else if self.loading_flag || self.menu_flag {
            self.post_race_flag = false;
        }

        // Track restarts
        if !self.menu_flag && !self.loading_flag {
            if Memory::current(memory.time_elapsed) < Memory::old(memory.time_elapsed) {
                if !self.restart_tracked_flag  {
                    self.post_race_flag = false;
                    self.buffered_restart_signal = true;
                    self.buffered_restart_detected_time = (ticks as f64) / TICK_RATE;
                }
                self.restart_tracked_flag = true;
            }
            else {
                self.restart_tracked_flag = false;
            }
        }
    }

    fn update_timer(&mut self, ticks: u64, settings: &Settings) {
        self.eval_split(settings);
        self.eval_reset(settings, ticks);
        self.eval_load(settings);
        self.eval_start(settings);
        self.eval_gametime();
    }

    fn eval_load(&self, settings: &Settings) {
        let should_pause = (self.loading_flag && settings.pause_on_loads) 
        || (self.menu_flag && settings.pause_on_menu)
        || (self.pause_flag && settings.pause_on_pause)
        || (self.post_race_flag && settings.pause_on_post_race);
        if should_pause {
            asr::timer::pause_game_time();
        }
        else {
            asr::timer::resume_game_time();
        }
    }

    fn eval_split(&mut self, settings: &Settings) {
        let can_split = asr::timer::state() == TimerState::Running;
        let should_split = can_split && ((self.race_finish_split_signal && settings.split_on_finish)
        || (self.lap_split_signal && settings.split_on_lap));
        if should_split {
            asr::print_message("SPLITTING");
            asr::timer::split();
        }
        self.lap_split_signal = false;
        self.race_finish_split_signal = false;
    }

    fn eval_reset(&mut self, settings: &Settings, ticks: u64) {
        let timer_can_reset = asr::timer::state() == TimerState::Running || asr::timer::state() == TimerState::Ended || asr::timer::state() == TimerState::Paused;

        if self.menu_reset_signal {
            self.buffered_restart_signal = false;
            self.menu_reset_signal = false;
            if settings.reset_on_entry && timer_can_reset && !self.race_finished_this_run_flag {
                asr::print_message("RESETTING ON LEVEL ENTRY");
                self.race_finished_this_run_flag = false;
                self.menu_restart_signal = true;
                asr::timer::reset();
            }
            return;
        }

        if self.menu_flag || self.loading_flag || self.race_finished_this_run_flag {
            self.buffered_restart_signal = false;
            return;
        }

        if self.buffered_restart_signal {
            let current_time = (ticks as f64) / TICK_RATE;
            let diff = current_time - self.buffered_restart_detected_time;
            if diff < 0.08 {
                return;
            }

            self.buffered_restart_signal = false;
            self.restart_start_signal = true;
            if settings.reset_on_restart && timer_can_reset {
                asr::print_message("RESETTING ON LEVEL RESTART");
                self.race_finished_this_run_flag = false;
                self.set_game_time_after_buffered_restart_signal = true;
                self.buffered_restart_new_game_time = diff;
                asr::timer::reset();
            }
        }
    }

    fn eval_gametime(&mut self) {
        if asr::timer::state() == TimerState::NotRunning || asr::timer::state() == TimerState::Ended {
            return;
        }
        if self.set_game_time_after_buffered_restart_signal {
            asr::print_message("UPDATING GAMETIME");
            self.set_game_time_after_buffered_restart_signal = false;
            asr::timer::set_game_time(asr::time::Duration::seconds_f64(self.buffered_restart_new_game_time));
        }
    }

    fn eval_start(&mut self, settings: &Settings) {
        let can_start = asr::timer::state() != TimerState::Running;
        if can_start && ((self.restart_start_signal && (settings.start_on_restart || settings.reset_on_restart))
        || (self.menu_start_signal && settings.start_on_entry)
        || (self.menu_restart_signal && settings.reset_on_entry)) {
            asr::print_message("STARTING");
            asr::timer::start();
        }
        self.restart_start_signal = false;
        self.menu_start_signal = false;
        self.menu_restart_signal = false;
    }
}