use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error("Failed to open file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to read ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML Parsing Error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("Missing attribute '{attr}' on manifest item")]
    MissingAttribute { attr: &'static str },
    #[error("Could not find the OPF rootfile")]
    RootfileNotFound,
}

pub type EpubResult<T> = Result<T, EpubError>;
