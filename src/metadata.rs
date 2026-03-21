use crate::error::MetadataError;

#[derive(Debug, Default)]
pub struct EpubLenientMetadata {
    pub identifiers: Vec<String>,
    pub titles: Vec<String>,
    pub subjects: Vec<String>,
    pub types: Vec<String>,
    pub creators: Vec<String>,
    pub contributors: Vec<String>,
    pub languages: Vec<String>,
    pub publishers: Vec<String>,
    pub dates: Vec<String>,
    pub last_modified: Option<String>,
    pub parse_warnings: Vec<MetadataError>,
}

impl From<&roxmltree::Node<'_, '_>> for EpubLenientMetadata {
    fn from(xml_metadata: &roxmltree::Node) -> Self {
        let mut res = Self::default();

        fn push_trim(vec: &mut Vec<String>, node: &roxmltree::Node) {
            if let Some(text) = node.text() {
                vec.push(String::from(text.trim()));
            }
        }

        for child in xml_metadata.children() {
            if matches!(
                child.tag_name().namespace(),
                Some("http://purl.org/dc/elements/1.1/")
            ) {
                match child.tag_name().name() {
                    "identifier" => push_trim(&mut res.identifiers, &child),
                    "title" => push_trim(&mut res.titles, &child),
                    "subject" => push_trim(&mut res.subjects, &child),
                    "type" => push_trim(&mut res.types, &child),
                    "contributor" => push_trim(&mut res.contributors, &child),
                    "creator" => push_trim(&mut res.creators, &child),
                    "language" => push_trim(&mut res.languages, &child),
                    "publisher" => push_trim(&mut res.publishers, &child),
                    "date" => {
                        if !res.dates.is_empty() {
                            res.parse_warnings.push(MetadataError::MultipleDates);
                        }

                        push_trim(&mut res.dates, &child);
                    }
                    _ => (),
                }
            } else {
                if child.tag_name().name() == "meta" {
                    if matches!(child.attribute("property"), Some("dcterms:modified")) {
                        if res.last_modified.is_some() {
                            res.parse_warnings.push(MetadataError::MultipleLastModified);
                        }
                        res.last_modified = child.text().map(|lm| String::from(lm.trim()));
                    }
                }
            }
        }

        if res.identifiers.is_empty() {
            res.parse_warnings.push(MetadataError::NoIdentifier);
        }

        if res.titles.is_empty() {
            res.parse_warnings.push(MetadataError::NoTitle);
        }

        if res.languages.is_empty() {
            res.parse_warnings.push(MetadataError::NoLanguage);
        }

        if res.last_modified.is_none() {
            res.parse_warnings.push(MetadataError::NoLastModified);
        }

        res
    }
}

pub struct EpubStrictMetadata {
    pub identifiers: Vec<String>,
    pub titles: Vec<String>,
    pub subjects: Vec<String>,
    pub types: Vec<String>,
    pub creators: Vec<String>,
    pub contributors: Vec<String>,
    pub languages: Vec<String>,
    pub publishers: Vec<String>,
    pub date: Option<String>,
    pub last_modified: String,
}

impl TryFrom<EpubLenientMetadata> for EpubStrictMetadata {
    type Error = Vec<MetadataError>;

    fn try_from(lenient: EpubLenientMetadata) -> Result<Self, Self::Error> {
        if !lenient.parse_warnings.is_empty() {
            Err(lenient.parse_warnings)
        } else {
            Ok(EpubStrictMetadata {
                identifiers: lenient.identifiers,
                titles: lenient.titles,
                subjects: lenient.subjects,
                types: lenient.types,
                creators: lenient.creators,
                contributors: lenient.contributors,
                languages: lenient.languages,
                publishers: lenient.publishers,
                date: lenient.dates.first().cloned(),
                last_modified: lenient.last_modified.unwrap(),
            })
        }
    }
}

impl TryFrom<&roxmltree::Node<'_, '_>> for EpubStrictMetadata {
    type Error = Vec<MetadataError>;

    fn try_from(xml_metadata: &roxmltree::Node) -> Result<Self, Self::Error> {
        let lenient = EpubLenientMetadata::from(xml_metadata);
        Self::try_from(lenient)
    }
}
