use crate::{
    event::{AppEvent, Event, EventHandler},
    simulation_widget::Simulation,
};
use color_eyre::eyre;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyModifiers},
    style::Color,
};

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Event handler.
    pub events: EventHandler,
    pub simulation_paused: bool,
    pub color: Color,
    pub simulation_widget: Simulation,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            simulation_paused: false,
            events: EventHandler::new(),
            color: Color::White,
            simulation_widget: Simulation::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| {
                let area = frame.area();
                self.simulation_widget.update_window_size((&area).into());
                frame.render_widget(&self, area)
            })?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> color_eyre::Result<()> {
        match self.events.next()? {
            Event::Tick => {
                self.tick()?;
            }
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event) => {
                    self.simulation_widget.handle_keyboard_event(&key_event)?;
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                        KeyCode::Char('c' | 'C')
                            if key_event.modifiers == KeyModifiers::CONTROL =>
                        {
                            self.events.send(AppEvent::Quit)
                        }
                        KeyCode::Enter => {
                            self.simulation_paused = !self.simulation_paused;
                        }
                        // Other handlers you could add here.
                        _ => {}
                    }
                }
                crossterm::event::Event::Mouse(mouse_event) => {
                    self.simulation_widget.handle_mouse_event(&mouse_event)?;
                }
                _ => {}
            },
            Event::App(app_event) => {
                self.simulation_widget.handle_app_event(&app_event)?;
                match app_event {
                    AppEvent::Quit => self.quit(),
                }
            }
        }
        Ok(())
    }

    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    pub fn tick(&mut self) -> eyre::Result<()> {
        if self.simulation_paused {
            return Ok(());
        }
        self.simulation_widget.handle_ticks()
    }
    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}
