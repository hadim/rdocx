use std::collections::HashMap;
use std::ops::Range;

use oxml_core::OxmlError;
use oxml_drawing::namespace::A_NS;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer, XmlVersion};

use crate::shape_tree::ClickHyperlink;

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

/// Reads the click hyperlink of one raw `p:cNvPr`, its `a:hlinkClick` child.
pub(crate) fn non_visual_click_hyperlink(xml: &[u8]) -> Option<ClickHyperlink> {
    locate_click_hyperlink(xml)
        .ok()?
        .hyperlink
        .map(|(_, hyperlink)| hyperlink)
}

/// Replaces the `a:hlinkClick` child of one raw `p:cNvPr` with a fresh
/// element that carries only the relationship id and action of `hyperlink`,
/// or removes it for `None`.
///
/// A new element goes first, where `CT_NonVisualDrawingProps` puts it. Every
/// other child keeps its bytes and its place, and a `p:cNvPr` left without
/// content becomes an empty element again.
pub(crate) fn set_non_visual_click_hyperlink(
    xml: &mut Vec<u8>,
    hyperlink: Option<&ClickHyperlink>,
) -> Result<(), OxmlError> {
    let location = locate_click_hyperlink(xml)?;
    let replacement = match hyperlink {
        Some(hyperlink) => {
            let mut start = BytesStart::new("a:hlinkClick");
            if let Some(id) = &hyperlink.relationship_id {
                start.push_attribute(("r:id", id.as_str()));
            }
            if let Some(action) = &hyperlink.action {
                start.push_attribute(("action", action.as_str()));
            }
            let mut writer = Writer::new(Vec::new());
            writer.write_event(Event::Empty(start))?;
            writer.into_inner()
        }
        None => Vec::new(),
    };
    let root_end = location.root_end;
    match (location.close, location.hyperlink) {
        (None, _) if replacement.is_empty() => {}
        (None, _) => {
            // An empty root ends with `/>`: open it around the new child.
            let mut opened = xml[..root_end - 2].to_vec();
            opened.push(b'>');
            opened.extend_from_slice(&replacement);
            opened.extend_from_slice(b"</");
            opened.extend_from_slice(&location.root_name);
            opened.push(b'>');
            opened.extend_from_slice(&xml[root_end..]);
            *xml = opened;
        }
        (Some(close), Some((range, _)))
            if replacement.is_empty() && range == (root_end..close.start) =>
        {
            let mut collapsed = xml[..root_end - 1].to_vec();
            collapsed.extend_from_slice(b"/>");
            collapsed.extend_from_slice(&xml[close.end..]);
            *xml = collapsed;
        }
        (Some(_), Some((range, _))) => {
            xml.splice(range, replacement);
        }
        (Some(_), None) => {
            xml.splice(root_end..root_end, replacement);
        }
    }
    Ok(())
}

/// Where one raw `p:cNvPr` keeps its click hyperlink.
struct ClickHyperlinkLocation {
    root_name: Vec<u8>,
    /// The end of the root start tag, where a new first child goes.
    root_end: usize,
    /// The root end tag, `None` for an empty root.
    close: Option<Range<usize>>,
    /// The bytes and the modelled attributes of the first `a:hlinkClick`.
    hyperlink: Option<(Range<usize>, ClickHyperlink)>,
}

