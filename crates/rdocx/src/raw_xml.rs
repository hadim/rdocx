//! Raw XML of one body element: read it as standalone XML and replace it with
//! checked XML, the way out when the model has no API for a feature.
//!
//! A replacement must hold exactly one element of the addressed kind, parse
//! with the model, keep every element it names, and reference only
//! relationships the main document part already has. Anything else is
//! refused before the document changes.

use std::collections::{BTreeMap, BTreeSet};

use quick_xml::events::Event;
use quick_xml::name::ResolveResult;
use quick_xml::{NsReader, Reader};
use rdocx_oxml::document::{BodyContent, CT_Body, CT_Document, CT_SectPr};
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_Tc};
use rdocx_oxml::text::{AcceptedRunPath, CT_P, CT_R};

use crate::document::{close_content_fragment_namespaces, story_namespace_scope_at};
use crate::{Document, Error, Result};

const W_NS: &[u8] = b"http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const R_NS: &[u8] = b"http://schemas.openxmlformats.org/officeDocument/2006/relationships";

/// A paragraph addressed by the indexes of [`Document::paragraph`], or of
/// [`Document::table`], [`crate::TableRef::cell`] and
/// [`crate::CellRef::paragraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlParagraph {
    /// A body paragraph, by its index in [`Document::paragraphs`].
    Body(usize),
    /// A paragraph of a table cell.
    Cell {
        table: usize,
        row: usize,
        cell: usize,
        paragraph: usize,
    },
}

/// One element whose XML [`Document::element_xml`] reads and
/// [`Document::replace_element_xml`] replaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XmlTarget {
    /// A `w:p`.
    Paragraph(XmlParagraph),
    /// A `w:r`, by the source path [`crate::Paragraph::run_path`] returns.
    Run(XmlParagraph, AcceptedRunPath),
    /// A `w:tbl`, by its index in [`Document::table`].
    Table(usize),
    /// A `w:tc`, by the indexes of [`crate::TableRef::cell`].
    Cell {
        table: usize,
        row: usize,
        cell: usize,
    },
    /// A `w:sectPr`, by its index in [`Document::section`].
    Section(usize),
}

impl XmlTarget {
    fn local_name(&self) -> &'static str {
        match self {
            Self::Paragraph(_) => "p",
            Self::Run(..) => "r",
            Self::Table(_) => "tbl",
            Self::Cell { .. } => "tc",
            Self::Section(_) => "sectPr",
        }
    }

    /// The local names from the document root down to the element, in the
    /// shell document [`Document::shell`] builds around it.
    fn shell_path(&self) -> &'static [&'static [u8]] {
        match self {
            Self::Paragraph(_) => &[b"document", b"body", b"p"],
            Self::Run(..) => &[b"document", b"body", b"p", b"r"],
            Self::Table(_) => &[b"document", b"body", b"tbl"],
            Self::Cell { .. } => &[b"document", b"body", b"tbl", b"tr", b"tc"],
            Self::Section(_) => &[b"document", b"body", b"sectPr"],
        }
    }

    /// The markup that wraps a fragment of this kind inside `w:body`.
    fn wrapper(&self) -> (&'static str, &'static str) {
        match self {
            Self::Run(..) => ("<w:p>", "</w:p>"),
            Self::Cell { .. } => ("<w:tbl><w:tr>", "</w:tr></w:tbl>"),
            _ => ("", ""),
        }
    }
}

/// One parsed replacement, ready to be written in place. It lives only for
/// one call, so the paragraph variant stays unboxed.
#[allow(clippy::large_enum_variant)]
enum Element {
    Paragraph(CT_P),
    Run(CT_R),
    Table(CT_Tbl),
    Cell(CT_Tc),
    Section(CT_SectPr),
}

impl Element {
    /// The body content of the shell document that serializes it.
    fn into_body(self) -> CT_Body {
        let (content, sect_pr) = match self {
            Self::Paragraph(paragraph) => (vec![BodyContent::Paragraph(paragraph)], None),
            Self::Run(run) => {
                let mut paragraph = CT_P::new();
                paragraph.runs.push(run);
                (vec![BodyContent::Paragraph(paragraph)], None)
            }
            Self::Table(table) => (vec![BodyContent::Table(table)], None),
            Self::Cell(cell) => {
                let mut row = CT_Row::new();
                row.cells.push(cell);
                let mut table = CT_Tbl::new();
                table.rows.push(row);
                (vec![BodyContent::Table(table)], None)
            }
            Self::Section(section) => (Vec::new(), Some(section)),
        };
        CT_Body { content, sect_pr }
    }
}

