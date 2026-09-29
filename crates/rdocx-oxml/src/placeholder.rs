//! Placeholder replacement across paragraphs, tables, and headers/footers.
//!
//! Handles the cross-run splitting problem: a placeholder like `{{name}}`
//! may be split across multiple `<w:r>` elements in the OOXML source.
//!
//! A replacement reads the direct runs of a paragraph and the runs of its
//! inline content controls, in document order, and searches their text left
//! to right as one. A match must lie within one stretch of those runs, see
//! [`replaceable_texts`]. A match that straddles a content-control boundary
//! is not replaced, and the search goes on after it, so no part of the text
//! it covers is replaced either.

use crate::content_control::{CT_Sdt, SdtContent};
use crate::header_footer::CT_HdrFtr;
use crate::namespace::W_NS;
use crate::numbering::{local_namespace_overrides, namespace_bindings, word_prefixes_at};
use crate::properties::is_word_element;
use crate::table::{CT_Row, CT_Tbl, CT_Tc, CellContent};
use crate::text::{
    AcceptedRunPath, AcceptedRunPathSegment, CT_P, CT_R, RunContent, raw_with_external_bindings,
};

/// Replace all occurrences of `placeholder` with `replacement` in a paragraph.
///
/// Handles placeholders split across multiple runs and reaches the runs of
/// inline content controls. A match that straddles a content-control
/// boundary is not replaced. Preserves the formatting of the first matched
/// run. Returns the number of replacements made.
pub fn replace_in_paragraph(para: &mut CT_P, placeholder: &str, replacement: &str) -> usize {
    if placeholder.is_empty() {
        return 0;
    }
    replace_matches(para, &mut |text, from| {
        let start = from + text[from..].find(placeholder)?;
        Some((start, start + placeholder.len(), replacement.to_owned()))
    })
}

/// The texts a replacement in `para` matches against, one per stretch of
/// runs. The direct runs between two inline content controls form one
/// stretch, and so do the runs of one control between its nested controls.
/// A match never spans two stretches.
#[doc(hidden)]
pub fn replaceable_texts(para: &CT_P) -> Vec<String> {
    let mut texts: Vec<String> = Vec::new();
    let mut previous = None;
    for (stretch, path) in text_runs(para) {
        if previous != Some(stretch) {
            texts.push(String::new());
            previous = Some(stretch);
        }
        if let (Some(text), Some(run)) = (texts.last_mut(), para.accepted_run(&path)) {
            text.extend(run.content.iter().filter_map(|content| match content {
                RunContent::Text(t) => Some(t.text.as_str()),
                _ => None,
            }));
        }
    }
    texts
}

/// Takes a text and the byte offset to search from, and returns the byte
/// range of the next match and the text that replaces it.
type MatchFinder<'a> = dyn FnMut(&str, usize) -> Option<(usize, usize, String)> + 'a;

/// Find each match that `next_match` reports in the text of `para` and
/// replace it.
fn replace_matches(para: &mut CT_P, next_match: &mut MatchFinder<'_>) -> usize {
    let runs = text_runs(para);
    let mut emptied = Vec::new();
    let mut total = 0;

    // Byte offset in the concatenated paragraph text at which to look for the
    // next match. Resuming *after* the text we just inserted is what keeps this
    // terminating when the replacement itself contains the placeholder
    // (`replace("NAME", "NAME Smith")`); restarting from 0 would rematch the
    // replacement forever.
    let mut search_from = 0usize;

    loop {
        // 1. Concatenate all run text and build a char map.
        let (full_text, char_map) = build_char_map(para, &runs);
        if search_from > full_text.len() {
            break;
        }

        // 2. Find the next match (byte offsets in full_text).
        let Some((byte_start, byte_end, replacement)) = next_match(&full_text, search_from) else {
            break;
        };
        let next_char = full_text[byte_start..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);

        // A zero-width match would neither consume input nor advance the
        // cursor, so step past it and the loop always makes progress.
        if byte_start == byte_end {
            search_from = byte_start + next_char;
            continue;
        }

        // Convert byte offsets to char indices (char_map is indexed by char position).
        let match_start = full_text[..byte_start].chars().count();
        let match_end = match_start + full_text[byte_start..byte_end].chars().count();

        // 3. Determine which runs are affected.
        let first_run = char_map[match_start].run_index;
        let last_run = char_map[match_end - 1].run_index;

        if runs[first_run].0 != runs[last_run].0 {
            // The match straddles a content-control boundary, so it is no
            // match. Look again after it, as after a replaced match, so that
            // no shorter match inside it, such as `\d+` finds in the digits
            // after a boundary, replaces part of the text it covers.
            search_from = byte_end;
            continue;
        }

        if first_run == last_run {
            // Single-run match: simple in-place replacement on that run's text content.
            edit_run(para, &runs[first_run].1, |run| {
                replace_in_single_run(run, &char_map, match_start, match_end, &replacement);
            });
        } else {
            // Cross-run match: put replacement in first run, clear matched parts from others.
            replace_across_runs(para, &runs, &char_map, match_start, match_end, &replacement);
            emptied.extend(first_run + 1..=last_run);
        }

        // Text before `byte_start` is untouched and the replacement now
        // occupies `byte_start..byte_start + replacement.len()`, so this stays
        // on a char boundary of the rebuilt text.
        search_from = byte_start + replacement.len();

        total += 1;
    }

    remove_emptied_runs(para, &runs, emptied);
    total
}

/// The runs of `para` that a replacement reads, in the order `CT_P::runs`
/// reads them, each with the stretch it belongs to and its address.
fn text_runs(para: &CT_P) -> Vec<(usize, AcceptedRunPath)> {
    let mut runs = Vec::new();
    let mut stretch = 0;
    for index in 0..=para.runs.len() {
        for (control, (_, _, _, sdt)) in para
            .content_controls
            .iter()
            .enumerate()
            .filter(|(_, (at, _, _, _))| *at == index)
        {
            let mut prefix = vec![AcceptedRunPathSegment::ContentControl(control)];
            stretch += 1;
            control_text_runs(sdt, &mut prefix, &mut stretch, &mut runs);
            stretch += 1;
        }
        if index < para.runs.len() {
            runs.push((
                stretch,
                AcceptedRunPath {
                    segments: vec![AcceptedRunPathSegment::Run(index)],
                },
            ));
        }
    }
    runs
}

fn control_text_runs(
    sdt: &CT_Sdt,
    prefix: &mut Vec<AcceptedRunPathSegment>,
    stretch: &mut usize,
    runs: &mut Vec<(usize, AcceptedRunPath)>,
) {
    for (index, content) in sdt.content.iter().enumerate() {
        match content {
            SdtContent::Run(_) => {
                prefix.push(AcceptedRunPathSegment::Run(index));
                runs.push((
                    *stretch,
                    AcceptedRunPath {
                        segments: prefix.clone(),
                    },
                ));
                prefix.pop();
            }
            SdtContent::ContentControl(nested) => {
                prefix.push(AcceptedRunPathSegment::ContentControl(index));
                *stretch += 1;
                control_text_runs(nested, prefix, stretch, runs);
                *stretch += 1;
                prefix.pop();
            }
            _ => {}
        }
    }
}

/// Apply `edit` to the run at `path`.
fn edit_run(para: &mut CT_P, path: &AcceptedRunPath, edit: impl FnOnce(&mut CT_R)) {
    if let Some(mut run) = para.accepted_run(path).cloned() {
        edit(&mut run);
        let replaced = para.replace_accepted_run(path, run);
        debug_assert!(matches!(replaced, Ok(true)), "text run path is stale");
    }
}

/// A mapping from character position in the concatenated text to its source run and content item.
#[derive(Debug)]
struct CharMapping {
    /// Index of the run in the list of text runs.
    run_index: usize,
    /// Index of the RunContent item within the run.
    content_index: usize,
    /// Byte offset within the text string of the RunContent::Text item.
    byte_offset: usize,
}

