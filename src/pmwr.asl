state("PMR")
{
	int loading_flag : 0x126488; // 0 if not loading, 1 if loading
    int location_id: 0x218518; // 0 for menu/awards ceremony, otherwise marks track ID
    int laps_completed: 0x126608; // [0,#total laps]
    int total_laps: 0x1264C8; // Total lap count, set on vehicle select
}

init
{
    // Whether loading is detected.
    vars.is_loading = false;
    // Game time value recorded at the start of the load.
    vars.load_start_time = 0.0;
    // Whether to set game time to the last recorded load start time. Flag should reset afterwards.
    vars.set_game_time = false;

    // True only when navigating menu or on awards ceremony
    vars.on_menu = false;

    // True once laps completed is zero'd out and not in menu or loads. Reset on race finished.
    vars.race_started = false;
    // True when race_started is true and lap count equals total laps. Reset on split.
    vars.race_finished = false;
}

update
{
    // Check for loading
    if (current.loading_flag != 0 && old.loading_flag == 0) {
        print("Loading started");
        vars.is_loading = true;
        vars.set_game_time = true;
        vars.load_start_time = timer.CurrentTime.GameTime.Value.TotalMilliseconds;
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
}

gameTime
{
    if (vars.set_game_time) {
        vars.set_game_time = false;
        return TimeSpan.FromMilliseconds(vars.load_start_time);
    }
}

start {
    // Start when new level load begins from menu
    return current.location_id != 0 && old.location_id == 0;
}

split {
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