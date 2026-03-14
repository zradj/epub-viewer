use std::{collections::HashMap, io::{BufReader, Read, Seek}};

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
    pub fn new(reader: impl Read + Seek) -> Result<EpubBook, EpubError> {
        let mut zip_archive = ZipArchive::new(reader)?;
        let container = zip_archive.by_name("META-INF/container.xml")?;

        let root_file_name = get_root_file_name_from_container(container)?;

        println!("{root_file_name}");
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

pub fn get_root_file_name_from_container<R>(container: ZipFile<'_, R>) -> Result<String, EpubError>
where R: Read
{
    let buf_reader = BufReader::new(container);
    let mut reader = Reader::from_reader(buf_reader);
    let mut buf = vec![];

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                if e.name().as_ref() == b"rootfile" {
                    for attr in e.attributes() {
                        let attr = attr.map_err(|e| EpubError::Xml(e.into()))?;
                        if attr.key.local_name().as_ref() == b"full-path" {
                            let path = attr
                                .decode_and_unescape_value(reader.decoder())?
                                .into_owned();

                            return Ok(path);
                        }
                    }
                }
            },
            _ => (),
        }
    }

    Err(EpubError::RootfileNotFound)
}