impl Document {
    /// Return the XML of one body element as a standalone fragment that
    /// declares every namespace prefix in scope at its position.
    pub fn element_xml(&self, target: &XmlTarget) -> Result<Vec<u8>> {
        let element = self.current_element(target)?;
        let xml = self.shell(element.into_body()).to_xml()?;
        let (start, end) = element_span(&xml, target.shell_path())?;
        let scope = story_namespace_scope_at(&xml, start)?;
        close_content_fragment_namespaces(&xml[start..end], &scope)
    }

    /// Replace one body element with `xml`, which must hold exactly one
    /// element of the same kind.
    ///
    /// Prefixes the document root declares may be used without declaring
    /// them. The replacement is refused, and the document left unchanged,
    /// when the XML is malformed, holds another element or more than one,
    /// names an element the model would drop, references a relationship id
    /// the main document part does not have, or adds or removes a section
    /// break (edit those through [`XmlTarget::Section`]). Identifiers such as
    /// bookmark or comment ids are written as given.
    pub fn replace_element_xml(&mut self, target: &XmlTarget, xml: &[u8]) -> Result<()> {
        let invalid = |message: String| {
            Error::Other(format!(
                "replace_xml expects one w:{} element: {message}",
                target.local_name()
            ))
        };
        let fragment = strip_xml_declaration(xml);
        let root = single_root_name(fragment).map_err(invalid)?;
        let (before, after) = target.wrapper();
        let mut wrapped = self.shell_root()?;
        wrapped.extend_from_slice(b"<w:body>");
        wrapped.extend_from_slice(before.as_bytes());
        wrapped.extend_from_slice(fragment);
        wrapped.extend_from_slice(after.as_bytes());
        wrapped.extend_from_slice(b"</w:body></w:document>");

        let parsed = CT_Document::from_xml(&wrapped)
            .map_err(|error| invalid(format!("the XML does not parse: {error}")))?;
        let element = extract(target, parsed.body).ok_or_else(|| {
            invalid(format!(
                "got {root}, which does not parse as a single w:{}",
                target.local_name()
            ))
        })?;
        let current = self.current_element(target)?;
        let section_breaks = |counts: &ElementCounts| {
            counts
                .get(&(W_NS.to_vec(), b"sectPr".to_vec()))
                .copied()
                .unwrap_or(0)
        };
        let target_depth = match target {
            XmlTarget::Paragraph(_) => Some(3),
            XmlTarget::Run(..) => Some(4),
            _ => None,
        };
        if let Some(depth) = target_depth
            && block_inside_paragraph(&wrapped, depth)?
        {
            return Err(invalid(
                "a paragraph holds a w:p or w:tbl only inside a text box, add block content with \
                 document.insert_paragraph or document.insert_table"
                    .to_owned(),
            ));
        }
        let input = element_counts(&wrapped)?;
        let current_counts = element_counts(&self.shell(current.into_body()).to_xml()?)?;
        if !matches!(target, XmlTarget::Section(_))
            && section_breaks(&input) != section_breaks(&current_counts)
        {
            return Err(invalid(
                "it adds or removes a section break, edit section properties with replace_section_xml"
                    .to_owned(),
            ));
        }
        let shell = self.shell(element.into_body());
        let written = shell.to_xml()?;
        let kept = element_counts(&written)?;
        let missing = input
            .into_iter()
            .filter(|(name, count)| kept.get(name).copied().unwrap_or(0) < *count)
            .map(|((_, local), _)| String::from_utf8_lossy(&local).into_owned())
            .collect::<BTreeSet<_>>();
        if !missing.is_empty() {
            return Err(invalid(format!(
                "rdocx would not keep {}, so the replacement was refused",
                missing.into_iter().collect::<Vec<_>>().join(", ")
            )));
        }
        self.check_relationship_references(&wrapped)?;
        let element = extract(target, shell.body).expect("the shell holds the parsed element");
        self.write_element(target, element)
    }

