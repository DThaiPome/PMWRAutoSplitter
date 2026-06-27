state("PMR")
{
	int loading_flag : 0x126488; // 0 if not loading, 1 if loading
    int location_id: 0x218518; // 0 for menu/awards ceremony, otherwise marks track ID
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

isLoading
{
    // Pause when loading or on menu
    return vars.is_loading || vars.on_menu;
}