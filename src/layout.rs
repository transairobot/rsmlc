use crate::auto::{Auto, Length, Size};
use serde::{Deserialize, Deserializer, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum AxisPos {
    Random,
    Min,
    Max,
    Length(Length),
}

impl Serialize for AxisPos {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            AxisPos::Random => serializer.serialize_str("random"),
            AxisPos::Min => serializer.serialize_str("min"),
            AxisPos::Max => serializer.serialize_str("max"),
            AxisPos::Length(length) => length.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AxisPos {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "random" => Ok(AxisPos::Random),
            "min" => Ok(AxisPos::Min),
            "max" => Ok(AxisPos::Max),
            _ => Length::from_str(&s)
                .map(AxisPos::Length)
                .map_err(serde::de::Error::custom),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Flex {
    direction: String,
}

#[derive(Debug, Deserialize)]
struct Properties {}

#[derive(Debug, Deserialize)]
struct Layout {
    selector: String,
    space_size: Option<Size>, // (x, y, z)
    pos_x: Option<AxisPos>,
    pos_y: Option<AxisPos>,
    pos_z: Option<AxisPos>,
    flex_direction: Option<String>,
    flex_basis: Auto<Length>,
}

#[derive(Debug, Deserialize)]
struct LayoutSheet {
    layouts: Vec<Layout>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_axis_pos_deserialize_random() {
        let json = "\"random\"";
        let axis_pos: AxisPos = serde_json::from_str(json).unwrap();
        assert_eq!(axis_pos, AxisPos::Random);
    }

    #[test]
    fn test_axis_pos_deserialize_min() {
        let json = "\"min\"";
        let axis_pos: AxisPos = serde_json::from_str(json).unwrap();
        assert_eq!(axis_pos, AxisPos::Min);
    }

    #[test]
    fn test_axis_pos_deserialize_max() {
        let json = "\"max\"";
        let axis_pos: AxisPos = serde_json::from_str(json).unwrap();
        assert_eq!(axis_pos, AxisPos::Max);
    }

    #[test]
    fn test_axis_pos_deserialize_length() {
        let json = "\"10cm\"";
        let axis_pos: AxisPos = serde_json::from_str(json).unwrap();
        assert_eq!(axis_pos, AxisPos::Length(Length::from_cm(10)));
    }

    #[test]
    fn test_axis_pos_serialize_random() {
        let axis_pos = AxisPos::Random;
        let json = serde_json::to_string(&axis_pos).unwrap();
        assert_eq!(json, "\"random\"");
    }

    #[test]
    fn test_axis_pos_serialize_min() {
        let axis_pos = AxisPos::Min;
        let json = serde_json::to_string(&axis_pos).unwrap();
        assert_eq!(json, "\"min\"");
    }

    #[test]
    fn test_axis_pos_serialize_max() {
        let axis_pos = AxisPos::Max;
        let json = serde_json::to_string(&axis_pos).unwrap();
        assert_eq!(json, "\"max\"");
    }

    #[test]
    fn test_axis_pos_serialize_length() {
        let axis_pos = AxisPos::Length(Length::from_cm(10));
        let json = serde_json::to_string(&axis_pos).unwrap();
        // Length serializes as a number (u32), not a string
        assert_eq!(json, "100");
    }
}
