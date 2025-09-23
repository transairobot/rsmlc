use crate::base::Length;
use crate::dim3::Dim3;
use crate::error::RsmlError;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

fn deserialize_size<'de, D>(deserializer: D) -> Result<Dim3<Length>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let parts: Vec<&str> = s.split_whitespace().collect();

    if parts.len() != 3 {
        return Err(serde::de::Error::custom(
            "Size must be in format 'length width height' with 3 components",
        ));
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum GeomType {
    Box,
    Mesh,
}

impl Default for GeomType {
    fn default() -> Self {
        GeomType::Mesh
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Object {
    #[serde(rename = "geom-type")]
    #[serde(default)]
    pub geom_type: GeomType,
    #[serde(deserialize_with = "deserialize_size")]
    pub size: Dim3<Length>,
    pub path: Option<String>,
}

impl<'de> Deserialize<'de> for Object {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ObjectHelper {
            #[serde(rename = "geom-type")]
            #[serde(default)]
            geom_type: GeomType,
            #[serde(deserialize_with = "deserialize_size")]
            size: Dim3<Length>,
            path: Option<String>,
        }

        let helper = ObjectHelper::deserialize(deserializer)?;
        Ok(Object {
            geom_type: helper.geom_type,
            size: helper.size,
            path: helper.path,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
    pub name: String,
    pub objects: std::collections::HashMap<String, Object>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub package: PackageInfo,
    pub objects: std::collections::HashMap<String, Object>,
    pub groups: Vec<Group>,
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
}

impl Package {
    /// Load a package from a TOML file
    pub fn from_file(path: &str) -> Result<Self, crate::error::RsmlError> {
        let contents = std::fs::read_to_string(path).map_err(|e| crate::error::RsmlError::Io(e))?;
        toml::from_str(&contents).map_err(|e| crate::error::RsmlError::ParseError {
            field: "package".to_string(),
            message: format!("Failed to parse package file '{}': {}", path, e),
        })
    }

    /// Get the space size for an object or group by name.
    /// For groups, returns the maximum size among all objects in the group.
    pub fn get_space_size(&self, name: &str) -> Option<Dim3<Length>> {
        // First, try to find in objects
        if let Some(object) = self.objects.get(name) {
            return Some(object.size);
        }

        // Then, try to find in groups
        for group in &self.groups {
            if group.name == name {
                // Calculate maximum size among all objects in the group
                let mut max_size: Option<Dim3<Length>> = None;

                for (_, object) in &group.objects {
                    match max_size {
                        None => {
                            max_size = Some(object.size);
                        }
                        Some(current_max) => {
                            // Update max_size with the component-wise maximum
                            let new_max = Dim3::new(
                                std::cmp::max(current_max.x, object.size.x),
                                std::cmp::max(current_max.y, object.size.y),
                                std::cmp::max(current_max.z, object.size.z),
                            );
                            max_size = Some(new_max);
                        }
                    }
                }

                return max_size;
            }
        }

        // Name not found
        None
    }
}
