use std::{cell::RefCell, collections::HashMap, fs::File, io::{BufReader, Read, Seek}, rc::Rc};

use zip::{ZipArchive, read::ZipFile};

use crate::error::{EpubError, EpubResult};

#[derive(Debug)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub spine: Vec<String>,
    resources: RefCell<HashMap<String, EpubResource>>,
    archive: RefCell<ZipArchive<File>>,
}

impl EpubBook {
    pub fn new(path: &str) -> EpubResult<Self> {
        let archive = File::open(path)?;
        let mut archive = ZipArchive::new(archive)?;

        let mut container = archive.by_name("META-INF/container.xml")?;
        let mut container_content = String::new();
        container.read_to_string(&mut container_content)?;

        let container = roxmltree::Document::parse(&container_content)?;
        let root = Self::extract_root_path(container)?;

        todo!();
    }

    fn extract_root_path(container: roxmltree::Document) -> EpubResult<String> {
        for desc in container.descendants() {
            if desc.tag_name().name() == "rootfile" {
                let attr = desc.attribute("full-path");
                if let Some(root) = attr {
                    return Ok(String::from(root));
                }
            }
        }

        Err(EpubError::RootfileNotFound)
    }
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
    pub media_type: String,
    pub content: Option<Rc<Vec<u8>>>,
}
