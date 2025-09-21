use crate::base::{Length, Percentage};
use crate::dim3::Dim3;
use anyhow::{Result, anyhow};
use rand::Rng;
use std::fmt;
use std::str::FromStr;

mod flex;
pub use flex::{AlignItem, AlignItems, FlexBasis, FlexDirection, JustifyContent};

/// Enum for size values, supporting Length, Percentage, and Auto.
#[derive(Debug, Clone, PartialEq)]
pub enum SizeValue {
    Length(Length),
    Percentage(Percentage),
    Auto,
}

impl Default for SizeValue {
    fn default() -> Self {
        SizeValue::Auto
    }
}

impl FromStr for SizeValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();
        if s == "auto" {
            return Ok(SizeValue::Auto);
        }
        if s.ends_with('%') {
            return Ok(SizeValue::Percentage(Percentage::from_str(&s)?));
        }
        Ok(SizeValue::Length(Length::from_str(&s)?))
    }
}

impl From<FlexBasis> for SizeValue {
    fn from(flex_basis: FlexBasis) -> Self {
        match flex_basis {
            FlexBasis::Length(length) => SizeValue::Length(length),
            FlexBasis::Percentage(percentage) => SizeValue::Percentage(percentage),
            FlexBasis::Auto => SizeValue::Auto,
        }
    }
}

impl From<Length> for SizeValue {
    fn from(length: Length) -> Self {
        Self::Length(length)
    }
}

/// 为 SizeValue 实现 Display trait
impl fmt::Display for SizeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SizeValue::Length(l) => write!(f, "{l}"),
            SizeValue::Percentage(p) => write!(f, "{p}"),
            SizeValue::Auto => write!(f, "auto"),
        }
    }
}

/// Enum for position values, supporting Length and Auto.
#[derive(Debug, Clone, PartialEq)]
pub enum PositionValue {
    Length(Length),
    Auto,
}

impl Default for PositionValue {
    fn default() -> Self {
        PositionValue::Auto
    }
}

impl FromStr for PositionValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.trim().to_lowercase();
        if s == "auto" {
            return Ok(PositionValue::Auto);
        }
        Ok(PositionValue::Length(Length::from_str(&s)?))
    }
}

impl From<Length> for PositionValue {
    fn from(length: Length) -> Self {
        Self::Length(length)
    }
}

/// 为 PositionValue 实现 Display trait
impl fmt::Display for PositionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionValue::Length(l) => write!(f, "{l}"),
            PositionValue::Auto => write!(f, "auto"),
        }
    }
}

impl SizeValue {
    // length > percentage > auto
    pub fn assign_priority(&mut self, other: Self) {
        if matches!(other, Self::Length(_)) {
            *self = other;
        } else if matches!(other, Self::Percentage(_)) && !matches!(self, Self::Length(_)) {
            *self = other;
        }
    }

    pub fn add(&mut self, other: &Self) {
        if let Self::Length(self_length) = self {
            match other {
                SizeValue::Length(other_length) => *self_length += *other_length,
                SizeValue::Percentage(percentage) => {}
                SizeValue::Auto => {}
            }
        }
    }

    pub fn max(&mut self, other: &Self) {
        if let Self::Length(self_length) = self {
            match other {
                SizeValue::Length(other_length) => *self_length = (*self_length).max(*other_length),
                SizeValue::Percentage(percentage) => {}
                SizeValue::Auto => {}
            }
        }
    }
    pub fn is_length(&self) -> bool {
        matches!(self, SizeValue::Length(_))
    }
}

/// SpaceSize结构体，用于表示三个维度的尺寸值
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceSize {
    pub x: SizeValue,
    pub y: SizeValue,
    pub z: SizeValue,
}

/// 为 SpaceSize 实现 Display trait
impl fmt::Display for SpaceSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

impl Default for SpaceSize {
    fn default() -> Self {
        Self {
            x: SizeValue::Auto,
            y: SizeValue::Auto,
            z: SizeValue::Auto,
        }
    }
}

impl SpaceSize {
    pub fn new(x: SizeValue, y: SizeValue, z: SizeValue) -> Self {
        Self { x, y, z }
    }

    pub fn from_dim3_length(dim3: Dim3<Length>) -> Self {
        Self {
            x: SizeValue::Length(dim3.x),
            y: SizeValue::Length(dim3.y),
            z: SizeValue::Length(dim3.z),
        }
    }

