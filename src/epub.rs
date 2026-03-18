use std::{cell::RefCell, collections::HashMap, fs::File, io::Read, rc::Rc};

use zip::{ZipArchive, read::ZipFile};

use crate::error::{EpubError, EpubResult};

#[derive(Debug)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub spine: Vec<String>,
    resources: RefCell<HashMap<String, Rc<RefCell<EpubResource>>>>,
    archive: RefCell<ZipArchive<File>>,
}

impl EpubBook {
    pub fn new(path: &str) -> EpubResult<Self> {
        let archive = File::open(path)?;
        let mut archive = ZipArchive::new(archive)?;

        let mime_result = archive
            .by_name("mimetype")
            .map_err(EpubError::from)
            .and_then(|mut f| Self::verify_mimetype(&mut f));
        
        if let Err(e) = mime_result {
            eprintln!("[Warning] MIME verification: {e}");
        }

        let root_path = {
            let mut container_file = archive.by_name("META-INF/container.xml")?;
            let mut container_content = String::new();
            container_file.read_to_string(&mut container_content)?;

            let container_doc = roxmltree::Document::parse(&container_content)?;
            Self::extract_root_path(&container_doc)?
        };

        let base_path = match root_path.rfind('/') {
            Some(i) => &root_path[..=i],
            None => "",
        };

        let root_content = {
            let mut root_file = archive.by_name(&root_path)?;
            let mut root_content = String::new();
            root_file.read_to_string(&mut root_content)?;

            root_content
        };

        let mut metadata = EpubMetadata::default();
        let mut resources = HashMap::new();
        let mut spine = vec![];

        let root_doc = roxmltree::Document::parse(&root_content)?;
        for child in root_doc.root_element().children() {
            match child.tag_name().name() {
                "metadata" => metadata = EpubMetadata::from(&child),
                "manifest" => resources = Self::read_manifest(&child, base_path)?,
                // TODO: check "toc" atrribute
                "spine" => spine = Self::read_spine(&child)?,
                _ => (),
            }
        }

        Ok(EpubBook {
            metadata,
            spine,
            resources: RefCell::new(resources),
            archive: RefCell::new(archive),
        })
    }

    pub fn get_resource(&self, id: &str) -> EpubResult<Rc<RefCell<EpubResource>>> {
        let resources = self.resources.borrow_mut();
        let rc_resource = resources
            .get(id)
            .ok_or(EpubError::ResourceNotFound(String::from(id)))?;
        let mut resource = rc_resource.borrow_mut();

        if resource.content.is_none() {
            let mut archive = self.archive.borrow_mut();
            let mut resource_file = archive.by_name(&resource.path)?;

            let mut buf = vec![];
            resource_file.read_to_end(&mut buf)?;

            resource.content = Some(buf);
        }

        Ok(Rc::clone(rc_resource))
    }

    fn verify_mimetype(mime_file: &mut ZipFile<'_, File>) -> EpubResult<()> {
        let mut content = String::new();
        mime_file.read_to_string(&mut content)?;

        if content.trim() != "application/epub+zip" {
            return Err(EpubError::IncorrectMimeType);
        }

        Ok(())
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
        base_path: &str,
    ) -> EpubResult<HashMap<String, Rc<RefCell<EpubResource>>>> {
        let mut res = HashMap::new();

        for child in xml_manifest.children() {
            if child.tag_name().name() == "item" {
                // TODO: implement error tolerance
                let href = child.attribute("href").ok_or(
                    EpubError::MissingAttribute { attr: "href", loc: "manifest item" }
                )?;
                let id = child.attribute("id").ok_or(
                    EpubError::MissingAttribute { attr: "id", loc: "manifest item" }
                )?;
                let media_type = child.attribute("media-type").ok_or(
                    EpubError::MissingAttribute { attr: "media-type", loc: "manifest item" }
                )?;

                let path = format!("{base_path}{href}");
                let id = String::from(id);
                let media_type = String::from(media_type);

                res.insert(
                    id, Rc::new(RefCell::new(EpubResource { path, media_type, content: None }))
                );
            }
        }

        Ok(res)
    }

    fn read_spine(xml_spine: &roxmltree::Node<'_, '_>) -> EpubResult<Vec<String>> {
        let mut res = vec![];

        for child in xml_spine.children() {
            if child.tag_name().name() == "itemref" {
                let idref = child.attribute("idref").ok_or(
                    EpubError::MissingAttribute { attr: "idref", loc: "spine item" }
                )?;

                res.push(String::from(idref));
            }
        }

        Ok(res)
    }
}

#[derive(Debug, Default)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub authors: Option<Vec<String>>,
    pub language: Option<String>,
}

impl From<&roxmltree::Node<'_, '_>> for EpubMetadata {
    fn from(xml_metadata: &roxmltree::Node<'_, '_>) -> Self {
        let mut res = Self::default();
        let mut authors = vec![];

        for child in xml_metadata.children() {
            match child.tag_name().name() {
                "title" => res.title = child.text().map(String::from),
                "creator" => {
                    if let Some(author) = child.text() {
                        authors.push(String::from(author));
                    }
                },
                "language" => res.language = child.text().map(String::from),
                _ => (),
            }
        }

        if !authors.is_empty() {
            res.authors = Some(authors);
        }

        res
    }
}

#[derive(Debug)]
pub struct EpubResource {
    pub path: String,
    pub media_type: String,
    pub content: Option<Vec<u8>>,
}
