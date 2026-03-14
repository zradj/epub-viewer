use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpubError {
    #[error("Failed to open file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to read ZIP archive: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("XML Parsing Error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Could not find the OPF rootfile")]
    RootfileNotFound,
    #[error("Invalid UTF-8 in {context}")]
    Utf8 { context: &'static str, #[source] source: std::str::Utf8Error },
}