use std::collections::HashMap;
use crate::style::Style;
use anyhow::{Result, anyhow};

/// SelectorStyle结构体，包含一个选择器和样式映射
#[derive(Debug, Clone, PartialEq)]
pub struct SelectorStyle {
    /// 选择器字符串
    pub selector: String,
    /// 样式映射
    pub style_map: HashMap<String, String>,
    /// ID选择器列表 (以 # 开头的选择器，不包含 #)
    pub id_selectors: Vec<String>,
    /// Class选择器列表 (以 . 开头的选择器，不包含 .)
    pub class_selectors: Vec<String>,
}

impl SelectorStyle {
    /// 创建新的SelectorStyle实例
    pub fn new(selector: String, style_map: HashMap<String, String>) -> Self {
        let (id_selectors, class_selectors) = Self::parse_selector_string(&selector);
        SelectorStyle {
            selector,
            style_map,
            id_selectors,
            class_selectors,
        }
    }

    /// 创建带有默认空样式映射的SelectorStyle实例
    pub fn with_selector(selector: String) -> Self {
        let (id_selectors, class_selectors) = Self::parse_selector_string(&selector);
        SelectorStyle {
            selector,
            style_map: HashMap::new(),
            id_selectors,
            class_selectors,
        }
    }

    /// 从选择器和样式字符串创建SelectorStyle实例
    pub fn from_selector_and_style_string(selector: String, style_string: &str) -> Self {
        let mut style_map = HashMap::new();
        
        // 解析样式字符串
        for declaration in style_string.split(';') {
            let declaration = declaration.trim();
            if declaration.is_empty() {
                continue;
            }

            let parts: Vec<&str> = declaration.split(':').collect();
            if parts.len() == 2 {
                let property = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                style_map.insert(property, value);
            }
        }

        let (id_selectors, class_selectors) = Self::parse_selector_string(&selector);

        SelectorStyle {
            selector,
            style_map,
            id_selectors,
            class_selectors,
        }
    }

    /// 获取指定属性的值
    pub fn get_style_value(&self, property: &str) -> Option<&String> {
        self.style_map.get(property)
    }

    /// 设置指定属性的值
    pub fn set_style_value(&mut self, property: String, value: String) {
        self.style_map.insert(property, value);
    }

    /// 从样式映射创建Style对象
    pub fn to_style(&self) -> Result<Style> {
        Style::from_style_map(&self.style_map)
    }

    /// 解析选择器字符串，提取ID和class选择器
    fn parse_selector_string(selector_str: &str) -> (Vec<String>, Vec<String>) {
        let mut id_selectors = Vec::new();
        let mut class_selectors = Vec::new();
        
        // 分割逗号分隔的选择器
        for selector in selector_str.split(',') {
            let selector = selector.trim();
            
            if selector.starts_with('#') {
                // ID选择器
                let id = &selector[1..]; // 去除'#'
                id_selectors.push(id.trim().to_string());
            } else if selector.starts_with('.') {
                // Class选择器
                let class = &selector[1..]; // 去除'.'
                class_selectors.push(class.trim().to_string());
            }
            // 其他类型的选择器（如标签选择器）暂时忽略
        }
        
        (id_selectors, class_selectors)
    }

    /// 从TOML字符串解析多个SelectorStyle
    pub fn parse_from_toml(toml_str: &str) -> Result<Vec<SelectorStyle>> {
        let toml_value: toml::Value = toml::from_str(toml_str)
            .map_err(|e| anyhow!("Failed to parse TOML: {}", e))?;

        let styles_array = toml_value
            .get("styles")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("No 'styles' array found in TOML"))?;

        let mut selector_styles = Vec::new();

        for style_value in styles_array {
            let table = style_value
                .as_table()
                .ok_or_else(|| anyhow!("Style entry is not a table"))?;

            let selector = table
                .get("selector")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow!("Style entry missing 'selector' field"))?
                .to_string();

            let mut style_map = HashMap::new();
            for (key, value) in table.iter() {
                if key != "selector" {
                    let value_str = match value {
                        toml::Value::String(s) => s.clone(),
                        toml::Value::Integer(i) => i.to_string(),
                        toml::Value::Float(f) => f.to_string(),
                        toml::Value::Boolean(b) => b.to_string(),
                        _ => continue, // Skip unsupported value types
                    };
                    style_map.insert(key.clone(), value_str);
                }
            }