fn build_char_map(para: &CT_P, runs: &[(usize, AcceptedRunPath)]) -> (String, Vec<CharMapping>) {
    let mut full_text = String::new();
    let mut char_map = Vec::new();

    for (run_idx, (_, path)) in runs.iter().enumerate() {
        let Some(run) = para.accepted_run(path) else {
            continue;
        };
        for (content_idx, content) in run.content.iter().enumerate() {
            if let RunContent::Text(t) = content {
                for (byte_pos, _ch) in t.text.char_indices() {
                    char_map.push(CharMapping {
                        run_index: run_idx,
                        content_index: content_idx,
                        byte_offset: byte_pos,
                    });
                }
                full_text.push_str(&t.text);
            }
        }
    }

    (full_text, char_map)
}

fn replace_in_single_run(
    run: &mut CT_R,
    char_map: &[CharMapping],
    match_start: usize,
    match_end: usize,
    replacement: &str,
) {
    let first = &char_map[match_start];
    let content_idx = first.content_index;
    let byte_start = first.byte_offset;

    // Compute byte end within the same text item.
    let last = &char_map[match_end - 1];
    let byte_end = if let RunContent::Text(t) = &run.content[content_idx] {
        // byte_end is past the last matched character
        let remaining = &t.text[last.byte_offset..];
        let ch_len = remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        last.byte_offset + ch_len
    } else {
        last.byte_offset + 1
    };

    if let RunContent::Text(t) = &mut run.content[content_idx] {
        let mut new_text =
            String::with_capacity(t.text.len() - (byte_end - byte_start) + replacement.len());
        new_text.push_str(&t.text[..byte_start]);
        new_text.push_str(replacement);
        new_text.push_str(&t.text[byte_end..]);
        t.text = new_text;
        // Keep the flag the producer wrote. Dropping it rewrites an unchanged
        // run, which a later comparison against the source then reports.
        t.preserve_space = t.preserve_space || t.text.starts_with(' ') || t.text.ends_with(' ');
    }
}

fn replace_across_runs(
    para: &mut CT_P,
    runs: &[(usize, AcceptedRunPath)],
    char_map: &[CharMapping],
    match_start: usize,
    match_end: usize,
    replacement: &str,
) {
    // Handle the first run: replace from match start to end of text in that content item.
    let first_mapping = &char_map[match_start];
    edit_run(para, &runs[first_mapping.run_index].1, |run| {
        if let RunContent::Text(t) = &mut run.content[first_mapping.content_index] {
            let mut new_text = String::new();
            new_text.push_str(&t.text[..first_mapping.byte_offset]);
            new_text.push_str(replacement);
            t.text = new_text;
            t.preserve_space = t.preserve_space || t.text.starts_with(' ') || t.text.ends_with(' ');
        }
    });

    // Handle the last run: replace from start to match end within that content item.
    let last_mapping = &char_map[match_end - 1];
    edit_run(para, &runs[last_mapping.run_index].1, |run| {
        if let RunContent::Text(t) = &mut run.content[last_mapping.content_index] {
            let remaining = &t.text[last_mapping.byte_offset..];
            let ch_len = remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            let byte_end = last_mapping.byte_offset + ch_len;
            t.text = t.text[byte_end..].to_string();
            t.preserve_space = t.preserve_space || t.text.starts_with(' ') || t.text.ends_with(' ');
        }

        // If the last run's text is now empty, remove its text content too.
        run.content.retain(|c| {
            if let RunContent::Text(t) = c {
                !t.text.is_empty()
            } else {
                true
            }
        });
    });

    // Clear text content from runs strictly between first and last.
    for (_, path) in &runs[first_mapping.run_index + 1..last_mapping.run_index] {
        edit_run(para, path, |run| {
            run.content.retain(|c| !matches!(c, RunContent::Text(_)));
        });
    }
}

/// Remove the runs at `candidates` (indices into `runs`) that the replacement
/// left without any content, keeping every anchor of the paragraph and of
/// its content controls on the boundary that remains.
fn remove_emptied_runs(
    para: &mut CT_P,
    runs: &[(usize, AcceptedRunPath)],
    mut candidates: Vec<usize>,
) {
    candidates.sort_unstable();
    candidates.dedup();
    let mut direct = vec![false; para.runs.len()];
    // Later runs first, so that removing one inside a control leaves the
    // addresses of the others valid.
    for index in candidates.into_iter().rev() {
        let path = &runs[index].1;
        // A run keeps the attributes of its start tag, such as `w:rsidR`, as
        // a raw record, which does not make it worth keeping once empty.
        let emptied = para.accepted_run(path).is_some_and(|run| {
            run.content.is_empty()
                && run
                    .extra_xml_positions
                    .iter()
                    .filter(|position| CT_R::raw_child_is_root_attributes(**position))
                    .count()
                    == run.extra_xml.len()
        });
        if !emptied {
            continue;
        }
        match path.segments() {
            [AcceptedRunPathSegment::Run(run)] => direct[*run] = true,
            [AcceptedRunPathSegment::ContentControl(control), rest @ ..] => {
                if let Some((_, _, _, sdt)) = para.content_controls.get_mut(*control) {
                    remove_control_run(sdt, rest);
                }
            }
            _ => {}
        }
    }
    if direct.contains(&true) {
        para.remove_runs(&direct);
    }
}

fn remove_control_run(sdt: &mut CT_Sdt, path: &[AcceptedRunPathSegment]) {
    match path {
        [AcceptedRunPathSegment::Run(index)] => sdt.remove_content(*index),
        [AcceptedRunPathSegment::ContentControl(index), rest @ ..] => {
            if let Some(SdtContent::ContentControl(nested)) = sdt.content.get_mut(*index) {
                remove_control_run(nested, rest);
            }
        }
        _ => {}
    }
}

/// Replace all occurrences of `placeholder` in all paragraphs of a slice.
pub fn replace_in_paragraphs(paras: &mut [CT_P], placeholder: &str, replacement: &str) -> usize {
    paras
        .iter_mut()
        .map(|p| replace_in_paragraph(p, placeholder, replacement))
        .sum()
}

/// Replace all occurrences of `placeholder` in a table (recursively handles
/// nested tables and content controls).
pub fn replace_in_table(table: &mut CT_Tbl, placeholder: &str, replacement: &str) -> usize {
    edit_table(table, &mut |paragraph| {
        replace_in_paragraph(paragraph, placeholder, replacement)
    })
}

/// Hand every paragraph of a table to `edit` and sum what it counts.
fn edit_table(table: &mut CT_Tbl, edit: &mut dyn FnMut(&mut CT_P) -> usize) -> usize {
    let mut count = 0;
    for (_, _, sdt) in &mut table.content_controls {
        count += edit_control(sdt, edit);
    }
    for row in &mut table.rows {
        count += edit_row(row, edit);
    }
    count
}

fn edit_control(sdt: &mut CT_Sdt, edit: &mut dyn FnMut(&mut CT_P) -> usize) -> usize {
    let mut count = 0;
    for content in &mut sdt.content {
        count += match content {
            SdtContent::Paragraph(paragraph) => edit(paragraph),
            SdtContent::Table(table) => edit_table(table, edit),
            SdtContent::Row(row) => edit_row(row, edit),
            SdtContent::Cell(cell) => edit_cell(cell, edit),
            SdtContent::ContentControl(nested) => edit_control(nested, edit),
            SdtContent::Run(_) | SdtContent::RawXml(_) => 0,
        };
    }
    count
}

fn edit_row(row: &mut CT_Row, edit: &mut dyn FnMut(&mut CT_P) -> usize) -> usize {
    let mut count = 0;
    for (_, _, sdt) in &mut row.content_controls {
        count += edit_control(sdt, edit);
    }
    for cell in &mut row.cells {
        count += edit_cell(cell, edit);
    }
    count
}

fn edit_cell(cell: &mut CT_Tc, edit: &mut dyn FnMut(&mut CT_P) -> usize) -> usize {
    let mut count = 0;
    for content in &mut cell.content {
        count += match content {
            CellContent::Paragraph(paragraph) => edit(paragraph),
            CellContent::Table(table) => edit_table(table, edit),
            CellContent::ContentControl(sdt) => edit_control(sdt, edit),
        };
    }
    count
}

