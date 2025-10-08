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

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreeJsScene {
    pub objects: Vec<ThreeJsObject>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ThreeJsObject {
    pub id: Option<String>,
    pub tag_name: String,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
    pub size: [f64; 3],
    pub geometry_type: String,
    pub resource_path: Option<String>,
    pub identifier: Option<String>,
}

pub struct ThreeJsGenerator;

impl ThreeJsGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, render_tree: &RenderTree) -> ThreeJsScene {
        let mut objects = Vec::new();

        // Collect all item objects from the render tree
        self.collect_item_objects(&render_tree.root, &mut objects);

        ThreeJsScene { objects }
    }

    fn collect_item_objects(&self, node: &Rc<RefCell<RenderNode>>, objects: &mut Vec<ThreeJsObject>) {
        let node_ref = node.borrow();

        // If it's an Item type node, convert it to a ThreeJsObject
        if node_ref.node_type == RenderNodeType::Item {
            if let Some(object) = &node_ref.computed_style.object {
                let position = match object.geom_type {
                    PackageGeomType::Mesh => node_ref.computed_style.content_pos(),
                    _ => {
                        if let Some(pos) = node_ref.computed_style.get_center_pos() {
                            pos
                        } else {
                            SpacePosition::zero()
                        }
                    }
                };

                // Get position values in meters with Y and Z swapped for Three.js Y-up coordinate system
                let pos_values = if let Some(pos) = position.get_length() {
                    [
                        Self::length_to_meters(pos.x),
                        Self::length_to_meters(pos.z),  // Z in render_tree becomes Y in Three.js
                        Self::length_to_meters(pos.y)   // Y in render_tree becomes Z in Three.js
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                };

                // Get size values in meters with Y and Z swapped for Three.js Y-up coordinate system
                let size_values = [
                    Self::length_to_meters(object.size.x),
                    Self::length_to_meters(object.size.z),  // Z in render_tree becomes Y in Three.js
                    Self::length_to_meters(object.size.y)   // Y in render_tree becomes Z in Three.js
                ];

                // Calculate scale based on size/mesh_actual_size
                // If mesh_actual_size is 0, set scale to 1
                let scale_values = if object.mesh_actual_size.0 != 0.0 && object.mesh_actual_size.1 != 0.0 && object.mesh_actual_size.2 != 0.0 {
                    [
                        size_values[0] / object.mesh_actual_size.0 as f64,
                        size_values[2] / object.mesh_actual_size.1 as f64,
                        size_values[1] / object.mesh_actual_size.2 as f64
                    ]
                } else {
                    [1.0, 1.0, 1.0]
                };

                println!("size={:?}, act_size={:?}", size_values, object.mesh_actual_size);

                // Create a ThreeJsObject
                let three_js_object = ThreeJsObject {
                    id: node_ref.id.clone(),
                    tag_name: node_ref.tag_name.clone(),
                    position: pos_values,
                    rotation: [0.0, 0.0, 0.0], // Default rotation
                    scale: scale_values,
                    size: size_values,
                    geometry_type: match object.geom_type {
                        PackageGeomType::Box => "box".to_string(),
                        PackageGeomType::Mesh => "mesh".to_string(),
                    },
                    resource_path: object.path.clone(),
                    identifier: Some(object.identifier.clone()),
                };

                objects.push(three_js_object);
            }
        }

        // Recursively process child nodes
        for child in &node_ref.children {
            self.collect_item_objects(child, objects);
        }
    }

    /// 将Length转换为米为单位的浮点数
    fn length_to_meters(length: Length) -> f64 {
        length.mm() as f64 / 1000.0
    }

    /// Generate Three.js JSON and save it along with assets to the specified directory
    pub fn generate_to_directory(
        &self,
        render_tree: &RenderTree,
        directory: &str,
    ) -> Result<()> {
        // Create threejs subdirectory
        let threejs_directory = Path::new(directory).join("threejs");
        fs::create_dir_all(&threejs_directory)?;

        // Generate the Three.js scene
        let scene = self.generate(render_tree);

        // Write the scene to a JSON file in the threejs subdirectory
        let file_path = threejs_directory.join("scene.json");
        let mut file = std::fs::File::create(file_path)?;
        let json_content = serde_json::to_string_pretty(&scene)?;
        file.write_all(json_content.as_bytes())?;

        // Process and copy asset files to the assets directory
        self.copy_assets(render_tree, &threejs_directory.to_string_lossy())?;

        Ok(())
    }

    /// Copy asset files (like GLB files) to the assets directory
    fn copy_assets(&self, render_tree: &RenderTree, directory: &str) -> Result<()> {
        // Create assets directory
        let assets_dir = Path::new(directory).join("assets");
        fs::create_dir_all(&assets_dir)?;

        // Collect and copy all asset files from the render tree
        self.collect_and_copy_asset_files(&render_tree.root, &assets_dir, render_tree.get_package())?;

        Ok(())
    }

    /// Collect and copy asset files from all nodes in the render tree
    fn collect_and_copy_asset_files(
        &self,
        node: &Rc<RefCell<RenderNode>>,
        assets_dir: &Path,
        package: &crate::package::Package,
    ) -> Result<()> {
        let node_ref = node.borrow();

        // Process item nodes that might have asset files
        if node_ref.node_type == RenderNodeType::Item {
            if let Some(object) = &node_ref.computed_style.object {
                // Use get_absolute_path method instead of direct access to object.path
                if let Some(path) = object.get_absolute_path(package) {
                    if path.to_lowercase().ends_with(".glb") {

                        // Copy the GLB file to the appropriate directory
                        let src_path = Path::new(&path);

                        let dest_path = assets_dir.join(object.path.clone().unwrap());
                        // Get the identifier to create a subdirectory
                        fs::create_dir_all(&dest_path.parent().unwrap())?;

                        std::fs::copy(src_path, &dest_path)?;
                        
                        println!("Copied GLB file: {} -> {}", path, dest_path.display());
                    }
                }
            }
        }

        // Recursively process child nodes
        for child in &node_ref.children {
            self.collect_and_copy_asset_files(child, assets_dir, package)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Length;
    use crate::dim3::Dim3;
    use crate::package::{GeomType as PackageGeomType, Object};
    use crate::render_tree::{RenderNode, RenderNodeType};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_length_to_meters() {
        // Test 1 meter
        let length = Length::from_m(1.0);
        assert_eq!(ThreeJsGenerator::length_to_meters(length), 1.0);

        // Test 10 centimeters
        let length = Length::from_cm(10);
        assert_eq!(ThreeJsGenerator::length_to_meters(length), 0.1);

        // Test 5 millimeters
        let length = Length::from_mm(5);
        assert_eq!(ThreeJsGenerator::length_to_meters(length), 0.005);
    }

    #[test]
    fn test_generate_empty_tree() {
        // Create a simple render node
        let node = Rc::new(RefCell::new(RenderNode::new(
            "test".to_string(),
            RenderNodeType::Space,
        )));

        let mut objects = Vec::new();
        let threejs_generator = ThreeJsGenerator::new();
        threejs_generator.collect_item_objects(&node, &mut objects);

        // Should have no objects since the root is a Space type
        assert_eq!(objects.len(), 0);
    }
}