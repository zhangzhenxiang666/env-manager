use crate::tui::app::App;

pub mod app;
pub mod components;
pub mod event;
pub mod theme;
pub mod ui;
pub mod utils;
pub mod widgets;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    App::run()
}