/// Hand the paragraphs of a table or a block content control kept as raw
/// XML to `edit`, and re-serialise the element in place when the edit
/// counts a change. Any other element, one the typed parsers refuse, or one
/// whose namespaces the rewrite cannot keep, see [`with_source_namespaces`],
/// keeps its bytes and counts nothing.
fn edit_raw_block(
    raw: &mut Vec<u8>,
    word_prefixes: &[String],
    edit: &mut dyn FnMut(&mut CT_P) -> usize,
) -> usize {
    use quick_xml::events::Event;
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(raw.as_slice());
    reader.config_mut().trim_text(false);
    let mut buffer = Vec::new();
    let Ok(Event::Start(start)) = reader.read_event_into(&mut buffer) else {
        return 0;
    };
    let Ok(prefixes) = word_prefixes_at(&start, word_prefixes) else {
        return 0;
    };
    // The namespaces the start tag declares other than the part does.
    let Ok(bindings) = local_namespace_overrides(&start, word_prefixes) else {
        return 0;
    };
    let mut writer = Writer::new(Vec::new());
    let count = if is_word_element(start.name().as_ref(), b"tbl", &prefixes) {
        let Ok(mut table) =
            CT_Tbl::from_xml_with_prefixes_and_owner_bindings(&mut reader, &prefixes, &bindings)
        else {
            return 0;
        };
        let count = edit_table(&mut table, edit);
        if count == 0 || table.to_xml(&mut writer).is_err() {
            return 0;
        }
        count
    } else if is_word_element(start.name().as_ref(), b"sdt", &prefixes) {
        let Some(mut control) = CT_Sdt::from_body_raw(raw, word_prefixes) else {
            return 0;
        };
        let count = edit_control(&mut control, edit);
        if count == 0 || control.to_xml(&mut writer).is_err() {
            return 0;
        }
        count
    } else {
        return 0;
    };
    let Some(rewritten) =
        with_source_namespaces(raw, &writer.into_inner(), &bindings, word_prefixes)
    else {
        return 0;
    };
    *raw = rewritten;
    count
}

/// Declare the namespaces that the start tag of `source` declares other
/// than the part does, `start_bindings`, again on `rewritten`, the element
/// the typed writers produced from it, since they leave them out. They leave
/// out a declaration below the start tag too, so a prefix of `rewritten`
/// left to the part must not be one that `source` declares with a namespace
/// the part does not bind it to. Otherwise the prefix would be unbound, or
/// name another namespace, and this returns `None`.
fn with_source_namespaces(
    source: &[u8],
    rewritten: &[u8],
    start_bindings: &[(String, String)],
    word_prefixes: &[String],
) -> Option<Vec<u8>> {
    let rewritten = raw_with_external_bindings(rewritten, start_bindings).ok()?;
    let (_, declared) = namespace_use(source)?;
    let (left_to_part, _) = namespace_use(&rewritten)?;
    let part = namespace_bindings(word_prefixes);
    let part_namespace = |prefix: &str| {
        part.iter()
            .find(|(candidate, _)| candidate == prefix)
            .map(|(_, namespace)| namespace.as_str())
            .or_else(|| word_prefixes.iter().any(|p| p == prefix).then_some(W_NS))
    };
    left_to_part
        .iter()
        .all(|prefix| {
            declared
                .iter()
                .filter(|(candidate, _)| candidate == prefix)
                .all(|(_, namespace)| part_namespace(prefix) == Some(namespace.as_str()))
        })
        .then_some(rewritten)
}

/// The prefixes an element uses outside every declaration it makes, and
/// every declaration it makes, as prefix and namespace.
type NamespaceUse = (Vec<String>, Vec<(String, String)>);

/// The [`NamespaceUse`] of `xml`, the empty prefix standing for the default
/// namespace of an unprefixed element.
fn namespace_use(xml: &[u8]) -> Option<NamespaceUse> {
    use quick_xml::Reader;
    use quick_xml::events::Event;

    let prefix_of = |name: &[u8]| {
        let name = std::str::from_utf8(name).ok()?;
        Some(name.split_once(':').map(|(prefix, _)| prefix.to_owned()))
    };
    let mut reader = Reader::from_reader(xml);
    let mut scopes: Vec<Vec<(String, String)>> = Vec::new();
    let mut unbound: Vec<String> = Vec::new();
    let mut declared = Vec::new();
    let mut buffer = Vec::new();
    loop {
        let (element, opens) = match reader.read_event_into(&mut buffer).ok()? {
            Event::Start(element) => (element.into_owned(), true),
            Event::Empty(element) => (element.into_owned(), false),
            Event::End(_) => {
                scopes.pop();
                buffer.clear();
                continue;
            }
            Event::Eof => break,
            _ => {
                buffer.clear();
                continue;
            }
        };
        buffer.clear();
        let declarations = local_namespace_overrides(&element, &[]).ok()?;
        let mut prefixes = vec![prefix_of(element.name().as_ref())?.unwrap_or_default()];
        for attribute in element.attributes() {
            let key = attribute.ok()?.key;
            let key = key.as_ref();
            if key != b"xmlns" && !key.starts_with(b"xmlns:") {
                prefixes.extend(prefix_of(key)?);
            }
        }
        for prefix in prefixes {
            let bound = prefix == "xml"
                || declarations
                    .iter()
                    .chain(scopes.iter().flatten())
                    .any(|(candidate, _)| *candidate == prefix);
            if !bound && !unbound.contains(&prefix) {
                unbound.push(prefix);
            }
        }
        declared.extend(declarations.iter().cloned());
        if opens {
            scopes.push(declarations);
        }
    }
    Some((unbound, declared))
}

/// Replace all occurrences of `placeholder` in a header or footer.
///
/// Reaches its paragraphs and those of the tables and block content controls
/// it keeps as raw XML. A raw element the replacement changed is written back
/// in its place, and the others keep their bytes.
pub fn replace_in_header_footer(hf: &mut CT_HdrFtr, placeholder: &str, replacement: &str) -> usize {
    edit_header_footer(hf, &mut |paragraph| {
        replace_in_paragraph(paragraph, placeholder, replacement)
    })
}

/// The texts a replacement in `hf` matches against, see [`replaceable_texts`].
#[doc(hidden)]
pub fn header_footer_replaceable_texts(hf: &CT_HdrFtr) -> Vec<String> {
    let mut texts = Vec::new();
    edit_header_footer(&mut hf.clone(), &mut |paragraph| {
        texts.extend(replaceable_texts(paragraph));
        0
    });
    texts
}

fn edit_header_footer(hf: &mut CT_HdrFtr, edit: &mut dyn FnMut(&mut CT_P) -> usize) -> usize {
    let mut count = 0;
    for paragraph in &mut hf.paragraphs {
        count += edit(paragraph);
    }
    for raw in &mut hf.extra_xml {
        count += edit_raw_block(raw, &hf.word_prefixes, edit);
    }
    count
}

/// Replace placeholders in text boxes and shapes within a raw XML part.
///
/// Walks the XML, finds `w:txbxContent` elements at any depth outside another
/// text box, parses their child `w:p` elements using `CT_P::from_xml`, and
/// their child tables and block content controls with the typed parsers,
/// performs replacement, and re-serializes the children it changed. Every
/// other child of the text box, such as a bookmark or a paragraph without a
/// match, is copied through verbatim in its place. A text box nested inside
/// another one is kept as it is, not edited. Returns the modified XML and
/// replacement count.
pub fn replace_in_xml_part(
    xml: &[u8],
    placeholder: &str,
    replacement: &str,
) -> crate::error::Result<(Vec<u8>, usize)> {
    replace_many_in_xml_part(xml, &[(placeholder, replacement)])
}

/// Apply several placeholder replacements to a raw XML part in one pass.
///
/// Equivalent to calling [`replace_in_xml_part`] once per pair, but parses and
/// re-serialises the part a single time — which matters when filling a template
/// with many fields.
pub fn replace_many_in_xml_part(
    xml: &[u8],
    replacements: &[(&str, &str)],
) -> crate::error::Result<(Vec<u8>, usize)> {
    rewrite_text_boxes(xml, &mut |paragraph| {
        replacements
            .iter()
            .map(|(placeholder, replacement)| {
                replace_in_paragraph(paragraph, placeholder, replacement)
            })
            .sum()
    })
}

