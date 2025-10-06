use std::collections::HashMap;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::{
        Widget,
        canvas::{Canvas, Points},
    },
};

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
        let canvas = Canvas::default()
            .x_bounds([area.x as f64, (area.x + area.width) as f64])
            .y_bounds([area.y as f64, (area.y + area.height) as f64])
            .paint(|context| {
                let mut points: HashMap<Color, Vec<(f64, f64)>> = HashMap::new();

                for ((x, y), color) in self.iter_cells() {
                    if let Some(v) = points.get_mut(&color) {
                        v.push((x, y));
                    } else {
                        points.insert(color, vec![(x, y)]);
                    }
                }
                for (color, values) in points.iter() {
                    context.draw(&Points {
                        coords: values,
                        color: *color,
                    });
                }
            });
        canvas.render(area, buf);
    }
}
