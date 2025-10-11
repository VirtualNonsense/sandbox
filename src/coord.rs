use std::{
    hash::Hash,
    ops::{Add, Div, Mul, Sub},
};

use color_eyre::eyre;
#[derive(Hash, PartialEq, Eq, Clone)]
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

#[derive(Debug, Clone)]
pub struct Vec2 {
    pub x: i16,
    pub y: i16,
}

impl From<Direction> for Vec2 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => Self { x: 0, y: -1 },
            Direction::UpRight => Self { x: 1, y: -1 },
            Direction::UpLeft => Self { x: -1, y: -1 },
            Direction::Right => Self { x: 1, y: 0 },
            Direction::Left => Self { x: -1, y: 0 },
            Direction::Down => Self { x: 0, y: 1 },
            Direction::DownRight => Self { x: 1, y: 1 },
            Direction::DownLeft => Self { x: -1, y: 1 },
        }
    }
}
impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Add for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Add<Vec2> for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}
impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}
impl Sub for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Sub<Vec2> for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<i16> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: i16) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Mul<i16> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: i16) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<i16> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: i16) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl Div<i16> for &Vec2 {
    type Output = Vec2;

    fn div(self, rhs: i16) -> Self::Output {
        Self::Output {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl TryInto<(u16, u16)> for Vec2 {
    type Error = color_eyre::Report;

    fn try_into(self) -> Result<(u16, u16), Self::Error> {
        if self.x < 0 || self.y < 0 {
            return Err(color_eyre::Report::msg(format!(
                "Some values are negative: ({}, {})",
                &self.x, &self.y
            )));
        }
        Ok((self.x as u16, self.y as u16))
    }
}
impl TryInto<(u16, u16)> for &Vec2 {
    type Error = color_eyre::Report;

    fn try_into(self) -> Result<(u16, u16), Self::Error> {
        if self.x < 0 || self.y < 0 {
            return Err(color_eyre::Report::msg(format!(
                "Some values are negative: ({}, {})",
                &self.x, &self.y
            )));
        }
        Ok((self.x as u16, self.y as u16))
    }
}

impl From<Vec2> for (i16, i16) {
    fn from(val: Vec2) -> Self {
        (val.x, val.y)
    }
}

impl From<&Vec2> for (i16, i16) {
    fn from(val: &Vec2) -> Self {
        (val.x, val.y)
    }
}

impl TryInto<u32> for Vec2 {
    type Error = eyre::Report;

    fn try_into(self) -> Result<u32, Self::Error> {
        let (x, y) = self.try_into()?;
        Ok(get_key_from_coords(x, y))
    }
}
impl TryInto<u32> for &Vec2 {
    type Error = eyre::Report;

    fn try_into(self) -> Result<u32, Self::Error> {
        let (x, y) = self.try_into()?;
        Ok(get_key_from_coords(x, y))
    }
}

impl From<u32> for Vec2 {
    fn from(value: u32) -> Self {
        get_coords_from_key(value).into()
    }
}

impl From<(i16, i16)> for Vec2 {
    fn from(value: (i16, i16)) -> Self {
        Self {
            x: value.0,
            y: value.1,
        }
    }
}
impl From<(u16, u16)> for Vec2 {
    fn from(value: (u16, u16)) -> Self {
        Self {
            x: value.0 as i16,
            y: value.1 as i16,
        }
    }
}

fn get_coords_from_key(key: u32) -> (u16, u16) {
    let x = (key >> 16) as u16;
    let y = ((u16::MAX as u32) & key) as u16;
    (x, y)
}

fn get_key_from_coords(x: u16, y: u16) -> u32 {
    (x as u32) << 16 | y as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_get_key_from_coords() {
        let x = 123u16;
        let y = 102u16;

        let key = get_key_from_coords(x, y);
        let actual = 8061030u32;
        assert_eq!(actual, key, "{} is not equal to {}", key, actual);
    }
    #[test]
    fn test_get_coords_from_key() {
        let x_actual = 123u16;
        let y_actual = 102u16;

        let key = 8061030u32;
        let (x, y) = get_coords_from_key(key);
        assert_eq!(x_actual, x, "{} is not equal to {}", x, x_actual);
        assert_eq!(y_actual, y, "{} is not equal to {}", y, y_actual);
    }
}
