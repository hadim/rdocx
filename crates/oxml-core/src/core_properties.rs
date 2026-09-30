//! Dublin Core metadata from `docProps/core.xml`.

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::error::Result;
use crate::xml::local_name;

/// Document metadata from `docProps/core.xml` (Dublin Core).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CoreProperties {
    /// Document title (`dc:title`).
    pub title: Option<String>,
    /// Document creator/author (`dc:creator`).
    pub creator: Option<String>,
    /// Subject (`dc:subject`).
    pub subject: Option<String>,
    /// Description/comments (`dc:description`).
    pub description: Option<String>,
    /// Keywords (`cp:keywords`).
    pub keywords: Option<String>,
    /// Last modified by (`cp:lastModifiedBy`).
    pub last_modified_by: Option<String>,
    /// Date created (`dcterms:created`).
    pub created: Option<String>,
    /// Date modified (`dcterms:modified`).
    pub modified: Option<String>,
    /// Category (`cp:category`).
    pub category: Option<String>,
    /// Content status, such as "Draft" (`cp:contentStatus`).
    pub content_status: Option<String>,
    /// Identifier (`dc:identifier`).
    pub identifier: Option<String>,
    /// Language (`dc:language`).
    pub language: Option<String>,
    /// Date last printed (`cp:lastPrinted`).
    pub last_printed: Option<String>,
    /// Revision number (`cp:revision`).
    pub revision: Option<String>,
    /// Version (`cp:version`).
    pub version: Option<String>,
}

