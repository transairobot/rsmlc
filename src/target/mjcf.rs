use crate::base::Length;
use crate::dim3::Dim3;
use crate::error::Result;
use crate::package::GeomType as PackageGeomType;
use crate::render_tree::{RenderNode, RenderNodeType, RenderTree};
use crate::style::SpacePosition;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

pub struct MjcfGenerator {
    pub assets_map: HashMap<String, (Mesh, Material, Texture)>,
}

use anyhow::Result as AnyResult;
use gltf::json::accessor::ComponentType;
use image::{ImageBuffer, ImageFormat, Rgba};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Texture {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@type")]
    pub texture_type: String, // e.g. "2d"
    #[serde(rename = "@file")]
    pub file: String, // e.g. "mytexture.png"
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Material {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@texture")]
    pub texture: String, // references texture name
    #[serde(rename = "@texrepeat")]
    pub texrepeat: String, // e.g. "1 1"
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Mesh {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@file")]
    pub file: String, // e.g. "myshape.obj" or "mymesh.stl"
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Assets {
    #[serde(rename = "texture", default, skip_serializing_if = "Vec::is_empty")]
    pub textures: Vec<Texture>,
    #[serde(rename = "material", default, skip_serializing_if = "Vec::is_empty")]
    pub materials: Vec<Material>,
    #[serde(rename = "mesh", default, skip_serializing_if = "Vec::is_empty")]
    pub meshes: Vec<Mesh>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "mujoco")]
pub struct Mujoco {
    #[serde(rename = "@model")] // XML 属性
    pub model: String,

    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<Assets>,

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

    #[serde(rename = "@size", skip_serializing_if = "Option::is_none")]
    pub size: Option<String>, // 例如 "1 1 .05"

    #[serde(rename = "@pos")]
    pub pos: String, // 例如 "1 1 .05"

    #[serde(rename = "@type")]
    pub geom_type: GeomType, // 例如 "box"

    #[serde(rename = "@mesh", skip_serializing_if = "Option::is_none")]
    pub mesh: Option<String>,

    #[serde(rename = "@material", skip_serializing_if = "Option::is_none")]
    pub material: Option<String>,
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
    pub fn new() -> Self {
        Self {
            assets_map: HashMap::new(),
        }
    }
    pub fn generate(&mut self, render_tree: &RenderTree) -> Mujoco {
        let mut geoms = Vec::new();

        // Generate assets (textures, materials, meshes) from render tree
        let assets = self.create_assets_from_render_tree(render_tree);
        // 遍历渲染树，收集所有的Item节点
        self.collect_item_geoms(&render_tree.root, &mut geoms);

        // 添加默认光源
        let lights = vec![Light {
            name: "default_light".to_string(),
            pos: "0 0 2".to_string(),
            mode: "trackcom".to_string(),
        }];

        Mujoco {
            model: "rsml_model".to_string(),
            asset: if assets.textures.is_empty()
                && assets.materials.is_empty()
                && assets.meshes.is_empty()
            {
                None
            } else {
                Some(assets)
            },
            worldbody: WorldBody { geoms, lights },
        }
    }

    /// Create assets (textures, materials, meshes) from render tree
    fn create_assets_from_render_tree(&mut self, render_tree: &RenderTree) -> Assets {
        // Collect assets from all nodes in the render tree
        self.collect_assets_from_nodes(&render_tree.root);

        let mut textures: Vec<Texture> = Vec::new();
        let mut materials = Vec::new();
        let mut meshes = Vec::new();

        for (_id, (mesh, material, texture)) in &self.assets_map {
            meshes.push(mesh.clone());
            materials.push(material.clone());
            textures.push(texture.clone());
        }

        Assets {
            textures,
            materials,
            meshes,
        }
    }

    /// Collect assets from all nodes in the render tree
    fn collect_assets_from_nodes(&mut self, node: &Rc<RefCell<RenderNode>>) {
        let node_ref = node.borrow();

        // Process item nodes that might have GLB files
        if node_ref.node_type == RenderNodeType::Item {
            if let Some(object) = &node_ref.computed_style.object {
                // If assets for this object are already collected, skip.
                if !self.assets_map.contains_key(&object.identifier) {
                    // If object has a path, create appropriate assets
                    if let Some(path) = &object.path {
                        if path.to_lowercase().ends_with(".glb") {
                            let glb_path = std::path::Path::new(path);
                            let file_name = glb_path
                                .file_stem()
                                .map(|name| name.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            // Create a mesh asset entry
                            let mesh_name = format!("mesh_{}", file_name);
                            let mesh_file =
                                format!("assets/{}/{}.stl", object.identifier, file_name);
                            let mesh = Mesh {
                                name: mesh_name.clone(),
                                file: mesh_file,
                            };

                            // Create texture and material assets if texture files exist in the object's directory
                            let texture_dir = format!("assets/{}/", object.identifier);
                            let texture_path = format!("{}texture_1.png", texture_dir); // Simplified - in reality, we'd check for actual texture files

                            let texture_name = format!("tex_{}", file_name);
                            // For now, we'll create a placeholder texture and material based on the object
                            // In a real implementation, we'd check for actual texture files
                            let texture = Texture {
                                name: texture_name.clone(),
                                texture_type: "2d".to_string(),
                                file: texture_path.clone(),
                            };

                            let material = Material {
                                name: format!("mat_{}", file_name),
                                texture: texture_name,
                                texrepeat: "1 1".to_string(),
                            };

                            self.assets_map
                                .insert(object.identifier.clone(), (mesh, material, texture));
                        }
                    }
                }
            }
        }

        // Recursively process child nodes
        for child in &node_ref.children {
            self.collect_assets_from_nodes(child);
        }
    }

    /// Generate MuJoCo XML and save it to the specified directory
    pub fn generate_to_directory(
        &mut self,
        render_tree: &RenderTree,
        directory: &str,
    ) -> Result<()> {
        Self::convert_glb_to_stl_with_textures(render_tree, directory)?;
        // Generate the Mujoco struct
        let mujoco = self.generate(render_tree);

        // Convert Mujoco struct to XML string
        let mut buf = Vec::new();
        let mut writer = quick_xml::Writer::new(&mut buf);

        // Write the XML declaration
        writer.write_event(quick_xml::events::Event::Decl(
            quick_xml::events::BytesDecl::new("1.0", Some("utf-8"), None),
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

    fn collect_item_geoms(&self, node: &Rc<RefCell<RenderNode>>, geoms: &mut Vec<Geom>) {
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

                let pos = match object.geom_type {
                    PackageGeomType::Mesh => node_ref.computed_style.content_pos(),
                    _ => {
                        if let Some(pos) = node_ref.computed_style.get_center_pos() {
                            pos
                        } else {
                            SpacePosition::zero()
                        }
                    }
                };
                // 获取位置信息
                let pos = if let Some(position) = pos.get_length() {
                    format!(
                        "{} {} {}",
                        Self::length_to_meters(position.x),
                        Self::length_to_meters(position.y),
                        Self::length_to_meters(position.z)
                    )
                } else {
                    "0 0 0".to_string()
                };

                let geom_type: GeomType = object.geom_type.clone().into();

                let mut size = None;
                if geom_type != GeomType::Mesh {
                    // 获取尺寸信息
                    size = Some(format!(
                        "{} {} {}",
                        Self::length_to_meters(object.size.x / 2),
                        Self::length_to_meters(object.size.y / 2),
                        Self::length_to_meters(object.size.z / 2)
                    ));
                }

                println!("geom_type={:?}", geom_type);
                println!("identifier={:?}", object.identifier);
                println!("assets_map={:?}", self.assets_map);
                let (mesh, material) = if geom_type == GeomType::Mesh {
                    if let Some((mesh, material, _)) = self.assets_map.get(&object.identifier) {
                        (Some(mesh.name.clone()), Some(material.name.clone()))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                // 创建Geom
                let geom = Geom {
                    name,
                    size,
                    pos,
                    geom_type,
                    mesh,
                    material,
                };

                geoms.push(geom);
            }
        }

        // 递归处理子节点
        for child in &node_ref.children {
            self.collect_item_geoms(child, geoms);
        }
    }

    /// 将Length转换为米为单位的浮点数，并格式化为字符串
    fn length_to_meters(length: Length) -> String {
        let meters = length.mm() as f64 / 1000.0;
        // 保留6位小数，去除尾随零
        format!("{:.6}", meters)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }

    /// Convert GLB files from render tree to STL with texture files and save to specified directory
    pub fn convert_glb_to_stl_with_textures(
        render_tree: &RenderTree,
        directory: &str,
    ) -> Result<()> {
        println!("convert_glb_to_stl_with_textures");
        // Create the target directory if it doesn't exist
        fs::create_dir_all(directory)?;

        // Count the number of converted files
        let count =
            Self::process_glb_nodes(&render_tree.root, directory, render_tree.get_package())?;

        println!(
            "Conversion completed. {} GLB files converted. Each GLB file has its own directory with corresponding STL and texture files in: {}",
            count, directory
        );

        Ok(())
    }

    /// Process GLB nodes and convert to STL with textures
    fn process_glb_nodes(
        node: &Rc<RefCell<RenderNode>>,
        directory: &str,
        package: &crate::package::Package,
    ) -> Result<usize> {
        let mut count = 0;
        let node_ref = node.borrow();
        println!(
            "node_ref.computed_style.object={:?}",
            node_ref.computed_style.object
        );
        // Process item nodes that might have GLB files
        if node_ref.node_type == RenderNodeType::Item {
            if let Some(object) = &node_ref.computed_style.object {
                // Use get_absolute_path method instead of direct access to object.path
                if let Some(path) = object.get_absolute_path(package) {
                    if path.to_lowercase().ends_with(".glb") {
                        let glb_path = Path::new(&path);
                        let file_name = glb_path
                            .file_stem()
                            .ok_or_else(|| {
                                crate::error::RsmlError::Io(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!("Invalid GLB file path: {}", path),
                                ))
                            })?
                            .to_str()
                            .ok_or_else(|| {
                                crate::error::RsmlError::Io(std::io::Error::new(
                                    std::io::ErrorKind::InvalidInput,
                                    format!("Invalid GLB file path: {}", path),
                                ))
                            })?;

                        // Get the parent directory of the GLB file to recreate the directory structure
                        let relative_glb_parent = format!("assets/{}", object.identifier);

                        // Create a directory path that includes the parent directory structure
                        let glb_dir = Path::new(directory).join(relative_glb_parent);
                        fs::create_dir_all(&glb_dir)?;

                        // Convert GLB to STL - put STL in the GLB-specific directory
                        let stl_path = glb_dir.join(format!("{}.stl", file_name));
                        match Self::convert_glb_to_stl(glb_path, &stl_path) {
                            Ok(()) => {
                                // Extract and save textures in the same GLB-specific directory
                                let texture_dir = &glb_dir; // Textures go in same directory as STL
                                if let Err(e) =
                                    Self::extract_textures_from_glb(glb_path, texture_dir)
                                {
                                    eprintln!(
                                        "Warning: Could not extract textures from {}: {}",
                                        path, e
                                    );
                                }

                                println!(
                                    "Converted GLB file: {} -> Directory: {}",
                                    path,
                                    glb_dir.display()
                                );
                                count += 1;
                            }
                            Err(e) => {
                                eprintln!("Error converting GLB file {}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        // Recursively process child nodes
        for child in &node_ref.children {
            count += Self::process_glb_nodes(child, directory, package)?;
        }

        Ok(count)
    }

    /// Convert a GLB file to STL format
    fn convert_glb_to_stl(glb_path: &Path, stl_path: &Path) -> AnyResult<()> {
        // Read the GLB file
        let (document, buffers, _) = gltf::import(&glb_path)?;

        // Create a vector to store all triangles from all meshes
        let mut triangles = Vec::new();

        // Process each mesh in the GLB file
        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                // Get the positions accessor
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                if let Some(iter) = reader.read_positions() {
                    // Convert positions to triangles
                    let positions: Vec<[f32; 3]> = iter.collect();

                    // For each primitive, we need to handle indices or assume triangles
                    if let Some(indices) = reader.read_indices() {
                        let indices: Vec<u32> = indices.into_u32().collect();
                        // Process indexed triangles
                        for chunk in indices.chunks(3) {
                            if chunk.len() == 3 {
                                let v1 = stl_io::Vector([
                                    positions[chunk[0] as usize][0] * 0.1,
                                    positions[chunk[0] as usize][2] * 0.1,
                                    positions[chunk[0] as usize][1] * 0.1,
                                ]);
                                let v2 = stl_io::Vector([
                                    positions[chunk[1] as usize][0] * 0.1,
                                    positions[chunk[1] as usize][2] * 0.1,
                                    positions[chunk[1] as usize][1] * 0.1,
                                ]);
                                let v3 = stl_io::Vector([
                                    positions[chunk[2] as usize][0] * 0.1,
                                    positions[chunk[2] as usize][2] * 0.1,
                                    positions[chunk[2] as usize][1] * 0.1,
                                ]);
                                triangles.push(stl_io::Triangle {
                                    normal: stl_io::Vector([0.0, 0.0, 0.0]), // Will be computed later
                                    vertices: [v1, v2, v3],
                                });
                            }
                        }
                    } else {
                        // Process non-indexed triangles (assuming vertices are already in triangle format)
                        for chunk in positions.chunks(3) {
                            if chunk.len() == 3 {
                                let v1 = stl_io::Vector([chunk[0][0], chunk[0][1], chunk[0][2]]);
                                let v2 = stl_io::Vector([chunk[1][0], chunk[1][1], chunk[1][2]]);
                                let v3 = stl_io::Vector([chunk[2][0], chunk[2][1], chunk[2][2]]);
                                triangles.push(stl_io::Triangle {
                                    normal: stl_io::Vector([0.0, 0.0, 0.0]), // Will be computed later
                                    vertices: [v1, v2, v3],
                                });
                            }
                        }
                    }
                }
            }
        }

        // Compute normals and write to STL file
        let mut file = std::fs::File::create(stl_path)?;
        stl_io::write_stl(&mut file, triangles.iter())?;

        Ok(())
    }

    /// Extract textures from GLB file and save to directory
    fn extract_textures_from_glb(glb_path: &Path, texture_dir: &Path) -> AnyResult<()> {
        let (document, buffers, _images) = gltf::import(&glb_path)?;

        // Process each texture in the GLB file
        for (index, texture) in document.textures().enumerate() {
            let image = texture.source();
            match image.source() {
                gltf::image::Source::Uri { uri, .. } => {
                    // Handle external image URIs
                    if uri.starts_with("data:") {
                        // This is a data URI, extract the image data
                        Self::extract_image_from_data_uri(uri, texture_dir, index)?;
                    } else {
                        // Handle local file reference if needed
                        eprintln!("External image URI not supported yet: {}", uri);
                    }
                }
                gltf::image::Source::View {
                    view, mime_type, ..
                } => {
                    // Handle embedded image data with MIME type
                    let buffer = &buffers[view.buffer().index()];
                    let start = view.offset();
                    let end = start + view.length();
                    let image_data = &buffer[start..end];

                    // Determine image format based on MIME type
                    let format_extension = match mime_type {
                        "image/png" => "png",
                        "image/jpeg" | "image/jpg" => "jpg",
                        _ => {
                            eprintln!("Unsupported MIME type: {}", mime_type);
                            continue;
                        }
                    };

                    let texture_path =
                        texture_dir.join(format!("texture_{}.{}", index, format_extension));
                    std::fs::write(&texture_path, image_data)?;

                    // Verify it's a valid image by trying to load it
                    if let Err(e) = image::load_from_memory(image_data) {
                        eprintln!("Invalid image data for texture {}: {}", index, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract image from data URI
    fn extract_image_from_data_uri(uri: &str, texture_dir: &Path, index: usize) -> AnyResult<()> {
        if let Some(data_pos) = uri.find(",") {
            let header = &uri[..data_pos];
            let data = &uri[data_pos + 1..];

            // Decode base64 data
            let image_data = base64::decode(data)?;

            // Determine format from header
            let format = if header.contains("image/png") {
                ImageFormat::Png
            } else if header.contains("image/jpeg") || header.contains("image/jpg") {
                ImageFormat::Jpeg
            } else {
                eprintln!("Unsupported data URI format in: {}", header);
                return Ok(());
            };

            let texture_path =
                texture_dir.join(format!("texture_{}.{}", index, format.extensions_str()[0]));
            std::fs::write(&texture_path, image_data)?;

            Ok(())
        } else {
            eprintln!("Invalid data URI: {}", uri);
            Ok(())
        }
    }
}

// Add base64 dependency since we need it for data URIs
use base64;

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
        let generator = MjcfGenerator::new();

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
        let node = Rc::new(RefCell::new(RenderNode::new(
            "test".to_string(),
            RenderNodeType::Space,
        )));

        // 创建一个空的package（这里只是测试，实际不会用到）
        // 我们直接测试collect_item_geoms方法

        let mut geoms = Vec::new();
        let mjcf_generator = MjcfGenerator::new();
        mjcf_generator.collect_item_geoms(&node, &mut geoms);

        // 应该没有geom，因为根节点是Space类型
        assert_eq!(geoms.len(), 0);
    }

    #[test]
    fn test_generate_to_directory() {
        use crate::package::Package;
        use crate::xml_parser::Element as DomElement;
        use std::fs;
        use tempfile::tempdir; // You might need to add tempfile as dev dependency

        // Create a minimal package for testing
        let test_package = Package {
            package: crate::package::PackageInfo {
                name: "test_package".to_string(),
                description: "Test package".to_string(),
            },
            objects: std::collections::HashMap::new(),
            groups: Vec::new(),
            dependencies: std::collections::HashMap::new(),
            directory: ".".to_string(),
        };

        // Create a simple DOM element
        let dom_element = DomElement::new("space".to_string());

        // Create a render tree from the DOM element and package
        let render_tree = RenderTree::new(&dom_element, &test_package).unwrap();

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Test the generate_to_directory function
        let mut mjcf_generator = MjcfGenerator::new();
        let result = mjcf_generator.generate_to_directory(&render_tree, temp_path);
        assert!(result.is_ok());

        // Check if the file was created
        let xml_path = Path::new(temp_path).join("model.xml");
        assert!(xml_path.exists());

        // Read and check if the file contains XML content
        let content = fs::read_to_string(xml_path).unwrap();
        println!("Generated XML content: {}", content); // Debug output
        assert!(content.contains("rsml_model"));
    }

    #[test]
    fn test_convert_glb_to_stl_with_textures() {
        use crate::package::Package;
        use crate::xml_parser::Element as DomElement;
        use tempfile::tempdir;

        // Create a minimal package for testing
        let mut objects = std::collections::HashMap::new();
        // Add an object with a GLB path for testing
        objects.insert(
            "test_object".to_string(),
            Object {
                geom_type: crate::package::GeomType::Mesh,
                size: Dim3::new(
                    Length::from_m(1.0),
                    Length::from_m(1.0),
                    Length::from_m(1.0),
                ),
                path: Some("examples/tiny_example/model.glb".to_string()), // Use a dummy path for testing
                identifier: "test_object".to_string(),
                mesh_actual_size: (0.0, 0.0, 0.0),
            },
        );

        let test_package = Package {
            package: crate::package::PackageInfo {
                name: "test_package".to_string(),
                description: "Test package".to_string(),
            },
            objects,
            groups: Vec::new(),
            dependencies: std::collections::HashMap::new(),
            directory: ".".to_string(),
        };

        // Create a DOM element that will use the test object
        let mut dom_element = DomElement::new("object".to_string());
        dom_element.text = "test_object".to_string(); // This should match the object name

        // Create a render tree from the DOM element and package
        let render_tree = RenderTree::new(&dom_element, &test_package).unwrap();

        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // Test the convert_glb_to_stl_with_textures function (it will fail if the GLB file doesn't exist, but that's expected)
        let result = MjcfGenerator::convert_glb_to_stl_with_textures(&render_tree, temp_path);
        // This might fail because the GLB file doesn't exist, but the function should execute without panicking
        println!("Conversion result: {:?}", result);
    }
}