/// Apply a regex replacement to text boxes and shapes within a raw XML part.
///
/// The regex counterpart to [`replace_in_xml_part`]; `replacement` supports
/// `$1`-style capture group references.
pub fn replace_regex_in_xml_part(
    xml: &[u8],
    re: &regex::Regex,
    replacement: &str,
) -> crate::error::Result<(Vec<u8>, usize)> {
    rewrite_text_boxes(xml, &mut |paragraph| {
        replace_regex_in_paragraph(paragraph, re, replacement)
    })
}

/// The texts a replacement in the text boxes of a raw XML part matches
/// against, see [`replaceable_texts`].
#[doc(hidden)]
pub fn xml_part_replaceable_texts(xml: &[u8]) -> crate::error::Result<Vec<String>> {
    let mut texts = Vec::new();
    rewrite_text_boxes(xml, &mut |paragraph| {
        texts.extend(replaceable_texts(paragraph));
        0
    })?;
    Ok(texts)
}

/// Walk `xml`, handing each paragraph of a `w:txbxContent` element to `edit`,
/// those of its tables and block content controls included, and
/// re-serialising a child in place when `edit` counts a change in it. Every
/// other child of the text box is copied through verbatim. Returns the
/// rewritten XML and the summed count.
fn rewrite_text_boxes(
    xml: &[u8],
    edit: &mut dyn FnMut(&mut CT_P) -> usize,
) -> crate::error::Result<(Vec<u8>, usize)> {
    use crate::error::OxmlError;
    use crate::namespace::{MC_NS, R_NS, matches_local_name};
    use crate::raw_xml::capture_element;
    use quick_xml::events::Event;
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();
    let mut total_count = 0;
    // The bindings `CT_P::from_xml` assumes for a paragraph cut out of the part.
    let word_prefixes = [
        "w".to_owned(),
        format!("\0r\0{R_NS}"),
        format!("\0mc\0{MC_NS}"),
    ];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) if matches_local_name(e.name().as_ref(), b"txbxContent") => {
                // We found a txbxContent element. Parse and edit each paragraph
                // and copy every other child through verbatim, in document order.
                writer.write_event(Event::Start(e.clone()))?;

                let mut inner_buf = Vec::new();

                loop {
                    match reader.read_event_into(&mut inner_buf) {
                        Ok(Event::Start(ref ie)) => {
                            if matches_local_name(ie.name().as_ref(), b"p") {
                                // Parse this paragraph: collect its XML, then parse via CT_P
                                let source = ie.to_owned();
                                let para_xml = capture_element(&mut reader, ie)?;
                                let mut para_reader = Reader::from_reader(para_xml.as_slice());
                                para_reader.config_mut().trim_text(true);
                                let mut prbuf = Vec::new();
                                // Advance past the <w:p> start tag
                                loop {
                                    match para_reader.read_event_into(&mut prbuf) {
                                        Ok(Event::Start(ref pe))
                                            if matches_local_name(pe.name().as_ref(), b"p") =>
                                        {
                                            break;
                                        }
                                        Ok(Event::Eof) => break,
                                        _ => {}
                                    }
                                    prbuf.clear();
                                }
                                let mut para = CT_P::from_xml(&mut para_reader)?;
                                let count = edit(&mut para);
                                if count == 0 {
                                    // Nothing changed, so the paragraph keeps its
                                    // bytes, start-tag attributes included.
                                    writer.get_mut().extend_from_slice(&para_xml);
                                } else {
                                    write_text_box_paragraph(&mut writer, &para, &source)?;
                                }
                                total_count += count;
                            } else {
                                // A table or a content control is edited through
                                // the typed model, and any other element the edit
                                // does not reach stays as it was.
                                let mut raw = capture_element(&mut reader, ie)?;
                                total_count += edit_raw_block(&mut raw, &word_prefixes, edit);
                                writer.get_mut().extend_from_slice(&raw);
                            }
                        }
                        // Every child element is consumed whole, so the first end
                        // tag at this level closes the txbxContent element.
                        Ok(Event::End(ie)) => {
                            writer.write_event(Event::End(ie))?;
                            break;
                        }
                        Ok(Event::Eof) => {
                            return Err(OxmlError::MissingElement("w:txbxContent end".to_owned()));
                        }
                        // Empty elements such as `<w:p/>` or a bookmark, whitespace,
                        // comments and processing instructions.
                        Ok(ev) => writer.write_event(ev)?,
                        Err(e) => return Err(e.into()),
                    }
                    inner_buf.clear();
                }
            }
            Ok(ev) => {
                writer.write_event(ev)?;
            }
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    Ok((writer.into_inner(), total_count))
}

/// Write a text-box paragraph that was parsed without its start tag.
///
/// The start tag keeps the attributes it was read with, such as `w:rsidR`,
/// `w14:paraId` or a local namespace declaration. The paragraph goes back to
/// the place it was read from, so every prefix they use resolves as before.
fn write_text_box_paragraph<W: std::io::Write>(
    writer: &mut quick_xml::Writer<W>,
    paragraph: &CT_P,
    source: &quick_xml::events::BytesStart<'_>,
) -> crate::error::Result<()> {
    use quick_xml::events::{BytesStart, Event};

    let mut xml = Vec::new();
    paragraph.to_xml(&mut quick_xml::Writer::new(&mut xml))?;
    let mut start = BytesStart::new("w:p");
    for attribute in source.attributes() {
        start.push_attribute(attribute?);
    }
    // Parsed without its start tag, the paragraph records no attribute of its
    // own, so it serializes with a bare start tag. Anything else is written as
    // it serialized.
    if xml == b"<w:p/>" {
        writer.write_event(Event::Empty(start))?;
    } else if let Some(content) = xml.strip_prefix(b"<w:p>") {
        writer.write_event(Event::Start(start))?;
        writer.get_mut().write_all(content)?;
    } else {
        writer.get_mut().write_all(&xml)?;
    }
    Ok(())
}

/// Replace placeholders in chart XML parts.
///
/// Chart text uses DrawingML runs: `a:r` → `a:t` (not `w:r`/`w:t`).
/// Also replaces in string cache values (`c:v`).
/// Returns modified XML and replacement count.
pub fn replace_in_chart_xml(
    xml: &[u8],
    placeholder: &str,
    replacement: &str,
) -> crate::error::Result<(Vec<u8>, usize)> {
    replace_many_in_chart_xml(xml, &[(placeholder, replacement)])
}

/// Apply several placeholder replacements to chart XML in one pass.
pub fn replace_many_in_chart_xml(
    xml: &[u8],
    replacements: &[(&str, &str)],
) -> crate::error::Result<(Vec<u8>, usize)> {
    use crate::namespace::matches_local_name;
    use quick_xml::events::{BytesEnd, BytesText, Event};
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();
    let mut total_count = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            // <a:t> holds DrawingML run text, <c:v> a chart value cache entry.
            Ok(Event::Start(ref e))
                if matches_local_name(e.name().as_ref(), b"t")
                    || matches_local_name(e.name().as_ref(), b"v") =>
            {
                let name = e.name();
                writer.write_event(Event::Start(e.borrow()))?;

                // Consume the element as a whole. Reading it event by event
                // would split the value at every entity reference and could
                // miss a placeholder straddling one.
                let mut text = crate::xml_text::read_element_text(&mut reader, name);
                for (placeholder, replacement) in replacements {
                    let occurrences = text.matches(placeholder).count();
                    if occurrences > 0 {
                        total_count += occurrences;
                        text = text.replace(placeholder, replacement);
                    }
                }
                writer.write_event(Event::Text(BytesText::new(&text)))?;

                writer.write_event(Event::End(BytesEnd::new(
                    std::str::from_utf8(name.as_ref()).unwrap_or_default(),
                )))?;
            }
            Ok(ev) => {
                writer.write_event(ev)?;
            }
            Err(e) => return Err(e.into()),
        }
        buf.clear();
    }

    Ok((writer.into_inner(), total_count))
}

