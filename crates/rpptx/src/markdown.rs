//! Markdown export of a presentation, one section per slide.
//!
//! The inline renderer follows the one in `rdocx-html`, which writes
//! emphasis CommonMark reads as the document means. rpptx does not depend on
//! the WordprocessingML crates, so the logic lives here as well.

use std::cmp::Reverse;

use oxml_drawing::text::{CT_TextCharacterProperties, TextRun, TextStrike};

use crate::{
    ExportOptions, Presentation, SlideContent, TableRef, TextParagraphRef, is_external_link,
    picture_alt_text,
};

impl Presentation {
    /// Writes the presentation as CommonMark with GitHub pipe tables.
    ///
    /// Each slide is a section headed by its title, or by `Slide N` when it
    /// has none. The rest follows [`crate::SlideRef::content`]: paragraphs
    /// are list items nested by their level, with bold, italic,
    /// strikethrough and external links, tables are pipe tables whose first
    /// row is the header, and pictures are images carrying their alt text.
    /// An image has an empty destination, since its bytes stay in the
    /// package. Speaker notes follow as a block quote when `options.notes`
    /// is set.
    pub fn to_markdown(&self, options: ExportOptions) -> String {
        let mut out = String::new();
        for (index, slide) in self.slides().enumerate() {
            if options.skip_hidden && slide.hidden() {
                continue;
            }
            let link = |relationship_id: &str| {
                self.hyperlink_address(index, relationship_id)
                    .filter(|address| is_external_link(address))
            };

            let title = slide
                .title()
                .and_then(|shape| shape.text_frame())
                .map(|frame| {
                    (0..frame.paragraph_count())
                        .filter_map(|paragraph| frame.paragraph(paragraph))
                        .map(|paragraph| paragraph_lines(paragraph, &link).join(" "))
                        .filter(|text| !text.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .filter(|title| !title.is_empty())
                .unwrap_or_else(|| format!("Slide {}", index + 1));
            out.push_str("# ");
            push_heading_text(&mut out, &title);
            out.push_str("\n\n");

            // The open list levels, each with the column its item text starts at.
            let mut list_levels: Vec<(u8, usize)> = Vec::new();
            for content in slide.content() {
                match content {
                    SlideContent::Paragraph(paragraph) => {
                        let lines = paragraph_lines(paragraph, &link);
                        if lines.is_empty() {
                            continue;
                        }
                        let level = paragraph.level();
                        while list_levels.last().is_some_and(|&(open, _)| open >= level) {
                            list_levels.pop();
                        }
                        let indent = list_levels.last().map_or(0, |&(_, column)| column);
                        list_levels.push((level, indent + 2));
                        let separator = format!("{LINE_BREAK}{}", " ".repeat(indent + 2));
                        out.push_str(&" ".repeat(indent));
                        out.push_str("- ");
                        out.push_str(&lines.join(&separator));
                        out.push('\n');
                    }
                    SlideContent::Table(table) => {
                        leave_list(&mut out, &mut list_levels);
                        emit_table(&mut out, table, &link);
                    }
                    SlideContent::Picture(picture) => {
                        leave_list(&mut out, &mut list_levels);
                        out.push_str("![");
                        let alt = picture_alt_text(picture).unwrap_or_default();
                        for character in
                            alt.split_whitespace().collect::<Vec<_>>().join(" ").chars()
                        {
                            push_escaped(&mut out, character);
                        }
                        out.push_str("]()\n\n");
                    }
                }
            }
            leave_list(&mut out, &mut list_levels);

            if options.notes {
                let notes = slide.notes_text().unwrap_or_default();
                let lines: Vec<String> = notes
                    .lines()
                    .map(|line| escape_text(line.trim()))
                    .filter(|line| !line.is_empty())
                    .collect();
                if !lines.is_empty() {
                    out.push_str("> **Notes**\n");
                    for line in lines {
                        out.push_str(">\n> ");
                        out.push_str(&escape_block_start(&line));
                        out.push('\n');
                    }
                    out.push('\n');
                }
            }
        }
        let end = out.trim_end().len();
        out.truncate(end);
        if !out.is_empty() {
            out.push('\n');
        }
        out
    }
}

/// End an open list before another kind of block.
///
/// A list item ends in a single newline, and a block that follows it without
/// a blank line is read as a lazy continuation of the last item.
fn leave_list(out: &mut String, list_levels: &mut Vec<(u8, usize)>) {
    if !list_levels.is_empty() && !out.ends_with("\n\n") {
        out.push('\n');
    }
    list_levels.clear();
}

/// Append the text of an ATX heading. A run of `#` at its end would be read
/// as the closing sequence, so the run is escaped.
fn push_heading_text(out: &mut String, text: &str) {
    let kept = text.trim_end_matches('#');
    if kept.len() == text.len() || kept.ends_with('\\') {
        out.push_str(text);
    } else {
        out.push_str(kept);
        out.push('\\');
        out.push_str(&text[kept.len()..]);
    }
}

/// A hard line break inside a paragraph.
const LINE_BREAK: &str = "  \n";

/// Render a paragraph as CommonMark inline text, one entry per line between
/// its line breaks. Blank lines are left out, since a blank line would end
/// the paragraph, and each line is escaped where it would start a block.
fn paragraph_lines<'a>(
    paragraph: TextParagraphRef<'_>,
    link: &impl Fn(&str) -> Option<&'a str>,
) -> Vec<String> {
    let mut out = String::new();
    // Consecutive runs in the same link are written as one link, and the
    // formatting inside and outside each link is rendered as a whole.
    let mut current_link: Option<&str> = None;
    let mut pieces: Vec<(String, Marks)> = Vec::new();
    for run in &paragraph.paragraph.runs {
        let (text, properties) = match run {
            TextRun::Run(run) => (run.text.value.as_str(), run.properties.as_ref()),
            TextRun::Field(field) => (
                field.text.as_ref().map_or("", |text| text.value.as_str()),
                field.run_properties.as_ref(),
            ),
            TextRun::Break(_) => {
                push_piece(&mut pieces, LINE_BREAK, Marks::default());
                continue;
            }
        };
        let in_link = properties
            .and_then(|properties| properties.hyperlink_click.as_ref())
            .and_then(|hyperlink| hyperlink.relationship_id.as_deref())
            .and_then(link);
        if in_link != current_link {
            push_inline(&mut out, &pieces, current_link);
            pieces.clear();
            current_link = in_link;
        }
        let mut raw = String::new();
        for character in text.chars() {
            match character {
                '\n' | '\r' | '\u{000b}' => raw.push_str(LINE_BREAK),
                character => push_escaped(&mut raw, character),
            }
        }
        push_piece(&mut pieces, &raw, Marks::from_properties(properties));
    }
    push_inline(&mut out, &pieces, current_link);

    out.split('\n')
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(escape_block_start)
        .collect()
}

/// Escape plain text, such as a speaker-note line, for inline use.
fn escape_text(text: &str) -> String {
    let mut out = String::new();
    for character in text.chars() {
        push_escaped(&mut out, character);
    }
    out
}

/// Escape the start of a line that CommonMark would otherwise read as a
/// heading, block quote, list item, thematic break or setext underline.
fn escape_block_start(line: &str) -> String {
    let ends_marker = |rest: &str| rest.is_empty() || rest.starts_with([' ', '\t']);
    let hashes = line.bytes().take_while(|&byte| byte == b'#').count();
    let digits = line.bytes().take_while(u8::is_ascii_digit).count();
    let first = line.chars().next();
    let heading = (1..=6).contains(&hashes) && ends_marker(&line[hashes..]);
    let bullet_or_rule = matches!(first, Some('-' | '+' | '='))
        && (ends_marker(&line[1..]) || line.trim_end().chars().all(|c| Some(c) == first));
    let ordered = (1..=9).contains(&digits)
        && matches!(line.as_bytes().get(digits), Some(b'.' | b')'))
        && ends_marker(&line[digits + 1..]);
    let escape_at = if heading || first == Some('>') || bullet_or_rule {
        Some(0)
    } else if ordered {
        Some(digits)
    } else {
        None
    };
    match escape_at {
        Some(at) => format!("{}\\{}", &line[..at], &line[at..]),
        None => line.to_owned(),
    }
}

/// The run formatting Markdown can spell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mark {
    Strike,
    Bold,
    Italic,
}

impl Mark {
    /// Every mark, outermost first when several start and end together.
    /// Strikethrough goes innermost, next to the text, because cmark-gfm
    /// reads a `~` beside a `*` run as a letter rather than punctuation.
    const ALL: [Mark; 3] = [Mark::Bold, Mark::Italic, Mark::Strike];

