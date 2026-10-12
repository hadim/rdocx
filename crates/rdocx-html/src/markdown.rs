//! Markdown generation from OXML types (GFM-compatible).

use std::cmp::Reverse;
use std::collections::HashMap;

use rdocx_oxml::document::CT_Body;
use rdocx_oxml::numbering::CT_Numbering;
use rdocx_oxml::properties::CT_PPr;
use rdocx_oxml::styles::CT_Styles;
use rdocx_oxml::table::CT_Tbl;
use rdocx_oxml::text::{BreakType, CT_P, CT_R, RunContent, SpecialCharacter};

use crate::{Block, body_blocks, cell_blocks};

/// Emit the full body content as Markdown.
pub(crate) fn emit_markdown(
    body: &CT_Body,
    styles: &CT_Styles,
    numbering: Option<&CT_Numbering>,
    hyperlink_urls: &HashMap<String, String>,
) -> String {
    let mut out = String::new();
    // The open list levels, each with the column its item text starts at.
    let mut list_levels: Vec<(u32, usize)> = Vec::new();

    for block in body_blocks(body, true) {
        match block {
            Block::Paragraph(p, boundaries) => {
                emit_paragraph(
                    &mut out,
                    &p,
                    &boundaries,
                    styles,
                    numbering,
                    hyperlink_urls,
                    &mut list_levels,
                );
            }
            Block::Table(tbl) => {
                let mut table = String::new();
                emit_table(&mut table, tbl, hyperlink_urls);
                if !table.is_empty() {
                    leave_list(&mut out, &mut list_levels);
                    out.push_str(&table);
                }
            }
        }
    }

    out
}

/// End an open list before another kind of block.
///
/// A list item ends in a single newline, and a block that follows it without
/// a blank line is read as a lazy continuation of the last item.
fn leave_list(out: &mut String, list_levels: &mut Vec<(u32, usize)>) {
    if !list_levels.is_empty() && !out.ends_with("\n\n") {
        out.push('\n');
    }
    list_levels.clear();
}

/// Detect heading level from paragraph properties/style.
fn detect_heading_level(ppr: Option<&CT_PPr>, styles: &CT_Styles) -> Option<u32> {
    let ppr = ppr?;

    if let Some(lvl) = ppr.outline_lvl {
        return Some(lvl + 1);
    }

    let style_id = ppr.style_id.as_deref()?;
    if let Some(level) = style_id.strip_prefix("Heading")
        && let Ok(n) = level.parse::<u32>()
        && (1..=6).contains(&n)
    {
        return Some(n);
    }

    if let Some(style) = styles.get_by_id(style_id)
        && let Some(spr) = &style.ppr
        && let Some(lvl) = spr.outline_lvl
    {
        return Some(lvl + 1);
    }

    None
}

/// Detect if a paragraph is a list item. Returns (is_ordered, ilvl).
fn detect_list(para: &CT_P, numbering: Option<&CT_Numbering>) -> Option<(bool, u32)> {
    let ppr = para.properties.as_ref()?;
    let num_id = ppr.num_id?;
    let ilvl = ppr.num_ilvl.unwrap_or(0);

    if num_id == 0 {
        return None;
    }

    let numbering = numbering?;

    let abstract_id = numbering
        .nums
        .iter()
        .find(|n| n.num_id == num_id)
        .map(|n| n.abstract_num_id)?;

    let abstract_num = numbering
        .abstract_nums
        .iter()
        .find(|a| a.abstract_num_id == abstract_id)?;

    let level = abstract_num.levels.iter().find(|l| l.ilvl == ilvl)?;

    use rdocx_oxml::numbering::ST_NumberFormat;
    let is_ordered = match level.num_fmt.as_ref() {
        Some(ST_NumberFormat::Bullet) => false,
        None
        | Some(
            ST_NumberFormat::Decimal
            | ST_NumberFormat::UpperRoman
            | ST_NumberFormat::LowerRoman
            | ST_NumberFormat::UpperLetter
            | ST_NumberFormat::LowerLetter
            | ST_NumberFormat::Ordinal
            | ST_NumberFormat::None,
        ) => true,
        Some(_) => return None,
    };

    Some((is_ordered, ilvl))
}

