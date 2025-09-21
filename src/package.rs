use crate::dim3::Dim3;
use crate::error::{RsmlError, Result};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fs;
use std::str::FromStr;

use crate::base::Length;

/// Custom deserializer for Dim3<Length> from string format "x y z"
fn deserialize_dim3_length<'de, D>(deserializer: D) -> anyhow::Result<Dim3<Length>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let parts: Vec<&str> = s.split_whitespace().collect();
    
    if parts.len() != 3 {
        return Err(serde::de::Error::custom(format!(
            "Expected 3 dimensions, got {}: {}",
            parts.len(),
            s
        )));
    }

    let x = parts[0]
        .parse::<Length>()
        .map_err(serde::de::Error::custom)?;
    let y = parts[1]
        .parse::<Length>()
        .map_err(serde::de::Error::custom)?;
    let z = parts[2]
        .parse::<Length>()
        .map_err(serde::de::Error::custom)?;

    Ok(Dim3::new(x, y, z))
}

/// 依赖项配置
#[derive(Debug, Deserialize, Clone)]
pub struct Dependency {
    #[serde(rename = "size-limit", deserialize_with = "deserialize_dim3_length")]
    pub size_limit: Dim3<Length>,
}

impl Dependency {
    /// 获取依赖项的尺寸限制
    pub fn size_limit(&self) -> &Dim3<Length> {
        &self.size_limit
    }
}

/// 对象配置
#[derive(Debug, Deserialize, Clone)]
pub struct Object {
    #[serde(rename = "geom-type")]
    pub geom_type: String,
    pub size: String,
}

impl Object {
    /// 获取对象的几何类型
    pub fn geom_type(&self) -> &str {
        &self.geom_type
    }

    /// 获取对象的尺寸
    pub fn size(&self) -> &str {
        &self.size
    }

    /// 解析对象的空间尺寸
    pub fn space_size(&self) -> Result<Dim3<Length>> {
        let parts: Vec<&str> = self.size.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(RsmlError::ParseError {
                field: "Object size".to_string(),
                message: format!("Invalid size format: {}", self.size)
            });
        }

        let x = Length::from_str(parts[0])?;
        let y = Length::from_str(parts[1])?;
        let z = Length::from_str(parts[2])?;

        Ok(Dim3::new(x, y, z))
    }
}

/// 组配置
#[derive(Debug, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub items: Vec<String>,
}

impl Group {
    /// 获取组的名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取组中的项目列表
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// 解析组的空间尺寸，计算组内所有对象的最大包围盒
    pub fn space_size(&self, package: &Package) -> Result<Dim3<Length>> {
        let mut max_x = Length::from_mm(0);
        let mut max_y = Length::from_mm(0);
        let mut max_z = Length::from_mm(0);

        // 遍历组内的所有项目
        for item_name in &self.items {
            // 尝试查找对象或组
            let size = if let Some(object) = package.get_object(item_name) {
                // If it's an object, get its space size
                object.space_size()?
            } else if let Some(dependency) = package.get_dependency(item_name) {
                // If it's a dependency, get its space size directly
                *dependency.size_limit()
            } else {
                // If it's neither an object nor a dependency, return an error
                return Err(RsmlError::InvalidStructure { 
                    message: format!("Item '{}' not found in package", item_name) 
                });
            };
            
            max_x = if size.x > max_x { size.x } else { max_x };
            max_y = if size.y > max_y { size.y } else { max_y };
            max_z = if size.z > max_z { size.z } else { max_z };
        }

        Ok(Dim3::new(max_x, max_y, max_z))
    }
}

/// Package配置
#[derive(Debug, Deserialize, Clone)]
pub struct Package {
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(default)]
    pub objects: HashMap<String, Object>,
    #[serde(default)]
    pub groups: Vec<Group>,
}

impl Package {
    /// 从文件解析Package配置
    pub fn from_file(file_path: &str) -> Result<Self> {
        let contents = fs::read_to_string(file_path)?;
        toml::from_str(&contents).map_err(RsmlError::from)
    }

    /// 获取所有依赖项
    pub fn dependencies(&self) -> &HashMap<String, Dependency> {
        &self.dependencies
    }

    /// 获取依赖项
    pub fn get_dependency(&self, name: &str) -> Option<&Dependency> {
        self.dependencies.get(name)
    }

    /// 获取所有对象
    pub fn objects(&self) -> &HashMap<String, Object> {
        &self.objects
    }

    /// 获取对象
    pub fn get_object(&self, name: &str) -> Option<&Object> {
        self.objects.get(name)
    }

    /// 获取所有组
    pub fn groups(&self) -> &[Group] {
        &self.groups
    }

