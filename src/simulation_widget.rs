use std::collections::HashMap;

use color_eyre::eyre::{self, Result};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Position, style::Color};

use crate::{
    event::AppEvent,
    particle::{self, Action, Cell},
    window::Window,
};

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
        let idx = Self::get_key_from_coords(cursor_x, cursor_y);

        self.src_buffer.entry(idx).or_insert(cell);
    }

    pub fn handle_app_event(&mut self, event: &AppEvent) -> Result<()> {
        match *event {
            AppEvent::Quit => {}
        }
        Ok(())
    }

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
    fn move_cell(
        pos: (i16, i16),
        delta_pos: (i16, i16),
        height: i16,
        width: i16,
        cell: particle::Cell,
        bffr: &mut HashMap<u32, particle::Cell>,
    ) -> eyre::Result<()> {
        let (x, y) = pos;
        let (dx, dy) = delta_pos;
        let (new_x, new_y) = if (dx >= 0 || x >= dx.abs())
            && x + dx <= width
            && (dy >= 0 || y >= dy.abs())
            && y + dx <= height
        {
            ((x + dx) as u16, (y + dy) as u16)
        } else {
            (x as u16, y as u16)
        };
        let new_idx = Simulation::get_key_from_coords(new_x, new_y);
        bffr.insert(new_idx, cell);
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

            if let Some(cell) = Self::find_cell(x, y + 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Down, cell);
            }

            if let Some(cell) = Self::find_cell(x + 1, y + 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::DownRight, cell);
            }
            if let Some(cell) = Self::find_cell(x - 1, y + 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::DownLeft, cell);
            }
            if let Some(cell) = Self::find_cell(x + 1, y, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Right, cell);
            }
            if let Some(cell) = Self::find_cell(x + 1, y - 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::UpRight, cell);
            }
            if let Some(cell) = Self::find_cell(x, y - 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Up, cell);
            }
            if let Some(cell) = Self::find_cell(x - 1, y - 1, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::UpLeft, cell);
            }
            if let Some(cell) = Self::find_cell(x - 1, y, columns, rows, &self.src_buffer) {
                neighbour_map.insert(particle::Direction::Left, cell);
            }
            if let Ok(action) = cell.update(neighbour_map) {
                match action {
                    Action::None => {
                        self.dst_buffer.insert(*idx, cell.clone());
                    }
                    Action::Replace(new_cell) => {
                        self.dst_buffer.insert(*idx, new_cell);
                    }
                    Action::Move(direction) => match direction {
                        particle::Direction::Up => {
                            Self::move_cell(
                                (x, y),
                                (0, -1),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::UpRight => {
                            Self::move_cell(
                                (x, y),
                                (1, -1),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::UpLeft => {
                            Self::move_cell(
                                (x, y),
                                (-1, -1),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::Right => {
                            Self::move_cell(
                                (x, y),
                                (1, 0),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::Left => {
                            Self::move_cell(
                                (x, y),
                                (-1, 0),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::Down => {
                            Self::move_cell(
                                (x, y),
                                (0, 1),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::DownRight => {
                            Self::move_cell(
                                (x, y),
                                (1, 1),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
                        }
                        particle::Direction::DownLeft => {
                            Self::move_cell(
                                (x, y),
                                (-1, 1),
                                rows,
                                columns,
                                cell.clone(),
                                &mut self.dst_buffer,
                            )?;
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
    pub fn iter_cells(&self) -> impl Iterator<Item = (Position, Color)> + '_ {
        self.src_buffer.iter().map(|(id, cell)| {
            let (x, y) = Self::get_coords_from_key(*id);
            let color = match *cell {
                Cell::Sand => Color::Yellow,
                Cell::Wood => Color::Rgb(25, 120, 25),
                Cell::Fire => Color::Red,
                Cell::Border => Color::Cyan,
            };
            (Position::from((x, y)), color)
        })
    }
    pub fn update_window_size(&mut self, window: Window) {
        self.window = Some(window)
    }
}

impl MaterialCanvas for Simulation {
    fn set_pixel(&mut self, x: u16, y: u16, cell: Cell) -> eyre::Result<()> {
        self.flip(x, y, cell);
        Ok(())
    }

    fn set_pixels(&mut self, points: Vec<(u16, u16)>, cell: Cell) -> eyre::Result<()> {
        for (x, y) in points {
            self.set_pixel(x, y, cell.clone())?;
        }
        Ok(())
    }

    fn remove_pixel(&mut self, x: u16, y: u16) -> eyre::Result<()> {
        let idx = Self::get_key_from_coords(x, y);
        self.src_buffer.remove(&idx);
        Ok(())
    }
}

pub trait MaterialCanvas {
    fn set_pixel(&mut self, x: u16, y: u16, cell: Cell) -> eyre::Result<()>;
    fn set_pixels(&mut self, points: Vec<(u16, u16)>, cell: Cell) -> eyre::Result<()>;
    fn remove_pixel(&mut self, x: u16, y: u16) -> eyre::Result<()>;
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

    #[cfg(test)]
    mod tests_finc_cell_h3_w3_empty_map {
        use super::*;
        const HEIGHT: i16 = 3;
        const WIDTH: i16 = 3;

        #[test]
        fn test_cell_m1_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(-1, -1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(0, -1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_1_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(1, -1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_2_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(2, -1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_3_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(3, -1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_m1_0_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(-1, 0, WIDTH, HEIGHT, &hash_map);
            let cell = cell_opt.unwrap();
            assert!(matches!(cell, Cell::Border));
        }
        #[test]
        fn test_cell_0_0_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(0, 0, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_1_0_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(1, 0, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_2_0_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(2, 0, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }

        #[test]
        fn test_cell_3_0_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(3, 0, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_m1_1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(-1, 1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_1_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(0, 1, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_1_1_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(1, 1, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_2_1_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(2, 1, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }

        #[test]
        fn test_cell_3_1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(3, 1, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_m1_2_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(-1, 2, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_2_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(0, 2, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_1_2_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(1, 2, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_2_2_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(2, 2, WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }

        #[test]
        fn test_cell_3_2_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(3, 2, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_m1_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(-1, 3, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(0, 3, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_1_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(1, 3, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_2_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(2, 3, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_3_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell(3, 3, WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
    }

    #[cfg(test)]
    mod test_move_cell_h3_w3 {
        use std::collections::{HashMap, hash_map};

        use crate::{particle::Cell, simulation_widget::Simulation};

        const HEIGHT: i16 = 3;
        const WIDTH: i16 = 3;
        #[test]
        fn test_down_1_0() {
            let mut hash_map = HashMap::new();
            let (x, y) = (1, 0);
            let delta_pos = (0, 1);

            Simulation::move_cell((x, y), delta_pos, HEIGHT, WIDTH, Cell::Sand, &mut hash_map)
                .expect("Failed to set the cell");
            let result_at_new_pos = hash_map.get(&Simulation::get_key_from_coords(
                (x + delta_pos.0) as u16,
                (y + delta_pos.1) as u16,
            ));
            assert_eq!(hash_map.len(), 1);
            assert!(result_at_new_pos.is_some_and(|value| matches!(value, Cell::Sand)))
        }
    }
}
