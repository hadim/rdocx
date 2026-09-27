//! Placeholder replacement across paragraphs, tables, and headers/footers.
//!
//! Handles the cross-run splitting problem: a placeholder like `{{name}}`
//! may be split across multiple `<w:r>` elements in the OOXML source.

use crate::header_footer::CT_HdrFtr;
use crate::table::CT_Tbl;
use crate::text::{CT_P, CT_R, RunContent};

/// Replace all occurrences of `placeholder` with `replacement` in a paragraph.
///
/// Handles placeholders split across multiple runs. Preserves the formatting
/// of the first matched run. Returns the number of replacements made.
pub fn replace_in_paragraph(para: &mut CT_P, placeholder: &str, replacement: &str) -> usize {
    if placeholder.is_empty() {
        return 0;
    }

    let mut total = 0;

    // Byte offset in the concatenated paragraph text at which to look for the
    // next match. Resuming *after* the text we just inserted is what keeps this
    // terminating when the replacement itself contains the placeholder
    // (`replace("NAME", "NAME Smith")`); restarting from 0 would rematch the
    // replacement forever.
    let mut search_from = 0usize;

    // We loop because after one replacement the text layout changes
    // and there may be more matches.
    loop {
        // 1. Concatenate all run text and build a char map.
        let (full_text, char_map) = build_char_map(&para.runs);

        if search_from >= full_text.len() {
            break;
        }

        // 2. Find the next match (byte offset in full_text).
        let Some(byte_start) = full_text[search_from..]
            .find(placeholder)
            .map(|i| search_from + i)
        else {
            break;
        };
        // Convert byte offsets to char indices (char_map is indexed by char position).
        let match_start = full_text[..byte_start].chars().count();
        let match_end = match_start + placeholder.chars().count();

        // 3. Determine which runs are affected.
        let first_char = &char_map[match_start];
        let last_char = &char_map[match_end - 1];
        let first_run = first_char.run_index;
        let last_run = last_char.run_index;

        if first_run == last_run {
            // Single-run match: simple in-place replacement on that run's text content.
            replace_in_single_run(
                &mut para.runs[first_run],
                &char_map,
                match_start,
                match_end,
                replacement,
            );
        } else {
            // Cross-run match: put replacement in first run, clear matched parts from others.
            replace_across_runs(
                &mut para.runs,
                &char_map,
                match_start,
                match_end,
                first_run,
                last_run,
                replacement,
            );
        }

        // Remove runs that became completely empty (no content at all).
        para.runs.retain(|r| !r.content.is_empty());

        // Update hyperlink spans to account for removed runs.
        reindex_hyperlinks(para);

        // Text before `byte_start` is untouched and the replacement now
        // occupies `byte_start..byte_start + replacement.len()`, so this stays
        // on a char boundary of the rebuilt text.
        search_from = byte_start + replacement.len();

        total += 1;
    }

    total
}

/// A mapping from character position in the concatenated text to its source run and content item.
#[derive(Debug)]
struct CharMapping {
    /// Index of the run in the paragraph's runs vec.
    run_index: usize,
    /// Index of the RunContent item within the run.
    content_index: usize,
    /// Byte offset within the text string of the RunContent::Text item.
    byte_offset: usize,
}

fn build_char_map(runs: &[CT_R]) -> (String, Vec<CharMapping>) {
    let mut full_text = String::new();
    let mut char_map = Vec::new();

    for (run_idx, run) in runs.iter().enumerate() {
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
        t.preserve_space = t.text.starts_with(' ') || t.text.ends_with(' ');
    }
}

fn replace_across_runs(
    runs: &mut [CT_R],
    char_map: &[CharMapping],
    match_start: usize,
    match_end: usize,
    first_run: usize,
    last_run: usize,
    replacement: &str,
) {
    // Handle the first run: replace from match start to end of text in that content item.
    let first_mapping = &char_map[match_start];
    let first_content_idx = first_mapping.content_index;
    let first_byte_offset = first_mapping.byte_offset;

    if let RunContent::Text(t) = &mut runs[first_run].content[first_content_idx] {
        let mut new_text = String::new();
        new_text.push_str(&t.text[..first_byte_offset]);
        new_text.push_str(replacement);
        t.text = new_text;
        t.preserve_space = t.text.starts_with(' ') || t.text.ends_with(' ');
    }

    // Handle the last run: replace from start to match end within that content item.
    let last_mapping = &char_map[match_end - 1];
    let last_content_idx = last_mapping.content_index;
    let last_byte_offset = last_mapping.byte_offset;

    if let RunContent::Text(t) = &mut runs[last_run].content[last_content_idx] {
        let remaining = &t.text[last_byte_offset..];
        let ch_len = remaining.chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        let byte_end = last_byte_offset + ch_len;
        t.text = t.text[byte_end..].to_string();
        t.preserve_space = t.text.starts_with(' ') || t.text.ends_with(' ');
    }

    // Clear text content from runs strictly between first and last.
    for run in &mut runs[(first_run + 1)..last_run] {
        run.content.retain(|c| !matches!(c, RunContent::Text(_)));
    }

    // If the last run's text is now empty, remove its text content too.
    if last_run != first_run {
        runs[last_run].content.retain(|c| {
            if let RunContent::Text(t) = c {
                !t.text.is_empty()
            } else {
                true
            }
        });
    }
}