    /// 获取组
    pub fn get_group(&self, name: &str) -> Option<&Group> {
        self.groups.iter().find(|group| group.name == name)
    }

    /// 获取对象或组的空间尺寸
    pub fn get_space_size(&self, name: &str) -> Result<Dim3<Length>> {
        if let Some(object) = self.get_object(name) {
            return object.space_size();
        }
        if let Some(group) = self.get_group(name) {
            return group.space_size(self);
        }
        Err(RsmlError::InvalidStructure {
            message: format!("Object or group with name '{}' not found in package", name)
        })
    }

    /// 检查是否存在指定的依赖项
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.contains_key(name)
    }

    /// 检查是否存在指定的对象
    pub fn has_object(&self, name: &str) -> bool {
        self.objects.contains_key(name)
    }

    /// 检查是否存在指定的组
    pub fn has_group(&self, name: &str) -> bool {
        self.groups.iter().any(|group| group.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_parsing() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");

        // 测试依赖项解析
        let bottle = package.get_dependency("bottle");
        assert!(bottle.is_some());
        let bottle_size = bottle.unwrap().size_limit();
        assert_eq!(bottle_size.x, Length::from_cm(10));
        assert_eq!(bottle_size.y, Length::from_cm(10));
        assert_eq!(bottle_size.z, Length::from_cm(10));

        // 测试对象解析
        let table_plane = package.get_object("table_plane");
        assert!(table_plane.is_some());
        let table_plane = table_plane.unwrap();
        assert_eq!(table_plane.geom_type(), "box");
        assert_eq!(table_plane.size(), "1m 1m 10cm");

        // 测试组解析
        let bottles_group = package.get_group("bottles");
        assert!(bottles_group.is_some());
        let bottles_group = bottles_group.unwrap();
        assert_eq!(bottles_group.name(), "bottles");
        assert_eq!(bottles_group.items(), &["box_bottle", "bottle"]);
    }

    #[test]
    fn test_package_collections() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");

        // 测试获取所有依赖项
        let dependencies = package.dependencies();
        assert_eq!(dependencies.len(), 1);
        assert!(dependencies.contains_key("bottle"));

        // 测试获取所有对象
        let objects = package.objects();
        assert_eq!(objects.len(), 4);
        assert!(objects.contains_key("table_plane"));
        assert!(objects.contains_key("table_leg"));
        assert!(objects.contains_key("floor"));
        assert!(objects.contains_key("box_bottle"));

        // 测试获取所有组
        let groups = package.groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].name(), "bottles");
    }

    #[test]
    fn test_package_contains_checks() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");

        // 测试存在性检查
        assert!(package.has_dependency("bottle"));
        assert!(!package.has_dependency("nonexistent"));

        assert!(package.has_object("table_plane"));
        assert!(!package.has_object("nonexistent"));

        assert!(package.has_group("bottles"));
        assert!(!package.has_group("nonexistent"));
    }

    #[test]
    fn test_dependency_methods() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");
        let bottle = package.get_dependency("bottle").unwrap();
        let size = bottle.size_limit();
        assert_eq!(size.x, Length::from_cm(10));
        assert_eq!(size.y, Length::from_cm(10));
        assert_eq!(size.z, Length::from_cm(10));
    }

    #[test]
    fn test_object_methods() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");
        let table_plane = package.get_object("table_plane").unwrap();
        assert_eq!(table_plane.geom_type(), "box");
        assert_eq!(table_plane.size(), "1m 1m 10cm");
    }

    #[test]
    fn test_group_methods() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");
        let bottles_group = package.get_group("bottles").unwrap();
        assert_eq!(bottles_group.name(), "bottles");
        assert_eq!(bottles_group.items(), &["box_bottle", "bottle"]);
    }

    #[test]
    fn test_get_space_size() {
        let package = Package::from_file("package.toml").expect("Failed to parse package.toml");

        // Test getting size of an object
        let table_plane_size = package.get_space_size("table_plane").unwrap();
        assert_eq!(table_plane_size.x, Length::from_m(1));
        assert_eq!(table_plane_size.y, Length::from_m(1));
        assert_eq!(table_plane_size.z, Length::from_cm(10));

        // Test getting size of a group that contains a dependency
        let bottles_group_size = package.get_space_size("bottles").unwrap();
        assert_eq!(bottles_group_size.x, Length::from_cm(10));
        assert_eq!(bottles_group_size.y, Length::from_cm(10));
        assert_eq!(bottles_group_size.z, Length::from_cm(10));

        // Test not found
        let result = package.get_space_size("nonexistent");
        assert!(result.is_err());
    }
}