    fn delimiter(self) -> &'static str {
        match self {
            Mark::Strike => "~~",
            Mark::Bold => "**",
            Mark::Italic => "*",
        }
    }

    fn tag(self) -> &'static str {
        match self {
            Mark::Strike => "del",
            Mark::Bold => "strong",
            Mark::Italic => "em",
        }
    }
}

/// The set of marks a stretch of text carries.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Marks {
    strike: bool,
    bold: bool,
    italic: bool,
}

impl Marks {
    /// The marks of direct run properties. Formatting a run inherits from
    /// its placeholder, layout or master is not read.
    fn from_properties(properties: Option<&CT_TextCharacterProperties>) -> Self {
        Marks {
            strike: properties.is_some_and(|properties| {
                matches!(
                    properties.strike,
                    Some(TextStrike::Single | TextStrike::Double)
                )
            }),
            bold: properties.is_some_and(|properties| properties.bold == Some(true)),
            italic: properties.is_some_and(|properties| properties.italic == Some(true)),
        }
    }

    fn has(self, mark: Mark) -> bool {
        match mark {
            Mark::Strike => self.strike,
            Mark::Bold => self.bold,
            Mark::Italic => self.italic,
        }
    }
}

/// Append formatted pieces to `out`, as the text of a link when `link` is set.
fn push_inline(out: &mut String, pieces: &[(String, Marks)], link: Option<&str>) {
    let text = render_inline(pieces);
    match link {
        Some(url) if !text.trim().is_empty() => {
            // `![` would start an image.
            if out.ends_with('!') {
                out.pop();
                out.push_str("\\!");
            }
            out.push('[');
            out.push_str(&text);
            out.push_str("](");
            push_link_destination(out, url);
            out.push(')');
        }
        _ => out.push_str(&text),
    }
}

