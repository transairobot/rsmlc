use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;
use crate::error::RsmlError;

/// An Auto type that can either be a specific value or automatically determined.
/// Similar to Option<T>, but with special parsing behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Auto<T> {
    /// The value should be automatically determined
    Auto,
    /// A specific value
    Value(T),
}

impl<T> Auto<T> {
    /// Returns true if the auto is a Auto value.
    pub fn is_auto(&self) -> bool {
        matches!(self, Auto::Auto)
    }

    /// Returns true if the auto is a Value.
    pub fn is_value(&self) -> bool {
        matches!(self, Auto::Value(_))
    }

    /// Converts from Auto<T> to Option<T>.
    pub fn value(self) -> Option<T> {
        match self {
            Auto::Auto => None,
            Auto::Value(value) => Some(value),
        }
    }

    /// Converts from &Auto<T> to Option<&T>.
    pub fn as_value(&self) -> Option<&T> {
        match self {
            Auto::Auto => None,
            Auto::Value(value) => Some(value),
        }
    }
}

impl<T> FromStr for Auto<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("auto") {
            Ok(Auto::Auto)
        } else {
            T::from_str(s)
                .map(Auto::Value)
                .map_err(|e| format!("Failed to parse '{}': {}", s, e))
        }
    }
}

impl<'de, T> Deserialize<'de> for Auto<T>
where
    T: FromStr,
    T::Err: fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Auto::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl<T> Default for Auto<T> {
    fn default() -> Self {
        Auto::Auto
    }
}

/// A length structure with millimeters as the base unit, stored as u32
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Length(u32);

/// A percentage structure stored as u32 (0-100%)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Percentage(u32);

impl Length {
    /// Create a new Length from millimeters
    pub const fn from_mm(mm: u32) -> Self {
        Length(mm)
    }

    /// Create a new Length from centimeters
    pub const fn from_cm(cm: u32) -> Self {
        Length(cm * 10)
    }

    /// Create a new Length from meters
    pub const fn from_m(m: u32) -> Self {
        Length(m * 1000)
    }

    /// Get the length in millimeters
    pub const fn mm(&self) -> u32 {
        self.0
    }

    /// Get the length in centimeters (truncated)
    pub const fn cm(&self) -> u32 {
        self.0 / 10
    }

    /// Get the length in meters (truncated)
    pub const fn m(&self) -> u32 {
        self.0 / 1000
    }
}

impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 % 1000 == 0 {
            write!(f, "{}m", self.0 / 1000)
        } else if self.0 % 10 == 0 {
            write!(f, "{}cm", self.0 / 10)
        } else {
            write!(f, "{}mm", self.0)
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl FromStr for Length {
    type Err = RsmlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(RsmlError::ParseError { 
                field: "Length".to_string(), 
                message: "Empty string".to_string() 
            });
        }

        let mut split_pos = s.len();
        for (i, ch) in s.char_indices() {
            if !ch.is_ascii_digit() && ch != '.' {
                split_pos = i;
                break;
            }
        }

        let number_str = &s[..split_pos];
        let number: f64 = number_str.parse().map_err(|_| RsmlError::ParseError {
            field: "Length".to_string(),
            message: format!("Invalid number: {}", number_str)
        })?;

        let unit_str = &s[split_pos..].trim_start().to_lowercase();
        let multiplier = match unit_str.as_str() {
            "mm" => 1.0,
            "cm" => 10.0,
            "m" => 1000.0,
            "" => 1.0,
            _ => return Err(RsmlError::ParseError {
                field: "Length".to_string(),
                message: format!("Unknown unit: {}", unit_str)
            }),
        };

        let mm_value = number * multiplier;

        if mm_value < 0.0 {
            return Err(RsmlError::ParseError {
                field: "Length".to_string(),
                message: "Length cannot be negative".to_string()
            });
        }

        if mm_value > u32::MAX as f64 {
            return Err(RsmlError::ParseError {
                field: "Length".to_string(),
                message: "Length value too large".to_string()
            });
        }

        Ok(Length(mm_value as u32))
    }
}

