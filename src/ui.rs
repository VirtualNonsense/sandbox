use ratatui::{buffer::Buffer, layout::Rect, style::Stylize, widgets::Widget};

use crate::{app::App, simulation_widget::Simulation};
impl Widget for &App {
    /// Renders the user interface widgets.
    ///
    // This is where you add new widgets.
    // See the following resources:
    // - https://docs.rs/ratatui/latest/ratatui/widgets/index.html
    // - https://github.com/ratatui/ratatui/tree/master/examples
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.simulation_widget.render(area, buf);
    }
}

impl Widget for &Simulation {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        for (position, color) in self.iter_cells() {
            let area = Rect::from((position, ratatui::layout::Size::new(1, 1))).clamp(area);
            let pixel_widget = ratatui::symbols::block::FULL.fg(color);
            pixel_widget.render(area, buf);
        }
    }
}
