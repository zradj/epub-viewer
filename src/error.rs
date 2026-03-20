use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error("Failed to open file: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML Parsing Error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("Incorrect MIME type")]
    IncorrectMimeType,
    #[error("Could not find the OPF package file")]
    PackageNotFound,
    #[error("Content cannot be converted to text")]
    NotTextContent,
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Missing attribute '{attr}': {loc}")]
    MissingAttribute {
        attr: &'static str,
        loc: &'static str,
    },
}

pub type EpubResult<T> = Result<T, EpubError>;