impl CoreProperties {
    /// Parse `docProps/core.xml` content.
    pub fn from_xml(xml: &[u8]) -> Result<Self> {
        let mut reader = Reader::from_reader(xml);
        reader.config_mut().trim_text(true);

        let mut props = CoreProperties::default();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let field = match local_name(name.as_ref()) {
                        b"title" => &mut props.title,
                        b"creator" => &mut props.creator,
                        b"subject" => &mut props.subject,
                        b"description" => &mut props.description,
                        b"keywords" => &mut props.keywords,
                        b"lastModifiedBy" => &mut props.last_modified_by,
                        b"created" => &mut props.created,
                        b"modified" => &mut props.modified,
                        b"category" => &mut props.category,
                        b"contentStatus" => &mut props.content_status,
                        b"identifier" => &mut props.identifier,
                        b"language" => &mut props.language,
                        b"lastPrinted" => &mut props.last_printed,
                        b"revision" => &mut props.revision,
                        b"version" => &mut props.version,
                        _ => {
                            buf.clear();
                            continue;
                        }
                    };
                    // Consume the whole element: a value containing an entity
                    // arrives as several events, so a single Text event is not
                    // enough to reconstruct it.
                    let text = crate::xml_text::read_element_text(&mut reader, name);
                    if !text.is_empty() {
                        *field = Some(text);
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.into()),
                _ => {}
            }
            buf.clear();
        }

        Ok(props)
    }

    /// Serialize to `docProps/core.xml` bytes.
    pub fn to_xml(&self) -> Result<Vec<u8>> {
        use quick_xml::Writer;
        use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText};

        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        writer.write_event(Event::Decl(BytesDecl::new(
            "1.0",
            Some("UTF-8"),
            Some("yes"),
        )))?;

        let mut root = BytesStart::new("cp:coreProperties");
        root.push_attribute((
            "xmlns:cp",
            "http://schemas.openxmlformats.org/package/2006/metadata/core-properties",
        ));
        root.push_attribute(("xmlns:dc", "http://purl.org/dc/elements/1.1/"));
        root.push_attribute(("xmlns:dcterms", "http://purl.org/dc/terms/"));
        root.push_attribute(("xmlns:dcmitype", "http://purl.org/dc/dcmitype/"));
        root.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
        writer.write_event(Event::Start(root))?;

        fn write_element<W: std::io::Write>(
            writer: &mut Writer<W>,
            tag: &str,
            value: &Option<String>,
        ) -> Result<()> {
            if let Some(val) = value {
                writer.write_event(Event::Start(BytesStart::new(tag)))?;
                writer.write_event(Event::Text(BytesText::new(val)))?;
                writer.write_event(Event::End(BytesEnd::new(tag)))?;
            }
            Ok(())
        }

        fn write_date_element<W: std::io::Write>(
            writer: &mut Writer<W>,
            tag: &str,
            value: &Option<String>,
        ) -> Result<()> {
            if let Some(val) = value {
                let mut e = BytesStart::new(tag);
                e.push_attribute(("xsi:type", "dcterms:W3CDTF"));
                writer.write_event(Event::Start(e))?;
                writer.write_event(Event::Text(BytesText::new(val)))?;
                writer.write_event(Event::End(BytesEnd::new(tag)))?;
            }
            Ok(())
        }

        write_element(&mut writer, "dc:title", &self.title)?;
        write_element(&mut writer, "dc:subject", &self.subject)?;
        write_element(&mut writer, "dc:creator", &self.creator)?;
        write_element(&mut writer, "cp:keywords", &self.keywords)?;
        write_element(&mut writer, "dc:description", &self.description)?;
        write_element(&mut writer, "cp:lastModifiedBy", &self.last_modified_by)?;
        write_element(&mut writer, "cp:revision", &self.revision)?;
        write_element(&mut writer, "cp:lastPrinted", &self.last_printed)?;
        write_date_element(&mut writer, "dcterms:created", &self.created)?;
        write_date_element(&mut writer, "dcterms:modified", &self.modified)?;
        write_element(&mut writer, "cp:category", &self.category)?;
        write_element(&mut writer, "cp:contentStatus", &self.content_status)?;
        write_element(&mut writer, "dc:identifier", &self.identifier)?;
        write_element(&mut writer, "dc:language", &self.language)?;
        write_element(&mut writer, "cp:version", &self.version)?;

        writer.write_event(Event::End(BytesEnd::new("cp:coreProperties")))?;

        Ok(writer.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_core_properties() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
                   xmlns:dc="http://purl.org/dc/elements/1.1/"
                   xmlns:dcterms="http://purl.org/dc/terms/"
                   xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <dc:title>Test Document</dc:title>
  <dc:creator>John Doe</dc:creator>
  <dc:subject>Testing</dc:subject>
  <dc:description>A test document</dc:description>
  <cp:keywords>test, document</cp:keywords>
  <cp:lastModifiedBy>Jane Doe</cp:lastModifiedBy>
  <dcterms:created xsi:type="dcterms:W3CDTF">2024-01-15T10:30:00Z</dcterms:created>
  <dcterms:modified xsi:type="dcterms:W3CDTF">2024-06-20T14:00:00Z</dcterms:modified>
  <cp:category>Reports</cp:category>
  <cp:contentStatus>Draft</cp:contentStatus>
  <dc:identifier>DOC-7</dc:identifier>
  <dc:language>en-GB</dc:language>
  <cp:lastPrinted>2024-06-21T09:00:00Z</cp:lastPrinted>
  <cp:revision>12</cp:revision>
  <cp:version>1.4</cp:version>
</cp:coreProperties>"#;

        let props = CoreProperties::from_xml(xml).unwrap();
        assert_eq!(props.title, Some("Test Document".to_string()));
        assert_eq!(props.creator, Some("John Doe".to_string()));
        assert_eq!(props.subject, Some("Testing".to_string()));
        assert_eq!(props.description, Some("A test document".to_string()));
        assert_eq!(props.keywords, Some("test, document".to_string()));
        assert_eq!(props.last_modified_by, Some("Jane Doe".to_string()));
        assert_eq!(props.created, Some("2024-01-15T10:30:00Z".to_string()));
        assert_eq!(props.modified, Some("2024-06-20T14:00:00Z".to_string()));
        assert_eq!(props.category.as_deref(), Some("Reports"));
        assert_eq!(props.content_status.as_deref(), Some("Draft"));
        assert_eq!(props.identifier.as_deref(), Some("DOC-7"));
        assert_eq!(props.language.as_deref(), Some("en-GB"));
        assert_eq!(props.last_printed.as_deref(), Some("2024-06-21T09:00:00Z"));
        assert_eq!(props.revision.as_deref(), Some("12"));
        assert_eq!(props.version.as_deref(), Some("1.4"));
    }

    #[test]
    fn round_trip_core_properties() {
        let text = |value: &str| Some(value.to_owned());
        let props = CoreProperties {
            title: text("My Title"),
            creator: text("Author"),
            subject: None,
            description: None,
            keywords: text("rust, docx"),
            last_modified_by: None,
            created: text("2024-01-01T00:00:00Z"),
            modified: text("2024-06-01T00:00:00Z"),
            category: text("Reports"),
            content_status: text("Final"),
            identifier: text("DOC-7"),
            language: text("fr-FR"),
            last_printed: text("2024-06-02T00:00:00Z"),
            revision: text("3"),
            version: text("2.0"),
        };

        let xml = props.to_xml().unwrap();
        let parsed = CoreProperties::from_xml(&xml).unwrap();

        assert_eq!(parsed, props);
        let xml = String::from_utf8(xml).unwrap();
        assert!(xml.contains("<cp:lastPrinted>2024-06-02T00:00:00Z</cp:lastPrinted>"));
        assert!(xml.contains("<dc:language>fr-FR</dc:language>"));
    }

    #[test]
    fn unset_added_properties_leave_the_serialized_part_unchanged() {
        let props = CoreProperties {
            title: Some("Title".to_owned()),
            created: Some("2024-01-01T00:00:00Z".to_owned()),
            ..CoreProperties::default()
        };

        let xml = String::from_utf8(props.to_xml().unwrap()).unwrap();

        assert_eq!(
            xml.lines().map(str::trim).collect::<String>(),
            concat!(
                r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#,
                r#"<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:dcterms="http://purl.org/dc/terms/" xmlns:dcmitype="http://purl.org/dc/dcmitype/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">"#,
                "<dc:title>Title</dc:title>",
                r#"<dcterms:created xsi:type="dcterms:W3CDTF">2024-01-01T00:00:00Z</dcterms:created>"#,
                "</cp:coreProperties>",
            )
        );
    }
}
