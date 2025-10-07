pub struct Window {
    pub height: u16,
    pub width: u16,
    pub x: u16,
    pub y: u16,
}

impl From<&ratatui::layout::Rect> for Window {
    fn from(value: &ratatui::layout::Rect) -> Self {
        Self {
            height: value.height,
            width: value.width,
            x: value.x,
            y: value.y,
        }
    }
}

impl Window {
    pub fn is_inside(&self, x: u16, y: u16) -> bool {
        self.x <= x && self.y <= y && self.x + self.width > x && self.y + self.height > y
    }
}