            selector_styles.push(SelectorStyle {
                selector,
                style_map,
                id_selectors: Vec::new(),
                class_selectors: Vec::new(),
            });
        }

        // Now update the id_selectors and class_selectors for each style
        for style in &mut selector_styles {
            let (id_selectors, class_selectors) = Self::parse_selector_string(&style.selector);
            style.id_selectors = id_selectors;
            style.class_selectors = class_selectors;
        }

        Ok(selector_styles)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selector_style_creation() {
        let mut style_map = HashMap::new();
        style_map.insert("display".to_string(), "flex".to_string());
        style_map.insert("size".to_string(), "10m 10m 10m".to_string());

        let selector_style = SelectorStyle::new("div".to_string(), style_map);
        
        assert_eq!(selector_style.selector, "div");
        assert_eq!(selector_style.id_selectors, Vec::<String>::new());  // No ID selectors
        assert_eq!(selector_style.class_selectors, Vec::<String>::new());  // No class selectors
        assert_eq!(selector_style.get_style_value("display"), Some(&"flex".to_string()));
        assert_eq!(selector_style.get_style_value("size"), Some(&"10m 10m 10m".to_string()));
    }

    #[test]
    fn test_selector_style_from_style_string() {
        let selector_style = SelectorStyle::from_selector_and_style_string(
            ".my-class".to_string(),
            "display:flex;justify-content:center;size:10m 10m 10m"
        );
        
        assert_eq!(selector_style.selector, ".my-class");
        assert_eq!(selector_style.id_selectors, Vec::<String>::new());  // No ID selectors
        assert_eq!(selector_style.class_selectors, vec!["my-class".to_string()]);  // One class selector
        assert_eq!(selector_style.get_style_value("display"), Some(&"flex".to_string()));
        assert_eq!(selector_style.get_style_value("justify-content"), Some(&"center".to_string()));
        assert_eq!(selector_style.get_style_value("size"), Some(&"10m 10m 10m".to_string()));
    }

    #[test]
    fn test_set_style_value() {
        let mut selector_style = SelectorStyle::with_selector("#my-id".to_string());
        selector_style.set_style_value("color".to_string(), "red".to_string());
        
        assert_eq!(selector_style.id_selectors, vec!["my-id".to_string()]);
        assert_eq!(selector_style.class_selectors, Vec::<String>::new());
        assert_eq!(selector_style.get_style_value("color"), Some(&"red".to_string()));
    }

    #[test]
    fn test_to_style() {
        let selector_style = SelectorStyle::from_selector_and_style_string(
            "body".to_string(),
            "display:flex;size:10m 10m 10m"
        );
        
        let style = selector_style.to_style().unwrap();
        assert_eq!(style.display.to_string(), "flex");
    }

    #[test]
    fn test_parse_from_toml() {
        let toml_str = r#"
[[styles]]
selector = "leg1"
margin = "10cm 10cm 10cm 10cm 0cm 0cm"

[[styles]]
selector = "leg2"
pos = "min max min"
margin = "10cm 10cm 10cm 10cm 0cm 0cm"

[[styles]]
selector = "leg3"
margin = "10cm 10cm 10cm 10cm 0cm 0cm"

[[styles]]
selector = "leg4"
margin = "10cm 10cm 10cm 10cm 0cm 0cm"
"#;

        let selector_styles = SelectorStyle::parse_from_toml(toml_str).unwrap();
        
        assert_eq!(selector_styles.len(), 4);
        
        assert_eq!(selector_styles[0].selector, "leg1");
        assert_eq!(selector_styles[0].id_selectors, Vec::<String>::new());
        assert_eq!(selector_styles[0].class_selectors, Vec::<String>::new());
        assert_eq!(selector_styles[0].get_style_value("margin"), Some(&"10cm 10cm 10cm 10cm 0cm 0cm".to_string()));
        
        assert_eq!(selector_styles[1].selector, "leg2");
        assert_eq!(selector_styles[1].id_selectors, Vec::<String>::new());
        assert_eq!(selector_styles[1].class_selectors, Vec::<String>::new());
        assert_eq!(selector_styles[1].get_style_value("pos"), Some(&"min max min".to_string()));
        assert_eq!(selector_styles[1].get_style_value("margin"), Some(&"10cm 10cm 10cm 10cm 0cm 0cm".to_string()));
        
        assert_eq!(selector_styles[2].selector, "leg3");
        assert_eq!(selector_styles[3].selector, "leg4");
    }
    
    #[test]
    fn test_parse_selector_string() {
        // Test ID selector
        let (id_selectors, class_selectors) = SelectorStyle::parse_selector_string("#myId");
        assert_eq!(id_selectors, vec!["myId".to_string()]);
        assert_eq!(class_selectors, Vec::<String>::new());
        
        // Test class selector
        let (id_selectors, class_selectors) = SelectorStyle::parse_selector_string(".myClass");
        assert_eq!(id_selectors, Vec::<String>::new());
        assert_eq!(class_selectors, vec!["myClass".to_string()]);
        
        // Test multiple selectors
        let (id_selectors, class_selectors) = SelectorStyle::parse_selector_string("#id1, .class1, .class2");
        assert_eq!(id_selectors, vec!["id1".to_string()]);
        assert_eq!(class_selectors, vec!["class1".to_string(), "class2".to_string()]);
        
        // Test selector with spaces
        let (id_selectors, class_selectors) = SelectorStyle::parse_selector_string("  # spacedId  ,  . spacedClass  ");
        assert_eq!(id_selectors, vec!["spacedId".to_string()]);
        assert_eq!(class_selectors, vec!["spacedClass".to_string()]);
    }
}