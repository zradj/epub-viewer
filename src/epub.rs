use std::{
    collections::HashMap, convert::Infallible, fmt, fs::File, io::Read, path::Path, str::FromStr,
};

use zip::{ZipArchive, read::ZipFile};

use crate::error::{EpubError, EpubResult};

#[derive(Debug)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub spine: Vec<SpineItem>,
    resources: HashMap<String, EpubResource>,
}

impl EpubBook {
    pub fn new<P: AsRef<Path>>(path: P) -> EpubResult<Self> {
        let archive = File::open(path)?;
        let mut archive = ZipArchive::new(archive)?;

        let _ = archive
            .by_name("mimetype")
            .map_err(EpubError::from)
            .and_then(|mut f| Self::verify_mimetype(&mut f))
            .inspect_err(|e| eprintln!("[Warning] MIME verification error: {e}"));

        let package_path = {
            let mut container_file = archive.by_name("META-INF/container.xml")?;
            let mut container_content = String::new();
            container_file.read_to_string(&mut container_content)?;

            let container_doc = roxmltree::Document::parse(&container_content)?;
            Self::extract_package_path(&container_doc)?
        };

        let base_path = match package_path.rfind('/') {
            Some(i) => &package_path[..=i],
            None => "",
        };

        let root_content = {
            let mut root_file = archive.by_name(&package_path)?;
            let mut root_content = String::new();
            root_file.read_to_string(&mut root_content)?;

            root_content
        };

        let root_doc = roxmltree::Document::parse(&root_content)?;
        let root_element = root_doc.root_element();

        let mut metadata = EpubMetadata::default();
        if let Some(meta_node) = root_element.children().find(|c| c.has_tag_name("metadata")) {
            metadata = EpubMetadata::from(&meta_node);
        }

        let manifest_node = root_element
            .children()
            .find(|c| c.has_tag_name("manifest"))
            .ok_or(EpubError::MissingAttribute {
                attr: "manifest",
                loc: "package document",
            })?;

        let (resources, id_to_path) = Self::read_manifest(&mut archive, &manifest_node, base_path)?;

        let spine_node = root_element
            .children()
            .find(|c| c.has_tag_name("spine"))
            .ok_or(EpubError::MissingAttribute {
                attr: "spine",
                loc: "package document",
            })?;

        let spine = Self::read_spine(&spine_node, &id_to_path)?;

        Ok(EpubBook {
            metadata,
            spine,
            resources,
        })
    }

    pub fn resource(&self, path: &str) -> EpubResult<&EpubResource> {
        self.resources
            .get(path)
            .ok_or(EpubError::ResourceNotFound(String::from(path)))
    }

    pub fn relative_resource(
        &self,
        current_doc_path: &str,
        href: &str,
    ) -> EpubResult<&EpubResource> {
        if href.starts_with('/') || href.contains("://") {
            return self.resource(href);
        }

        let base_dir = match current_doc_path.rfind('/') {
            Some(i) => &current_doc_path[..=i],
            None => "",
        };

        let path = normalize_zip_path(base_dir, href);

        self.resource(&path)
    }

    fn verify_mimetype(mime_file: &mut ZipFile<'_, File>) -> EpubResult<()> {
        let mut content = String::new();
        mime_file.read_to_string(&mut content)?;

        if content.trim() != "application/epub+zip" {
            return Err(EpubError::IncorrectMimeType);
        }

        Ok(())
    }

    fn extract_package_path(container: &roxmltree::Document) -> EpubResult<String> {
        for desc in container.descendants() {
            if desc.tag_name().name() == "rootfile" {
                let attr = desc.attribute("full-path");
                if let Some(root) = attr {
                    return Ok(String::from(root));
                }
            }
        }

        Err(EpubError::PackageNotFound)
    }

    fn read_manifest(
        archive: &mut ZipArchive<File>,
        xml_manifest: &roxmltree::Node<'_, '_>,
        base_path: &str,
    ) -> EpubResult<(HashMap<String, EpubResource>, HashMap<String, String>)> {
        let mut resources = HashMap::new();
        let mut id_to_path = HashMap::new();

        for child in xml_manifest.children() {
            if child.tag_name().name() == "item" {
                // TODO: implement error tolerance
                let get_attr = |attr| {
                    child.attribute(attr).ok_or(EpubError::MissingAttribute {
                        attr,
                        loc: "manifest item",
                    })
                };

                let path = normalize_zip_path(base_path, get_attr("href")?);
                let id = String::from(get_attr("id")?);
                let media_type = String::from(get_attr("media-type")?);

                let mut item_file = archive.by_name(&path)?;
                let mut buf = vec![];
                item_file.read_to_end(&mut buf)?;

                id_to_path.insert(id, path.clone());

                resources.insert(
                    path,
                    EpubResource {
                        media_type: media_type.parse().unwrap(),
                        content: buf,
                    },
                );
            }
        }

        Ok((resources, id_to_path))
    }

    fn read_spine(
        xml_spine: &roxmltree::Node<'_, '_>,
        id_to_path: &HashMap<String, String>,
    ) -> EpubResult<Vec<SpineItem>> {
        let mut res = vec![];

        for child in xml_spine.children() {
            if child.tag_name().name() == "itemref" {
                let idref = child
                    .attribute("idref")
                    .ok_or(EpubError::MissingAttribute {
                        attr: "idref",
                        loc: "spine item",
                    })?;

                let path = id_to_path
                    .get(idref)
                    .ok_or(EpubError::ResourceNotFound(String::from(idref)))?
                    .clone();

                let linear = !matches!(child.attribute("linear"), Some("no"));

                let properties = child.attribute("properties").map(String::from);

                res.push(SpineItem {
                    path,
                    linear,
                    properties,
                });
            }
        }

        Ok(res)
    }
}