impl<'de> Deserialize<'de> for Length {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Length::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl std::ops::Add for Length {
    type Output = Length;

    fn add(self, other: Length) -> Length {
        Length(self.0.saturating_add(other.0))
    }
}

impl std::ops::Sub for Length {
    type Output = Length;

    fn sub(self, other: Length) -> Length {
        Length(self.0.saturating_sub(other.0))
    }
}

impl std::ops::Mul<u32> for Length {
    type Output = Length;

    fn mul(self, scalar: u32) -> Length {
        Length(self.0.saturating_mul(scalar))
    }
}

impl std::ops::Div<u32> for Length {
    type Output = Length;

    fn div(self, scalar: u32) -> Length {
        if scalar == 0 {
            Length(0)
        } else {
            Length(self.0 / scalar)
        }
    }
}

impl std::ops::AddAssign for Length {
    fn add_assign(&mut self, other: Length) {
        self.0 = self.0.saturating_add(other.0);
    }
}

impl std::iter::Sum for Length {
    fn sum<I: Iterator<Item = Length>>(iter: I) -> Length {
        iter.fold(Length(0), |acc, x| acc + x)
    }
}

impl Percentage {
    pub const fn new(value: u32) -> Self {
        Percentage(value)
    }

    pub const fn value(&self) -> u32 {
        self.0
    }

    pub fn float(&self) -> f32 {
        self.0 as f32 / 100.0
    }
}

impl fmt::Display for Percentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}%", self.0)
    }
}

impl Default for Percentage {
    fn default() -> Self {
        Self(0)
    }
}

impl FromStr for Percentage {
    type Err = RsmlError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(RsmlError::ParseError {
                field: "Percentage".to_string(),
                message: "Empty string".to_string()
            });
        }

        if !s.ends_with('%') {
            return Err(RsmlError::ParseError {
                field: "Percentage".to_string(),
                message: "Percentage must end with %".to_string()
            });
        }

        let number_str = &s[..s.len() - 1];
        let number: u32 = number_str.parse().map_err(|_| RsmlError::ParseError {
            field: "Percentage".to_string(),
            message: format!("Invalid number: {}", number_str)
        })?;

        if number > 100 {
            return Err(RsmlError::ParseError {
                field: "Percentage".to_string(),
                message: "Percentage cannot be greater than 100%".to_string()
            });
        }

        Ok(Percentage(number))
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Percentage::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_from_str() {
        let result: Auto<i32> = "auto".parse().unwrap();
        assert_eq!(result, Auto::Auto);

        let result: Auto<i32> = "Auto".parse().unwrap();
        assert_eq!(result, Auto::Auto);

        let result: Auto<i32> = "AUTO".parse().unwrap();
        assert_eq!(result, Auto::Auto);

        let result: Auto<i32> = "42".parse().unwrap();
        assert_eq!(result, Auto::Value(42));

        let result: Auto<String> = "hello".parse().unwrap();
        assert_eq!(result, Auto::Value("hello".to_string()));

        let result: Result<Auto<i32>, _> = "not_a_number".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_length_from_str() {
        let len: Length = "3mm".parse().unwrap();
        assert_eq!(len.mm(), 3);

        let len: Length = "5cm".parse().unwrap();
        assert_eq!(len.mm(), 50);

        let len: Length = "2m".parse().unwrap();
        assert_eq!(len.mm(), 2000);

        let len: Length = "10".parse().unwrap();
        assert_eq!(len.mm(), 10);
    }

    #[test]
    fn test_length_from_str_errors() {
        let result: Result<Length, _> = "abcmm".parse();
        assert!(result.is_err());

        let result: Result<Length, _> = "5km".parse();
        assert!(result.is_err());

        let result: Result<Length, _> = "-5mm".parse();
        assert!(result.is_err());

        let result: Result<Length, _> = "".parse();
        assert!(result.is_err());
    }
}