/// Replace all regex matches in a paragraph with the replacement string.
///
/// The `replacement` string supports capture group references: `$1`, `$2`, etc.
/// Uses the same cross-run char map algorithm as literal replacement, so it
/// reaches the runs of inline content controls and leaves a match that
/// straddles a content-control boundary as it is.
/// Returns the number of replacements made.
pub fn replace_regex_in_paragraph(para: &mut CT_P, re: &regex::Regex, replacement: &str) -> usize {
    // `captures_at` (rather than slicing) keeps anchors and look-around
    // evaluating against the full paragraph text.
    replace_matches(para, &mut |text, from| {
        let captures = re.captures_at(text, from)?;
        let matched = captures.get(0)?;
        // Expand capture groups in replacement
        let mut expanded = String::new();
        captures.expand(replacement, &mut expanded);
        Some((matched.start(), matched.end(), expanded))
    })
}

/// Replace regex matches in all paragraphs.
pub fn replace_regex_in_paragraphs(
    paras: &mut [CT_P],
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    paras
        .iter_mut()
        .map(|p| replace_regex_in_paragraph(p, re, replacement))
        .sum()
}

/// Replace regex matches in a table (recursively handles nested tables and
/// content controls).
pub fn replace_regex_in_table(table: &mut CT_Tbl, re: &regex::Regex, replacement: &str) -> usize {
    edit_table(table, &mut |paragraph| {
        replace_regex_in_paragraph(paragraph, re, replacement)
    })
}