#[derive(Debug, Default)]
pub struct EpubMetadata {
    pub title: Option<String>,
    pub authors: Vec<String>,
    pub language: Option<String>,
}

impl From<&roxmltree::Node<'_, '_>> for EpubMetadata {
    fn from(xml_metadata: &roxmltree::Node<'_, '_>) -> Self {
        let mut res = Self::default();

        for child in xml_metadata.children() {
            match child.tag_name().name() {
                "title" => res.title = child.text().map(String::from),
                "creator" => {
                    if let Some(author) = child.text() {
                        res.authors.push(String::from(author));
                    }
                }
                "language" => res.language = child.text().map(String::from),
                _ => (),
            }
        }

        res
    }
}

#[derive(Debug)]
pub struct EpubResource {
    pub media_type: MediaType,
    pub content: Vec<u8>,
}

impl EpubResource {
    pub fn is_text(&self) -> bool {
        match &self.media_type {
            MediaType::Xhtml
            | MediaType::Css
            | MediaType::Js
            | MediaType::ImageSvg
            | MediaType::LegacyNcx
            | MediaType::MediaOverlay => true,
            MediaType::Other(s) => s.starts_with("text/"),
            _ => false,
        }
    }

    pub fn content_str(&self) -> EpubResult<&str> {
        if self.is_text() {
            Ok(std::str::from_utf8(&self.content)?)
        } else {
            Err(EpubError::NotTextContent)
        }
    }

    pub fn content_str_lossy(&self) -> EpubResult<String> {
        if self.is_text() {
            Ok(String::from_utf8_lossy(&self.content).into_owned())
        } else {
            Err(EpubError::NotTextContent)
        }
    }
}

#[derive(Debug)]
pub enum MediaType {
    Xhtml,
    Css,
    Js,
    ImageGif,
    ImageJpeg,
    ImagePng,
    ImageSvg,
    ImageWebp,
    AudioMpeg,
    AudioMp4,
    AudioOgg,
    FontTtf,
    FontOtf,
    FontSfnt,
    FontWoff,
    FontWoff2,
    LegacyNcx,
    MediaOverlay,
    Other(String),
}

impl FromStr for MediaType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "application/xhtml+xml" => Self::Xhtml,
            "text/css" => Self::Css,
            "application/javascript" | "application/ecmascript" | "text/javascript" => Self::Js,
            "image/gif" => Self::ImageGif,
            "image/jpeg" => Self::ImageJpeg,
            "image/png" => Self::ImagePng,
            "image/svg+xml" => Self::ImageSvg,
            "image/webp" => Self::ImageWebp,
            "audio/mpeg" => Self::AudioMpeg,
            "audio/mp4" => Self::AudioMp4,
            "audio/ogg" | "audio/ogg; codecs=opus" => Self::AudioOgg,
            "font/ttf" => Self::FontTtf,
            "font/otf" | "application/vnd.ms-opentype" => Self::FontOtf,
            "application/font-sfnt" => Self::FontSfnt,
            "font/woff" | "application/font-woff" => Self::FontWoff,
            "font/woff2" => Self::FontWoff2,
            "application/x-dtbncx+xml" => Self::LegacyNcx,
            "application/smil+xml" => Self::MediaOverlay,
            _ => Self::Other(String::from(s)),
        })
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Xhtml => "application/xhtml+xml",
            Self::Css => "text/css",
            Self::Js => "application/javascript",
            Self::ImageGif => "image/gif",
            Self::ImageJpeg => "image/jpeg",
            Self::ImagePng => "image/png",
            Self::ImageSvg => "image/svg+xml",
            Self::ImageWebp => "image/webp",
            Self::AudioMpeg => "audio/mpeg",
            Self::AudioMp4 => "audio/mp4",
            Self::AudioOgg => "audio/ogg; codecs=opus",
            Self::FontTtf => "font/ttf",
            Self::FontOtf => "font/otf",
            Self::FontSfnt => "application/font-sfnt",
            Self::FontWoff => "font/woff",
            Self::FontWoff2 => "font/woff2",
            Self::LegacyNcx => "application/x-dtbncx+xml",
            Self::MediaOverlay => "application/smil+xml",
            Self::Other(s) => s,
        };

        f.write_str(s)
    }
}

#[derive(Debug)]
pub struct SpineItem {
    pub path: String,
    pub linear: bool,
    pub properties: Option<String>,
}

fn normalize_zip_path(base: &str, href: &str) -> String {
    let base_path = std::path::Path::new(base);
    let href_path = std::path::Path::new(href);
    let joined = base_path.join(href_path);

    let mut components: Vec<&std::ffi::OsStr> = vec![];
    for comp in joined.components() {
        match comp {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::Normal(s) if s != ".." && s != "." => components.push(s),
            _ => (),
        }
    }

    let normalized = components
        .iter()
        .map(|s| s.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");

    if normalized.is_empty() {
        String::from(".")
    } else {
        normalized
    }
}
