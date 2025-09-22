use crate::error::{Result, RsmlError};
use serde::Deserialize;
use std::fs;
use std::io::copy;
use std::path::Path;

/// Represents a single 3D asset from the API
#[derive(Deserialize, Debug, Clone)]
pub struct Robot3DAsset {
    pub id: String,
    pub name: String,
    /// The direct download URL for the asset file
    pub resource_url: String,
    /// Asset dimensions in millimeters, assuming the API provides them.
    pub x_len: f64,
    pub y_len: f64,
    pub z_len: f64,
}

/// Represents a paginated list response from the API
#[derive(Deserialize, Debug)]
pub struct PaginationListResp<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub total_pages: i64,
}

/// Response item for a 3D asset category
#[derive(Deserialize, Debug)]
pub struct Robot3DAssetCategoryRespItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub utime: i64,
    pub ctime: i64,
    pub asset_count: i64,
}

/// Generic API response wrapper
#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub data: T,
    pub code: i32,
    pub message: String,
}

/// Fetches dependency information from the remote asset server.
pub fn fetch_dependency(name: &str) -> Result<Robot3DAssetCategoryRespItem> {
    let base_url = "https://transairobot.com";
    let url = format!("{}/api/robot-3d-assets/categories/name", base_url);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .query(&[("name", name)])
        .send()
        .map_err(|e| RsmlError::NetworkError(e.to_string()))?;

    if response.status().is_success() {
        let api_response: ApiResponse<Robot3DAssetCategoryRespItem> =
            response.json().map_err(|e| RsmlError::ParseError {
                field: "API Response".to_string(),
                message: e.to_string(),
            })?;
        
        // Check if the API returned a success code (assuming 0 or 200 means success)
        if api_response.code == 0 || api_response.code == 200 {
            Ok(api_response.data)
        } else {
            // Handle API-level errors (like authentication failures)
            Err(RsmlError::ApiError {
                status: api_response.code as u16,
                message: api_response.message,
            })
        }
    } else {
        Err(RsmlError::ApiError {
            status: response.status().as_u16(),
            message: response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string()),
        })
    }
}

/// Fetches a paginated list of 3D assets within a specific category.
pub fn fetch_assets_in_category(
    category_id: String,
    page: u32,
    limit: u32,
) -> Result<PaginationListResp<Robot3DAsset>> {
    let base_url = "https://transairobot.com";
    let url = format!("{}/api/robot-3d-assets/{}/assets", base_url, category_id);

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .query(&[("page", page), ("limit", limit)])
        .send()
        .map_err(|e| RsmlError::NetworkError(e.to_string()))?;

    if response.status().is_success() {
        let api_response: ApiResponse<PaginationListResp<Robot3DAsset>> =
            response.json().map_err(|e| RsmlError::ParseError {
                field: "API Response".to_string(),
                message: e.to_string(),
            })?;
        
        // Check if the API returned a success code (assuming 0 or 200 means success)
        if api_response.code == 0 || api_response.code == 200 {
            Ok(api_response.data)
        } else {
            // Handle API-level errors (like authentication failures)
            Err(RsmlError::ApiError {
                status: api_response.code as u16,
                message: api_response.message,
            })
        }
    } else {
        Err(RsmlError::ApiError {
            status: response.status().as_u16(),
            message: response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string()),
        })
    }
}

/// Downloads a file from a given URL and saves it to a specified path.
pub fn download_file(url: &str, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut response =
        reqwest::blocking::get(url).map_err(|e| RsmlError::NetworkError(e.to_string()))?;
    let mut dest = fs::File::create(path)?;

    if response.status().is_success() {
        copy(&mut response, &mut dest)?;
        Ok(())
    } else {
        Err(RsmlError::ApiError {
            status: response.status().as_u16(),
            message: format!("Failed to download file from {}", url),
        })
    }
}
