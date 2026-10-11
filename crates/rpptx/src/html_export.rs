//! Semantic HTML export of a presentation, one section per slide.
//!
//! The HTML import in `html.rs` needs an HTML parser behind the
//! `default-template` feature. The export needs none, so it lives apart and
//! every build of the facade has it.

use oxml_drawing::text::{CT_TextCharacterProperties, TextRun, TextStrike, TextUnderline};

use crate::{
    ExportOptions, Presentation, SlideContent, TableRef, TextParagraphRef, is_external_link,
    picture_alt_text,
};

impl Presentation {
    /// Writes the presentation as one semantic HTML document.
    ///
    /// Each slide is a `section` headed by its title, or by `Slide N` when
    /// it has none. The rest follows [`crate::SlideRef::content`] as the
    /// Markdown export reads it: paragraphs are list items nested by level,
    /// with direct bold, italic, underline, strikethrough, superscript,
    /// subscript and external links, tables keep their merged cells and the
    /// header row their style marks, and pictures are images embedded as
    /// data URIs with their alt text. Speaker notes follow in an `aside`
    /// when `options.notes` is set. Slide geometry is not reproduced, which
    /// the PNG export covers. Text and attributes are escaped and void
    /// elements self-closed, so the document also parses as XML.
    pub fn to_html(&self, options: ExportOptions) -> String {
        let document_title = self
            .core_properties()
            .and_then(|properties| properties.title.as_deref())
            .map(str::trim)
            .filter(|title| !title.is_empty())
            .unwrap_or("Presentation");
        let mut out =
            String::from("<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\" />\n<title>");
        push_html_text(&mut out, document_title);
        out.push_str("</title>\n</head>\n<body>\n");
        for (index, slide) in self.slides().enumerate() {
            if options.skip_hidden && slide.hidden() {
                continue;
            }
            let link = |relationship_id: &str| {
                self.hyperlink_address(index, relationship_id)
                    .filter(|address| is_external_link(address))
            };
            out.push_str(&format!("<section id=\"slide-{}\">\n<h1>", index + 1));
            let title = slide
                .title()
                .and_then(|shape| shape.text_frame())
                .map(|frame| {
                    (0..frame.paragraph_count())
                        .filter_map(|paragraph| frame.paragraph(paragraph))
                        .filter(|paragraph| !paragraph.text().trim().is_empty())
                        .map(|paragraph| html_inline(paragraph, &link))
                        .collect::<Vec<_>>()
                        .join("<br />")
                })
                .filter(|title| !title.is_empty())
                .unwrap_or_else(|| format!("Slide {}", index + 1));
            out.push_str(&title);
            out.push_str("</h1>\n");

            // The levels of the open list items, one open list each.
            let mut open_items: Vec<u8> = Vec::new();
            for content in slide.content() {
                match content {
                    SlideContent::Paragraph(paragraph) => {
                        if paragraph.text().trim().is_empty() {
                            continue;
                        }
                        // The item closes the deeper items before it and
                        // becomes a sibling of the last one it closes, or
                        // opens a list of its own when it closes none.
                        let level = paragraph.level();
                        let mut closed = 0;
                        while open_items.last().is_some_and(|&open| open >= level) {
                            if closed > 0 {
                                out.push_str("</ul>\n");
                            }
                            out.push_str("</li>\n");
                            open_items.pop();
                            closed += 1;
                        }
                        if closed == 0 {
                            out.push_str(if open_items.is_empty() {
                                "<ul>\n"
                            } else {
                                "\n<ul>\n"
                            });
                        }
                        open_items.push(level);
                        out.push_str("<li>");
                        out.push_str(&html_inline(paragraph, &link));
                    }
                    SlideContent::Table(table) => {
                        close_html_list(&mut out, &mut open_items);
                        push_html_table(&mut out, table, &link);
                    }
                    SlideContent::Picture(picture) => {
                        close_html_list(&mut out, &mut open_items);
                        let alt = picture_alt_text(picture).unwrap_or_default();
                        let image = picture
                            .non_visual_id()
                            .and_then(|id| self.picture_image(index, id).ok())
                            .filter(|image| image.content_type.starts_with("image/"));
                        if let Some(image) = image {
                            out.push_str("<img src=\"data:");
                            push_html_text(&mut out, &image.content_type);
                            out.push_str(";base64,");
                            push_base64(&mut out, image.bytes);
                            out.push_str("\" alt=\"");
                            push_html_text(&mut out, &alt);
                            out.push_str("\" />\n");
                        } else if !alt.is_empty() {
                            // An image the package does not hold keeps its
                            // description as text.
                            out.push_str("<p>");
                            push_html_text(&mut out, &alt);
                            out.push_str("</p>\n");
                        }
                    }
                }
            }
            close_html_list(&mut out, &mut open_items);

            if options.notes {
                let notes = slide.notes_text().unwrap_or_default();
                let lines: Vec<&str> = notes
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .collect();
                if !lines.is_empty() {
                    out.push_str("<aside class=\"notes\">\n<h2>Notes</h2>\n");
                    for line in lines {
                        out.push_str("<p>");
                        push_html_text(&mut out, line);
                        out.push_str("</p>\n");
                    }
                    out.push_str("</aside>\n");
                }
            }
            out.push_str("</section>\n");
        }
        out.push_str("</body>\n</html>\n");
        out
    }
}

