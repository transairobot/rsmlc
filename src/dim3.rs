use std::ops::{Add, Sub, Mul, Div};
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dim3<T> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: Default> Default for Dim3<T> {
    fn default() -> Self {
        Self {
            x: T::default(),
            y: T::default(),
            z: T::default(),
        }
    }
}

impl<T> Dim3<T> {
    pub fn new(x: T, y: T, z: T) -> Self {
        Dim3 { x, y, z }
    }

    pub fn x(&self) -> T where T: Copy {
        self.x
    }

    pub fn y(&self) -> T where T: Copy {
        self.y
    }

    pub fn z(&self) -> T where T: Copy {
        self.z
    }
}

impl<T: Add<Output = T>> Add for Dim3<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl<T: Sub<Output = T>> Sub for Dim3<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl<T: Mul<Output = T>> Mul for Dim3<T> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl<T: Div<Output = T>> Div for Dim3<T> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Dim3<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}
