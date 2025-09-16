use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;
use std::str::FromStr;

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

impl FromStr for Length {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Trim whitespace
        let s = s.trim();
        
        // Check if string is empty
        if s.is_empty() {
            return Err("Empty string".to_string());
        }
        
        // Find the position where the number ends and the unit begins
        let mut split_pos = s.len();
        for (i, ch) in s.char_indices() {
            if !ch.is_ascii_digit() && ch != '.' {
                split_pos = i;
                break;
            }
        }
        
        // Parse the number part
        let number_str = &s[..split_pos];
        let number: f64 = number_str.parse().map_err(|_| format!("Invalid number: {}", number_str))?;
        
        // Parse the unit part (trim leading whitespace)
        let unit_str = &s[split_pos..].trim_start().to_lowercase();
        let multiplier = match unit_str.as_str() {
            "mm" => 1.0,
            "cm" => 10.0,
            "m" => 1000.0,
            "" => 1.0, // Default to mm if no unit specified
            _ => return Err(format!("Unknown unit: {}", unit_str)),
        };
        
        // Calculate the value in mm
        let mm_value = number * multiplier;
        
        // Check for negative values
        if mm_value < 0.0 {
            return Err("Length cannot be negative".to_string());
        }
        
        // Convert to u32, checking for overflow
        if mm_value > u32::MAX as f64 {
            return Err("Length value too large".to_string());
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
            // Avoid division by zero
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_from_str() {
        // Test parsing "auto" (case insensitive)
        let result: Auto<i32> = "auto".parse().unwrap();
        assert_eq!(result, Auto::Auto);

        let result: Auto<i32> = "Auto".parse().unwrap();
        assert_eq!(result, Auto::Auto);

        let result: Auto<i32> = "AUTO".parse().unwrap();
        assert_eq!(result, Auto::Auto);

        // Test parsing specific values
        let result: Auto<i32> = "42".parse().unwrap();
        assert_eq!(result, Auto::Value(42));

        let result: Auto<String> = "hello".parse().unwrap();
        assert_eq!(result, Auto::Value("hello".to_string()));

        // Test parsing invalid values
        let result: Result<Auto<i32>, _> = "not_a_number".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_methods() {
        let auto: Auto<i32> = Auto::Auto;
        assert!(auto.is_auto());
        assert!(!auto.is_value());
        assert_eq!(auto.as_value(), None);
        // Clone for the second assertion since value() takes ownership
        assert_eq!(auto.clone().value(), None);

        let value: Auto<i32> = Auto::Value(42);
        assert!(!value.is_auto());
        assert!(value.is_value());
        assert_eq!(value.as_value(), Some(&42));
        // Clone for the second assertion since value() takes ownership
        assert_eq!(value.clone().value(), Some(42));
    }
}

#[cfg(test)]
mod length_tests {
    use super::*;

    #[test]
    fn test_length_creation() {
        let len = Length::from_mm(1000);
        assert_eq!(len.mm(), 1000);
        assert_eq!(len.cm(), 100);
        assert_eq!(len.m(), 1);

        let len = Length::from_cm(150);
        assert_eq!(len.mm(), 1500);
        assert_eq!(len.cm(), 150);
        assert_eq!(len.m(), 1);

        let len = Length::from_m(2);
        assert_eq!(len.mm(), 2000);
        assert_eq!(len.cm(), 200);
        assert_eq!(len.m(), 2);
    }

    #[test]
    fn test_length_from_str() {
        // Test mm
        let len: Length = "3mm".parse().unwrap();
        assert_eq!(len.mm(), 3);

        // Test cm
        let len: Length = "5cm".parse().unwrap();
        assert_eq!(len.mm(), 50);

        // Test m
        let len: Length = "2m".parse().unwrap();
        assert_eq!(len.mm(), 2000);

        // Test default (mm)
        let len: Length = "10".parse().unwrap();
        assert_eq!(len.mm(), 10);

        // Test fractional values
        let len: Length = "1.5cm".parse().unwrap();
        assert_eq!(len.mm(), 15);

        // Test with whitespace
        let len: Length = " 3 mm ".parse().unwrap();
        assert_eq!(len.mm(), 3);
    }

    #[test]
    fn test_length_display() {
        // Test mm display
        let len = Length::from_mm(5);
        assert_eq!(format!("{}", len), "5mm");

        // Test cm display
        let len = Length::from_mm(50);
        assert_eq!(format!("{}", len), "5cm");

        // Test m display
        let len = Length::from_mm(5000);
        assert_eq!(format!("{}", len), "5m");

        // Test mixed display (should show as mm if not exact cm or m)
        let len = Length::from_mm(55);
        assert_eq!(format!("{}", len), "55mm");

        let len = Length::from_mm(550);
        assert_eq!(format!("{}", len), "55cm");
    }

    #[test]
    fn test_length_from_str_errors() {
        // Test invalid number
        let result: Result<Length, _> = "abcmm".parse();
        assert!(result.is_err());

        // Test unknown unit
        let result: Result<Length, _> = "5km".parse();
        assert!(result.is_err());

        // Test negative value
        let result: Result<Length, _> = "-5mm".parse();
        assert!(result.is_err());

        // Test empty string
        let result: Result<Length, _> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_length_sum() {
        let lengths = vec![Length::from_mm(100), Length::from_mm(200), Length::from_mm(300)];
        let total: Length = lengths.iter().copied().sum();
        assert_eq!(total, Length::from_mm(600));
        
        // Test with empty iterator
        let empty: Vec<Length> = vec![];
        let total: Length = empty.iter().copied().sum();
        assert_eq!(total, Length::from_mm(0));
    }

    #[test]
    fn test_length_add_assign() {
        let mut len1 = Length::from_mm(100);
        let len2 = Length::from_mm(200);
        
        len1 += len2;
        assert_eq!(len1, Length::from_mm(300));
        
        // Test saturation
        let mut len_max = Length(u32::MAX);
        len_max += Length::from_mm(1);
        assert_eq!(len_max, Length(u32::MAX));
    }
}