    pub fn zero() -> Self {
        Self {
            x: SizeValue::Length(Length::from_mm(0)),
            y: SizeValue::Length(Length::from_mm(0)),
            z: SizeValue::Length(Length::from_mm(0)),
        }
    }

    pub fn assign_priority(&mut self, other: Self) {
        self.x.assign_priority(other.x);
        self.y.assign_priority(other.y);
        self.z.assign_priority(other.z);
    }
    
    /// 判断SpaceSize的任意一个维度是否为Auto
    pub fn has_auto(&self) -> bool {
        self.x == SizeValue::Auto || self.y == SizeValue::Auto || self.z == SizeValue::Auto
    }

    pub fn all_length(&self) -> bool {
        self.x.is_length() && self.y.is_length() && self.z.is_length()
    }

    pub fn create_self_by_auto_to_zero(&self) -> Self {
        let x = if SizeValue::Auto == self.x {
            SizeValue::Length(Length::from_cm(0))
        } else {
            self.x.clone()
        };
        let y = if SizeValue::Auto == self.y {
            SizeValue::Length(Length::from_cm(0))
        } else {
            self.y.clone()
        };
        let z = if SizeValue::Auto == self.z {
            SizeValue::Length(Length::from_cm(0))
        } else {
            self.z.clone()
        };
        return Self { x: x, y: y, z: z };
    }
    
    /// 将SpaceSize转换为Dim3<Length>，如果所有维度都是Length类型
    pub fn get_length(&self) -> Option<Dim3<Length>> {
        match (&self.x, &self.y, &self.z) {
            (SizeValue::Length(x), SizeValue::Length(y), SizeValue::Length(z)) => {
                Some(Dim3::new(*x, *y, *z))
            }
            _ => None,
        }
    }
}

/// Display属性枚举
#[derive(Debug, Clone, PartialEq)]
pub enum Display {
    Flex,
    Cube,
}

impl FromStr for Display {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_lowercase().as_str() {
            "flex" => Ok(Display::Flex),
            "cube" => Ok(Display::Cube),
            _ => Err(anyhow!("Invalid display value: {}", s)),
        }
    }
}

/// 为 Display 实现 Display trait
impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Display::Flex => write!(f, "flex"),
            Display::Cube => write!(f, "cube"),
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

/// 为 AxisPos 实现 Display trait
impl fmt::Display for AxisPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AxisPos::Min => write!(f, "min"),
            AxisPos::Max => write!(f, "max"),
            AxisPos::Random => write!(f, "random"),
            AxisPos::Length(l) => write!(f, "{l}"),
        }
    }
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

/// SpacePosition结构体，用于表示三个维度的位置值
#[derive(Debug, Clone, PartialEq)]
pub struct SpacePosition {
    pub x: PositionValue,
    pub y: PositionValue,
    pub z: PositionValue,
}

impl Default for SpacePosition {
    fn default() -> Self {
        Self {
            x: PositionValue::Auto,
            y: PositionValue::Auto,
            z: PositionValue::Auto,
        }
    }
}

/// 为 SpacePosition 实现 Display trait
impl fmt::Display for SpacePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.x, self.y, self.z)
    }
}

/// Style结构体，包含所有支持的样式属性
#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub size: SpaceSize,                 // size: 三个维度的尺寸 (x, y, z)
    pub display: Display,                // display: block 或 flex
    pub justify_content: JustifyContent, // justify-content: 对齐方式
    pub align_items: AlignItems,         // align-items: 交叉轴对齐方式
    pub flex_direction: FlexDirection,   // flex-direction: x, y, z
    pub position: SpacePosition,         // pos: 三个轴的定位
    pub flex_basis: FlexBasis,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            size: SpaceSize::default(),
            display: Display::Flex,                    // default flex
            justify_content: JustifyContent::default(), // default
            align_items: AlignItems::default(),         // default
            flex_direction: FlexDirection::default(),   // default ReverseZ
            position: SpacePosition::default(),         // 默认位置为auto
            flex_basis: FlexBasis::default(),
        }
    }
}

impl Style {
    /// 创建新的Style实例
    pub fn new() -> Self {
        Style::default()
    }

    /// 获取x维度的尺寸
    pub fn size_x(&self) -> &SizeValue {
        &self.size.x
    }

    /// 获取y维度的尺寸
    pub fn size_y(&self) -> &SizeValue {
        &self.size.y
    }

    /// 获取z维度的尺寸
    pub fn size_z(&self) -> &SizeValue {
        &self.size.z
    }

    /// 获取x维度的位置
    pub fn position_x(&self) -> &PositionValue {
        &self.position.x
    }

