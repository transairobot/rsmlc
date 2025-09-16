use crate::auto::{Auto, Length};
use std::str::FromStr;
use anyhow::{Result, anyhow};

/// Display属性枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Display {
    Block,
    Flex,
}

impl FromStr for Display {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "block" => Ok(Display::Block),
            "flex" => Ok(Display::Flex),
            _ => Err(anyhow!("Invalid display value: {}", s)),
        }
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
}

impl FromStr for FlexDirection {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "x" => Ok(FlexDirection::X),
            "y" => Ok(FlexDirection::Y),
            "z" => Ok(FlexDirection::Z),
            _ => Err(anyhow!("Invalid flex-direction value: {}", s)),
        }
    }
}

/// Position属性枚举，支持每个轴的定位
#[derive(Debug, Clone, PartialEq)]
pub enum AxisPos {
    Min,
    Max,
    Random,
    Length(Length),
}

impl FromStr for AxisPos {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "min" => Ok(AxisPos::Min),
            "max" => Ok(AxisPos::Max),
            "random" => Ok(AxisPos::Random),
            _ => {
                // 尝试解析为长度
                match Length::from_str(s) {
                    Ok(length) => Ok(AxisPos::Length(length)),
                    Err(_) => Err(anyhow!("Invalid position value: {}", s)),
                }
            }
        }
    }
}

/// Position结构体，包含三个轴的定位信息
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub x: AxisPos,
    pub y: AxisPos,
    pub z: AxisPos,
}

impl Position {
    pub fn new(x: AxisPos, y: AxisPos, z: AxisPos) -> Self {
        Position { x, y, z }
    }
}

impl FromStr for Position {
    type Err = anyhow::Error;
    
    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();
        if parts.len() != 3 {
            return Err(anyhow!("Position must have exactly 3 values (x, y, z)"));
        }
        
        let x = AxisPos::from_str(parts[0])?;
        let y = AxisPos::from_str(parts[1])?;
        let z = AxisPos::from_str(parts[2])?;
        
        Ok(Position::new(x, y, z))
    }
}

/// Style结构体，包含所有支持的样式属性
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub size: (Auto<Length>, Auto<Length>, Auto<Length>), // size: 三个维度的尺寸 (x, y, z)
    pub display: Display,                // display: block 或 flex
    pub justify_content: Option<JustifyContent>,  // justify-content: 对齐方式
    pub flex_direction: Option<FlexDirection>,    // flex-direction: x, y, z
    pub position: Option<Position>,      // pos: 三个轴的定位
}

impl Default for Style {
    fn default() -> Self {
        Style {
            size: (Auto::Auto, Auto::Auto, Auto::Auto), // 默认尺寸为auto
            display: Display::Block,         // 默认显示为block
            justify_content: None,           // 默认无justify-content
            flex_direction: None,            // 默认无flex-direction
            position: None,                  // 默认无位置
        }
    }
}

impl Style {
    /// 创建新的Style实例
    pub fn new() -> Self {
        Style::default()
    }
    
    /// 获取x维度的尺寸
    pub fn size_x(&self) -> &Auto<Length> {
        &self.size.0
    }
    
    /// 获取y维度的尺寸
    pub fn size_y(&self) -> &Auto<Length> {
        &self.size.1
    }
    
    /// 获取z维度的尺寸
    pub fn size_z(&self) -> &Auto<Length> {
        &self.size.2
    }
    
    /// 从样式字符串解析Style对象
    /// 支持格式如: "size:10m 10m 10m;display:flex;justify-content:flex-end;"
    pub fn from_style_string(style_str: &str) -> Result<Self> {
        let mut style = Style::new();
        
        // 分割样式声明
        for declaration in style_str.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }
            
            // 分割属性名和值
            let parts: Vec<&str> = declaration.split(':').collect();
            if parts.len() != 2 {
                return Err(anyhow!("Invalid style declaration: {}", declaration));
            }
            
            let property = parts[0].trim();
            let value = parts[1].trim();
            
            match property {
                "size" => {
                    // 解析尺寸，格式如 "10m 10m 10m" 或 "auto auto auto"
                    let size_parts: Vec<&str> = value.split_whitespace().collect();
                    if size_parts.len() != 3 {
                        return Err(anyhow!("Size must have exactly 3 values (x, y, z)"));
                    }
                    
                    let x = parse_auto_length(size_parts[0])?;
                    let y = parse_auto_length(size_parts[1])?;
                    let z = parse_auto_length(size_parts[2])?;
                    
                    style.size = (x, y, z);
                }
                "display" => {
                    style.display = Display::from_str(value)?;
                }
                "justify-content" => {
                    style.justify_content = Some(JustifyContent::from_str(value)?);
                }
                "flex-direction" => {
                    style.flex_direction = Some(FlexDirection::from_str(value)?);
                }
                "pos" => {
                    style.position = Some(Position::from_str(value)?);
                }
                _ => {
                    // 忽略未知属性而不是报错，以提高兼容性
                    eprintln!("Warning: Unknown style property '{}'", property);
                }
            }
        }
        
        Ok(style)
    }
}