/// Write a URL as a link destination. A space or control character ends a
/// destination and an unbalanced parenthesis ends the link, so those are
/// percent-encoded or escaped.
fn push_link_destination(out: &mut String, url: &str) {
    for character in url.chars() {
        match character {
            ' ' => out.push_str("%20"),
            character if character.is_ascii_control() => {
                out.push_str(&format!("%{:02X}", u32::from(character)));
            }
            '(' | ')' | '<' | '>' | '\\' => {
                out.push('\\');
                out.push(character);
            }
            character => out.push(character),
        }
    }
}

/// Append `text` with `marks`, merged into the previous piece when its marks
/// are the same.
fn push_piece(pieces: &mut Vec<(String, Marks)>, text: &str, marks: Marks) {
    if text.is_empty() {
        return;
    }
    match pieces.last_mut() {
        Some((last, last_marks)) if *last_marks == marks => last.push_str(text),
        _ => pieces.push((text.to_owned(), marks)),
    }
}

/// Append a character of slide text, escaping the punctuation that would
/// otherwise start emphasis, a code span, a link, raw HTML, an entity
/// reference or strikethrough.
fn push_escaped(text: &mut String, character: char) {
    if matches!(
        character,
        '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '~' | '&'
    ) {
        text.push('\\');
    }
    text.push(character);
}

/// One emphasis, strikethrough or bold span between two boundaries.
struct Span {
    mark: Mark,
    /// The boundaries it opens and closes at.
    open: usize,
    close: usize,
    /// Written as an HTML element because CommonMark would not read
    /// delimiters at one of its boundaries as intended.
    html: bool,
}