    /// Fail when `xml` references a relationship id the main document part
    /// does not have.
    fn check_relationship_references(&self, xml: &[u8]) -> Result<()> {
        let known = self
            .package
            .get_part_rels(&self.doc_part_name)
            .map(|relationships| {
                relationships
                    .items
                    .iter()
                    .map(|relationship| relationship.id.clone())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        let mut reader = NsReader::from_reader(xml);
        let mut buffer = Vec::new();
        loop {
            let event = reader.read_event_into(&mut buffer).map_err(xml_error)?;
            let element = match &event {
                Event::Start(element) | Event::Empty(element) => element,
                Event::Eof => return Ok(()),
                _ => {
                    buffer.clear();
                    continue;
                }
            };
            for attribute in element.attributes() {
                let attribute = attribute.map_err(|error| xml_error(error.into()))?;
                let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
                if matches!(namespace, ResolveResult::Bound(uri) if uri.as_ref() == R_NS) {
                    let value = attribute
                        .decoded_and_normalized_value(
                            quick_xml::XmlVersion::Implicit1_0,
                            element.decoder(),
                        )
                        .map_err(xml_error)?;
                    if !known.contains(value.as_ref()) {
                        return Err(Error::Other(format!(
                            "replace_xml refused: r:{}=\"{value}\" names no relationship of the \
                             document part, add the target first (for example with add_picture \
                             or add_hyperlink) and reference the id it creates",
                            String::from_utf8_lossy(local.as_ref())
                        )));
                    }
                }
            }
            buffer.clear();
        }
    }

    /// A document holding `body` under this document's root element, so it
    /// declares the same namespaces.
    fn shell(&self, body: CT_Body) -> CT_Document {
        CT_Document {
            body,
            extra_namespaces: self.document.extra_namespaces.clone(),
            background_xml: None,
            background_extra_xml: Vec::new(),
            root_attributes: self.document.root_attributes.clone(),
        }
    }

    /// The start tag of this document's root element, with its declarations.
    fn shell_root(&self) -> Result<Vec<u8>> {
        let xml = self
            .shell(CT_Body {
                content: Vec::new(),
                sect_pr: None,
            })
            .to_xml()?;
        let (start, _) = element_span(&xml, &[b"document"])?;
        let mut reader = Reader::from_reader(&xml[start..]);
        let mut buffer = Vec::new();
        loop {
            match reader.read_event_into(&mut buffer).map_err(xml_error)? {
                Event::Start(_) => {
                    let end = start + reader.buffer_position() as usize;
                    return Ok(xml[start..end].to_vec());
                }
                Event::Eof => return Err(Error::Other("document has no root".to_owned())),
                _ => buffer.clear(),
            }
        }
    }

    fn current_element(&self, target: &XmlTarget) -> Result<Element> {
        let missing = || Error::Other(format!("no w:{} at {target:?}", target.local_name()));
        let paragraph = |at: &XmlParagraph| match *at {
            XmlParagraph::Body(index) => self.paragraph(index).map(|p| p.inner.clone()),
            XmlParagraph::Cell {
                table,
                row,
                cell,
                paragraph,
            } => self.table(table).and_then(|table| {
                let cell = table.cell(row, cell)?;
                cell.paragraph(paragraph).map(|p| p.inner.clone())
            }),
        };
        match target {
            XmlTarget::Paragraph(at) => paragraph(at).map(Element::Paragraph),
            XmlTarget::Run(at, path) => paragraph(at)
                .and_then(|paragraph| paragraph.accepted_run(path).cloned())
                .map(Element::Run),
            XmlTarget::Table(index) => self.table(*index).map(|t| Element::Table(t.inner.clone())),
            XmlTarget::Cell { table, row, cell } => self.table(*table).and_then(|table| {
                table
                    .cell(*row, *cell)
                    .map(|c| Element::Cell(c.inner.clone()))
            }),
            XmlTarget::Section(index) => self
                .section(*index)
                .map(|section| Element::Section(section.inner.clone())),
        }
        .ok_or_else(missing)
    }

    fn write_element(&mut self, target: &XmlTarget, element: Element) -> Result<()> {
        let missing = || Error::Other(format!("no w:{} at {target:?}", target.local_name()));
        fn with_paragraph(
            document: &mut Document,
            at: &XmlParagraph,
            edit: impl FnOnce(&mut CT_P) -> Result<()>,
        ) -> Option<Result<()>> {
            match *at {
                XmlParagraph::Body(index) => document
                    .paragraph_mut(index)
                    .map(|paragraph| edit(paragraph.inner)),
                XmlParagraph::Cell {
                    table,
                    row,
                    cell,
                    paragraph,
                } => {
                    let mut table = document.table_mut(table)?;
                    let mut cell = table.cell(row, cell)?;
                    let paragraph = cell.paragraph_mut(paragraph)?;
                    Some(edit(paragraph.inner))
                }
            }
        }
        let written = match (target, element) {
            (XmlTarget::Paragraph(at), Element::Paragraph(new)) => {
                with_paragraph(self, at, |paragraph| {
                    *paragraph = new;
                    Ok(())
                })
            }
            (XmlTarget::Run(at, path), Element::Run(new)) => {
                with_paragraph(self, at, |paragraph| {
                    if paragraph.replace_accepted_run(path, new)? {
                        Ok(())
                    } else {
                        Err(Error::Other("accepted run path is stale".to_owned()))
                    }
                })
            }
            (XmlTarget::Table(index), Element::Table(new)) => self.table_mut(*index).map(|table| {
                *table.inner = new;
                Ok(())
            }),
            (XmlTarget::Cell { table, row, cell }, Element::Cell(new)) => {
                self.table_mut(*table).and_then(|mut table| {
                    table.cell(*row, *cell).map(|cell| {
                        *cell.inner = new;
                        Ok(())
                    })
                })
            }
            (XmlTarget::Section(index), Element::Section(new)) => {
                self.section_mut(*index).map(|section| {
                    *section.inner = new;
                    Ok(())
                })
            }
            _ => unreachable!("extract returns the element kind of its target"),
        };
        written.ok_or_else(missing)?
    }
}

/// The element of `target`'s kind a parsed shell body holds, when it holds
/// exactly that and nothing else.
fn extract(target: &XmlTarget, body: CT_Body) -> Option<Element> {
    let CT_Body {
        mut content,
        sect_pr,
    } = body;
    if let XmlTarget::Section(_) = target {
        return content.is_empty().then_some(sect_pr?).map(Element::Section);
    }
    if sect_pr.is_some() || content.len() != 1 {
        return None;
    }
    match (target, content.pop()?) {
        (XmlTarget::Paragraph(_), BodyContent::Paragraph(paragraph)) => {
            Some(Element::Paragraph(paragraph))
        }
        (XmlTarget::Run(..), BodyContent::Paragraph(paragraph)) => {
            let [run] = paragraph.runs.as_slice() else {
                return None;
            };
            let mut bare = CT_P::new();
            bare.runs.push(run.clone());
            (paragraph == bare).then(|| Element::Run(run.clone()))
        }
        (XmlTarget::Table(_), BodyContent::Table(table)) => Some(Element::Table(table)),
        (XmlTarget::Cell { .. }, BodyContent::Table(mut table)) => {
            let mut row = table.rows.pop()?;
            let cell = row.cells.pop()?;
            (table.rows.is_empty() && row.cells.is_empty()).then_some(Element::Cell(cell))
        }
        _ => None,
    }
}

/// The byte span of the first element whose ancestry of local names is
/// `path`, from its start tag to the end of its end tag.
fn element_span(xml: &[u8], path: &[&[u8]]) -> Result<(usize, usize)> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let mut stack: Vec<Vec<u8>> = Vec::new();
    let mut start = None;
    loop {
        let before = reader.buffer_position() as usize;
        match reader.read_event_into(&mut buffer).map_err(xml_error)? {
            Event::Start(element) => {
                stack.push(element.local_name().as_ref().to_vec());
                if start.is_none() && stack.iter().map(Vec::as_slice).eq(path.iter().copied()) {
                    start = Some((before, stack.len()));
                }
            }
            Event::Empty(element) => {
                stack.push(element.local_name().as_ref().to_vec());
                if start.is_none() && stack.iter().map(Vec::as_slice).eq(path.iter().copied()) {
                    return Ok((before, reader.buffer_position() as usize));
                }
                stack.pop();
            }
            Event::End(_) => {
                if let Some((begin, depth)) = start
                    && depth == stack.len()
                {
                    return Ok((begin, reader.buffer_position() as usize));
                }
                stack.pop();
            }
            Event::Eof => {
                return Err(Error::Other(
                    "serialized shell lost the element it was built from".to_owned(),
                ));
            }
            _ => {}
        }
        buffer.clear();
    }
}

/// Whether a `w:p` or `w:tbl` sits below depth `target` (the root element
/// is depth 1) without a `w:txbxContent` between them, which Word refuses.
fn block_inside_paragraph(xml: &[u8], target: usize) -> Result<bool> {
    let mut reader = NsReader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut stack: Vec<bool> = Vec::new();
    loop {
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(xml_error)?;
        let word = matches!(namespace, ResolveResult::Bound(uri) if uri.as_ref() == W_NS);
        match event {
            Event::Start(ref element) | Event::Empty(ref element) => {
                let local = element.local_name();
                let text_box = word && local.as_ref() == b"txbxContent";
                let inside_box = stack.iter().skip(target).any(|boxed| *boxed);
                if stack.len() >= target
                    && !inside_box
                    && word
                    && matches!(local.as_ref(), b"p" | b"tbl")
                {
                    return Ok(true);
                }
                if matches!(event, Event::Start(_)) {
                    stack.push(text_box);
                }
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Eof => return Ok(false),
            _ => {}
        }
        buffer.clear();
    }
}

/// Element counts by namespace and local name.
type ElementCounts = BTreeMap<(Vec<u8>, Vec<u8>), usize>;

/// How many elements of each namespace and local name `xml` holds.
fn element_counts(xml: &[u8]) -> Result<ElementCounts> {
    let mut reader = NsReader::from_reader(xml);
    let mut buffer = Vec::new();
    let mut counts = BTreeMap::new();
    loop {
        let (namespace, event) = reader
            .read_resolved_event_into(&mut buffer)
            .map_err(xml_error)?;
        match event {
            Event::Start(element) | Event::Empty(element) => {
                let namespace = match namespace {
                    ResolveResult::Bound(uri) => uri.as_ref().to_vec(),
                    _ => Vec::new(),
                };
                *counts
                    .entry((namespace, element.local_name().as_ref().to_vec()))
                    .or_insert(0) += 1;
            }
            Event::Eof => return Ok(counts),
            _ => {}
        }
        buffer.clear();
    }
}

/// The qualified name of the fragment's only root element, or why it has
/// not exactly one.
fn single_root_name(fragment: &[u8]) -> std::result::Result<String, String> {
    let mut reader = Reader::from_reader(fragment);
    let mut buffer = Vec::new();
    let mut depth = 0usize;
    let mut root = None;
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| format!("the XML is malformed: {error}"))?;
        match event {
            Event::Start(ref element) | Event::Empty(ref element) if depth == 0 => {
                if root.is_some() {
                    return Err("it holds more than one root element".to_owned());
                }
                root = Some(String::from_utf8_lossy(element.name().as_ref()).into_owned());
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                }
            }
            Event::Start(_) => depth += 1,
            Event::End(_) => depth = depth.saturating_sub(1),
            Event::Text(text) if depth == 0 && !text.iter().all(u8::is_ascii_whitespace) => {
                return Err("it holds text outside its root element".to_owned());
            }
            Event::Eof if depth > 0 => {
                return Err("the XML is malformed: an element is not closed".to_owned());
            }
            Event::Eof => return root.ok_or_else(|| "it holds no element".to_owned()),
            _ => {}
        }
        buffer.clear();
    }
}

fn strip_xml_declaration(xml: &[u8]) -> &[u8] {
    let trimmed = xml.trim_ascii_start();
    if trimmed.starts_with(b"<?xml")
        && let Some(end) = trimmed.windows(2).position(|pair| pair == b"?>")
    {
        return &trimmed[end + 2..];
    }
    xml
}

fn xml_error(error: quick_xml::Error) -> Error {
    Error::Oxml(rdocx_oxml::OxmlError::from(error))
}
