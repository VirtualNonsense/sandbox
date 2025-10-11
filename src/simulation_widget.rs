use std::collections::HashMap;

use color_eyre::eyre::{self, Result};
use crossterm::event::{KeyEvent, MouseEvent};
use ratatui::{layout::Position, style::Color};

use crate::{
    coord::{self, Vec2},
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
                    crossterm::event::MouseButton::Right => Cell::Water,
                    crossterm::event::MouseButton::Middle => Cell::Fire,
                };
                self.flip(&(event.column, event.row).into(), cell)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn flip(&mut self, pos: &Vec2, cell: Cell) -> color_eyre::Result<()> {
        self.src_buffer.entry(pos.try_into()?).or_insert(cell);
        Ok(())
    }

    pub fn handle_app_event(&mut self, event: &AppEvent) -> Result<()> {
        match *event {
            AppEvent::Quit => {}
        }
        Ok(())
    }

    fn find_cell(pos: Vec2, columns: i16, rows: i16, map: &HashMap<u32, Cell>) -> Option<&Cell> {
        if pos.x < 0 || pos.y < 0 || pos.x >= columns || pos.y >= rows {
            return Some(&Cell::Border);
        }
        map.get(
            &pos.try_into()
                .expect("conversion into u32 should work if pos.x/y is positive"),
        )
    }

    fn fill_neighbour<'a>(
        &'a self,
        pos: &Vec2,
        direction: coord::Direction,
        width: i16,
        height: i16,
        map: &mut HashMap<coord::Direction, &'a Cell>,
    ) {
        let direction_vec: Vec2 = direction.clone().into();
        if let Some(cell) = Self::find_cell(pos + direction_vec, width, height, &self.src_buffer) {
            map.insert(direction, cell);
        }
    }
    pub fn handle_ticks(&mut self) -> Result<()> {
        self.dst_buffer.clear();
        let (width, height) = if let Some(window) = &self.window {
            (window.width as i16, window.height as i16)
        } else {
            (i16::MAX, i16::MAX)
        };
        for (idx, cell) in self.src_buffer.iter() {
            let pos: Vec2 = (*idx).into();

            let mut neighbour_map: HashMap<coord::Direction, &particle::Cell> = HashMap::new();

            self.fill_neighbour(
                &pos,
                coord::Direction::Down,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::DownRight,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::Right,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::UpRight,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::Up,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::UpLeft,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::Left,
                width,
                height,
                &mut neighbour_map,
            );
            self.fill_neighbour(
                &pos,
                coord::Direction::DownLeft,
                width,
                height,
                &mut neighbour_map,
            );
            if let Ok(action) = cell.update(neighbour_map) {
                match action {
                    Action::None => {
                        self.dst_buffer.insert(*idx, cell.clone());
                    }
                    Action::Replace(new_cell) => {
                        self.dst_buffer.insert(*idx, new_cell);
                    }
                    Action::Move(direction) => {
                        let cell = cell.clone();
                        let new_pos = &pos + &direction.into();

                        if new_pos.x < width
                            && new_pos.y < height
                            && let Ok(new_idx) = new_pos.try_into()
                        {
                            self.dst_buffer.insert(new_idx, cell);
                        } else if let Ok(old_idx) = pos.try_into() {
                            self.dst_buffer.insert(old_idx, cell);
                        };
                    }

                    Action::Vanish => {
                        // do nothing
                    }
                }
            }
        }

        std::mem::swap(&mut self.src_buffer, &mut self.dst_buffer);
        Ok(())
    }
    pub fn iter_cells(&self) -> impl Iterator<Item = (Position, Color)> + '_ {
        let iter = self.src_buffer.iter().map(|(id, cell)| {
            let pos: Vec2 = (*id).into();
            let color = match *cell {
                Cell::Sand => Color::Yellow,
                Cell::Wood => Color::Rgb(25, 120, 25),
                Cell::Fire => Color::Red,
                Cell::Border => Color::Cyan,
                Cell::Water => Color::Blue,
            };
            let res: Result<(u16, u16), eyre::Report> = pos.try_into();
            if let Ok(pos) = res {
                return Some((Position::from(pos), color));
            }
            None
        });
        iter.flatten()
    }
    pub fn update_window_size(&mut self, window: Window) {
        self.window = Some(window)
    }
}

impl MaterialCanvas for Simulation {
    fn set_pixel(&mut self, pos: &Vec2, cell: Cell) -> eyre::Result<()> {
        self.flip(pos, cell)?;
        Ok(())
    }

    fn set_pixels(&mut self, points: &[Vec2], cell: Cell) -> eyre::Result<()> {
        for pos in points {
            self.set_pixel(pos, cell.clone())?;
        }
        Ok(())
    }

    fn remove_pixel(&mut self, pos: &Vec2) -> eyre::Result<()> {
        let idx = pos.try_into()?;
        self.src_buffer.remove(&idx);
        Ok(())
    }
}

pub trait MaterialCanvas {
    fn set_pixel(&mut self, pos: &Vec2, cell: Cell) -> eyre::Result<()>;
    fn set_pixels(&mut self, points: &[Vec2], cell: Cell) -> eyre::Result<()>;
    fn remove_pixel(&mut self, pos: &Vec2) -> eyre::Result<()>;
}

#[cfg(test)]
mod tests {

    use super::*;

    #[cfg(test)]
    mod tests_finc_cell_h3_w3_empty_map {
        use super::*;
        const HEIGHT: i16 = 3;
        const WIDTH: i16 = 3;

        #[test]
        fn test_cell_m1_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((-1i16, -1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((0i16, -1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_1_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((1i16, -1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_2_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((2i16, -1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_3_m1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((3i16, -1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_m1_0_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((-1i16, 0).into(), WIDTH, HEIGHT, &hash_map);
            let cell = cell_opt.unwrap();
            assert!(matches!(cell, Cell::Border));
        }
        #[test]
        fn test_cell_0_0_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((0i16, 0).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_1_0_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((1i16, 0).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_2_0_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((2i16, 0).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }

        #[test]
        fn test_cell_3_0_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((3i16, 0).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_m1_1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((-1i16, 1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_1_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((0i16, 1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_1_1_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((1i16, 1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_2_1_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((2i16, 1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }

        #[test]
        fn test_cell_3_1_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((3i16, 1).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_m1_2_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((-1i16, 2).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_2_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((0i16, 2).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_1_2_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((1i16, 2).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }
        #[test]
        fn test_cell_2_2_is_empty() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((2i16, 2).into(), WIDTH, HEIGHT, &hash_map);
            assert!(cell_opt.is_none());
        }

        #[test]
        fn test_cell_3_2_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((3i16, 2).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_m1_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((-1i16, 3).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_0_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((0i16, 3).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_1_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((1i16, 3).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
        #[test]
        fn test_cell_2_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((2i16, 3).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }

        #[test]
        fn test_cell_3_3_is_border() {
            let hash_map = HashMap::new();

            let cell_opt = Simulation::find_cell((3i16, 3).into(), WIDTH, HEIGHT, &hash_map);
            assert!(matches!(*cell_opt.unwrap(), Cell::Border));
        }
    }
}