/// Render formatted pieces as CommonMark inline text.
///
/// Whitespace at the edge of a piece is written outside the delimiters,
/// because a delimiter next to whitespace on its inner side does not open or
/// close. A piece of whitespace alone takes no delimiters. Marks are opened
/// and closed only where they change, so neighbouring pieces with the same
/// marks share one pair of delimiters, and spans nest. A span whose
/// delimiters CommonMark would read as literal text at a boundary, next to
/// punctuation inside a word for example, is written as an HTML element.
fn render_inline(pieces: &[(String, Marks)]) -> String {
    // The text between whitespace, and the whitespace before, between and
    // after it. `gaps` has one more entry than `cores`.
    let mut cores: Vec<(String, Marks)> = Vec::new();
    let mut gaps = vec![String::new()];
    for (text, marks) in pieces {
        let core = text.trim();
        if core.is_empty() {
            gaps.last_mut().expect("gaps is never empty").push_str(text);
            continue;
        }
        let start = text.len() - text.trim_start().len();
        let gap = gaps.last_mut().expect("gaps is never empty");
        gap.push_str(&text[..start]);
        match cores.last_mut() {
            Some((last, last_marks)) if gap.is_empty() && last_marks == marks => {
                last.push_str(core);
            }
            _ => {
                cores.push((core.to_owned(), *marks));
                gaps.push(String::new());
            }
        }
        gaps.last_mut()
            .expect("gaps is never empty")
            .push_str(&text[start + core.len()..]);
    }

    // Boundary `b` lies before core `b`, and the last boundary after every
    // core. Each lists the spans it closes, innermost first, those that stay
    // open across it, then those it opens, outermost first. Each mark is
    // open at most once, so none of these lists is longer than three.
    let boundaries = cores.len() + 1;
    let mut spans: Vec<Span> = Vec::new();
    let mut closes: Vec<Vec<usize>> = vec![Vec::new(); boundaries];
    let mut encloses: Vec<Vec<usize>> = vec![Vec::new(); boundaries];
    let mut opens: Vec<Vec<usize>> = vec![Vec::new(); boundaries];
    let mut stack: Vec<usize> = Vec::new();
    for boundary in 0..boundaries {
        let wanted = cores
            .get(boundary)
            .map_or(Marks::default(), |(_, marks)| *marks);
        // Spans nest, so a mark that ends closes every span opened after it.
        if let Some(depth) = stack.iter().position(|&span| !wanted.has(spans[span].mark)) {
            for span in stack.drain(depth..).rev() {
                spans[span].close = boundary;
                closes[boundary].push(span);
            }
        }
        encloses[boundary].clone_from(&stack);
        let mut starting: Vec<Mark> = Mark::ALL
            .into_iter()
            .filter(|&mark| wanted.has(mark) && !stack.iter().any(|&span| spans[span].mark == mark))
            .collect();
        // The longest lasting mark goes outermost, so it need not be closed
        // and reopened when a shorter one ends.
        starting.sort_by_key(|&mark| {
            Reverse(
                cores[boundary..]
                    .iter()
                    .take_while(|(_, marks)| marks.has(mark))
                    .count(),
            )
        });
        for mark in starting {
            stack.push(spans.len());
            opens[boundary].push(spans.len());
            spans.push(Span {
                mark,
                open: boundary,
                close: boundaries,
                html: false,
            });
        }
    }

    // A span becomes HTML when one of its boundaries fails, which changes
    // the neighbours of the delimiters at its other boundary, so that one is
    // checked again. Fewer delimiters never make another boundary fail.
    let mut pending: Vec<usize> = (0..boundaries).collect();
    while let Some(boundary) = pending.pop() {
        let before = boundary
            .checked_sub(1)
            .and_then(|core| cores[core].0.chars().last());
        let after = cores
            .get(boundary)
            .and_then(|(core, _)| core.chars().next());
        let closing: Vec<(&Span, bool)> = closes[boundary]
            .iter()
            .map(|&span| (&spans[span], false))
            .collect();
        let opening: Vec<(&Span, bool)> = opens[boundary]
            .iter()
            .map(|&span| (&spans[span], true))
            .collect();
        let enclosed = encloses[boundary]
            .iter()
            .any(|&span| !spans[span].html && spans[span].mark != Mark::Strike);
        let gap = &gaps[boundary];
        let holds = if gap.is_empty() {
            delimiters_hold(before, &[closing, opening].concat(), after, enclosed)
        } else {
            delimiters_hold(before, &closing, gap.chars().next(), enclosed)
                && delimiters_hold(gap.chars().last(), &opening, after, enclosed)
        };
        if holds {
            continue;
        }
        for &span in closes[boundary].iter().chain(&opens[boundary]) {
            if !spans[span].html {
                spans[span].html = true;
                pending.push(spans[span].open);
                pending.push(spans[span].close);
            }
        }
    }

    let mut out = String::new();
    for boundary in 0..boundaries {
        for &span in &closes[boundary] {
            let Span { mark, html, .. } = spans[span];
            if html {
                out.push_str(&format!("</{}>", mark.tag()));
            } else {
                out.push_str(mark.delimiter());
            }
        }
        out.push_str(&gaps[boundary]);
        for &span in &opens[boundary] {
            let Span { mark, html, .. } = spans[span];
            if html {
                out.push_str(&format!("<{}>", mark.tag()));
            } else {
                out.push_str(mark.delimiter());
            }
        }
        if let Some((core, _)) = cores.get(boundary) {
            out.push_str(core);
        }
    }
    out
}

/// How a delimiter run sees the character on one side of it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Flank {
    Whitespace,
    Punctuation,
    Other,
}