/// Re-index hyperlink spans after runs may have been removed.
fn reindex_hyperlinks(para: &mut CT_P) {
    // After retain, run indices may have shifted. We rebuild by checking
    // which runs still exist. Since retain preserves order and only removes
    // empty runs, the relative order is maintained. However, hyperlink spans
    // referenced by index need adjustment.
    //
    // For simplicity: we decrement indices for each removed slot.
    // But since we already called retain, the runs are already compacted.
    // We need to adjust hyperlinks based on the new run count.
    //
    // The simplest correct approach: hyperlinks that referenced removed runs
    // get their range clamped/invalidated.
    para.hyperlinks.retain(|hl| hl.run_start < para.runs.len());
    for hl in &mut para.hyperlinks {
        if hl.run_end > para.runs.len() {
            hl.run_end = para.runs.len();
        }
    }
}

/// Replace all occurrences of `placeholder` in all paragraphs of a slice.
pub fn replace_in_paragraphs(paras: &mut [CT_P], placeholder: &str, replacement: &str) -> usize {
    paras
        .iter_mut()
        .map(|p| replace_in_paragraph(p, placeholder, replacement))
        .sum()
}

/// Replace all occurrences of `placeholder` in a table (recursively handles nested tables).
pub fn replace_in_table(table: &mut CT_Tbl, placeholder: &str, replacement: &str) -> usize {
    let mut count = 0;
    for (_, _, sdt) in &mut table.content_controls {
        count += replace_in_sdt(sdt, placeholder, replacement);
    }
    for row in &mut table.rows {
        count += replace_in_row(row, placeholder, replacement);
    }
    count
}

fn replace_in_sdt(
    sdt: &mut crate::content_control::CT_Sdt,
    placeholder: &str,
    replacement: &str,
) -> usize {
    use crate::content_control::SdtContent;

    let mut count = 0;
    for content in &mut sdt.content {
        count += match content {
            SdtContent::Paragraph(paragraph) => {
                replace_in_paragraph(paragraph, placeholder, replacement)
            }
            SdtContent::Table(table) => replace_in_table(table, placeholder, replacement),
            SdtContent::Row(row) => replace_in_row(row, placeholder, replacement),
            SdtContent::Cell(cell) => replace_in_cell(cell, placeholder, replacement),
            SdtContent::ContentControl(nested) => replace_in_sdt(nested, placeholder, replacement),
            SdtContent::Run(_) | SdtContent::RawXml(_) => 0,
        };
    }
    count
}

fn replace_in_row(row: &mut crate::table::CT_Row, placeholder: &str, replacement: &str) -> usize {
    let controls = row
        .content_controls
        .iter_mut()
        .map(|(_, _, sdt)| replace_in_sdt(sdt, placeholder, replacement))
        .sum::<usize>();
    controls
        + row
            .cells
            .iter_mut()
            .map(|cell| replace_in_cell(cell, placeholder, replacement))
            .sum::<usize>()
}

fn replace_in_cell(cell: &mut crate::table::CT_Tc, placeholder: &str, replacement: &str) -> usize {
    use crate::table::CellContent;

    cell.content
        .iter_mut()
        .map(|content| match content {
            CellContent::Paragraph(paragraph) => {
                replace_in_paragraph(paragraph, placeholder, replacement)
            }
            CellContent::Table(table) => replace_in_table(table, placeholder, replacement),
            CellContent::ContentControl(sdt) => replace_in_sdt(sdt, placeholder, replacement),
        })
        .sum()
}

/// Replace all occurrences of `placeholder` in a header or footer.
pub fn replace_in_header_footer(hf: &mut CT_HdrFtr, placeholder: &str, replacement: &str) -> usize {
    replace_in_paragraphs(&mut hf.paragraphs, placeholder, replacement)
}

/// Replace placeholders in text boxes and shapes within a raw XML part.
///
/// Walks the XML, finds `w:txbxContent` elements at any depth outside another
/// text box, parses their child `w:p` elements using `CT_P::from_xml`,
/// performs replacement, and re-serializes the paragraphs it changed. Every
/// other child of the text box, such as a table, a content control, a
/// bookmark or a paragraph without a match, is copied through verbatim in its
/// place. A text box nested inside another one is kept as it is, not edited.
/// Returns the modified XML and replacement count.
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

