use crossterm::event::{DisableMouseCapture, EnableMouseCapture};

use crate::app::App;

pub mod app;
pub mod event;
pub mod particle;
mod simulation_widget;
pub mod ui;
pub mod window;
fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    crossterm::execute!(std::io::stdout(), EnableMouseCapture)?;
    let result = App::new().run(terminal);
    ratatui::restore();
    crossterm::execute!(std::io::stdout(), DisableMouseCapture)?;
    result
}