/// Close every open list item and list.
fn close_html_list(out: &mut String, open_items: &mut Vec<u8>) {
    for _ in open_items.drain(..) {
        out.push_str("</li>\n</ul>\n");
    }
}

/// The run formatting the HTML export writes as elements.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
struct HtmlMarks {
    bold: bool,
    italic: bool,
    underline: bool,
    strike: bool,
    /// Positive for superscript, negative for subscript.
    baseline: i8,
}

impl HtmlMarks {
    /// The marks of direct run properties. Formatting a run inherits from
    /// its placeholder, layout or master is not read.
    fn from_properties(properties: Option<&CT_TextCharacterProperties>) -> Self {
        let Some(properties) = properties else {
            return Self::default();
        };
        let baseline = properties
            .baseline
            .as_deref()
            .and_then(|baseline| baseline.trim_end_matches('%').parse::<f64>().ok())
            .unwrap_or(0.0);
        Self {
            bold: properties.bold == Some(true),
            italic: properties.italic == Some(true),
            underline: properties
                .underline
                .is_some_and(|underline| underline != TextUnderline::None),
            strike: matches!(
                properties.strike,
                Some(TextStrike::Single | TextStrike::Double)
            ),
            baseline: if baseline > 0.0 {
                1
            } else if baseline < 0.0 {
                -1
            } else {
                0
            },
        }
    }

    /// The elements to open, outermost first.
    fn tags(self) -> Vec<&'static str> {
        [
            (self.bold, "strong"),
            (self.italic, "em"),
            (self.underline, "u"),
            (self.strike, "s"),
            (self.baseline > 0, "sup"),
            (self.baseline < 0, "sub"),
        ]
        .into_iter()
        .filter_map(|(on, tag)| on.then_some(tag))
        .collect()
    }
}

/// Render a paragraph's runs, breaks and fields as escaped inline HTML.
/// Neighbouring runs with the same formatting share their elements, and
/// neighbouring runs with the same external link share one anchor.
fn html_inline<'a>(
    paragraph: TextParagraphRef<'_>,
    link: &impl Fn(&str) -> Option<&'a str>,
) -> String {
    // Escaped text, its marks, and its link.
    let mut pieces: Vec<(String, HtmlMarks, Option<&'a str>)> = Vec::new();
    let mut push = |text: &str, marks: HtmlMarks, link: Option<&'a str>| match pieces.last_mut() {
        Some((last, last_marks, last_link)) if *last_marks == marks && *last_link == link => {
            last.push_str(text);
        }
        _ => pieces.push((text.to_owned(), marks, link)),
    };
    let mut current_link = None;
    for run in &paragraph.paragraph.runs {
        let (text, properties) = match run {
            TextRun::Run(run) => (run.text.value.as_str(), run.properties.as_ref()),
            TextRun::Field(field) => (
                field.text.as_ref().map_or("", |text| text.value.as_str()),
                field.run_properties.as_ref(),
            ),
            TextRun::Break(_) => {
                push("<br />", HtmlMarks::default(), current_link);
                continue;
            }
        };
        current_link = properties
            .and_then(|properties| properties.hyperlink_click.as_ref())
            .and_then(|hyperlink| hyperlink.relationship_id.as_deref())
            .and_then(link);
        let marks = HtmlMarks::from_properties(properties);
        for (line, segment) in text.split(['\n', '\r', '\u{000b}']).enumerate() {
            if line > 0 {
                push("<br />", HtmlMarks::default(), current_link);
            }
            let mut escaped = String::new();
            push_html_text(&mut escaped, segment);
            if !escaped.is_empty() {
                push(&escaped, marks, current_link);
            }
        }
    }

    let mut out = String::new();
    let mut open_link = None;
    for (text, marks, link) in &pieces {
        if *link != open_link {
            if open_link.is_some() {
                out.push_str("</a>");
            }
            if let Some(address) = link {
                out.push_str("<a href=\"");
                push_html_text(&mut out, address);
                out.push_str("\">");
            }
            open_link = *link;
        }
        let tags = marks.tags();
        for tag in &tags {
            out.push_str(&format!("<{tag}>"));
        }
        out.push_str(text);
        for tag in tags.iter().rev() {
            out.push_str(&format!("</{tag}>"));
        }
    }
    if open_link.is_some() {
        out.push_str("</a>");
    }
    out
}