/// Emit a paragraph as Markdown.
fn emit_paragraph(
    out: &mut String,
    para: &CT_P,
    boundaries: &[usize],
    styles: &CT_Styles,
    numbering: Option<&CT_Numbering>,
    hyperlink_urls: &HashMap<String, String>,
    list_levels: &mut Vec<(u32, usize)>,
) {
    let heading_level = detect_heading_level(para.properties.as_ref(), styles);
    let list_info = detect_list(para, numbering);

    // Collect inline text for the paragraph
    let text = collect_paragraph_text(para, hyperlink_urls, boundaries);

    // Skip empty paragraphs (but still emit blank line for spacing)
    if text.is_empty() && heading_level.is_none() && list_info.is_none() {
        out.push('\n');
        return;
    }

    if let Some(level) = heading_level {
        // Heading
        leave_list(out, list_levels);
        // An ATX heading is one line, so a line break is an HTML one.
        let hashes = "#".repeat(level.min(6) as usize);
        out.push_str(&hashes);
        out.push(' ');
        out.push_str(&text.trim().replace(LINE_BREAK, "<br>"));
        out.push_str("\n\n");
    } else if let Some((is_ordered, ilvl)) = list_info {
        // A nested item is indented to the text column of its parent item,
        // which is three columns after `1. ` and two after `- `.
        while list_levels.last().is_some_and(|&(level, _)| level >= ilvl) {
            list_levels.pop();
        }
        let indent = list_levels.last().map_or(0, |&(_, column)| column);
        let marker = if is_ordered { "1. " } else { "- " };
        list_levels.push((ilvl, indent + marker.len()));
        out.push_str(&format!("{}{marker}{}\n", " ".repeat(indent), text.trim()));
    } else {
        // Retain original edge whitespace and terminal hard-break spelling.
        leave_list(out, list_levels);
        out.push_str(&text);
        out.push_str("\n\n");
    }
}

