use crate::geom::{Box, Cylinder, ObjectGroup, Sphere};
use serde::Deserialize;
use thiserror::Error;
use anyhow::{Result, bail};

#[derive(Debug, Deserialize)]
pub struct SpaceLayoutSheet {
    // #[serde(rename = "@path")]
    // pub path: String,
    #[serde(rename = "$text")]
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct Head {
    #[serde(rename = "layout")]
    pub layout: SpaceLayoutSheet,
}

#[derive(Debug, Deserialize)]
pub enum SpaceChildren {
    // 关键：Box 把大小变成已知，允许递归
    #[serde(rename = "space")]
    Space(Space),

    #[serde(rename = "group")]
    ObjectGroup(ObjectGroup),

    #[serde(rename = "box")]
    Box(Box),

    #[serde(rename = "cylinder")]
    Cylinder(Cylinder),

    #[serde(rename = "sphere")]
    Sphere(Sphere),
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Space cannot contain both child spaces and object groups")]
    MixedSpaceAndGroup,
    #[error("Space cannot contain more than one Geom(ObjectGroup, Sphere, Cylinder)")]
    MultipleGeoms,
}

#[derive(Debug, Deserialize)]
pub struct Space {
    #[serde(rename = "@id", default)]
    pub id: String,

    #[serde(rename = "$value", default)]
    pub children: Vec<SpaceChildren>,
}

impl Space {
    pub fn validate(&self) -> Result<()> {
        let mut has_space = false;
        let mut other_count = 0;

        for child in &self.children {
            match child {
                SpaceChildren::Space(_) => has_space = true,
                _ => other_count += 1,
            }
        }

        // 检查是否同时包含Space和ObjectGroup
        if has_space && other_count > 0 {
            bail!(ParseError::MixedSpaceAndGroup);
        }

        // 检查是否包含多个ObjectGroup
        if other_count > 1 {
            bail!(ParseError::MultipleGeoms);
        }

        // 递归验证子空间
        for child in &self.children {
            if let SpaceChildren::Space(space) = child {
                space.validate()?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Body {
    #[serde(rename = "space", default)]
    pub spaces: Vec<Space>,
}

impl Body {
    pub fn validate(&self) -> Result<()> {
        for space in &self.spaces {
            space.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Root {
    #[serde(rename = "head")]
    pub head: Head,
    #[serde(rename = "body")]
    pub body: Body,
}

impl Root {
    pub fn validate(&self) -> Result<()> {
        self.body.validate()
    }
}