/// Replace regex matches in a header or footer, see
/// [`replace_in_header_footer`].
pub fn replace_regex_in_header_footer(
    hf: &mut CT_HdrFtr,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    edit_header_footer(hf, &mut |paragraph| {
        replace_regex_in_paragraph(paragraph, re, replacement)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::namespace::{MC_NS, W_NS};
    use crate::properties::CT_RPr;

    fn make_para(texts: &[&str]) -> CT_P {
        let mut p = CT_P::new();
        for text in texts {
            p.add_run(text);
        }
        p
    }

    #[test]
    fn replace_single_run() {
        let mut p = make_para(&["Hello {{name}}, welcome!"]);
        let count = replace_in_paragraph(&mut p, "{{name}}", "Alice");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Alice, welcome!");
    }

    #[test]
    fn replace_cross_two_runs() {
        let mut p = make_para(&["Hello {{", "name}}, welcome!"]);
        let count = replace_in_paragraph(&mut p, "{{name}}", "Bob");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Bob, welcome!");
    }

    #[test]
    fn replace_cross_three_runs() {
        let mut p = make_para(&["{{", "na", "me}}"]);
        let count = replace_in_paragraph(&mut p, "{{name}}", "Charlie");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Charlie");
    }

    #[test]
    fn replace_preserves_formatting() {
        let mut p = CT_P::new();
        // Run 0: bold "Hello "
        let mut r0 = CT_R::new("Hello ");
        r0.properties = Some(CT_RPr {
            bold: Some(true),
            ..Default::default()
        });
        p.runs.push(r0);

        // Run 1: italic "{{name}}"
        let mut r1 = CT_R::new("{{name}}");
        r1.properties = Some(CT_RPr {
            italic: Some(true),
            ..Default::default()
        });
        p.runs.push(r1);

        // Run 2: "!"
        p.add_run("!");

        let count = replace_in_paragraph(&mut p, "{{name}}", "Alice");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Alice!");

        // Run 0 should still be bold
        assert_eq!(p.runs[0].properties.as_ref().unwrap().bold, Some(true));
        // Run 1 (now "Alice") should still be italic
        assert_eq!(p.runs[1].properties.as_ref().unwrap().italic, Some(true));
    }

    #[test]
    fn replace_keeps_the_producer_space_flag() {
        let mut preserved = CT_R::new("WORD");
        if let RunContent::Text(text) = &mut preserved.content[0] {
            text.preserve_space = true;
        }
        let mut p = CT_P::new();
        p.runs.push(preserved.clone());
        p.runs.push(preserved);
        p.add_run("tail");

        assert_eq!(replace_in_paragraph(&mut p, "WORD", "WORD"), 2);
        assert_eq!(replace_in_paragraph(&mut p, "DWO", "D-WO"), 1);
        assert_eq!(replace_in_paragraph(&mut p, "tail", " end"), 1);
        let flags = p
            .runs
            .iter()
            .map(|run| match &run.content[0] {
                RunContent::Text(text) => (text.text.as_str(), text.preserve_space),
                _ => unreachable!(),
            })
            .collect::<Vec<_>>();
        assert_eq!(
            flags,
            [("WORD-WO", true), ("RD", true), (" end", true)],
            "a rewritten run keeps a producer flag and gains one at a text edge"
        );
    }

    fn paragraph_xml(paragraph: &CT_P) -> String {
        let mut writer = quick_xml::Writer::new(Vec::new());
        paragraph.to_xml(&mut writer).unwrap();
        String::from_utf8(writer.into_inner()).unwrap()
    }

    /// A match across runs removes the runs it empties. The anchors after
    /// them are kept by run index, and used to slide one run later, past the
    /// text they preceded. A ruby annotation slid onto the run after its base.
    #[test]
    fn removing_emptied_runs_keeps_later_anchors_in_place() {
        let source = format!(
            r#"<w:p xmlns:w="{W_NS}"><w:r><w:t>{{{{na</w:t></w:r><w:r><w:t>me}}}}</w:t></w:r><w:bookmarkStart w:id="1" w:name="mark"/><w:commentRangeStart w:id="2"/><w:sdt><w:sdtPr><w:tag w:val="kept"/></w:sdtPr><w:sdtContent><w:r><w:t>control</w:t></w:r></w:sdtContent></w:sdt><w:proofErr w:type="spellStart"/><w:ruby><w:rt><w:r><w:t>ann</w:t></w:r></w:rt><w:rubyBase><w:r><w:t>BASE</w:t></w:r></w:rubyBase></w:ruby><w:r><w:t>x</w:t></w:r><w:bookmarkEnd w:id="1"/><w:commentRangeEnd w:id="2"/></w:p>"#
        );
        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        for regex in [false, true] {
            let mut p = CT_P::from_xml_fragment(source.as_bytes()).unwrap();
            let count = if regex {
                replace_regex_in_paragraph(&mut p, &re, "Bob")
            } else {
                replace_in_paragraph(&mut p, "{{name}}", "Bob")
            };
            assert_eq!(count, 1);
            assert_eq!(p.runs.len(), 3);
            let xml = paragraph_xml(&p);
            let positions = [
                ">Bob<",
                "bookmarkStart",
                "commentRangeStart",
                "<w:sdt>",
                "proofErr",
                "<w:ruby>",
                ">BASE<",
                "</w:ruby>",
                ">x<",
                "bookmarkEnd",
                "commentRangeEnd",
            ]
            .map(|marker| {
                xml.find(marker)
                    .unwrap_or_else(|| panic!("{marker}: {xml}"))
            });
            assert!(positions.is_sorted(), "{xml}");
        }
    }

    /// A run whose only child is raw XML, such as a drawing Word writes in
    /// `mc:AlternateContent`, has no typed content, and neither has an
    /// empty run. A match anywhere in the paragraph used to remove both.
    #[test]
    fn replace_keeps_the_runs_it_did_not_empty() {
        let source = format!(
            r#"<w:p xmlns:w="{W_NS}" xmlns:mc="{MC_NS}"><w:r><w:t>Hello {{{{name}}}}</w:t></w:r><w:r><w:rPr><w:b/></w:rPr></w:r><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing/></mc:Choice><mc:Fallback><w:pict/></mc:Fallback></mc:AlternateContent></w:r></w:p>"#
        );
        let mut p = CT_P::from_xml_fragment(source.as_bytes()).unwrap();

        assert_eq!(replace_in_paragraph(&mut p, "{{name}}", "Ada"), 1);

        assert_eq!(p.runs.len(), 3);
        assert!(paragraph_xml(&p).contains("<mc:AlternateContent>"));
    }

    /// Inside an inline control the replacement removes the runs it empties
    /// too. A later run of the control keeps the bytes it was read with.
    #[test]
    fn a_control_run_after_emptied_ones_keeps_its_source_bytes() {
        let tail = r#"<w:r w:rsidR="00CC"><w:t xml:space="preserve"> tail</w:t></w:r>"#;
        let source = format!(
            r#"<w:p xmlns:w="{W_NS}"><w:r><w:t xml:space="preserve">Dear </w:t></w:r><w:sdt><w:sdtPr><w:tag w:val="goog_rdk_0"/></w:sdtPr><w:sdtContent><w:r w:rsidR="00AA"><w:t>{{{{na</w:t></w:r><w:r w:rsidR="00BB"><w:t>me}}}}</w:t></w:r>{tail}</w:sdtContent></w:sdt></w:p>"#
        );
        let mut p = CT_P::from_xml_fragment(source.as_bytes()).unwrap();

        assert_eq!(replace_in_paragraph(&mut p, "{{name}}", "Bob"), 1);

        assert_eq!(p.text(), "Dear Bob tail");
        assert_eq!(p.content_controls[0].3.content.len(), 2);
        let xml = paragraph_xml(&p);
        assert!(xml.contains(tail), "{xml}");
    }

    /// Each inline control starts a stretch of its own, and so does each
    /// control nested in it. A match never spans two stretches.
    #[test]
    fn a_match_stays_within_one_stretch_of_runs() {
        let source = format!(
            r#"<w:p xmlns:w="{W_NS}"><w:r><w:t>al</w:t></w:r><w:sdt><w:sdtContent><w:r><w:t>X</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t>pha</w:t></w:r><w:sdt><w:sdtContent><w:r><w:t>b</w:t></w:r><w:sdt><w:sdtContent><w:r><w:t>c</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t>d</w:t></w:r><w:r><w:t>e</w:t></w:r></w:sdtContent></w:sdt></w:p>"#
        );
        let mut p = CT_P::from_xml_fragment(source.as_bytes()).unwrap();
        assert_eq!(replaceable_texts(&p), ["al", "X", "pha", "b", "c", "de"]);

        for straddling in ["alX", "Xpha", "phab", "bc", "cd"] {
            assert_eq!(replace_in_paragraph(&mut p, straddling, "-"), 0);
        }
        let re = regex::Regex::new(r"^al|bcd|e$").unwrap();
        assert_eq!(replace_regex_in_paragraph(&mut p, &re, "-"), 2);

        assert_eq!(p.text(), "-Xphabcd-");
    }

    /// A quantified pattern also matches the digits after a boundary alone.
    /// The search goes on after the straddling match, so it no longer
    /// replaces part of the number the reader sees.
    #[test]
    fn no_match_starts_inside_a_straddling_one() {
        let source = format!(
            r#"<w:p xmlns:w="{W_NS}"><w:r><w:t>12</w:t></w:r><w:sdt><w:sdtContent><w:r><w:t xml:space="preserve">34 end</w:t></w:r></w:sdtContent></w:sdt><w:r><w:t xml:space="preserve"> 56</w:t></w:r></w:p>"#
        );
        for pattern in [r"\d+", r"\d{2,}"] {
            let mut p = CT_P::from_xml_fragment(source.as_bytes()).unwrap();
            let re = regex::Regex::new(pattern).unwrap();

            assert_eq!(replace_regex_in_paragraph(&mut p, &re, "N"), 1, "{pattern}");

            assert_eq!(p.text(), "1234 end N", "{pattern}");
        }
    }

    #[test]
    fn replace_multiple_occurrences() {
        let mut p = make_para(&["{{x}} and {{x}}"]);
        let count = replace_in_paragraph(&mut p, "{{x}}", "Y");
        assert_eq!(count, 2);
        assert_eq!(p.text(), "Y and Y");
    }

    #[test]
    fn replace_no_match() {
        let mut p = make_para(&["Hello World"]);
        let count = replace_in_paragraph(&mut p, "{{missing}}", "X");
        assert_eq!(count, 0);
        assert_eq!(p.text(), "Hello World");
    }

    #[test]
    fn replace_in_table_recursive() {
        use crate::table::{CT_Row, CT_Tc, CellContent};

        let mut table = CT_Tbl::new();
        let mut row = CT_Row::new();
        let mut cell = CT_Tc::new();

        // Add a paragraph with placeholder
        let mut p = CT_P::new();
        p.add_run("Value: {{val}}");
        cell.content = vec![CellContent::Paragraph(p)];

        // Add a nested table with placeholder
        let mut nested = CT_Tbl::new();
        let mut nrow = CT_Row::new();
        let mut ncell = CT_Tc::new();
        let mut np = CT_P::new();
        np.add_run("Nested: {{val}}");
        ncell.content = vec![CellContent::Paragraph(np)];
        nrow.cells.push(ncell);
        nested.rows.push(nrow);
        cell.content.push(CellContent::Table(nested));

        row.cells.push(cell);
        table.rows.push(row);

        let count = replace_in_table(&mut table, "{{val}}", "42");
        assert_eq!(count, 2);

        // Verify
        let para = match &table.rows[0].cells[0].content[0] {
            CellContent::Paragraph(p) => p,
            _ => panic!("expected paragraph"),
        };
        assert_eq!(para.text(), "Value: 42");
    }

    #[test]
    fn replace_in_header_footer_test() {
        let mut hf = CT_HdrFtr::new();
        let mut p = CT_P::new();
        p.add_run("Company: {{company}}");
        hf.paragraphs.push(p);

        let count = replace_in_header_footer(&mut hf, "{{company}}", "Acme Corp");
        assert_eq!(count, 1);
        assert_eq!(hf.text(), "Company: Acme Corp");
    }

    /// The tables and block content controls of a header are raw XML in the
    /// model, and replacement used to skip them. One the replacement changes
    /// is written back in its place, and one it does not change keeps its
    /// bytes, under the prefix the part uses.
    #[test]
    fn replace_in_header_footer_reaches_its_tables_and_controls() {
        let untouched = r#"<q:sdt><q:sdtPr><q:tag q:val="kept"/></q:sdtPr><q:sdtContent><q:p><q:r><q:t>no tag</q:t></q:r></q:p></q:sdtContent></q:sdt>"#;
        let xml = format!(
            r#"<q:hdr xmlns:q="{W_NS}"><q:tbl><q:tblGrid/><q:tr><q:tc><q:p><q:r><q:t>cell {{{{x}}}}</q:t></q:r></q:p></q:tc></q:tr></q:tbl><q:p><q:r><q:t>paragraph {{{{x}}}}</q:t></q:r></q:p><q:sdt><q:sdtPr><q:tag q:val="control"/></q:sdtPr><q:sdtContent><q:p><q:r><q:t>control {{{{x}}}}</q:t></q:r></q:p></q:sdtContent></q:sdt>{untouched}</q:hdr>"#
        );
        let parsed = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();
        assert_eq!(
            header_footer_replaceable_texts(&parsed),
            ["paragraph {{x}}", "cell {{x}}", "control {{x}}", "no tag"]
        );

        let re = regex::Regex::new(r"\{\{x\}\}").unwrap();
        for regex in [false, true] {
            let mut hf = parsed.clone();
            let count = if regex {
                replace_regex_in_header_footer(&mut hf, &re, "Y")
            } else {
                replace_in_header_footer(&mut hf, "{{x}}", "Y")
            };
            assert_eq!(count, 3);
            let written = String::from_utf8(hf.to_xml_header().unwrap()).unwrap();
            assert!(!written.contains("{{x}}"), "{written}");
            let positions = ["cell Y", "paragraph Y", "control Y", untouched].map(|marker| {
                written
                    .find(marker)
                    .unwrap_or_else(|| panic!("{marker}: {written}"))
            });
            assert!(positions.is_sorted(), "{written}");
        }
    }

    /// A header rewrites its raw tables as a text box does, see
    /// `replace_in_textbox_keeps_the_namespaces_its_tables_declare`. A cell
    /// that declares a namespace the root binds the same way is replaced.
    #[test]
    fn replace_in_header_footer_keeps_the_namespaces_its_tables_declare() {
        let on_cell = table_declaring_w14(false, "cell {{x}}");
        let tables = [table_declaring_w14(true, "table {{x}}"), on_cell.clone()].concat();
        let w14 = r#" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
        for (root, expected) in [("", 1), (w14, 2)] {
            let xml = format!(r#"<w:hdr xmlns:w="{W_NS}"{root}>{tables}</w:hdr>"#);
            let mut hf = CT_HdrFtr::from_xml(xml.as_bytes()).unwrap();

            assert_eq!(replace_in_header_footer(&mut hf, "{{x}}", "Y"), expected);

            let written = String::from_utf8(hf.to_xml_header().unwrap()).unwrap();
            assert_every_prefix_is_bound(&written);
            assert!(written.contains(">table Y<"), "{written}");
            assert_eq!(written.contains(&on_cell), expected == 1, "{written}");
        }
    }

    #[test]
    fn replace_empty_placeholder_noop() {
        let mut p = make_para(&["Hello"]);
        let count = replace_in_paragraph(&mut p, "", "X");
        assert_eq!(count, 0);
    }

    #[test]
    fn replace_in_textbox_xml() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape">
<w:body>
<w:p><w:r><mc:AlternateContent><mc:Choice>
<w:drawing><wp:anchor xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing">
<a:graphic xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<a:graphicData><wps:wsp><wps:txbx>
<w:txbxContent>
<w:p><w:r><w:t>Hello {{name}}</w:t></w:r></w:p>
</w:txbxContent>
</wps:txbx></wps:wsp></a:graphicData></a:graphic>
</wp:anchor></w:drawing>
</mc:Choice></mc:AlternateContent></w:r></w:p>
</w:body>
</w:document>"#;

        let (result, count) = replace_in_xml_part(xml, "{{name}}", "Alice").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Hello Alice"));
        assert!(!result_str.contains("{{name}}"));
    }

    /// Word writes revision-save and paragraph identities on text-box
    /// paragraphs too. An edited paragraph used to be written back with a
    /// bare start tag, so a replacement dropped them.
    #[test]
    fn replace_in_textbox_keeps_the_paragraph_start_tag_attributes() {
        const START: &str = r#"<w:p w:rsidR="00A1B2C3" w14:paraId="1A2B3C4D" w14:textId="5E6F7A8B" xmlns:x="urn:producer" x:id="7">"#;
        let xml = format!(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r><w:pict><v:shape><v:textbox><w:txbxContent>{START}<w:r><w:t>Hello {{{{name}}}}</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p></w:body></w:document>"#
        );
        let expected = format!("{START}<w:r><w:t>Hello Alice</w:t></w:r></w:p>");

        let (result, count) = replace_in_xml_part(xml.as_bytes(), "{{name}}", "Alice").unwrap();
        assert_eq!(count, 1);
        let result = String::from_utf8(result).unwrap();
        assert!(result.contains(&expected), "{result}");

        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        let (result, count) = replace_regex_in_xml_part(xml.as_bytes(), &re, "Alice").unwrap();
        assert_eq!(count, 1);
        let result = String::from_utf8(result).unwrap();
        assert!(result.contains(&expected), "{result}");
    }

    #[test]
    fn replace_in_vml_textbox() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:v="urn:schemas-microsoft-com:vml">
<w:body>
<w:p><w:r><w:pict><v:shape>
<v:textbox>
<w:txbxContent>
<w:p><w:r><w:t>Company: {{company}}</w:t></w:r></w:p>
</w:txbxContent>
</v:textbox>
</v:shape></w:pict></w:r></w:p>
</w:body>
</w:document>"#;

        let (result, count) = replace_in_xml_part(xml, "{{company}}", "Acme").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Company: Acme"));
    }

    /// A text box whose paragraphs sit among a table, a block content
    /// control, a bookmark, an empty paragraph, a comment and whitespace.
    /// A second text box without a placeholder follows. The walker rewrites
    /// every text box of the part, so that one must come back unchanged too.
    /// The paragraphs without a placeholder keep their identity attributes.
    const TEXT_BOX_WITH_EVERY_KIND_OF_CHILD: &str = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:body><w:p><w:r><w:pict><v:shape><v:textbox><w:txbxContent>
<w:p w:rsidR="00A1B2C3" w14:paraId="1234ABCD" w14:textId="77AA88BB"><w:r><w:t>Title</w:t></w:r></w:p>
<w:p><w:r><w:t>Hello {{name}}</w:t></w:r></w:p>
<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><!--kept-->
<w:sdt><w:sdtPr><w:alias w:val="Signature"/><w:id w:val="42"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>control</w:t></w:r></w:p></w:sdtContent></w:sdt><w:bookmarkStart w:id="7" w:name="boxed"/><w:p/><w:bookmarkEnd w:id="7"/><w:p><w:r><w:t>{{name}} again</w:t></w:r></w:p>
</w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p><w:p><w:r><w:pict><v:shape><v:textbox><w:txbxContent><w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>other cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl><w:p w:rsidR="00A1B2C3" w14:paraId="5678CDEF" w14:textId="77AA88BC"><w:r><w:t>No placeholder here</w:t></w:r></w:p></w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p></w:body></w:document>"#;

    #[test]
    fn replace_in_textbox_keeps_every_other_child_in_order() {
        let xml = TEXT_BOX_WITH_EVERY_KIND_OF_CHILD;
        let expected = xml.replace("{{name}}", "Alice");

        let (result, count) = replace_in_xml_part(xml.as_bytes(), "{{name}}", "Alice").unwrap();
        assert_eq!(count, 2);
        assert_eq!(String::from_utf8(result).unwrap(), expected);

        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        let (result, count) = replace_regex_in_xml_part(xml.as_bytes(), &re, "Alice").unwrap();
        assert_eq!(count, 2);
        assert_eq!(String::from_utf8(result).unwrap(), expected);
    }

    /// The tables and block content controls of a text box are edited
    /// through the typed model. Those without a match keep their bytes.
    #[test]
    fn replace_in_textbox_reaches_its_tables_and_controls() {
        let xml = TEXT_BOX_WITH_EVERY_KIND_OF_CHILD
            .replace(">cell<", ">cell {{name}}<")
            .replace(">control<", ">control {{name}}<");
        let other_table = r#"<w:tbl><w:tblPr><w:tblW w:w="0" w:type="auto"/></w:tblPr><w:tblGrid><w:gridCol w:w="2000"/></w:tblGrid><w:tr><w:tc><w:p><w:r><w:t>other cell</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#;
        assert_eq!(
            xml_part_replaceable_texts(xml.as_bytes()).unwrap(),
            [
                "Title",
                "Hello {{name}}",
                "cell {{name}}",
                "control {{name}}",
                "{{name}} again",
                "other cell",
                "No placeholder here"
            ]
        );

        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        for regex in [false, true] {
            let (result, count) = if regex {
                replace_regex_in_xml_part(xml.as_bytes(), &re, "Alice").unwrap()
            } else {
                replace_in_xml_part(xml.as_bytes(), "{{name}}", "Alice").unwrap()
            };
            assert_eq!(count, 4);
            let result = String::from_utf8(result).unwrap();
            assert!(!result.contains("{{name}}"), "{result}");
            let positions = [
                "Hello Alice",
                "cell Alice",
                "<!--kept-->",
                "control Alice",
                "Alice again",
                other_table,
            ]
            .map(|marker| {
                result
                    .find(marker)
                    .unwrap_or_else(|| panic!("{marker}: {result}"))
            });
            assert!(positions.is_sorted(), "{result}");
        }
    }

    /// Panic on an element or attribute prefix that no declaration binds.
    fn assert_every_prefix_is_bound(xml: &str) {
        use quick_xml::events::Event;
        use quick_xml::name::ResolveResult;

        let mut reader = quick_xml::NsReader::from_str(xml);
        loop {
            let (namespace, event) = reader.read_resolved_event().unwrap();
            assert!(!matches!(namespace, ResolveResult::Unknown(_)), "{xml}");
            match event {
                Event::Start(element) | Event::Empty(element) => {
                    for attribute in element.attributes() {
                        let key = attribute.unwrap().key;
                        let (namespace, _) = reader.resolver().resolve_attribute(key);
                        assert!(!matches!(namespace, ResolveResult::Unknown(_)), "{xml}");
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
    }

    /// A table with a paragraph that uses `w14`, declared on the table or on
    /// its cell.
    fn table_declaring_w14(on_table: bool, text: &str) -> String {
        let declaration = r#" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml""#;
        let (table, cell) = if on_table {
            (declaration, "")
        } else {
            ("", declaration)
        };
        format!(
            r#"<w:tbl{table}><w:tblGrid/><w:tr><w:tc{cell}><w:p w14:paraId="0000ABCD"><w:r><w:t>{text}</w:t></w:r></w:p></w:tc></w:tr></w:tbl>"#
        )
    }

    /// The typed writers leave out the namespace declarations of a table, so
    /// a paragraph that used a prefix declared on the table came back with
    /// it unbound. The table declares it again. A declaration on a cell
    /// cannot be put back, so that table keeps its bytes and counts nothing.
    #[test]
    fn replace_in_textbox_keeps_the_namespaces_its_tables_declare() {
        let on_cell = table_declaring_w14(false, "cell {{name}}");
        let xml = format!(
            r#"<w:document xmlns:w="{W_NS}" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r><w:pict><v:shape><v:textbox><w:txbxContent>{}{on_cell}</w:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p></w:body></w:document>"#,
            table_declaring_w14(true, "table {{name}}")
        );

        let (result, count) = replace_in_xml_part(xml.as_bytes(), "{{name}}", "Ada").unwrap();

        assert_eq!(count, 1);
        let result = String::from_utf8(result).unwrap();
        assert_every_prefix_is_bound(&result);
        assert!(result.contains(">table Ada<"), "{result}");
        assert!(result.contains(&on_cell), "{result}");
    }

    /// The end tag used to be written as `w:txbxContent` whatever prefix the
    /// start tag carried, which left the part ill-formed.
    #[test]
    fn replace_in_textbox_closes_it_with_its_own_prefix() {
        let xml = r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r><w:pict><v:shape><v:textbox><q:txbxContent xmlns:q="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:p><w:r><w:t>Hello {{name}}</w:t></w:r></w:p></q:txbxContent></v:textbox></v:shape></w:pict></w:r></w:p></w:body></w:document>"#;

        let (result, count) = replace_in_xml_part(xml.as_bytes(), "{{name}}", "Alice").unwrap();
        assert_eq!(count, 1);
        assert_eq!(
            String::from_utf8(result).unwrap(),
            xml.replace("{{name}}", "Alice")
        );
    }

    /// A text box cut short used to keep the walker reading past the end of
    /// the part forever.
    #[test]
    fn replace_in_unterminated_textbox_is_an_error() {
        for tail in ["<w:p/>", "<w:p><w:r><w:t>Hello {{name}}"] {
            let xml = format!(
                r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:pict><w:txbxContent>{tail}"#
            );
            assert!(replace_in_xml_part(xml.as_bytes(), "{{name}}", "Alice").is_err());
        }
    }

    #[test]
    fn replace_in_xml_part_no_textbox() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:body><w:p><w:r><w:t>Hello {{name}}</w:t></w:r></w:p></w:body>
</w:document>"#;

        let (_, count) = replace_in_xml_part(xml, "{{name}}", "Alice").unwrap();
        // No text boxes, so no replacements from the raw XML pass
        assert_eq!(count, 0);
    }

    #[test]
    fn replace_in_chart_title() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart>
<c:title><c:tx><c:rich>
<a:p><a:r><a:t>{{chart_title}}</a:t></a:r></a:p>
</c:rich></c:tx></c:title>
</c:chart>
</c:chartSpace>"#;

        let (result, count) = replace_in_chart_xml(xml, "{{chart_title}}", "Sales Report").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Sales Report"));
        assert!(!result_str.contains("{{chart_title}}"));
    }

    #[test]
    fn replace_in_chart_axis_and_cache() {
        let xml = br#"<?xml version="1.0" encoding="UTF-8"?>
<c:chartSpace xmlns:c="http://schemas.openxmlformats.org/drawingml/2006/chart"
              xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
<c:chart>
<c:plotArea>
<c:catAx><c:title><c:tx><c:rich>
<a:p><a:r><a:t>{{axis}}</a:t></a:r></a:p>
</c:rich></c:tx></c:title></c:catAx>
<c:barChart><c:ser><c:cat><c:strRef><c:strCache>
<c:pt idx="0"><c:v>{{label}}</c:v></c:pt>
</c:strCache></c:strRef></c:cat></c:ser></c:barChart>
</c:plotArea>
</c:chart>
</c:chartSpace>"#;

        let (result, count) = replace_in_chart_xml(xml, "{{axis}}", "Quarter").unwrap();
        assert_eq!(count, 1);
        let result_str = String::from_utf8(result).unwrap();
        assert!(result_str.contains("Quarter"));

        let (result2, count2) =
            replace_in_chart_xml(result_str.as_bytes(), "{{label}}", "Q1").unwrap();
        assert_eq!(count2, 1);
        let result2_str = String::from_utf8(result2).unwrap();
        assert!(result2_str.contains("Q1"));
    }

    #[test]
    fn replace_utf8_multibyte_placeholder() {
        // Test that multi-byte UTF-8 placeholders work correctly.
        // This verifies the byte-offset to char-index conversion in build_char_map.
        let mut p = make_para(&["こんにちは{{名前}}さん"]);
        let count = replace_in_paragraph(&mut p, "{{名前}}", "太郎");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "こんにちは太郎さん");
    }

    #[test]
    fn replace_utf8_cross_run() {
        // Multi-byte placeholder split across runs
        let mut p = make_para(&["こんにちは{{", "名前}}さん"]);
        let count = replace_in_paragraph(&mut p, "{{名前}}", "花子");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "こんにちは花子さん");
    }

    #[test]
    fn replace_utf8_multiple_occurrences() {
        let mut p = make_para(&["{{名前}}と{{名前}}"]);
        let count = replace_in_paragraph(&mut p, "{{名前}}", "太郎");
        assert_eq!(count, 2);
        assert_eq!(p.text(), "太郎と太郎");
    }

    #[test]
    fn replace_emoji_placeholder() {
        // Emojis are 4-byte UTF-8, good stress test
        let mut p = make_para(&["Hello {{🎉name🎉}}, welcome!"]);
        let count = replace_in_paragraph(&mut p, "{{🎉name🎉}}", "World");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello World, welcome!");
    }

    // ---- Regex replacement tests ----

    #[test]
    fn regex_replace_simple() {
        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        let mut p = make_para(&["Hello {{name}}, welcome!"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "Alice");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Alice, welcome!");
    }

    #[test]
    fn regex_replace_with_capture_groups() {
        let re = regex::Regex::new(r"(\d+)-(\d+)-(\d+)").unwrap();
        let mut p = make_para(&["Date: 2024-01-15"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "$3/$2/$1");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Date: 15/01/2024");
    }

    #[test]
    fn regex_replace_multiple_matches() {
        let re = regex::Regex::new(r"\b[A-Z]\w+").unwrap();
        let mut p = make_para(&["Hello World Today"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "X");
        assert_eq!(count, 3);
        assert_eq!(p.text(), "X X X");
    }

    #[test]
    fn regex_replace_cross_run() {
        let re = regex::Regex::new(r"\{\{name\}\}").unwrap();
        let mut p = make_para(&["Hello {{", "name}}, welcome!"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "Bob");
        assert_eq!(count, 1);
        assert_eq!(p.text(), "Hello Bob, welcome!");
    }

    #[test]
    fn regex_replace_no_match() {
        let re = regex::Regex::new(r"xyz\d+").unwrap();
        let mut p = make_para(&["Hello World"]);
        let count = replace_regex_in_paragraph(&mut p, &re, "X");
        assert_eq!(count, 0);
        assert_eq!(p.text(), "Hello World");
    }

    #[test]
    fn regex_replace_in_table() {
        use crate::table::{CT_Row, CT_Tc, CellContent};

        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        let mut table = CT_Tbl::new();
        let mut row = CT_Row::new();
        let mut cell = CT_Tc::new();
        let mut p = CT_P::new();
        p.add_run("Value: {{item}}");
        cell.content = vec![CellContent::Paragraph(p)];
        row.cells.push(cell);
        table.rows.push(row);

        let count = replace_regex_in_table(&mut table, &re, "[$1]");
        assert_eq!(count, 1);

        let para = match &table.rows[0].cells[0].content[0] {
            CellContent::Paragraph(p) => p,
            _ => panic!("expected paragraph"),
        };
        assert_eq!(para.text(), "Value: [item]");
    }
}
