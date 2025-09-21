use crate::base::{Length, Percentage};
use crate::style::{SizeValue, SpaceSize};
use anyhow::{Result, anyhow};
use std::str::FromStr;

/// FlexBasis 枚举，支持 <length> | <percentage> | auto
#[derive(Debug, Clone, PartialEq)]
pub enum FlexBasis {
    Length(Length),
    Percentage(Percentage),
    Auto,
}

impl Default for FlexBasis {
    fn default() -> Self {
        Self::Auto
    }
}

impl FromStr for FlexBasis {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();
        
        if s == "auto" {
            return Ok(FlexBasis::Auto);
        }
        
        // Try to parse as percentage first (ends with %)
        if s.ends_with('%') {
            match Percentage::from_str(&s) {
                Ok(percentage) => Ok(FlexBasis::Percentage(percentage)),
                Err(e) => Err(anyhow!("Invalid percentage value: {}", e)),
            }
        } else {
            // Try to parse as length
            match Length::from_str(&s) {
                Ok(length) => Ok(FlexBasis::Length(length)),
                Err(e) => Err(anyhow!("Invalid flex-basis value '{}': {}", s, e)),
            }
        }
    }
}

impl FlexBasis {
    /// Convert FlexBasis to SpaceSize based on FlexDirection
    /// Only the dimension corresponding to the flex direction will have the flex-basis value,
    /// other dimensions will be set to SizeValue::Auto
    pub fn to_space_size(&self, direction: &FlexDirection) -> SpaceSize {
        let size_value: SizeValue = self.clone().into(); // Convert FlexBasis to SizeValue
        
        match direction {
            FlexDirection::X | FlexDirection::ReverseX => {
                SpaceSize::new(size_value, SizeValue::Auto, SizeValue::Auto)
            }
            FlexDirection::Y | FlexDirection::ReverseY => {
                SpaceSize::new(SizeValue::Auto, size_value, SizeValue::Auto)
            }
            FlexDirection::Z | FlexDirection::ReverseZ => {
                SpaceSize::new(SizeValue::Auto, SizeValue::Auto, size_value)
            }
        }
    }
}

/// align-items属性枚举
#[derive(Debug, Clone, PartialEq)]
pub enum AlignItem {
    FlexStart,
    FlexEnd,
    Center,
}

impl FromStr for AlignItem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "flex-start" => Ok(AlignItem::FlexStart),
            "flex-end" => Ok(AlignItem::FlexEnd),
            "center" => Ok(AlignItem::Center),
            _ => Err(anyhow!("Invalid align-item value: {}", s)),
        }
    }
}

/// align-items属性，支持两个交叉轴的对齐方式
#[derive(Debug, Clone, PartialEq)]
pub struct AlignItems {
    pub cross1: AlignItem,  // 第一个交叉轴对齐方式
    pub cross2: AlignItem,  // 第二个交叉轴对齐方式
}

impl Default for AlignItems {
    fn default() -> Self {
        Self {
            cross1: AlignItem::FlexStart,
            cross2: AlignItem::FlexStart,
        }
    }
}

impl FromStr for AlignItems {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        
        if parts.len() != 2 {
            return Err(anyhow!("align-items must have exactly 2 values (cross1 cross2)"));
        }
        
        let cross1 = AlignItem::from_str(parts[0])?;
        let cross2 = AlignItem::from_str(parts[1])?;
        
        Ok(AlignItems { cross1, cross2 })
    }
}

/// justify-content属性枚举
#[derive(Debug, Clone, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::FlexStart
    }
}

impl FromStr for JustifyContent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "flex-start" => Ok(JustifyContent::FlexStart),
            "flex-end" => Ok(JustifyContent::FlexEnd),
            "center" => Ok(JustifyContent::Center),
            "space-between" => Ok(JustifyContent::SpaceBetween),
            "space-around" => Ok(JustifyContent::SpaceAround),
            "space-evenly" => Ok(JustifyContent::SpaceEvenly),
            _ => Err(anyhow!("Invalid justify-content value: {}", s)),
        }
    }
}

/// flex-direction属性枚举
#[derive(Debug, Clone, PartialEq)]
pub enum FlexDirection {
    X,
    Y,
    Z,
    ReverseX,
    ReverseY,
    ReverseZ,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::ReverseZ
    }
}

