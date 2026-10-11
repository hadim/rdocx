use std::collections::HashMap;

use oxml_core::OxmlError;
use oxml_drawing::namespace::A_NS;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

/// PresentationML main namespace URI.
pub const P_NS: &str = "http://schemas.openxmlformats.org/presentationml/2006/main";

/// Fixed PresentationML prefix used when writing XML.
pub const P_PREFIX: &str = "p";

/// Office document relationships namespace URI.
pub const R_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// Fixed office document relationships prefix used when writing XML.
pub const R_PREFIX: &str = "r";

/// Markup Compatibility namespace URI.
pub const MC_NS: &str = "http://schemas.openxmlformats.org/markup-compatibility/2006";

pub(crate) const FIXED_MODEL_PREFIXES: &[&str] = &["p", "a", "r"];
pub(crate) const FIXED_SHAPE_TREE_PREFIXES: &[&str] = &["p", "a", "r", "mc"];

/// Deepest nesting the readers accept for the recursive structures of a
/// slide: groups and alternate content in a shape tree, and time node lists
/// in `p:timing`.
///
/// PowerPoint nests these a handful of levels deep. The parsers, the
/// serialisers and the walkers of the layout and render crates recurse once
/// per level, so a bound far above anything a producer writes keeps every
/// one of them within the stack. At this depth a debug build opens, renders
/// and saves a slide in about 1.7 MB of stack, little more than a flat
/// slide needs, which fits the 2 MB default of spawned Rust threads. At 32
/// levels it took close to 2 MB.
pub const MAX_NESTING: usize = 16;

/// Whether elements named in `locals` nest more than `limit` deep along any
/// path in `xml`, found without recursing.
///
/// An element counts whatever its namespace, which can only over-count.
/// Malformed XML counts up to the error, so the parser still reports it.
pub(crate) fn nesting_exceeds(xml: &[u8], locals: &[&[u8]], limit: usize) -> bool {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut open_counts = Vec::new();
    let mut depth = 0usize;
    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(start)) => {
                let name = start.name();
                let local = name.as_ref().rsplit(|byte| *byte == b':').next();
                let counts = locals.iter().any(|candidate| local == Some(*candidate));
                if counts {
                    depth += 1;
                    if depth > limit {
                        return true;
                    }
                }
                open_counts.push(counts);
            }
            Ok(Event::End(_)) => {
                if open_counts.pop() == Some(true) {
                    depth -= 1;
                }
            }
            Ok(Event::Eof) | Err(_) => return false,
            Ok(_) => {}
        }
        buffer.clear();
    }
}

/// The error for XML that [`nesting_exceeds`] rejects.
pub(crate) fn nesting_error(what: &str) -> OxmlError {
    OxmlError::InvalidValue(format!("{what} nesting exceeds {MAX_NESTING} levels"))
}

#[derive(Clone, Debug, Default)]
pub(crate) struct NamespaceBindings {
    default: Option<String>,
    prefixes: HashMap<String, String>,
}

impl NamespaceBindings {
    pub(crate) fn from_entries(entries: &[(String, String)]) -> Self {
        let mut bindings = Self::default();
        for (prefix, uri) in entries {
            if prefix.is_empty() {
                bindings.default = Some(uri.clone());
            } else {
                bindings.prefixes.insert(prefix.clone(), uri.clone());
            }
        }
        bindings
    }

    pub(crate) fn with_start(&self, start: &BytesStart<'_>) -> Result<Self, OxmlError> {
        let mut bindings = self.clone();
        for (name, value) in all_attributes(start)? {
            if name == "xmlns" {
                bindings.default = Some(value);
            } else if let Some(prefix) = name.strip_prefix("xmlns:") {
                bindings.prefixes.insert(prefix.to_owned(), value);
            }
        }
        Ok(bindings)
    }