/// 计算后的样式，包含绝对的尺寸和位置
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedStyle {
    pub size: (Length, Length, Length),
    pub pos: (Length, Length, Length),
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            size: (Length::from_m(0), Length::from_m(0), Length::from_m(0)),
            pos: (Length::from_m(0), Length::from_m(0), Length::from_m(0)),
        }
    }
}

/// 解析Auto<Length>值
fn parse_auto_length(s: &str) -> Result<Auto<Length>> {
    if s.to_lowercase() == "auto" {
        Ok(Auto::Auto)
    } else {
        match Length::from_str(s) {
            Ok(length) => Ok(Auto::Value(length)),
            Err(e) => Err(anyhow!("Invalid length value '{}': {}", s, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_display_parsing() {
        assert_eq!(Display::from_str("block").unwrap(), Display::Block);
        assert_eq!(Display::from_str("flex").unwrap(), Display::Flex);
        assert!(Display::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_justify_content_parsing() {
        assert_eq!(JustifyContent::from_str("flex-start").unwrap(), JustifyContent::FlexStart);
        assert_eq!(JustifyContent::from_str("flex-end").unwrap(), JustifyContent::FlexEnd);
        assert_eq!(JustifyContent::from_str("center").unwrap(), JustifyContent::Center);
        assert_eq!(JustifyContent::from_str("space-between").unwrap(), JustifyContent::SpaceBetween);
        assert_eq!(JustifyContent::from_str("space-around").unwrap(), JustifyContent::SpaceAround);
        assert_eq!(JustifyContent::from_str("space-evenly").unwrap(), JustifyContent::SpaceEvenly);
        assert!(JustifyContent::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_flex_direction_parsing() {
        assert_eq!(FlexDirection::from_str("x").unwrap(), FlexDirection::X);
        assert_eq!(FlexDirection::from_str("y").unwrap(), FlexDirection::Y);
        assert_eq!(FlexDirection::from_str("z").unwrap(), FlexDirection::Z);
        assert!(FlexDirection::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_axis_pos_parsing() {
        assert_eq!(AxisPos::from_str("min").unwrap(), AxisPos::Min);
        assert_eq!(AxisPos::from_str("max").unwrap(), AxisPos::Max);
        assert_eq!(AxisPos::from_str("random").unwrap(), AxisPos::Random);
        assert_eq!(AxisPos::from_str("10cm").unwrap(), AxisPos::Length(Length::from_cm(10)));
        assert!(AxisPos::from_str("invalid").is_err());
    }
    
    #[test]
    fn test_position_parsing() {
        let pos = Position::from_str("min max 10cm").unwrap();
        assert_eq!(pos.x, AxisPos::Min);
        assert_eq!(pos.y, AxisPos::Max);
        assert_eq!(pos.z, AxisPos::Length(Length::from_cm(10)));
        
        assert!(Position::from_str("min max").is_err());
        assert!(Position::from_str("min max 10cm extra").is_err());
    }
    
    #[test]
    fn test_style_parsing() {
        // 测试完整的样式字符串解析
        let style_str = "size:10m 5m auto;display:flex;justify-content:flex-end;flex-direction:x;pos:min max 10cm";
        let style = Style::from_style_string(style_str).unwrap();
        
        // 验证size
        assert_eq!(style.size_x(), &Auto::Value(Length::from_m(10)));
        assert_eq!(style.size_y(), &Auto::Value(Length::from_m(5)));
        assert_eq!(style.size_z(), &Auto::Auto);
        
        // 验证display
        assert_eq!(style.display, Display::Flex);
        
        // 验证justify-content
        assert_eq!(style.justify_content, Some(JustifyContent::FlexEnd));
        
        // 验证flex-direction
        assert_eq!(style.flex_direction, Some(FlexDirection::X));
        
        // 验证position
        let pos = style.position.unwrap();
        assert_eq!(pos.x, AxisPos::Min);
        assert_eq!(pos.y, AxisPos::Max);
        assert_eq!(pos.z, AxisPos::Length(Length::from_cm(10)));
    }
    
    #[test]
    fn test_style_parsing_with_defaults() {
        // 测试只包含部分属性的样式字符串
        let style_str = "display:block";
        let style = Style::from_style_string(style_str).unwrap();
        
        // 验证默认值
        assert_eq!(style.display, Display::Block);
        assert_eq!(style.size_x(), &Auto::Auto);
        assert_eq!(style.size_y(), &Auto::Auto);
        assert_eq!(style.size_z(), &Auto::Auto);
        assert_eq!(style.justify_content, None);
        assert_eq!(style.flex_direction, None);
        assert_eq!(style.position, None);
    }
    
    #[test]
    fn test_invalid_style_parsing() {
        // 测试无效的样式字符串
        assert!(Style::from_style_string("invalid").is_err());
        assert!(Style::from_style_string("size:10m 5m").is_err()); // 少于3个值
        assert!(Style::from_style_string("display:invalid").is_err());
    }
}