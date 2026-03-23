use crate::error::MetadataError;

#[derive(Debug, Default)]
pub struct LenientMetadata {
    pub identifiers: Vec<String>,
    pub titles: Vec<String>,
    pub languages: Vec<String>,
    pub contributors: Vec<String>,
    pub coverages: Vec<String>,
    pub creators: Vec<String>,
    pub dates: Vec<String>,
    pub descriptions: Vec<String>,
    pub formats: Vec<String>,
    pub publishers: Vec<String>,
    pub relations: Vec<String>,
    pub rights: Vec<String>,
    pub sources: Vec<String>,
    pub subjects: Vec<String>,
    pub types: Vec<String>,
    pub last_modified: Option<String>,
    pub cover_image: Option<String>,
    pub parse_warnings: Vec<MetadataError>,
}

impl From<&roxmltree::Node<'_, '_>> for LenientMetadata {
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
                    "language" => push_trim(&mut res.languages, &child),
                    "contributor" => push_trim(&mut res.contributors, &child),
                    "coverage" => push_trim(&mut res.coverages, &child),
                    "creator" => push_trim(&mut res.creators, &child),
                    "date" => {
                        if !res.dates.is_empty() {
                            res.parse_warnings.push(MetadataError::MultipleDates);
                        }

                        push_trim(&mut res.dates, &child);
                    }
                    "description" => push_trim(&mut res.descriptions, &child),
                    "format" => push_trim(&mut res.formats, &child),
                    "publisher" => push_trim(&mut res.publishers, &child),
                    "relation" => push_trim(&mut res.relations, &child),
                    "rights" => push_trim(&mut res.rights, &child),
                    "source" => push_trim(&mut res.sources, &child),
                    "subject" => push_trim(&mut res.subjects, &child),
                    "type" => push_trim(&mut res.types, &child),
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

                    if matches!(child.attribute("name"), Some("cover")) {
                        res.cover_image = child.attribute("content").map(String::from);
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

pub struct StrictMetadata {
    pub identifiers: Vec<String>,
    pub titles: Vec<String>,
    pub languages: Vec<String>,
    pub contributors: Vec<String>,
    pub coverages: Vec<String>,
    pub creators: Vec<String>,
    pub date: Option<String>,
    pub descriptions: Vec<String>,
    pub formats: Vec<String>,
    pub publishers: Vec<String>,
    pub relations: Vec<String>,
    pub rights: Vec<String>,
    pub sources: Vec<String>,
    pub subjects: Vec<String>,
    pub types: Vec<String>,
    pub last_modified: String,
    pub cover_image: Option<String>,
}

impl TryFrom<LenientMetadata> for StrictMetadata {
    type Error = Vec<MetadataError>;

    fn try_from(mut lenient: LenientMetadata) -> Result<Self, Self::Error> {
        if !lenient.parse_warnings.is_empty() {
            Err(lenient.parse_warnings)
        } else {
            Ok(StrictMetadata {
                identifiers: lenient.identifiers,
                titles: lenient.titles,
                languages: lenient.languages,
                contributors: lenient.contributors,
                coverages: lenient.coverages,
                creators: lenient.creators,
                date: lenient.dates.pop(),
                descriptions: lenient.descriptions,
                formats: lenient.formats,
                publishers: lenient.publishers,
                relations: lenient.relations,
                rights: lenient.rights,
                sources: lenient.sources,
                subjects: lenient.subjects,
                types: lenient.types,
                cover_image: lenient.cover_image,
                last_modified: lenient.last_modified.unwrap(),
            })
        }
    }
}

impl TryFrom<&roxmltree::Node<'_, '_>> for StrictMetadata {
    type Error = Vec<MetadataError>;

    fn try_from(xml_metadata: &roxmltree::Node) -> Result<Self, Self::Error> {
        let lenient = LenientMetadata::from(xml_metadata);
        Self::try_from(lenient)
    }
}