/// A hard line break inside a paragraph.
const LINE_BREAK: &str = "  \n";
/// A page break, written as a thematic break between blank lines. Without
/// the blank line before it, `---` turns the line above into a heading.
const PAGE_BREAK: &str = "\n\n---\n\n";

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
    fn from_rpr(rpr: Option<&rdocx_oxml::properties::CT_RPr>) -> Self {
        Marks {
            strike: rpr.is_some_and(|r| r.strike == Some(true) || r.dstrike == Some(true)),
            bold: rpr.is_some_and(|r| r.bold == Some(true)),
            italic: rpr.is_some_and(|r| r.italic == Some(true)),
        }
    }

    fn overlay(self, properties: Option<&rdocx_oxml::properties::CT_RPr>) -> Self {
        let Some(properties) = properties else {
            return self;
        };
        Marks {
            strike: properties
                .strike
                .or(properties.dstrike)
                .unwrap_or(self.strike),
            bold: properties.bold.unwrap_or(self.bold),
            italic: properties.italic.unwrap_or(self.italic),
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

/// Collect all text from the accepted view of a paragraph, applying inline
/// formatting.
fn collect_paragraph_text(
    para: &CT_P,
    hyperlink_urls: &HashMap<String, String>,
    boundaries: &[usize],
) -> String {
    let mut out = String::new();

    // The runs the text readers read, those of content controls, tracked
    // insertions, smart tags and custom XML included, deleted ones left out.
    let (view, projected_boundaries) = para.accepted_view_with_field_owners();
    let para = &view;

    // Build hyperlink map
    let mut hyperlink_map: HashMap<usize, &str> = HashMap::new();
    for hl in &para.hyperlinks {
        // Markdown renderers turn `[text](javascript:...)` into a live link too,
        // so the same scheme allowlist applies here.
        if let Some(rel_id) = &hl.rel_id
            && let Some(url) = hyperlink_urls
                .get(rel_id)
                .map(String::as_str)
                .and_then(crate::sanitize::safe_url)
        {
            for i in hl.run_start..hl.run_end {
                hyperlink_map.insert(i, url);
            }
        }
    }

    // Consecutive runs in the same link are written as one link, and the
    // formatting inside and outside each link is rendered as a whole.
    let mut current_link: Option<&str> = None;
    let mut pieces: Vec<(String, Marks)> = Vec::new();

    for (run_idx, run) in para.runs.iter().enumerate() {
        if boundaries.contains(&run_idx) || projected_boundaries.contains(&run_idx) {
            pieces.push((String::new(), Marks::default()));
        }
        let in_link = hyperlink_map.get(&run_idx).copied();
        if in_link != current_link {
            push_inline(&mut out, &pieces, current_link);
            pieces.clear();
            current_link = in_link;
        }
        collect_run_pieces(run, &mut pieces);
    }
    push_inline(&mut out, &pieces, current_link);

    out
}

/// Append formatted pieces to `out`, as the text of a link when `link` is set.
fn push_inline(out: &mut String, pieces: &[(String, Marks)], link: Option<&str>) {
    let mut start = 0;
    for (index, (text, _)) in pieces.iter().enumerate() {
        if text == PAGE_BREAK {
            push_inline_segment(out, &pieces[start..index], link);
            out.push_str(PAGE_BREAK);
            start = index + 1;
        }
    }
    push_inline_segment(out, &pieces[start..], link);
}

fn push_inline_segment(out: &mut String, pieces: &[(String, Marks)], link: Option<&str>) {
    let text = render_inline(pieces);
    if text.is_empty() {
        return;
    }
    match link {
        Some(url) => {
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
        None => out.push_str(&text),
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
        Some((last, last_marks)) if !last.is_empty() && *last_marks == marks => last.push_str(text),
        _ => pieces.push((text.to_owned(), marks)),
    }
}

/// Append a character of document text, escaping the punctuation that would
/// otherwise start emphasis, a code span, a link, raw HTML or strikethrough.
fn push_escaped(text: &mut String, character: char) {
    if matches!(character, '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '~') {
        text.push('\\');
    }
    text.push(character);
}

/// Collect the text of a single run as formatted pieces.
fn collect_run_pieces(run: &CT_R, pieces: &mut Vec<(String, Marks)>) {
    let marks = Marks::from_rpr(run.properties.as_ref());
    let mut raw = String::new();

    for content in &run.content {
        match content {
            RunContent::Text(t) | RunContent::DeletedText(t) => {
                for character in t.text.chars() {
                    push_escaped(&mut raw, character);
                }
            }
            RunContent::Tab => raw.push('\t'),
            RunContent::Break(bt) => match bt {
                BreakType::Line | BreakType::Column => {
                    push_piece(pieces, &std::mem::take(&mut raw), marks);
                    pieces.push((String::new(), Marks::default()));
                    pieces.push((LINE_BREAK.to_owned(), Marks::default()));
                    pieces.push((String::new(), Marks::default()));
                }
                BreakType::Page => {
                    push_piece(pieces, &std::mem::take(&mut raw), marks);
                    pieces.push((String::new(), Marks::default()));
                    pieces.push((PAGE_BREAK.to_owned(), Marks::default()));
                    pieces.push((String::new(), Marks::default()));
                }
            },
            RunContent::Field(field) => {
                push_piece(pieces, &std::mem::take(&mut raw), marks);
                pieces.push((String::new(), Marks::default()));
                for (text, properties) in field.cached_display_segments() {
                    let field_marks = marks.overlay(properties);
                    let mut display = String::new();
                    for character in text.chars() {
                        match character {
                            '\t' => display.push('\t'),
                            '\n' | '\u{000b}' => {
                                push_piece(pieces, &std::mem::take(&mut display), field_marks);
                                pieces.push((String::new(), Marks::default()));
                                pieces.push((LINE_BREAK.to_owned(), Marks::default()));
                                pieces.push((String::new(), Marks::default()));
                            }
                            '\u{000c}' => {
                                push_piece(pieces, &std::mem::take(&mut display), field_marks);
                                pieces.push((String::new(), Marks::default()));
                                pieces.push((PAGE_BREAK.to_owned(), Marks::default()));
                                pieces.push((String::new(), Marks::default()));
                            }
                            character => push_escaped(&mut display, character),
                        }
                    }
                    push_piece(pieces, &display, field_marks);
                }
                pieces.push((String::new(), Marks::default()));
            }
            RunContent::SpecialCharacter(character) => match character {
                SpecialCharacter::CarriageReturn => {
                    push_piece(pieces, &std::mem::take(&mut raw), marks);
                    pieces.push((String::new(), Marks::default()));
                    pieces.push((LINE_BREAK.to_owned(), Marks::default()));
                    pieces.push((String::new(), Marks::default()));
                }
                SpecialCharacter::NoBreakHyphen => raw.push('\u{2011}'),
                SpecialCharacter::SoftHyphen => raw.push('\u{00ad}'),
                SpecialCharacter::PositionalTab { .. } => raw.push('\t'),
            },
            // A symbol is font-encoded rather than Unicode, so there is no
            // portable Markdown spelling for it.
            RunContent::Symbol { .. }
            | RunContent::Drawing(_)
            | RunContent::FootnoteRef { .. }
            | RunContent::EndnoteRef { .. }
            | RunContent::CommentReference { .. } => {}
        }
    }

    push_piece(pieces, &raw, marks);
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
    // An empty piece is a physical field/break boundary, not empty source text.
    // Render each side independently so equal marks cannot cross that owner.
    pieces
        .split(|(text, _)| text.is_empty())
        .map(render_inline_segment)
        .collect()
}

fn render_inline_segment(pieces: &[(String, Marks)]) -> String {
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

/// Emit a table as a GFM pipe table.
fn emit_table(out: &mut String, tbl: &CT_Tbl, hyperlink_urls: &HashMap<String, String>) {
    if tbl.rows().is_empty() {
        return;
    }

    // Collect all rows as cell text vectors
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut max_cols = 0;

    // Rows and cells that content controls wrap are written in place.
    for row in tbl
        .rows()
        .into_iter()
        .filter(|row| !row.accepted_view_removes())
    {
        let mut cells: Vec<String> = Vec::new();
        for cell in row.cells() {
            let text = collect_cell_text(cell, hyperlink_urls);
            cells.push(text);
        }
        if cells.len() > max_cols {
            max_cols = cells.len();
        }
        rows.push(cells);
    }

    if rows.is_empty() || max_cols == 0 {
        return;
    }

    // Pad rows to same length
    for row in &mut rows {
        while row.len() < max_cols {
            row.push(String::new());
        }
    }

    // Emit header row (first row)
    out.push('|');
    for cell in &rows[0] {
        out.push_str(&format!(" {} |", cell.trim()));
    }
    out.push('\n');

    // Emit separator
    out.push('|');
    for _ in 0..max_cols {
        out.push_str(" --- |");
    }
    out.push('\n');

    // Emit data rows
    for row in rows.iter().skip(1) {
        out.push('|');
        for cell in row {
            out.push_str(&format!(" {} |", cell.trim()));
        }
        out.push('\n');
    }

    out.push('\n');
}

/// Collect text from a table cell.
fn collect_cell_text(
    cell: &rdocx_oxml::table::CT_Tc,
    hyperlink_urls: &HashMap<String, String>,
) -> String {
    let mut parts = Vec::new();

    for block in cell_blocks(cell) {
        match block {
            Block::Paragraph(p, boundaries) => {
                // A pipe table row is one line, so breaks inside a cell are
                // written as HTML line breaks. A pipe would end the cell, in
                // a link destination too, so every pipe is escaped, and GFM
                // removes the escape before it reads the cell's inlines.
                let text = collect_paragraph_text(&p, hyperlink_urls, &boundaries)
                    .replace(PAGE_BREAK, " ");
                let trimmed = text.trim().replace(LINE_BREAK, "<br>").replace('|', "\\|");
                if !trimmed.is_empty() {
                    parts.push(trimmed);
                }
            }
            Block::Table(table) if table.accepted_view_removes() => {}
            Block::Table(_) => {
                parts.push("(nested table)".to_string());
            }
        }
    }

    parts.join(" ")
}
