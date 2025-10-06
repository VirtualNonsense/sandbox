use std::collections::HashMap;

use color_eyre::eyre::{self, Result};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Rect, style::Color};

use crate::{
    event::AppEvent,
    particle::{self, Action, Cell},
};

pub struct Window {
    pub height: u16,
    pub width: u16,
    pub x: u16,
    pub y: u16,
}

impl From<&Rect> for Window {
    fn from(value: &Rect) -> Self {
        Self {
            height: value.height,
            width: value.width,
            x: value.x,
            y: value.y,
        }
    }
}

#[derive(Default)]
pub struct Simulation {
    src_buffer: HashMap<u32, particle::Cell>,
    /// hash value is coord since coords seem to be stored as u16
    /// layout: xxxx xxxx xxxx xxxx yyyy yyyy yyyy yyyy
    dst_buffer: HashMap<u32, particle::Cell>,
    window: Option<Window>,
}

impl Simulation {
    pub fn handle_keyboard_event(&mut self, _event: &KeyEvent) -> Result<()> {
        Ok(())
    }

    pub fn handle_mouse_event(&mut self, event: &MouseEvent) -> Result<()> {
        match event.kind {
            crossterm::event::MouseEventKind::Up(button)
            | crossterm::event::MouseEventKind::Drag(button) => {
                let cell = match button {
                    crossterm::event::MouseButton::Left => Cell::Sand,
                    crossterm::event::MouseButton::Right => Cell::Wood,
                    crossterm::event::MouseButton::Middle => Cell::Fire,
                };
                self.flip(event.column, event.row, cell);
            }
            _ => {}
        }
        Ok(())
    }

    fn flip(&mut self, cursor_x: u16, cursor_y: u16, cell: Cell) {
        let (cursor_x, cursor_y) = if let Some(win) = &self.window
            && win.height >= cursor_y
        {
            (cursor_x, win.height - cursor_y)
        } else {
            (cursor_x, cursor_y)
        };

        let idx = Self::get_key_from_coords(cursor_x, cursor_y);

        self.src_buffer.entry(idx).or_insert(cell);
    }

    pub fn handle_app_event(&mut self, event: &AppEvent) -> Result<()> {
        match *event {
            AppEvent::Quit => {}
        }
        Ok(())
    }

    pub fn handle_ticks(&mut self) -> Result<()> {
        self.dst_buffer.clear();
        let (rows, columns) = if let Some(win) = &self.window {
            (win.height as i16, win.width as i16)
        } else {
            (i16::MAX, i16::MAX)
        };
        for (idx, cell) in self.src_buffer.iter() {
            let (x, y) = Self::get_coords_from_key(*idx);

            let (x, y) = (x as i16, y as i16);
            let mut neighbour_map: HashMap<particle::Direction, &particle::Cell> = HashMap::new();

            fn find_cell(
                x: i16,
                y: i16,
                columns: i16,
                rows: i16,
                map: &HashMap<u32, Cell>,
            ) -> Option<&Cell> {
                if x < 0 || y < 0 || x >= columns || y >= rows {
                    return Some(&Cell::Border);
                }
                map.get(&Simulation::get_key_from_coords(x as u16, y as u16))
            }

            if let Some(cell) = find_cell(x, y - 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Down, cell);
            }

            if let Some(cell) = find_cell(x + 1, y - 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::DownRight, cell);
            }
            if let Some(cell) = find_cell(x - 1, y - 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::DownLeft, cell);
            }
            if let Some(cell) = find_cell(x + 1, y, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Right, cell);
            }
            if let Some(cell) = find_cell(x + 1, y + 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::UpRight, cell);
            }
            if let Some(cell) = find_cell(x, y + 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Up, cell);
            }
            if let Some(cell) = find_cell(x - 1, y + 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::UpLeft, cell);
            }
            if let Some(cell) = find_cell(x - 1, y, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Left, cell);
            }
            if let Ok(action) = cell.update(neighbour_map) {
                fn move_cell(
                    x: i16,
                    y: i16,
                    dx: i16,
                    dy: i16,
                    cell: particle::Cell,
                    bffr: &mut HashMap<u32, particle::Cell>,
                ) -> eyre::Result<()> {
                    let x = if x >= dx.abs() {
                        (x + dx) as u16
                    } else {
                        x as u16
                    };

                    let y = if y >= dy.abs() {
                        (y + dy) as u16
                    } else {
                        y as u16
                    };
                    let new_idx = Simulation::get_key_from_coords(x, y);
                    bffr.insert(new_idx, cell);
                    Ok(())
                }
                match action {
                    Action::None => {
                        self.dst_buffer.insert(*idx, cell.clone());
                    }
                    Action::Replace(new_cell) => {
                        self.dst_buffer.insert(*idx, new_cell);
                    }
                    Action::Move(direction) => match direction {
                        particle::Direction::Up => {
                            move_cell(x, y, 0, 1, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::UpRight => {
                            move_cell(x, y, 1, 1, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::UpLeft => {
                            move_cell(x, y, -1, 1, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::Right => {
                            move_cell(x, y, 1, 0, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::Left => {
                            move_cell(x, y, -1, 0, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::Down => {
                            move_cell(x, y, 0, -1, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::DownRight => {
                            move_cell(x, y, 1, -1, cell.clone(), &mut self.dst_buffer)?;
                        }
                        particle::Direction::DownLeft => {
                            move_cell(x, y, -1, -1, cell.clone(), &mut self.dst_buffer)?;
                        }
                    },
                    Action::Vanish => {
                        // do nothing
                    }
                }
            }
        }

        std::mem::swap(&mut self.src_buffer, &mut self.dst_buffer);
        Ok(())
    }
    fn get_coords_from_key(key: u32) -> (u16, u16) {
        let x = (key >> 16) as u16;
        let y = ((u16::MAX as u32) & key) as u16;
        (x, y)
    }

    fn get_key_from_coords(x: u16, y: u16) -> u32 {
        (x as u32) << 16 | y as u32
    }
    pub fn iter_cells(&self) -> impl Iterator<Item = ((f64, f64), Color)> + '_ {
        self.src_buffer.iter().map(|(id, cell)| {
            let (x, y) = Self::get_coords_from_key(*id);
            let color = match *cell {
                Cell::Sand => Color::Yellow,
                Cell::Wood => Color::Rgb(25, 120, 25),
                Cell::Fire => Color::Red,
                Cell::Border => Color::Cyan,
            };
            ((x as f64, y as f64), color)
        })
    }
    pub fn update_window_size(&mut self, window: Window) {
        self.window = Some(window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_key_from_coords() {
        let x = 123u16;
        let y = 102u16;

        let key = Simulation::get_key_from_coords(x, y);
        let actual = 8061030u32;
        assert_eq!(actual, key, "{} is not equal to {}", key, actual);
    }
    #[test]
    fn test_get_coords_from_key() {
        let x_actual = 123u16;
        let y_actual = 102u16;

        let key = 8061030u32;
        let (x, y) = Simulation::get_coords_from_key(key);
        assert_eq!(x_actual, x, "{} is not equal to {}", x, x_actual);
        assert_eq!(y_actual, y, "{} is not equal to {}", y, y_actual);
    }
}