    /// 获取y维度的位置
    pub fn position_y(&self) -> &PositionValue {
        &self.position.y
    }

    /// 获取z维度的位置
    pub fn position_z(&self) -> &PositionValue {
        &self.position.z
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
                    // 解析尺寸，格式如 "10m 50% auto"
                    let size_parts: Vec<&str> = value.split_whitespace().collect();
                    if size_parts.len() != 3 {
                        return Err(anyhow!("Size must have exactly 3 values (x, y, z)"));
                    }

                    let x = SizeValue::from_str(size_parts[0])?;
                    let y = SizeValue::from_str(size_parts[1])?;
                    let z = SizeValue::from_str(size_parts[2])?;

                    style.size = SpaceSize::new(x, y, z);
                }
                "display" => {
                    style.display = Display::from_str(value)?;
                }
                "justify-content" => {
                    style.justify_content = JustifyContent::from_str(value)?;
                }
                "align-items" => {
                    style.align_items = AlignItems::from_str(value)?;
                }
                "flex-direction" => {
                    style.flex_direction = FlexDirection::from_str(value)?;
                }
                "pos" => {
                    // 解析位置，格式如 "min max 10cm" 或 "auto auto auto"
                    let pos_parts: Vec<&str> = value.split_whitespace().collect();
                    if pos_parts.len() != 3 {
                        return Err(anyhow!("Position must have exactly 3 values (x, y, z)"));
                    }

                    let x = PositionValue::from_str(pos_parts[0])?;
                    let y = PositionValue::from_str(pos_parts[1])?;
                    let z = PositionValue::from_str(pos_parts[2])?;

                    style.position = SpacePosition { x, y, z };
                }
                "flex-basis" => {
                    style.flex_basis = FlexBasis::from_str(value)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Percentage;

    #[test]
    fn test_display_parsing() {
        assert_eq!(Display::from_str("cube").unwrap(), Display::Cube);
        assert_eq!(Display::from_str("flex").unwrap(), Display::Flex);
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
    fn test_align_items_parsing() {
        // 测试align-items属性的解析
        let style_str = "align-items:flex-start center";
        let style = Style::from_style_string(style_str).unwrap();
        
        assert_eq!(style.align_items.cross1, AlignItem::FlexStart);
        assert_eq!(style.align_items.cross2, AlignItem::Center);
        
        // 测试默认值
        let style_str = "display:flex";
        let style = Style::from_style_string(style_str).unwrap();
        
        assert_eq!(style.align_items.cross1, AlignItem::FlexStart);
        assert_eq!(style.align_items.cross2, AlignItem::FlexStart);
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
        assert_eq!(
            AxisPos::from_str("10cm").unwrap(),
            AxisPos::Length(Length::from_cm(10))
        );
        assert!(AxisPos::from_str("invalid").is_err());
    }

    #[test]
    fn test_flex_basis_to_size_value_conversion() {
        // Test Length conversion
        let flex_basis_length = FlexBasis::Length(Length::from_mm(100));
        let size_value: SizeValue = flex_basis_length.into();
        assert_eq!(size_value, SizeValue::Length(Length::from_mm(100)));

        // Test Percentage conversion
        let flex_basis_percentage = FlexBasis::Percentage(Percentage::new(50));
        let size_value: SizeValue = flex_basis_percentage.into();
        assert_eq!(size_value, SizeValue::Percentage(Percentage::new(50)));

        // Test Auto conversion
        let flex_basis_auto = FlexBasis::Auto;
        let size_value: SizeValue = flex_basis_auto.into();
        assert_eq!(size_value, SizeValue::Auto);
    }

    #[test]
    fn test_style_parsing() {
        // 测试完整的样式字符串解析
        let style_str = "size:10m 50% auto;display:flex;justify-content:flex-end;align-items:flex-start center;flex-direction:x;pos:10cm 20cm 30cm;flex-basis:50%";
        let style = Style::from_style_string(style_str).unwrap();

        // 验证size
        assert_eq!(style.size_x(), &SizeValue::Length(Length::from_m(10)));
        assert_eq!(style.size_y(), &SizeValue::Percentage(Percentage::new(50)));
        assert_eq!(style.size_z(), &SizeValue::Auto);

        // 验证display
        assert_eq!(style.display, Display::Flex);

        // 验证justify-content
        assert_eq!(style.justify_content, JustifyContent::FlexEnd);

        // 验证align-items
        assert_eq!(style.align_items.cross1, AlignItem::FlexStart);
        assert_eq!(style.align_items.cross2, AlignItem::Center);

        // 验证flex-direction
        assert_eq!(style.flex_direction, FlexDirection::X);

        // 验证position
        assert_eq!(style.position_x(), &PositionValue::Length(Length::from_cm(10)));
        assert_eq!(style.position_y(), &PositionValue::Length(Length::from_cm(20)));
        assert_eq!(style.position_z(), &PositionValue::Length(Length::from_cm(30)));

        // 验证flex-basis
        assert_eq!(style.flex_basis, FlexBasis::Percentage(Percentage::new(50)));
    }

    #[test]
    fn test_style_parsing_with_defaults() {
        // 测试只包含部分属性的样式字符串
        let style_str = "display:flex";
        let style = Style::from_style_string(style_str).unwrap();

        // 验证默认值
        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.size_x(), &SizeValue::Auto);
        assert_eq!(style.size_y(), &SizeValue::Auto);
        assert_eq!(style.size_z(), &SizeValue::Auto);
        assert_eq!(style.justify_content, JustifyContent::FlexStart);
        assert_eq!(style.align_items.cross1, AlignItem::FlexStart);
        assert_eq!(style.align_items.cross2, AlignItem::FlexStart);
        assert_eq!(style.flex_direction, FlexDirection::default());
        // 验证position默认值为auto
        assert_eq!(style.position_x(), &PositionValue::Auto);
        assert_eq!(style.position_y(), &PositionValue::Auto);
        assert_eq!(style.position_z(), &PositionValue::Auto);
        assert_eq!(style.flex_basis, FlexBasis::Auto);
    }

    #[test]
    fn test_space_position_parsing() {
        // 测试位置值的解析
        let style_str = "pos:10cm 20cm auto";
        let style = Style::from_style_string(style_str).unwrap();

        // 验证position值
        assert_eq!(style.position_x(), &PositionValue::Length(Length::from_cm(10)));
        assert_eq!(style.position_y(), &PositionValue::Length(Length::from_cm(20)));
        assert_eq!(style.position_z(), &PositionValue::Auto);
    }

    #[test]
    fn test_position_value_parsing() {
        // 测试PositionValue的解析
        assert_eq!(PositionValue::from_str("auto").unwrap(), PositionValue::Auto);
        assert_eq!(PositionValue::from_str("10cm").unwrap(), PositionValue::Length(Length::from_cm(10)));
        assert_eq!(PositionValue::from_str("5m").unwrap(), PositionValue::Length(Length::from_m(5)));
        assert_eq!(PositionValue::from_str("20mm").unwrap(), PositionValue::Length(Length::from_mm(20)));
    }

    #[test]
    fn test_position_value_display() {
        // 测试PositionValue的显示
        assert_eq!(format!("{}", PositionValue::Auto), "auto");
        assert_eq!(format!("{}", PositionValue::Length(Length::from_cm(10))), "10cm");
        assert_eq!(format!("{}", PositionValue::Length(Length::from_m(5))), "5m");
    }

    #[test]
    fn test_space_position_default() {
        // 测试SpacePosition的默认值
        let position = SpacePosition::default();
        assert_eq!(position.x, PositionValue::Auto);
        assert_eq!(position.y, PositionValue::Auto);
        assert_eq!(position.z, PositionValue::Auto);
    }

    #[test]
    fn test_space_position_display() {
        // 测试SpacePosition的显示
        let position = SpacePosition {
            x: PositionValue::Length(Length::from_cm(10)),
            y: PositionValue::Length(Length::from_m(5)),
            z: PositionValue::Auto,
        };
        assert_eq!(format!("{}", position), "10cm 5m auto");
    }

    #[test]
    fn test_invalid_style_parsing() {
        // 测试无效的样式字符串
        assert!(Style::from_style_string("invalid").is_err());
        assert!(Style::from_style_string("size:10m 5m").is_err()); // 少于3个值
        assert!(Style::from_style_string("display:invalid").is_err());
    }

    #[test]
    fn test_space_size_from_dim3_length() {
        let dim3 = Dim3::new(Length::from_m(10), Length::from_cm(20), Length::from_mm(30));
        let space_size = SpaceSize::from_dim3_length(dim3);

        assert_eq!(space_size.x, SizeValue::Length(Length::from_m(10)));
        assert_eq!(space_size.y, SizeValue::Length(Length::from_cm(20)));
        assert_eq!(space_size.z, SizeValue::Length(Length::from_mm(30)));
    }
}