    pub(crate) fn element_uri<'a>(&'a self, name: &[u8]) -> Option<&'a str> {
        match qname_prefix(name) {
            Some(prefix) => self.prefixes.get(prefix).map(String::as_str),
            None => self.default.as_deref(),
        }
    }

    pub(crate) fn attribute_uri<'a>(&'a self, name: &[u8]) -> Option<&'a str> {
        qname_prefix(name).and_then(|prefix| self.prefixes.get(prefix).map(String::as_str))
    }

    pub(crate) fn entries(&self) -> Vec<(String, String)> {
        let mut entries: Vec<_> = self
            .prefixes
            .iter()
            .map(|(prefix, uri)| (prefix.clone(), uri.clone()))
            .collect();
        sort_namespace_entries(&mut entries);
        if let Some(uri) = &self.default {
            entries.push((String::new(), uri.clone()));
        }
        entries
    }

    pub(crate) fn reject_writer_conflicts(&self, fixed_prefixes: &[&str]) -> Result<(), OxmlError> {
        for prefix in fixed_prefixes {
            let expected = canonical_uri(prefix).expect("fixed prefixes have canonical URIs");
            if let Some(actual) = self.prefixes.get(*prefix)
                && actual != expected
            {
                return Err(OxmlError::InvalidValue(format!(
                    "xmlns:{prefix} conflicts with the fixed writer namespace"
                )));
            }
        }
        Ok(())
    }
}

pub(crate) fn all_attributes(start: &BytesStart<'_>) -> Result<Vec<(String, String)>, OxmlError> {
    let mut attributes = Vec::new();
    for attribute in start.attributes() {
        let attribute = attribute?;
        let name = std::str::from_utf8(attribute.key.as_ref())?.to_owned();
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, start.decoder())?
            .replace(['\r', '\n', '\t'], " ");
        attributes.push((name, value));
    }
    Ok(attributes)
}

pub(crate) fn non_visual_drawing_id(xml: &[u8]) -> Option<u32> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    loop {
        match reader.read_event_into(&mut buffer).ok()? {
            Event::Start(start) | Event::Empty(start) => {
                return all_attributes(&start)
                    .ok()?
                    .into_iter()
                    .find(|(name, _)| name == "id")
                    .and_then(|(_, value)| value.parse().ok());
            }
            Event::Eof => return None,
            _ => {}
        }
        buffer.clear();
    }
}

pub(crate) fn non_visual_drawing_name(start: &BytesStart<'_>) -> Result<Option<String>, OxmlError> {
    Ok(all_attributes(start)?
        .into_iter()
        .find(|(name, _)| name == "name")
        .map(|(_, value)| value))
}

