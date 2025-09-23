use thiserror::Error;

#[derive(Error, Debug)]
pub enum RsmlError {
    #[error("XML parsing error: {0}")]
    XmlParse(#[from] quick_xml::Error),

    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] quick_xml::events::attributes::AttrError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Invalid RSML structure: {message}")]
    InvalidStructure { message: String },

    #[error("Missing required element: {element}")]
    MissingElement { element: String },

    #[error("Invalid attribute value: {attribute} = {value}")]
    InvalidAttribute { attribute: String, value: String },

    #[error("Parse error for {field}: {message}")]
    ParseError { field: String, message: String },

    #[error("Render tree error: {message}")]
    RenderTree { message: String },

    #[error("Style computation error: {message}")]
    StyleComputation { message: String },

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("API error: status={status}, message={message}")]
    ApiError { status: i32, message: String },

    #[error("Cube display must explicit set size: length(mm/cm) or percentage(%)")]
    CubeSizeError,

    #[error("package config error: {0}")]
    PackageConfigError(String),
}

pub type Result<T> = anyhow::Result<T, RsmlError>;
