state("PMR")
{
	int loading_flag : 0x126488;
}

init
{
    vars.is_loading = false;
    vars.set_game_time = false;
    vars.load_start_time = 0.0;
}

update
{
    // print("Loading state: " + current.loading_flag);
    if (current.loading_flag != 0 && !vars.is_loading) {
        print("Loading started");
        vars.is_loading = true;
        vars.set_game_time = true;
        vars.load_start_time = timer.CurrentTime.GameTime.Value.TotalMilliseconds;
    }
    else if (current.loading_flag == 0 && vars.is_loading) {
        print("Loading ended");
        vars.is_loading = false;
    }
}

gameTime
{
    if (vars.set_game_time) {
        vars.set_game_time = false;
        return TimeSpan.FromMilliseconds(vars.load_start_time);
    }
    // return TimeSpan.FromMilliseconds(current.loading_flag);
}

isLoading
{
    return vars.is_loading;
}