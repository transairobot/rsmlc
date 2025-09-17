use crate::auto::{Auto, Length};
use crate::dim3::Dim3;
use anyhow::{anyhow, Result};
use rand::Rng;
use std::str::FromStr;

/// Display属性枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Display {
    Block,
    Stack,
}

impl FromStr for Display {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "block" => Ok(Display::Block),
            "flex" | "stack" => Ok(Display::Stack),
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
pub enum StackDirection {
    X,
    Y,
    Z,
}

impl FromStr for StackDirection {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "x" => Ok(StackDirection::X),
            "y" => Ok(StackDirection::Y),
            "z" => Ok(StackDirection::Z),
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

impl AxisPos {
    pub fn absolute_pos(&self, min: Length, max: Length) -> Length {
        match self {
            AxisPos::Min => min,
            AxisPos::Max => max,
            AxisPos::Random => {
                // 生成min和max之间的随机值
                let min_val = min.mm();
                let max_val = max.mm();
                let mut rng = rand::thread_rng();
                let rand_val = rng.gen_range(min_val..=max_val);
                Length::from_mm(rand_val)
            }
            AxisPos::Length(length) => *length,
        }
    }
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

/// Position类型，支持每个轴的定位，与size一样支持auto
pub type Position = (Auto<AxisPos>, Auto<AxisPos>, Auto<AxisPos>);

/// Style结构体，包含所有支持的样式属性
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub size: Dim3<Auto<Length>>, // size: 三个维度的尺寸 (x, y, z)
    pub display: Display,                                 // display: block 或 flex
    pub justify_content: Option<JustifyContent>,          // justify-content: 对齐方式
    pub stack_direction: Option<StackDirection>,          // stack-direction: x, y, z
    pub position: Position,                               // pos: 三个轴的定位
}

impl Default for Style {
    fn default() -> Self {
        Style {
            size: Dim3::new(Auto::Auto, Auto::Auto, Auto::Auto), // 默认尺寸为auto
            display: Display::Block,                    // 默认显示为block
            justify_content: None,                      // 默认无justify-content
            stack_direction: None,                      // 默认无flex-direction
            position: (Auto::Auto, Auto::Auto, Auto::Auto), // 默认位置为auto
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
        &self.size.x
    }

    /// 获取y维度的尺寸
    pub fn size_y(&self) -> &Auto<Length> {
        &self.size.y
    }

    /// 获取z维度的尺寸
    pub fn size_z(&self) -> &Auto<Length> {
        &self.size.z
    }

    /// 获取x维度的位置
    pub fn position_x(&self) -> &Auto<AxisPos> {
        &self.position.0
    }

    /// 获取y维度的位置
    pub fn position_y(&self) -> &Auto<AxisPos> {
        &self.position.1
    }

    /// 获取z维度的位置
    pub fn position_z(&self) -> &Auto<AxisPos> {
        &self.position.2
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

                    style.size = Dim3::new(x, y, z);
                }
                "display" => {
                    style.display = Display::from_str(value)?;
                }
                "justify-content" => {
                    style.justify_content = Some(JustifyContent::from_str(value)?);
                }
                "flex-direction" => {
                    style.stack_direction = Some(StackDirection::from_str(value)?);
                }
                "pos" => {
                    // 解析位置，格式如 "min max 10cm" 或 "auto auto auto"
                    let pos_parts: Vec<&str> = value.split_whitespace().collect();
                    if pos_parts.len() != 3 {
                        return Err(anyhow!("Position must have exactly 3 values (x, y, z)"));
                    }

                    let x = parse_auto_axis_pos(pos_parts[0])?;
                    let y = parse_auto_axis_pos(pos_parts[1])?;
                    let z = parse_auto_axis_pos(pos_parts[2])?;

                    style.position = (x, y, z);
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
    pub size: Dim3<Length>,
    pub pos: Dim3<Length>,
}

impl Default for ComputedStyle {
    fn default() -> Self {
        Self {
            size: Dim3::default(),
            pos: Dim3::default(),
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

/// 解析Auto<AxisPos>值
fn parse_auto_axis_pos(s: &str) -> Result<Auto<AxisPos>> {
    if s.to_lowercase() == "auto" {
        Ok(Auto::Auto)
    } else {
        match AxisPos::from_str(s) {
            Ok(pos) => Ok(Auto::Value(pos)),
            Err(e) => Err(anyhow!("Invalid position value '{}': {}", s, e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_parsing() {
        assert_eq!(Display::from_str("block").unwrap(), Display::Block);
        assert_eq!(Display::from_str("stack").unwrap(), Display::Stack);
        assert!(Display::from_str("invalid").is_err());
    }

    #[test]
    fn test_justify_content_parsing() {
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
    fn test_flex_direction_parsing() {
        assert_eq!(StackDirection::from_str("x").unwrap(), StackDirection::X);
        assert_eq!(StackDirection::from_str("y").unwrap(), StackDirection::Y);
        assert_eq!(StackDirection::from_str("z").unwrap(), StackDirection::Z);
        assert!(StackDirection::from_str("invalid").is_err());
    }

    #[test]
    fn test_axis_pos_parsing() {
        assert_eq!(AxisPos::from_str("min").unwrap(), AxisPos::Min);
        assert_eq!(AxisPos::from_str("max").unwrap(), AxisPos::Max);
        assert_eq!(AxisPos::from_str("random").unwrap(), AxisPos::Random);
        assert_eq!(
            AxisPos::from_str("10cm").unwrap(),
            AxisPos::Length(Length::from_cm(10))
        );
        assert!(AxisPos::from_str("invalid").is_err());
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
        assert_eq!(style.display, Display::Stack);

        // 验证justify-content
        assert_eq!(style.justify_content, Some(JustifyContent::FlexEnd));

        // 验证flex-direction
        assert_eq!(style.stack_direction, Some(StackDirection::X));

        // 验证position
        assert_eq!(style.position_x(), &Auto::Value(AxisPos::Min));
        assert_eq!(style.position_y(), &Auto::Value(AxisPos::Max));
        assert_eq!(
            style.position_z(),
            &Auto::Value(AxisPos::Length(Length::from_cm(10)))
        );
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
        assert_eq!(style.stack_direction, None);
        // 验证position默认值为auto
        assert_eq!(style.position_x(), &Auto::Auto);
        assert_eq!(style.position_y(), &Auto::Auto);
        assert_eq!(style.position_z(), &Auto::Auto);
    }

    #[test]
    fn test_invalid_style_parsing() {
        // 测试无效的样式字符串
        assert!(Style::from_style_string("invalid").is_err());
        assert!(Style::from_style_string("size:10m 5m").is_err()); // 少于3个值
        assert!(Style::from_style_string("display:invalid").is_err());
    }
}