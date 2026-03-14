use std::{collections::HashMap, error::Error, io::{Read, Seek}};

use quick_xml::{Reader, events::Event};
use zip::{ZipArchive, read::ZipFile};

use crate::error::EpubError;

#[derive(Debug)]
pub struct EpubBook {
    pub metadata: EpubMetadata,
    pub resources: HashMap<String, EpubResource>,
    pub spine: Vec<String>,
}

impl EpubBook {
    pub fn new(reader: impl Read + Seek) -> Result<EpubBook, Box<dyn Error>> {
        let mut zip_archive = ZipArchive::new(reader)?;
        let mut container = zip_archive.by_name("META-INF/container.xml")?;

        let mut contents = String::new();
        container.read_to_string(&mut contents)?;

        println!("{}", contents);
        todo!();
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
    pub mime_type: String,
    pub content: Vec<u8>,
}

fn get_root_file_name_from_container<R>(mut container: ZipFile<'_, R>) -> Result<String, EpubError>
where R: Read
{
    let mut content = String::new();
    container.read_to_string(&mut content)?;

    let mut reader = Reader::from_str(&content);
    loop {
        match reader.read_event() {
            Err(e) => return Err(EpubError::Xml(e)),
            Ok(Event::Eof) => return Err(EpubError::RootfileNotFound),
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                if e.name().as_ref() == b"rootfile" {
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| EpubError::Xml(quick_xml::Error::InvalidAttr(e)))?;
                        if attr.key.local_name().as_ref() == b"full-path" {
                            return Ok(
                                str::from_utf8(&attr.value)
                                    .map_err(|e| EpubError::Utf8 { context: "rootfile path", source: e })?
                                    .to_string()
                            )
                        }
                    }

                    return Err(EpubError::RootfileNotFound);
                }
            },
            _ => (),
        }
    }
}
