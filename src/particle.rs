use std::collections::HashMap;

use color_eyre::eyre::{self, Ok};
use rand;
#[derive(Clone)]
pub enum Cell {
    Sand,
    Wood,
    Fire,
}

pub enum Action {
    None,
    Replace(Cell),
    Move(Direction),
    Vanish,
}
#[derive(Hash, PartialEq, Eq)]
pub enum Direction {
    Up,
    UpRight,
    UpLeft,
    Right,
    Left,
    Down,
    DownRight,
    DownLeft,
}

impl Cell {
    pub fn update(&self, neighbours: HashMap<Direction, &Cell>) -> eyre::Result<Action> {
        match self {
            Cell::Sand => self.handle_sand(neighbours),
            Cell::Wood => self.handle_wood(neighbours),
            Cell::Fire => self.handle_fire(neighbours),
        }
    }

    fn handle_sand(&self, neighbours: HashMap<Direction, &Cell>) -> eyre::Result<Action> {
        // self is of type Sand

        if !neighbours.contains_key(&Direction::Down) {
            return Ok(Action::Move(Direction::Down));
        }

        let (first, second) = if rand::random_bool(0.5) {
            (Direction::DownRight, Direction::DownLeft)
        } else {
            (Direction::DownLeft, Direction::DownRight)
        };
        if !neighbours.contains_key(&first) {
            return Ok(Action::Move(first));
        };
        if !neighbours.contains_key(&second) {
            return Ok(Action::Move(second));
        }
        Ok(Action::None)
    }
    fn handle_fire(&self, _neighbours: HashMap<Direction, &Cell>) -> eyre::Result<Action> {
        Ok(Action::Vanish)
    }
    fn handle_wood(&self, neighbours: HashMap<Direction, &Cell>) -> eyre::Result<Action> {
        let filer: Vec<()> = neighbours
            .iter()
            .filter_map(|(_direction, cell)| {
                if matches!(cell, Cell::Fire) {
                    return Some(());
                }
                None
            })
            .collect();
        if !filer.is_empty() {
            return Ok(Action::Replace(Cell::Fire));
        }
        Ok(Action::None)
    }
}
