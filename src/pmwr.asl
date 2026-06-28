state("PMR")
{
	int loading_flag : 0x126488; // 0 if not loading, 1 if loading
    int location_id: 0x218518; // 0 for menu/awards ceremony, otherwise marks track ID
    int laps_completed: 0x126608; // [0,#total laps]
    int total_laps: 0x1264C8; // Total lap count, set on vehicle select
    int time_elapsed: 0x217A74; // Ticks elapsed since entering/restarting a track (Not sure what exactly the ticks measure)
}

init
{
    // Whether loading is detected.
    vars.is_loading = false;

    // True only when navigating menu or on awards ceremony
    vars.on_menu = false;

    // True once laps completed is zero'd out and not in menu or loads. Reset on race finished.
    vars.race_started = false;
    // True when race_started is true and lap count equals total laps. Reset on split.
    vars.race_finished = false;
    // Track if a split has happened
    vars.split_flag = false;

    // True once 0 ticks has been detected
    vars.reset_tracked = false;
    // Set this to true to reset the timer. Cleared after reset
    vars.reset_flag = false;
    // Set after reset_flag is cleared to start the timer again
    vars.restart_flag = false;

    // State for tracking a buffer after detecting a reset, to prevent data race issue when exiting to main menu
    vars.restart_detected_time = 0.0;
    vars.set_restart_game_time = false;
    vars.restart_time = 0.0;
}

update
{
    // Check for loading
    if (current.loading_flag != 0 && old.loading_flag == 0) {
        print("Loading started");
        vars.is_loading = true;
    }
    else if (current.loading_flag == 0 && old.loading_flag != 0) {
        print("Loading ended");
        vars.is_loading = false;
    }
    
    // Check for menu
    vars.on_menu = current.location_id == 0;

    // Check for race status
    if (!vars.race_started && current.loading_flag == 0 && current.location_id != 0 && current.laps_completed == 0) {
        vars.race_started = true;
    }
    if (vars.race_started && current.laps_completed == current.total_laps) {
        vars.race_started = false;
        vars.race_finished = true;
    }

    // Track restarts
    if (!vars.on_menu && !vars.is_loading) {
        if (current.time_elapsed < old.time_elapsed) {
            if (!vars.reset_tracked) {
                print("RESET DETECTED");
                vars.reset_flag = true;
                vars.restart_detected_time = timer.CurrentTime.GameTime.Value.TotalMilliseconds;
            }
            vars.reset_tracked = true;
        }
        else {
            vars.reset_tracked = false;
        }
    }
}

gameTime
{
    if (vars.set_restart_game_time) {
        vars.set_restart_game_time = false;
        return TimeSpan.FromMilliseconds(vars.restart_time);
    }
}

reset
{
    if (vars.on_menu || vars.is_loading || vars.split_flag) {
        print("Reset skipped: Wrong state");
        vars.reset_flag = false;
        return false;
    }
    if (vars.reset_flag) {

        double current_game_time = timer.CurrentTime.GameTime.Value.TotalMilliseconds;
        double diff = current_game_time - vars.restart_detected_time;
        if (diff < 80) {
        print("Reset buffered until future frame");
            return false;
        }

        print("Reset passed buffer: resetting for real...");
        vars.reset_flag = false;
        vars.restart_flag = true;
        vars.set_restart_game_time = true;
        vars.restart_detected_time = diff;
        return true;
    }
    return false;
}

start
{
    // Start when new level load begins from menu
    if (vars.restart_flag) {
        vars.restart_flag = false;
        return true;
    }
    return current.location_id != 0 && old.location_id == 0;
}

split
{
    // Split on race finish
    if (vars.race_finished) {
        vars.race_finished = false;
        return true;
    }
}

isLoading
{
    // Pause when loading or on menu
    return vars.is_loading || vars.on_menu;
}

onSplit
{
    vars.split_flag = true;
}

onReset
{
    vars.split_flag = false;
}