/// Walk `xml`, handing each paragraph of a `w:txbxContent` element to `edit`
/// and re-serialising it in place when `edit` counts a change. Every other
/// paragraph and child of the text box is copied through verbatim. Returns the
/// rewritten XML and the summed count.
fn rewrite_text_boxes(
    xml: &[u8],
    edit: &mut dyn FnMut(&mut CT_P) -> usize,
) -> crate::error::Result<(Vec<u8>, usize)> {
    use crate::error::OxmlError;
    use crate::namespace::matches_local_name;
    use crate::raw_xml::capture_element;
    use quick_xml::events::Event;
    use quick_xml::{Reader, Writer};

    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();
    let mut total_count = 0;

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
                                    para.to_xml(&mut writer)?;
                                }
                                total_count += count;
                            } else {
                                // A table, a content control or any other element
                                // the edit does not reach stays as it was.
                                let raw = capture_element(&mut reader, ie)?;
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
/// Uses the same cross-run char map algorithm as literal replacement.
/// Returns the number of replacements made.
pub fn replace_regex_in_paragraph(para: &mut CT_P, re: &regex::Regex, replacement: &str) -> usize {
    let mut total = 0;

    // See `replace_in_paragraph`: resume after the inserted text so a
    // replacement that itself matches the pattern cannot loop forever.
    // `captures_at` (rather than slicing) keeps anchors and look-around
    // evaluating against the full paragraph text.
    let mut search_from = 0usize;

    loop {
        let (full_text, char_map) = build_char_map(&para.runs);
        if char_map.is_empty() || search_from > full_text.len() {
            break;
        }

        // Find the next match
        let Some(m) = re.captures_at(&full_text, search_from).and_then(|caps| {
            let mat = caps.get(0)?;
            // Expand capture groups in replacement
            let mut expanded = String::new();
            caps.expand(replacement, &mut expanded);
            Some((mat.start(), mat.end(), expanded))
        }) else {
            break;
        };

        let (byte_start, byte_end, expanded_replacement) = m;

        // A zero-width match would neither consume input nor advance the
        // cursor; step past it so the loop always makes progress.
        if byte_start == byte_end {
            search_from = full_text[byte_start..]
                .chars()
                .next()
                .map_or(full_text.len() + 1, |c| byte_start + c.len_utf8());
            continue;
        }

        // Convert byte offsets to char indices
        let match_start = full_text[..byte_start].chars().count();
        let match_end = match_start + full_text[byte_start..byte_end].chars().count();

        if match_start >= char_map.len() || match_end == 0 || match_end > char_map.len() {
            break;
        }

        // Determine which runs are affected
        let first_run = char_map[match_start].run_index;
        let last_run = char_map[match_end - 1].run_index;

        if first_run == last_run {
            replace_in_single_run(
                &mut para.runs[first_run],
                &char_map,
                match_start,
                match_end,
                &expanded_replacement,
            );
        } else {
            replace_across_runs(
                &mut para.runs,
                &char_map,
                match_start,
                match_end,
                first_run,
                last_run,
                &expanded_replacement,
            );
        }

        para.runs.retain(|r| !r.content.is_empty());
        reindex_hyperlinks(para);
        search_from = byte_start + expanded_replacement.len();
        total += 1;
    }

    total
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

/// Replace regex matches in a table (recursively handles nested tables).
pub fn replace_regex_in_table(table: &mut CT_Tbl, re: &regex::Regex, replacement: &str) -> usize {
    let mut count = 0;
    for (_, _, sdt) in &mut table.content_controls {
        count += replace_regex_in_sdt(sdt, re, replacement);
    }
    for row in &mut table.rows {
        count += replace_regex_in_row(row, re, replacement);
    }
    count
}

fn replace_regex_in_sdt(
    sdt: &mut crate::content_control::CT_Sdt,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    use crate::content_control::SdtContent;

    let mut count = 0;
    for content in &mut sdt.content {
        count += match content {
            SdtContent::Paragraph(paragraph) => {
                replace_regex_in_paragraph(paragraph, re, replacement)
            }
            SdtContent::Table(table) => replace_regex_in_table(table, re, replacement),
            SdtContent::Row(row) => replace_regex_in_row(row, re, replacement),
            SdtContent::Cell(cell) => replace_regex_in_cell(cell, re, replacement),
            SdtContent::ContentControl(nested) => replace_regex_in_sdt(nested, re, replacement),
            SdtContent::Run(_) | SdtContent::RawXml(_) => 0,
        };
    }
    count
}

fn replace_regex_in_row(
    row: &mut crate::table::CT_Row,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    let controls = row
        .content_controls
        .iter_mut()
        .map(|(_, _, sdt)| replace_regex_in_sdt(sdt, re, replacement))
        .sum::<usize>();
    controls
        + row
            .cells
            .iter_mut()
            .map(|cell| replace_regex_in_cell(cell, re, replacement))
            .sum::<usize>()
}

fn replace_regex_in_cell(
    cell: &mut crate::table::CT_Tc,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    use crate::table::CellContent;

    cell.content
        .iter_mut()
        .map(|content| match content {
            CellContent::Paragraph(paragraph) => {
                replace_regex_in_paragraph(paragraph, re, replacement)
            }
            CellContent::Table(table) => replace_regex_in_table(table, re, replacement),
            CellContent::ContentControl(sdt) => replace_regex_in_sdt(sdt, re, replacement),
        })
        .sum()
}

/// Replace regex matches in a header or footer.
pub fn replace_regex_in_header_footer(
    hf: &mut CT_HdrFtr,
    re: &regex::Regex,
    replacement: &str,
) -> usize {
    replace_regex_in_paragraphs(&mut hf.paragraphs, re, replacement)
}

#[cfg(test)]
mod tests {
    use super::*;
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