impl FromStr for FlexDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "x" => Ok(FlexDirection::X),
            "y" => Ok(FlexDirection::Y),
            "z" => Ok(FlexDirection::Z),
            "x-reverse" => Ok(FlexDirection::ReverseX),
            "y-reverse" => Ok(FlexDirection::ReverseY),
            "z-reverse" => Ok(FlexDirection::ReverseZ),
            _ => Err(anyhow!("Invalid flex-direction value: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flex_basis_from_str() {
        assert_eq!(FlexBasis::from_str("auto").unwrap(), FlexBasis::Auto);
        assert_eq!(
            FlexBasis::from_str("50%").unwrap(),
            FlexBasis::Percentage(Percentage::new(50))
        );
        assert_eq!(
            FlexBasis::from_str("100mm").unwrap(),
            FlexBasis::Length(Length::from_mm(100))
        );
        assert!(FlexBasis::from_str("invalid").is_err());
    }

    #[test]
    fn test_align_item_from_str() {
        assert_eq!(
            AlignItem::from_str("flex-start").unwrap(),
            AlignItem::FlexStart
        );
        assert_eq!(
            AlignItem::from_str("flex-end").unwrap(),
            AlignItem::FlexEnd
        );
        assert_eq!(
            AlignItem::from_str("center").unwrap(),
            AlignItem::Center
        );
        assert!(AlignItem::from_str("invalid").is_err());
    }

    #[test]
    fn test_align_items_from_str() {
        let align_items = AlignItems::from_str("flex-start center").unwrap();
        assert_eq!(align_items.cross1, AlignItem::FlexStart);
        assert_eq!(align_items.cross2, AlignItem::Center);
        
        let align_items = AlignItems::from_str("flex-end flex-start").unwrap();
        assert_eq!(align_items.cross1, AlignItem::FlexEnd);
        assert_eq!(align_items.cross2, AlignItem::FlexStart);
        
        assert!(AlignItems::from_str("invalid").is_err());
        assert!(AlignItems::from_str("flex-start").is_err());
        assert!(AlignItems::from_str("flex-start center extra").is_err());
    }

    #[test]
    fn test_justify_content_from_str() {
        assert_eq!(
            JustifyContent::from_str("flex-start").unwrap(),
            JustifyContent::FlexStart
        );
        assert_eq!(
            JustifyContent::from_str("flex-end").unwrap(),
            JustifyContent::FlexEnd
        );
        assert_eq!(
            JustifyContent::from_str("center").unwrap(),
            JustifyContent::Center
        );
        assert_eq!(
            JustifyContent::from_str("space-between").unwrap(),
            JustifyContent::SpaceBetween
        );
        assert_eq!(
            JustifyContent::from_str("space-around").unwrap(),
            JustifyContent::SpaceAround
        );
        assert_eq!(
            JustifyContent::from_str("space-evenly").unwrap(),
            JustifyContent::SpaceEvenly
        );
        assert!(JustifyContent::from_str("invalid").is_err());
    }

    #[test]
    fn test_flex_direction_from_str() {
        assert_eq!(FlexDirection::from_str("x").unwrap(), FlexDirection::X);
        assert_eq!(FlexDirection::from_str("y").unwrap(), FlexDirection::Y);
        assert_eq!(FlexDirection::from_str("z").unwrap(), FlexDirection::Z);
        assert_eq!(
            FlexDirection::from_str("x-reverse").unwrap(),
            FlexDirection::ReverseX
        );
        assert_eq!(
            FlexDirection::from_str("y-reverse").unwrap(),
            FlexDirection::ReverseY
        );
        assert_eq!(
            FlexDirection::from_str("z-reverse").unwrap(),
            FlexDirection::ReverseZ
        );
        assert!(FlexDirection::from_str("invalid").is_err());
    }

    #[test]
    fn test_flex_basis_to_space_size() {
        // Test with Length
        let flex_basis = FlexBasis::Length(Length::from_mm(100));
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::X),
            SpaceSize::new(SizeValue::Length(Length::from_mm(100)), SizeValue::Auto, SizeValue::Auto)
        );
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::Y),
            SpaceSize::new(SizeValue::Auto, SizeValue::Length(Length::from_mm(100)), SizeValue::Auto)
        );
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::Z),
            SpaceSize::new(SizeValue::Auto, SizeValue::Auto, SizeValue::Length(Length::from_mm(100)))
        );
        
        // Test with Percentage
        let flex_basis = FlexBasis::Percentage(Percentage::new(50));
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::X),
            SpaceSize::new(SizeValue::Percentage(Percentage::new(50)), SizeValue::Auto, SizeValue::Auto)
        );
        
        // Test with Auto
        let flex_basis = FlexBasis::Auto;
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::X),
            SpaceSize::new(SizeValue::Auto, SizeValue::Auto, SizeValue::Auto)
        );
        
        // Test with reverse directions (should behave the same as normal directions)
        let flex_basis = FlexBasis::Length(Length::from_mm(100));
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::ReverseX),
            flex_basis.to_space_size(&FlexDirection::X)
        );
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::ReverseY),
            flex_basis.to_space_size(&FlexDirection::Y)
        );
        
        assert_eq!(
            flex_basis.to_space_size(&FlexDirection::ReverseZ),
            flex_basis.to_space_size(&FlexDirection::Z)
        );
    }
}