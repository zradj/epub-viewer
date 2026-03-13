use std::collections::HashMap;

#[derive(Debug)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub resources: HashMap<String, EpubResource>,
    pub spine: Vec<String>,
}

#[derive(Debug, Default)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug)]
pub struct EpubResource {
    pub id: String,
    pub path: String,
    pub mime_type: String,
    pub content: Vec<u8>,
}