impl Flank {
    /// The readings of a character. `None` is the edge of the inline text,
    /// which counts as whitespace. cmark-gfm does not count `~` as
    /// punctuation, unlike the CommonMark spec, and CommonMark versions
    /// differ on which other Unicode symbols are, so those are read both ways.
    fn of(character: Option<char>) -> &'static [Flank] {
        match character {
            None => &[Flank::Whitespace],
            Some(c) if c.is_whitespace() => &[Flank::Whitespace],
            Some(c) if c.is_alphanumeric() => &[Flank::Other],
            Some(c) if c.is_ascii_punctuation() && c != '~' => &[Flank::Punctuation],
            Some(_) => &[Flank::Punctuation, Flank::Other],
        }
    }
}

/// Whether CommonMark reads each delimiter in `tokens`, written between the
/// characters `before` and `after`, as the opener or closer it is meant to
/// be. `enclosed` says whether a `*` span stays open across the boundary.
///
/// Neighbouring delimiters of the same character form one delimiter run. A
/// run that closes must be right-flanking and one that opens left-flanking.
/// A run that is both could close an enclosing `*` span instead of opening,
/// and past three characters the rule of three can pair it differently. A
/// `~` run must be exactly `~~`. Every reading of a neighbouring character
/// must hold.
fn delimiters_hold(
    before: Option<char>,
    tokens: &[(&Span, bool)],
    after: Option<char>,
    enclosed: bool,
) -> bool {
    let character = |span: &Span| if span.mark == Mark::Strike { '~' } else { '*' };
    // A neighbouring token is an HTML tag or a delimiter of the other character.
    let neighbour = |span: &Span| Flank::of(Some(if span.html { '<' } else { character(span) }));

    let mut start = 0;
    while start < tokens.len() {
        let (span, _) = tokens[start];
        if span.html {
            start += 1;
            continue;
        }
        let delimiter = character(span);
        let mut end = start;
        while end < tokens.len() && !tokens[end].0.html && character(tokens[end].0) == delimiter {
            end += 1;
        }
        let run = &tokens[start..end];
        let previous = start
            .checked_sub(1)
            .map_or(Flank::of(before), |index| neighbour(tokens[index].0));
        let next = tokens
            .get(end)
            .map_or(Flank::of(after), |(span, _)| neighbour(span));
        let length: usize = run
            .iter()
            .map(|(span, _)| span.mark.delimiter().len())
            .sum();
        let closes = run.iter().any(|&(_, opens)| !opens);
        let opens = run.iter().any(|&(_, opens)| opens);
        for &previous in previous {
            for &next in next {
                let left = next != Flank::Whitespace
                    && (next != Flank::Punctuation || previous != Flank::Other);
                let right = previous != Flank::Whitespace
                    && (previous != Flank::Punctuation || next != Flank::Other);
                if (closes && !right)
                    || (opens && !left)
                    || (opens && right && enclosed && delimiter == '*')
                    || (left && right && length > 3)
                    || (delimiter == '~' && length != 2)
                {
                    return false;
                }
            }
        }
        start = end;
    }
    true
}

/// Emit a table as a GFM pipe table whose first row is the header. A cell
/// that continues a merge is left empty.
fn emit_table<'a>(out: &mut String, table: TableRef<'_>, link: &impl Fn(&str) -> Option<&'a str>) {
    let columns = table.column_count();
    if table.row_count() == 0 || columns == 0 {
        return;
    }
    for row in 0..table.row_count() {
        out.push('|');
        for column in 0..columns {
            // A pipe table row is one line, so the paragraphs and line
            // breaks of a cell are joined by HTML line breaks. A pipe would
            // end the cell, in a link destination too, so every pipe is
            // escaped, and GFM removes the escape before it reads the
            // cell's inlines.
            let text = table
                .cell(row, column)
                .filter(|cell| !cell.is_spanned())
                .and_then(|cell| {
                    let frame = cell.text_frame()?;
                    Some(
                        (0..frame.paragraph_count())
                            .filter_map(|paragraph| frame.paragraph(paragraph))
                            .map(|paragraph| paragraph_lines(paragraph, link).join("<br>"))
                            .filter(|text| !text.is_empty())
                            .collect::<Vec<_>>()
                            .join("<br>"),
                    )
                })
                .unwrap_or_default();
            out.push(' ');
            out.push_str(&text.replace('|', "\\|"));
            out.push_str(" |");
        }
        out.push('\n');
        if row == 0 {
            out.push('|');
            out.push_str(&" --- |".repeat(columns));
            out.push('\n');
        }
    }
    out.push('\n');
}
