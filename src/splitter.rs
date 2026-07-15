use asr::timer::TimerState;

use crate::Memory;
use crate::TICK_RATE;
use crate::settings::Settings;
use crate::settings::TimingMethod;

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
    lap_times: [i32; 5],
    igt_time: i64,
    igt_time_display: i64,

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
            self.igt_time = 0;
            self.igt_time_display = 0;
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
            self.lap_times.fill(0);
        }
        let current_laps_completed = Memory::current(memory.laps_completed);
        let old_laps_completed = Memory::old(memory.laps_completed);
        let total_laps = Memory::current(memory.total_laps);
        
        let in_race = self.race_started_flag && !self.loading_flag;
        let single_lap_complete = in_race && current_laps_completed > old_laps_completed;
        let final_lap_complete = in_race && current_laps_completed == total_laps;
        if final_lap_complete  {
            self.race_started_flag = false;
            self.race_finish_split_signal = true;
            self.race_finished_this_run_flag = true;
            self.post_race_flag = true;
            
            // IGT time
            let lap_times = Memory::current(memory.lap_times);
            let mut track_time: i64 = 0;
            for i in 0..5 {
                track_time += lap_times[i] as i64;
            }
            self.igt_time += ((track_time / 10) as i64) * 10;
            self.igt_time_display = self.igt_time;
            // asr::print_message(&format!("IGT TIME: {}", self.igt_time));
        }
        else if single_lap_complete {
            self.lap_split_signal = true;
            let lap_time = (Memory::current(memory.lap_times)[(Memory::current(memory.laps_completed) - 1) as usize] / 10) * 10;
            self.igt_time_display += lap_time as i64;
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
        self.eval_gametime(settings);
        self.eval_split(settings);
        self.eval_reset(settings, ticks);
        self.eval_load(settings);
        self.eval_start(settings);
    }

    fn eval_load(&self, settings: &Settings) {
        match settings.timing_method.current {
            TimingMethod::LoadRemovedTime => {
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
            },
            TimingMethod::InGameTime => {
                asr::timer::pause_game_time();
            }
        }

    }

    fn eval_split(&mut self, settings: &Settings) {
        let can_split = asr::timer::state() == TimerState::Running;
        let should_split = can_split && ((self.race_finish_split_signal && (settings.split_on_finish || settings.split_on_lap))
        || (self.lap_split_signal && settings.split_on_lap));
        if should_split {
            asr::print_message("SPLITTING");
            // if settings.timing_method.current == TimingMethod::InGameTime {
            //     asr::timer::set_game_time(asr::time::Duration::milliseconds(self.igt_time));
            // }
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
                self.igt_time = 0;
                self.igt_time_display = 0;
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

    fn eval_gametime(&mut self, settings: &Settings) {
        if asr::timer::state() == TimerState::NotRunning || asr::timer::state() == TimerState::Ended {
            return;
        }
        match settings.timing_method.current {
            TimingMethod::LoadRemovedTime => {
                if self.set_game_time_after_buffered_restart_signal {
                    asr::print_message("UPDATING GAMETIME");
                    self.set_game_time_after_buffered_restart_signal = false;
                    asr::timer::set_game_time(asr::time::Duration::seconds_f64(self.buffered_restart_new_game_time));
                }
            }
            TimingMethod::InGameTime => {
                asr::timer::set_game_time(asr::time::Duration::milliseconds(self.igt_time_display));
            }
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