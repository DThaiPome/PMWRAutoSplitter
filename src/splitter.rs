use asr::timer::TimerState;

use crate::Memory;
use crate::TICK_RATE;

#[derive(Default)]
pub struct Splitter {
    is_loading: bool,
    on_menu: bool,
    race_started: bool,
    race_finished: bool,
    reset_tracked: bool,
    reset_flag: bool,
    restart_flag: bool,
    restart_detected_time: f64,
    set_restart_game_time: bool,
    restart_time: f64
}

impl Splitter {
    pub fn update(&mut self, ticks: u64, memory: &Memory) {
        self.update_vars(ticks, &memory);
        self.update_timer(ticks);
    }

    fn update_vars(&mut self, ticks: u64, memory: &Memory) {
        // Check for loading
        if Memory::current(memory.loading_flag) && !Memory::old(memory.loading_flag) {
            self.is_loading = true;
        }
        else if !Memory::current(memory.loading_flag) && Memory::old(memory.loading_flag) {
            self.is_loading = false;
        }

        // Check for menu
        self.on_menu = Memory::current(memory.location_id) == 0;

        // Check for menu->level run start
        if asr::timer::state() == TimerState::NotRunning && Memory::current(memory.location_id) != 0 && Memory::old(memory.location_id) == 0 {
            self.restart_flag = true;
        }

        // Check for race status
        if !self.race_started && !self.is_loading && !self.on_menu && Memory::current(memory.laps_completed) == 0 {
            self.race_started = true;
        }
        if self.race_started && Memory::current(memory.laps_completed) == Memory::current(memory.total_laps)  {
            self.race_started = false;
            self.race_finished = true;
        }

        // Track restarts
        if !self.on_menu && !self.is_loading {
            if Memory::current(memory.time_elapsed) < Memory::old(memory.time_elapsed) {
                if !self.reset_tracked  {
                    self.reset_flag = true;
                    self.restart_detected_time = (ticks as f64) / TICK_RATE;
                }
                self.reset_tracked = true;
            }
            else {
                self.reset_tracked = false;
            }
        }
    }

    fn update_timer(&mut self, ticks: u64) {
        self.eval_split();
        self.eval_reset(ticks);
        self.eval_gametime();
        self.eval_load();
        self.eval_start();
    }

    fn eval_load(&self) {
        let should_pause = self.is_loading || self.on_menu;
        if should_pause {
            asr::timer::pause_game_time();
        }
        else {
            asr::timer::resume_game_time();
        }
    }

    fn eval_split(&mut self) {
        if asr::timer::state() != TimerState::Running {
            return;
        }
        // Split on race finish
        if self.race_finished {
            asr::print_message("SPLITTING");
            self.race_finished = false;
            asr::timer::split();
        }
    }

    fn eval_reset(&mut self, ticks: u64) {
        if asr::timer::state() == TimerState::Ended {
            return;
        }
        if self.on_menu || self.is_loading || asr::timer::current_split_index().unwrap_or(0) > 0 {
            self.reset_flag = false;
            return;
        }

        if self.reset_flag {
            let current_time = (ticks as f64) / TICK_RATE;
            let diff = current_time - self.restart_detected_time;
            if diff < 0.08 {
                return;
            }
            asr::print_message("RESETTING");

            self.reset_flag = false;
            self.restart_flag = true;
            self.set_restart_game_time = true;
            self.restart_time = diff;
            asr::timer::reset();
        }
    }

    fn eval_gametime(&mut self) {
        if asr::timer::state() == TimerState::NotRunning || asr::timer::state() == TimerState::Ended {
            return;
        }
        if self.set_restart_game_time {
            asr::print_message("UPDATING GAMETIME");
            self.set_restart_game_time = false;
            asr::timer::set_game_time(asr::time::Duration::seconds_f64(self.restart_time));
        }
    }

    fn eval_start(&mut self) {
        if asr::timer::state() == TimerState::Running {
            return;
        }
        if self.restart_flag {
            asr::print_message("STARTING");
            self.restart_flag = false;
            asr::timer::start();
        }
    }
}