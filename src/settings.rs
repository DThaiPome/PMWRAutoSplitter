use asr::{settings::Gui};

#[derive(Gui)]
pub struct Settings {
    /// My Setting
    #[default = true]
    my_setting: bool,
    // TODO: Change these settings.
}