/// Write a table, its cells' paragraphs joined by line breaks. A merge
/// origin spans its cells, which are left out, and the first row is a
/// header row when the table style marks it.
fn push_html_table<'a>(
    out: &mut String,
    table: TableRef<'_>,
    link: &impl Fn(&str) -> Option<&'a str>,
) {
    let (rows, columns) = (table.row_count(), table.column_count());
    if rows == 0 || columns == 0 {
        return;
    }
    let header_rows = usize::from(table.first_row());
    out.push_str("<table>\n");
    for row in 0..rows {
        let in_header = row < header_rows;
        if row == 0 && in_header {
            out.push_str("<thead>\n");
        } else if row == header_rows {
            out.push_str("<tbody>\n");
        }
        // A row span ends with its row group.
        let group_end = if in_header { header_rows } else { rows };
        out.push_str("<tr>");
        for column in 0..columns {
            let Some(cell) = table.cell(row, column).filter(|cell| !cell.is_spanned()) else {
                continue;
            };
            let tag = if in_header { "th" } else { "td" };
            out.push('<');
            out.push_str(tag);
            let column_span = (cell.span_width() as usize).min(columns - column);
            let row_span = (cell.span_height() as usize).min(group_end - row);
            if column_span > 1 {
                out.push_str(&format!(" colspan=\"{column_span}\""));
            }
            if row_span > 1 {
                out.push_str(&format!(" rowspan=\"{row_span}\""));
            }
            out.push('>');
            if let Some(frame) = cell.text_frame() {
                let paragraphs: Vec<String> = (0..frame.paragraph_count())
                    .filter_map(|paragraph| frame.paragraph(paragraph))
                    .filter(|paragraph| !paragraph.text().trim().is_empty())
                    .map(|paragraph| html_inline(paragraph, link))
                    .collect();
                out.push_str(&paragraphs.join("<br />"));
            }
            out.push_str(&format!("</{tag}>"));
        }
        out.push_str("</tr>\n");
        if row + 1 == header_rows {
            out.push_str("</thead>\n");
        }
    }
    if rows > header_rows {
        out.push_str("</tbody>\n");
    }
    out.push_str("</table>\n");
}

/// Append text escaped for an HTML text node or a quoted attribute value.
/// Control characters other than tab have no place in either and are left
/// out.
fn push_html_text(out: &mut String, text: &str) {
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            '\t' => out.push('\t'),
            character if character.is_control() => {}
            character => out.push(character),
        }
    }
}

/// Append bytes as standard padded base64.
fn push_base64(out: &mut String, bytes: &[u8]) {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    for chunk in bytes.chunks(3) {
        let word = chunk
            .iter()
            .enumerate()
            .fold(0_u32, |word, (index, &byte)| {
                word | u32::from(byte) << (16 - 8 * index)
            });
        for index in 0..4 {
            if index <= chunk.len() {
                out.push(char::from(
                    ALPHABET[(word >> (18 - 6 * index)) as usize & 63],
                ));
            } else {
                out.push('=');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_pads_a_last_group_of_one_or_two_bytes() {
        for (bytes, expected) in [
            (&b""[..], ""),
            (b"M", "TQ=="),
            (b"Ma", "TWE="),
            (b"Man", "TWFu"),
            (b"\xff\xfe\xfd\xfc", "//79/A=="),
        ] {
            let mut out = String::new();
            push_base64(&mut out, bytes);
            assert_eq!(out, expected);
        }
    }
}
