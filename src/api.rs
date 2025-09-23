use crate::error::{Result, RsmlError};
use serde::Deserialize;
use serde_json;
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
    pub data: Option<T>,
    pub code: i32,
    pub message: String,
}

/// Generic API call function for robot-3d-assets endpoints
fn api_call<T: for<'de> Deserialize<'de>>(path: &str, method: &str) -> Result<T> {
    let base_url = "https://transairobot.com";
    let url = format!("{}/api/{}", base_url, path);
    
    let client = reqwest::blocking::Client::new();
    let response = match method.to_lowercase().as_str() {
        "get" => client.get(&url).send(),
        "post" => client.post(&url).send(),
        _ => return Err(RsmlError::ParseError {
            field: "HTTP Method".to_string(),
            message: format!("Unsupported method: {}", method),
        }),
    }
    .map_err(|e| RsmlError::NetworkError(e.to_string()))?;

    if response.status().is_success() {
        let text = response.text().map_err(|e| RsmlError::NetworkError(e.to_string()))?;
        
        let api_response: ApiResponse<T> = serde_json::from_str(&text).map_err(|e| RsmlError::ParseError {
            field: "API Response".to_string(),
            message: e.to_string(),
        })?;
        
        if api_response.code == 0 || api_response.code == 200 {
            api_response.data.ok_or_else(|| RsmlError::ApiError {
                status: api_response.code,
                message: "API returned success code but no data".to_string(),
            })
        } else {
            Err(RsmlError::ApiError {
                status: api_response.code,
                message: api_response.message,
            })
        }
    } else {
        Err(RsmlError::ApiError {
            status: response.status().as_u16() as i32,
            message: response.text().unwrap_or_else(|_| "Unknown error".to_string()),
        })
    }
}

/// Fetches dependency information from the remote asset server.
pub fn fetch_dependency(name: &str) -> Result<Robot3DAssetCategoryRespItem> {
    let path = format!("robot-3d-assets/categories/name?name={}", name);
    api_call(&path, "get")
}

/// Fetches a paginated list of 3D assets within a specific category.
pub fn fetch_assets_in_category(
    category_id: String,
    page: u32,
    limit: u32,
) -> Result<PaginationListResp<Robot3DAsset>> {
    let path = format!("robot-3d-assets/assets?category_id={}&page={}&limit={}", category_id, page, limit);
    api_call(&path, "get")
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
            status: response.status().as_u16() as i32,
            message: format!("Failed to download file from {}", url),
        })
    }
}
