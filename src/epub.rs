use std::{cell::RefCell, collections::HashMap, fs::File, io::Read, rc::Rc};

use zip::ZipArchive;

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

        let root_path = {
            let mut container_file = archive.by_name("META-INF/container.xml")?;
            let mut container_content = String::new();
            container_file.read_to_string(&mut container_content)?;

            let container_doc = roxmltree::Document::parse(&container_content)?;
            Self::extract_root_path(&container_doc)?
        };

        let mut root_file = archive.by_name(&root_path)?;
        let mut root_content = String::new();
        root_file.read_to_string(&mut root_content)?;

        let mut metadata = EpubMetadata::default();

        let root_doc = roxmltree::Document::parse(&root_content)?;
        for child in root_doc.root_element().children() {
            match child.tag_name().name() {
                "metadata" => metadata = EpubMetadata::from(&child),
                "manifest" => 
                _ => (),
            }
        }

        todo!();
    }

    fn extract_root_path(container: &roxmltree::Document) -> EpubResult<String> {
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

    fn read_manifest(
        xml_manifest: &roxmltree::Node<'_, '_>,
    ) -> EpubResult<RefCell<HashMap<String, EpubResource>>> {
        let mut res = HashMap::new();

        for child in xml_manifest.children() {
            if child.tag_name().name() == "item" {
                // TODO: implement error tolerance
                let href = child.attribute("href").ok_or(
                    EpubError::MissingAttribute { attr: "href" }
                )?;
                let id = child.attribute("id").ok_or(EpubError::MissingAttribute { attr: "id" })?;
                let media_type = child.attribute("media-type").ok_or(
                    EpubError::MissingAttribute { attr: "media-type" }
                )?;

                let href = String::from(href);
                let id = String::from(id);
                let media_type = String::from(media_type);

                res.insert(id, EpubResource { path: href, media_type, content: None });
            }
        }

        Ok(RefCell::new(res))
    }
}

#[derive(Debug, Default)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
}

impl EpubMetadata {
    pub fn from(xml_metadata: &roxmltree::Node<'_, '_>) -> Self {
        let mut res = Self::default();

        for child in xml_metadata.children() {
            match child.tag_name().name() {
                "title" => {
                    if let Some(title) = child.text() {
                        res.title = Some(String::from(title));
                    }
                },
                "creator" => {
                    if let Some(author) = child.text() {
                        res.author = Some(String::from(author));
                    }
                },
                "language" => {
                    if let Some(lang) = child.text() {
                        res.language = Some(String::from(lang));
                    }
                },
                _ => (),
            }
        }

        res
    }
}

#[derive(Debug)]
pub struct EpubResource {
    pub path: String,
    pub media_type: String,
    pub content: Option<Rc<Vec<u8>>>,
}