pub(crate) fn set_non_visual_drawing_name(xml: &mut Vec<u8>, name: &str) -> Result<(), OxmlError> {
    oxml_core::xml::reject_non_xml_characters("shape name", name)?;
    let mut reader = Reader::from_reader(xml.as_slice());
    let mut buffer = Vec::new();
    loop {
        let event = reader.read_event_into(&mut buffer)?;
        let is_empty = matches!(&event, Event::Empty(_));
        match event {
            Event::Start(start) | Event::Empty(start) => {
                let end = reader.buffer_position() as usize;
                let qualified_name = std::str::from_utf8(start.name().as_ref())?.to_owned();
                let mut replacement = BytesStart::new(qualified_name);
                let mut replaced = false;
                for attribute in start.attributes().with_checks(false) {
                    let attribute = attribute?;
                    if attribute.key.as_ref() == b"name" {
                        replacement.push_attribute(("name", name));
                        replaced = true;
                    } else {
                        replacement.push_attribute(attribute);
                    }
                }
                if !replaced {
                    replacement.push_attribute(("name", name));
                }
                let mut writer = Writer::new(Vec::new());
                if is_empty {
                    writer.write_event(Event::Empty(replacement))?;
                } else {
                    writer.write_event(Event::Start(replacement))?;
                    writer.get_mut().extend_from_slice(&xml[end..]);
                }
                *xml = writer.into_inner();
                return Ok(());
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "non-visual drawing properties".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

pub(crate) fn root_attributes(
    start: &BytesStart<'_>,
    fixed_prefixes: &[&str],
) -> Result<Vec<(String, String)>, OxmlError> {
    Ok(all_attributes(start)?
        .into_iter()
        .filter(|(name, _)| !is_fixed_xmlns(name, fixed_prefixes))
        .collect())
}

pub(crate) fn self_contained_attributes(
    start: &BytesStart<'_>,
    fixed_prefixes: &[&str],
    inherited: &[(String, String)],
) -> Result<Vec<(String, String)>, OxmlError> {
    let mut attributes = root_attributes(start, fixed_prefixes)?;
    for (prefix, uri) in inherited {
        let name = if prefix.is_empty() {
            "xmlns".to_owned()
        } else {
            format!("xmlns:{prefix}")
        };
        if is_fixed_xmlns(&name, fixed_prefixes)
            || attributes.iter().any(|(existing, _)| existing == &name)
        {
            continue;
        }
        attributes.push((name, uri.clone()));
    }
    Ok(attributes)
}

pub(crate) fn is_fixed_xmlns(name: &str, fixed_prefixes: &[&str]) -> bool {
    name.strip_prefix("xmlns:")
        .is_some_and(|prefix| fixed_prefixes.contains(&prefix))
}

fn canonical_uri(prefix: &str) -> Option<&'static str> {
    match prefix {
        "p" => Some(P_NS),
        "a" => Some(A_NS),
        "r" => Some(R_NS),
        "mc" => Some(MC_NS),
        _ => None,
    }
}

fn qname_prefix(name: &[u8]) -> Option<&str> {
    let position = name.iter().position(|byte| *byte == b':')?;
    std::str::from_utf8(&name[..position]).ok()
}

fn sort_namespace_entries(entries: &mut [(String, String)]) {
    entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
}

#[cfg(test)]
mod tests {
    use super::{non_visual_drawing_id, set_non_visual_drawing_name, sort_namespace_entries};

    #[test]
    fn namespace_entries_have_deterministic_prefix_order() {
        let mut entries = vec![
            ("z".to_owned(), "urn:z".to_owned()),
            ("a".to_owned(), "urn:a".to_owned()),
            ("m".to_owned(), "urn:m".to_owned()),
        ];
        sort_namespace_entries(&mut entries);

        assert_eq!(
            entries,
            vec![
                ("a".to_owned(), "urn:a".to_owned()),
                ("m".to_owned(), "urn:m".to_owned()),
                ("z".to_owned(), "urn:z".to_owned()),
            ]
        );
    }

    #[test]
    fn non_visual_id_accepts_any_prefix_and_only_an_unqualified_id() {
        assert_eq!(
            non_visual_drawing_id(br#"<q:cNvPr xmlns:q="urn:p" id="42" q:id="7"/>"#),
            Some(42)
        );
        assert_eq!(
            non_visual_drawing_id(br#"<q:cNvPr xmlns:q="urn:p" q:id="7"/>"#),
            None
        );
        assert_eq!(
            non_visual_drawing_id(
                br#"<q:cNvPr xmlns:q="urn:p"><x:cNvPr xmlns:x="urn:extension" id="99"/></q:cNvPr>"#
            ),
            None
        );
    }

    #[test]
    fn non_visual_name_rewrite_escapes_the_value_and_preserves_children() {
        let mut xml = br#"<q:cNvPr xmlns:q="urn:p" id="42" name="old" producer="one&#x20;two"><x:raw xmlns:x="urn:x">one &amp; two</x:raw><!--note--></q:cNvPr>"#.to_vec();

        set_non_visual_drawing_name(&mut xml, "A & B \"quoted\"").unwrap();

        assert_eq!(
            xml,
            br#"<q:cNvPr xmlns:q="urn:p" id="42" name="A &amp; B &quot;quoted&quot;" producer="one&#x20;two"><x:raw xmlns:x="urn:x">one &amp; two</x:raw><!--note--></q:cNvPr>"#
        );
    }
}
