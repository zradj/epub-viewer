use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error("Failed to open file: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP archive error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML Parsing Error: {0}")]
    Xml(#[from] roxmltree::Error),
    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
    #[error("Incorrect MIME type")]
    IncorrectMimeType,
    #[error("Could not find the OPF package file")]
    PackageNotFound,
    #[error("Content cannot be converted to text")]
    NotTextContent,
    #[error("Book metadata error: {0}")]
    Metadata(MetadataError),
    #[error("Resource not found: {0}")]
    ResourceNotFound(String),
    #[error("Missing attribute '{attr}': {loc}")]
    MissingAttribute {
        attr: &'static str,
        loc: &'static str,
    },
}

#[derive(Error, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum MetadataError {
    #[error("The book has no identifier")]
    NoIdentifier,
    #[error("The book has no title")]
    NoTitle,
    #[error("The book does not specify the language of the content")]
    NoLanguage,
    #[error("The book does not specify the last modified date")]
    NoLastModified,
    #[error("The book provides multiple publication dates")]
    MultipleDates,
    #[error("The book provides multiple last modified dates")]
    MultipleLastModified,
}

pub type EpubResult<T> = Result<T, EpubError>;
