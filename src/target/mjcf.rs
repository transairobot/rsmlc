use crate::base::Length;
use crate::dim3::Dim3;
use crate::package::GeomType as PackageGeomType;
use crate::render_tree::{RenderNode, RenderNodeType, RenderTree};
use std::cell::RefCell;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use crate::error::Result;

pub struct MjcfGenerator;

use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "mujoco")]
pub struct Mujoco {
    #[serde(rename = "@model")] // XML 属性
    pub model: String,

    #[serde(rename = "worldbody")]
    pub worldbody: WorldBody,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct WorldBody {
    #[serde(rename = "geom")]
    pub geoms: Vec<Geom>,

    #[serde(rename = "light")]
    pub lights: Vec<Light>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum GeomType {
    #[serde(rename = "box")]
    Box, // 长宽高
    #[serde(rename = "mesh")]
    Mesh,
}

impl From<PackageGeomType> for GeomType {
    fn from(geom_type: PackageGeomType) -> Self {
        match geom_type {
            PackageGeomType::Box => GeomType::Box,
            PackageGeomType::Mesh => GeomType::Mesh,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Geom {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@size")]
    pub size: String, // 例如 "1 1 .05"

    #[serde(rename = "@pos")]
    pub pos: String, // 例如 "1 1 .05"

    #[serde(rename = "@type")]
    pub geom_type: GeomType, // 例如 "box"
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Light {
    #[serde(rename = "@name")]
    pub name: String,

    #[serde(rename = "@pos")]
    pub pos: String, // 例如 "0 0 2"

    #[serde(rename = "@mode")]
    pub mode: String, // 例如 "trackcom"
}

impl MjcfGenerator {
    pub fn generate(render_tree: &RenderTree) -> Mujoco {
        let mut geoms = Vec::new();
        
        // 遍历渲染树，收集所有的Item节点
        Self::collect_item_geoms(&render_tree.root, &mut geoms);
        
        // 添加默认光源
        let lights = vec![
            Light {
                name: "default_light".to_string(),
                pos: "0 0 2".to_string(),
                mode: "trackcom".to_string(),
            }
        ];
        
        Mujoco {
            model: "rsml_model".to_string(),
            worldbody: WorldBody {
                geoms,
                lights,
            },
        }
    }

    /// Generate MuJoCo XML and save it to the specified directory
    pub fn generate_to_directory(render_tree: &RenderTree, directory: &str) -> Result<()> {
        // Generate the Mujoco struct
        let mujoco = Self::generate(render_tree);

        // Convert Mujoco struct to XML string
        let mut buf = Vec::new();
        let mut writer = quick_xml::Writer::new(&mut buf);
        
        // Write the XML declaration
        writer.write_event(quick_xml::events::Event::Decl(
            quick_xml::events::BytesDecl::new("1.0", Some("utf-8"), None)
        ))?;
        
        // Serialize the struct to XML
        let content = quick_xml::se::to_string(&mujoco)?;
                buf.extend_from_slice(content.as_bytes());

        let xml_string = String::from_utf8(buf)?;

        // Ensure the directory exists
        fs::create_dir_all(directory)?;

        // Write the XML to a file in the specified directory
        let file_path = Path::new(directory).join("model.xml");
        let mut file = File::create(file_path)?;
        file.write_all(xml_string.as_bytes())?;

        Ok(())
    }
    
    fn collect_item_geoms(node: &Rc<RefCell<RenderNode>>, geoms: &mut Vec<Geom>) {
        let node_ref = node.borrow();
        
        // 如果是Item类型的节点，转换为Geom
        if node_ref.node_type == RenderNodeType::Item {
            if let Some(object) = &node_ref.computed_style.object {
                // 生成Geom名称
                let name = if let Some(id) = &node_ref.id {
                    id.clone()
                } else {
                    format!("{}_{}", node_ref.tag_name, geoms.len())
                };
                
                // 获取位置信息
                let pos = if let Some(position) = node_ref.computed_style.position.get_length() {
                    format!("{} {} {}", 
                        Self::length_to_meters(position.x), 
                        Self::length_to_meters(position.y), 
                        Self::length_to_meters(position.z)
                    )
                } else {
                    "0 0 0".to_string()
                };
                
                // 获取尺寸信息
                let size = format!("{} {} {}", 
                    Self::length_to_meters(object.size.x), 
                    Self::length_to_meters(object.size.y), 
                    Self::length_to_meters(object.size.z)
                );
                
                // 创建Geom
                let geom = Geom {
                    name,
                    size,
                    pos,
                    geom_type: object.geom_type.clone().into(),
                };
                
                geoms.push(geom);
            }
        }
        
        // 递归处理子节点
        for child in &node_ref.children {
            Self::collect_item_geoms(child, geoms);
        }
    }
    
    /// 将Length转换为米为单位的浮点数，并格式化为字符串
    fn length_to_meters(length: Length) -> String {
        let meters = length.mm() as f64 / 1000.0;
        // 保留6位小数，去除尾随零
        format!("{:.6}", meters).trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Length;
    use crate::dim3::Dim3;
    use crate::package::{GeomType as PackageGeomType, Object};
    use crate::render_tree::{RenderNode, RenderNodeType};
    use crate::style::{ComputedStyle, SpacePosition, SpaceSize};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_length_to_meters() {
        let generator = MjcfGenerator;
        
        // Test 1 meter
        let length = Length::from_m(1.0);
        assert_eq!(MjcfGenerator::length_to_meters(length), "1");
        
        // Test 10 centimeters
        let length = Length::from_cm(10);
        assert_eq!(MjcfGenerator::length_to_meters(length), "0.1");
        
        // Test 5 millimeters
        let length = Length::from_mm(5);
        assert_eq!(MjcfGenerator::length_to_meters(length), "0.005");
    }
    
    #[test]
    fn test_geom_type_conversion() {
        let box_type: GeomType = PackageGeomType::Box.into();
        assert_eq!(box_type, GeomType::Box);
        
        let mesh_type: GeomType = PackageGeomType::Mesh.into();
        assert_eq!(mesh_type, GeomType::Mesh);
    }
    
    #[test]
    fn test_generate_empty_tree() {
        // 创建一个简单的渲染节点
        let node = Rc::new(RefCell::new(RenderNode::new("test".to_string(), RenderNodeType::Space)));
        
        // 创建一个空的package（这里只是测试，实际不会用到）
        // 我们直接测试collect_item_geoms方法
        
        let mut geoms = Vec::new();
        MjcfGenerator::collect_item_geoms(&node, &mut geoms);
        
        // 应该没有geom，因为根节点是Space类型
        assert_eq!(geoms.len(), 0);
    }

    #[test]
    fn test_generate_to_directory() {
        use std::fs;
        use tempfile::tempdir; // You might need to add tempfile as dev dependency
        use crate::xml_parser::Element as DomElement;
        use crate::package::Package;

        // Create a minimal package for testing
        let test_package = Package {
            package: crate::package::PackageInfo {
                name: "test_package".to_string(),
                description: "Test package".to_string(),
            },
            objects: std::collections::HashMap::new(),
            groups: Vec::new(),
            dependencies: std::collections::HashMap::new(),
        };

        // Create a simple DOM element
        let dom_element = DomElement::new("space".to_string());
        
        // Create a render tree from the DOM element and package
        let render_tree = RenderTree::new(&dom_element, &test_package).unwrap();

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Test the generate_to_directory function
        let result = MjcfGenerator::generate_to_directory(&render_tree, temp_path);
        assert!(result.is_ok());

        // Check if the file was created
        let xml_path = Path::new(temp_path).join("model.xml");
        assert!(xml_path.exists());

        // Read and check if the file contains XML content
        let content = fs::read_to_string(xml_path).unwrap();
        println!("Generated XML content: {}", content); // Debug output
        assert!(content.contains("rsml_model"));
    }
}