fn locate_click_hyperlink(xml: &[u8]) -> Result<ClickHyperlinkLocation, OxmlError> {
    let mut reader = Reader::from_reader(xml);
    let mut buffer = Vec::new();
    let (mut location, root) = loop {
        match reader.read_event_into(&mut buffer)? {
            Event::Start(start) => {
                break (
                    ClickHyperlinkLocation {
                        root_name: start.name().as_ref().to_vec(),
                        root_end: reader.buffer_position() as usize,
                        close: None,
                        hyperlink: None,
                    },
                    NamespaceBindings::default().with_start(&start)?,
                );
            }
            Event::Empty(start) => {
                return Ok(ClickHyperlinkLocation {
                    root_name: start.name().as_ref().to_vec(),
                    root_end: reader.buffer_position() as usize,
                    close: None,
                    hyperlink: None,
                });
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "non-visual drawing properties".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    };
    let mut depth = 0usize;
    let mut open = None;
    loop {
        buffer.clear();
        let begin = reader.buffer_position() as usize;
        let event = reader.read_event_into(&mut buffer)?;
        let end = reader.buffer_position() as usize;
        match event {
            Event::Start(start) => {
                if depth == 0 && location.hyperlink.is_none() && open.is_none() {
                    open = click_hyperlink(&start, &root)?.map(|hyperlink| (begin, hyperlink));
                }
                depth += 1;
            }
            Event::Empty(start) => {
                if depth == 0 && location.hyperlink.is_none() {
                    location.hyperlink =
                        click_hyperlink(&start, &root)?.map(|hyperlink| (begin..end, hyperlink));
                }
            }
            Event::End(_) if depth == 0 => {
                location.close = Some(begin..end);
                return Ok(location);
            }
            Event::End(_) => {
                depth -= 1;
                if depth == 0
                    && let Some((start, hyperlink)) = open.take()
                {
                    location.hyperlink = Some((start..end, hyperlink));
                }
            }
            Event::Eof => {
                return Err(OxmlError::MissingElement(
                    "closing non-visual drawing properties".to_owned(),
                ));
            }
            _ => {}
        }
    }
}

/// Reads `start` as a click hyperlink when it is one.
///
/// A raw `p:cNvPr` does not carry the declarations of its ancestors, so a
/// prefix it does not bind itself is taken to be the expected one: the
/// element matches by its local name unless its prefix is bound to another
/// namespace than DrawingML, and the relationship id is the prefixed `id`
/// attribute whose prefix is not bound to another namespace than the
/// relationships one.
fn click_hyperlink(
    start: &BytesStart<'_>,
    parent: &NamespaceBindings,
) -> Result<Option<ClickHyperlink>, OxmlError> {
    let scope = parent.with_start(start)?;
    let name = start.name();
    if name.local_name().as_ref() != b"hlinkClick"
        || scope
            .element_uri(name.as_ref())
            .is_some_and(|uri| uri != A_NS)
    {
        return Ok(None);
    }
    let mut hyperlink = ClickHyperlink::default();
    for (name, value) in all_attributes(start)? {
        if name == "action" {
            hyperlink.action = Some(value);
        } else if name.split_once(':').is_some_and(|(prefix, local)| {
            prefix != "xmlns"
                && local == "id"
                && scope
                    .attribute_uri(name.as_bytes())
                    .is_none_or(|uri| uri == R_NS)
        }) {
            hyperlink.relationship_id = Some(value).filter(|value| !value.is_empty());
        }
    }
    Ok(Some(hyperlink))
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
    use super::{
        non_visual_click_hyperlink, non_visual_drawing_id, set_non_visual_click_hyperlink,
        set_non_visual_drawing_name, sort_namespace_entries,
    };
    use crate::shape_tree::ClickHyperlink;

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

    #[test]
    fn click_hyperlink_reads_its_relationship_and_action_by_namespace() {
        let xml = br#"<q:cNvPr xmlns:q="urn:p" id="2" name="Link"><x:hlinkClick xmlns:x="urn:x" r:id="rId1"/><a:hlinkClick rel:id="rId3" action="ppaction://hlinksldjump" tooltip="Go" xmlns:rel="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><a:snd r:embed="rId9" name="click"/></a:hlinkClick></q:cNvPr>"#;
        assert_eq!(
            non_visual_click_hyperlink(xml),
            Some(ClickHyperlink {
                relationship_id: Some("rId3".to_owned()),
                action: Some("ppaction://hlinksldjump".to_owned()),
            })
        );
        assert_eq!(
            non_visual_click_hyperlink(
                br#"<p:cNvPr id="2"><a:hlinkClick xmlns:x="urn:x" x:id="rId3" id="rId4" r:id=""/></p:cNvPr>"#
            ),
            Some(ClickHyperlink::default())
        );
        assert_eq!(
            non_visual_click_hyperlink(br#"<p:cNvPr id="2" name="Plain"/>"#),
            None
        );
        assert_eq!(
            non_visual_click_hyperlink(
                br#"<p:cNvPr id="2"><a:hlinkHover r:id="rId5"/><a:extLst><a:ext uri="{X}"><a:hlinkClick r:id="rId6"/></a:ext></a:extLst></p:cNvPr>"#
            ),
            None
        );
    }

    #[test]
    fn click_hyperlink_write_replaces_only_its_element_and_restores_an_empty_root() {
        let empty = br#"<p:cNvPr id="2" name="A &amp; B"/>"#.to_vec();
        let link = ClickHyperlink {
            relationship_id: Some("rId4".to_owned()),
            action: None,
        };
        let mut xml = empty.clone();
        set_non_visual_click_hyperlink(&mut xml, None).unwrap();
        assert_eq!(xml, empty);
        set_non_visual_click_hyperlink(&mut xml, Some(&link)).unwrap();
        assert_eq!(
            xml,
            br#"<p:cNvPr id="2" name="A &amp; B"><a:hlinkClick r:id="rId4"/></p:cNvPr>"#
        );
        assert_eq!(non_visual_click_hyperlink(&xml), Some(link.clone()));
        set_non_visual_click_hyperlink(&mut xml, None).unwrap();
        assert_eq!(xml, empty);

        let mut xml = br#"<p:cNvPr id="3"><a:hlinkClick r:id="rId1" tooltip="old"><a:snd r:embed="rId2"/></a:hlinkClick><a:hlinkHover r:id="rId5"/><a:extLst><a:ext uri="{X}"/></a:extLst></p:cNvPr>"#.to_vec();
        let jump = ClickHyperlink {
            relationship_id: Some("rId6".to_owned()),
            action: Some("ppaction://hlinksldjump".to_owned()),
        };
        set_non_visual_click_hyperlink(&mut xml, Some(&jump)).unwrap();
        assert_eq!(
            xml,
            br#"<p:cNvPr id="3"><a:hlinkClick r:id="rId6" action="ppaction://hlinksldjump"/><a:hlinkHover r:id="rId5"/><a:extLst><a:ext uri="{X}"/></a:extLst></p:cNvPr>"#
        );
        set_non_visual_click_hyperlink(&mut xml, None).unwrap();
        assert_eq!(
            xml,
            br#"<p:cNvPr id="3"><a:hlinkHover r:id="rId5"/><a:extLst><a:ext uri="{X}"/></a:extLst></p:cNvPr>"#
        );
        set_non_visual_click_hyperlink(&mut xml, Some(&link)).unwrap();
        assert_eq!(
            xml,
            br#"<p:cNvPr id="3"><a:hlinkClick r:id="rId4"/><a:hlinkHover r:id="rId5"/><a:extLst><a:ext uri="{X}"/></a:extLst></p:cNvPr>"#
        );
    }
}
