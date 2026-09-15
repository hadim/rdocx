//! CLI command implementations.

use std::path::{Path, PathBuf};

use oxml_cli_support::{
    StagedOutputSet, default_output_path, ensure_output_paths_available, json_envelope, parse_range,
};
use rdocx::{
    BodyItemRef, Document, RasterFormat, RasterOptions, RasterOutput, RevisionKind, RunRange,
};
use rdocx_oxml::content_control::{CT_Sdt, SdtContent};
use rdocx_oxml::document::{BodyContent, CT_Document};
use rdocx_oxml::table::{CT_Row, CT_Tbl, CT_Tc, CellContent};
use rdocx_oxml::text::{CT_P, CT_R};
use serde_json::{Value, json};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct ImageOptions<'a> {
    pub pages: Option<&'a str>,
    pub quality: u8,
    pub transparent: bool,
}

pub struct RenderOptions<'a> {
    pub page: Option<usize>,
    pub pages: Option<&'a str>,
    pub format: &'a str,
    pub quality: u8,
    pub transparent: bool,
}

#[derive(Clone, Copy)]
pub enum RevisionAction {
    Accept,
    Reject,
}

pub struct RevisionSelector<'a> {
    pub id: Option<i32>,
    pub author: Option<&'a str>,
    pub start_date: Option<&'a str>,
    pub end_date: Option<&'a str>,
}

/// Inspect a DOCX file and print structure information.
pub fn inspect(file: &Path, json: bool) -> Result<()> {
    let doc = Document::open(file)?;

    let paragraph_count = doc.paragraph_count();
    let table_count = doc.table_count();
    let content_count = doc.content_count();

    let title = doc.title().map(|s| s.to_string());
    let author = doc.author().map(|s| s.to_string());
    let subject = doc.subject().map(|s| s.to_string());
    let keywords = doc.keywords().map(|s| s.to_string());

    // Collect style IDs used in paragraphs
    let mut style_ids: Vec<String> = Vec::new();
    for para in doc.paragraphs() {
        if let Some(style) = para.style_id() {
            let s = style.to_string();
            if !style_ids.contains(&s) {
                style_ids.push(s);
            }
        }
    }

    if json {
        let obj = inspect_json(file, &doc, style_ids)?;
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        println!("File: {}", file.display());
        println!("Paragraphs: {paragraph_count}");
        println!("Tables: {table_count}");
        println!("Content elements: {content_count}");
        println!();
        println!("Metadata:");
        if let Some(t) = &title {
            println!("  Title: {t}");
        }
        if let Some(a) = &author {
            println!("  Author: {a}");
        }
        if let Some(s) = &subject {
            println!("  Subject: {s}");
        }
        if let Some(k) = &keywords {
            println!("  Keywords: {k}");
        }
        if title.is_none() && author.is_none() && subject.is_none() && keywords.is_none() {
            println!("  (none)");
        }
        println!();
        println!("Styles used:");
        if style_ids.is_empty() {
            println!("  (none)");
        } else {
            for sid in &style_ids {
                println!("  - {sid}");
            }
        }
    }

    Ok(())
}

fn inspect_json(file: &Path, doc: &Document, style_ids: Vec<String>) -> Result<Value> {
    Ok(json_envelope(json!({
        "file": file.display().to_string(),
        "paragraphs": doc.paragraph_count(),
        "tables": doc.table_count(),
        "content_elements": doc.content_count(),
        "metadata": {
            "title": doc.title(),
            "author": doc.author(),
            "subject": doc.subject(),
            "keywords": doc.keywords(),
        },
        "styles_used": style_ids,
    }))?)
}

/// Extract plain text from a DOCX file.
///
/// Body paragraphs and table cell text are both emitted, in document order —
/// printing only `paragraphs()` would silently drop everything inside tables.
pub fn text(file: &Path, json_output: bool) -> Result<()> {
    let doc = Document::open(file)?;
    if json_output {
        let document = parsed_main_document(file)?;
        let mut paragraphs = Vec::new();
        for (body_index, content) in document.body.content.iter().enumerate() {
            match content {
                BodyContent::Paragraph(paragraph) => {
                    paragraphs.push(paragraph_json(body_index, &[], paragraph));
                }
                BodyContent::Table(table) => {
                    collect_table_paragraphs(body_index, &[], table, &mut paragraphs);
                }
                BodyContent::ContentControl(control) => {
                    collect_control_paragraphs(body_index, &[], control, &mut paragraphs);
                }
                BodyContent::RawXml(_) => {}
            }
        }
        print_json(json!({
            "scope": "main",
            "revision_view": "accepted",
            "paragraphs": paragraphs,
        }))?;
    } else {
        print!("{}", doc.text());
    }
    Ok(())
}

/// Emit deterministic point-space extents for every direct body item.
pub fn layout(file: &Path, json_output: bool) -> Result<()> {
    let doc = Document::open(file)?;
    let layout = doc.layout_deterministic()?;
    let body_items = doc
        .body_items()
        .enumerate()
        .map(|(body_index, item)| {
            let kind = match item {
                BodyItemRef::Paragraph(_) => "paragraph",
                BodyItemRef::Table(_) => "table",
                BodyItemRef::ContentControl(_) => "content-control",
                BodyItemRef::UnsupportedXml(_) => "unsupported-xml",
            };
            let fragments = layout
                .body_layout_fragments(body_index)
                .unwrap_or_default()
                .iter()
                .map(|fragment| {
                    json!({
                        "physical_page": fragment.physical_page,
                        "displayed_page": fragment.displayed_page,
                        "x": fragment.x,
                        "y": fragment.y,
                        "width": fragment.width,
                        "height": fragment.height,
                    })
                })
                .collect::<Vec<_>>();
            json!({
                "body_index": body_index,
                "kind": kind,
                "fragments": fragments,
            })
        })
        .collect::<Vec<_>>();

    if json_output {
        print_json(json!({
            "scope": "main",
            "revision_view": "accepted",
            "units": "points",
            "page_count": layout.layout.pages.len(),
            "body_items": body_items,
        }))?;
    } else {
        println!("Pages: {}", layout.layout.pages.len());
        for item in body_items {
            println!(
                "Body item {} ({}): {} fragment(s)",
                item["body_index"],
                item["kind"].as_str().unwrap_or("unknown"),
                item["fragments"].as_array().map_or(0, Vec::len)
            );
        }
    }
    Ok(())
}

fn parsed_main_document(file: &Path) -> Result<CT_Document> {
    let package = oxml_opc::OpcPackage::open(file)?;
    let part = package
        .main_document_part()
        .ok_or("package declares no main document relationship")?;
    let bytes = package
        .get_part(&part)
        .ok_or_else(|| format!("main document part {part} is missing"))?;
    Ok(CT_Document::from_xml(bytes)?)
}

fn path_segment(kind: &str, index: usize) -> Value {
    json!({ "kind": kind, "index": index })
}

fn paragraph_json(body_index: usize, path: &[Value], paragraph: &CT_P) -> Value {
    let runs = paragraph
        .accepted_bookmark_runs()
        .into_iter()
        .enumerate()
        .map(|(index, run)| {
            json!({
                "index": index,
                "text": run.text(),
                "formatting": run_formatting_json(run),
            })
        })
        .collect::<Vec<_>>();
    let style = paragraph
        .properties
        .as_ref()
        .and_then(|properties| properties.style_id.as_deref());
    let numbering = paragraph.properties.as_ref().and_then(|properties| {
        properties.num_id.map(|num_id| {
            json!({
                "num_id": num_id,
                "level": properties.num_ilvl.unwrap_or(0),
            })
        })
    });
    json!({
        "body_index": body_index,
        "path": path,
        "style": style,
        "numbering": numbering,
        "text": runs.iter().filter_map(|run| run["text"].as_str()).collect::<String>(),
        "runs": runs,
    })
}

fn run_formatting_json(run: &CT_R) -> Value {
    let Some(properties) = run.properties.as_ref() else {
        return Value::Null;
    };
    json!({
        "bold": properties.bold,
        "italic": properties.italic,
        "strike": properties.strike,
        "underline": properties.underline.map(|value| value.to_str()),
        "font": properties.font_ascii,
        "size_points": properties.sz.map(|value| value.to_pt()),
        "color": properties.color,
        "highlight": properties.highlight.map(|value| value.to_str()),
        "language": properties.language,
        "style": properties.style_id,
    })
}

fn collect_table_paragraphs(
    body_index: usize,
    path: &[Value],
    table: &CT_Tbl,
    output: &mut Vec<Value>,
) {
    for (row_index, row) in table.rows.iter().enumerate() {
        let mut row_path = path.to_vec();
        row_path.push(path_segment("row", row_index));
        collect_row_paragraphs(body_index, &row_path, row, output);
    }
}

fn collect_row_paragraphs(
    body_index: usize,
    path: &[Value],
    row: &CT_Row,
    output: &mut Vec<Value>,
) {
    for (cell_index, cell) in row.cells.iter().enumerate() {
        let mut cell_path = path.to_vec();
        cell_path.push(path_segment("cell", cell_index));
        collect_cell_paragraphs(body_index, &cell_path, cell, output);
    }
}

fn collect_cell_paragraphs(
    body_index: usize,
    path: &[Value],
    cell: &CT_Tc,
    output: &mut Vec<Value>,
) {
    for (content_index, content) in cell.content.iter().enumerate() {
        let mut content_path = path.to_vec();
        match content {
            CellContent::Paragraph(paragraph) => {
                content_path.push(path_segment("paragraph", content_index));
                output.push(paragraph_json(body_index, &content_path, paragraph));
            }
            CellContent::Table(table) => {
                content_path.push(path_segment("table", content_index));
                collect_table_paragraphs(body_index, &content_path, table, output);
            }
            CellContent::ContentControl(control) => {
                content_path.push(path_segment("content-control", content_index));
                collect_control_paragraphs(body_index, &content_path, control, output);
            }
        }
    }
}

fn collect_control_paragraphs(
    body_index: usize,
    path: &[Value],
    control: &CT_Sdt,
    output: &mut Vec<Value>,
) {
    for (content_index, content) in control.content.iter().enumerate() {
        let mut content_path = path.to_vec();
        match content {
            SdtContent::Paragraph(paragraph) => {
                content_path.push(path_segment("paragraph", content_index));
                output.push(paragraph_json(body_index, &content_path, paragraph));
            }
            SdtContent::Table(table) => {
                content_path.push(path_segment("table", content_index));
                collect_table_paragraphs(body_index, &content_path, table, output);
            }
            SdtContent::Row(row) => {
                content_path.push(path_segment("row", content_index));
                collect_row_paragraphs(body_index, &content_path, row, output);
            }
            SdtContent::Cell(cell) => {
                content_path.push(path_segment("cell", content_index));
                collect_cell_paragraphs(body_index, &content_path, cell, output);
            }
            SdtContent::ContentControl(nested) => {
                content_path.push(path_segment("content-control", content_index));
                collect_control_paragraphs(body_index, &content_path, nested, output);
            }
            SdtContent::Run(_) | SdtContent::RawXml(_) => {}
        }
    }
}

/// Convert a DOCX file to another format.
pub fn convert(
    file: &Path,
    to: &str,
    output: Option<&Path>,
    dpi: u32,
    font_dir: Option<&Path>,
    image: ImageOptions<'_>,
) -> Result<()> {
    let doc = Document::open(file)?;

    let default_ext = match to {
        "pdf" => "pdf",
        "html" => "html",
        "md" | "markdown" => "md",
        "png" => "png",
        "jpg" | "jpeg" => "jpg",
        "tif" | "tiff" => "tiff",
        other => {
            return Err(format!(
                "Unknown format: {other}. Supported: pdf, html, md, png, jpeg, tiff"
            )
            .into());
        }
    };

    let output_path = match output {
        Some(p) => p.to_path_buf(),
        None => default_output_path(file, default_ext),
    };

    match to {
        "pdf" => {
            let bytes = if let Some(dir) = font_dir {
                let font_files = Document::load_fonts_from_dir(dir);
                let font_refs: Vec<(&str, &[u8])> = font_files
                    .iter()
                    .map(|f| (f.family.as_str(), f.data.as_slice()))
                    .collect();
                doc.to_pdf_with_fonts(&font_refs)?
            } else {
                doc.to_pdf()?
            };
            std::fs::write(&output_path, bytes)?;
        }
        "html" => {
            let html = doc.to_html();
            std::fs::write(&output_path, html)?;
        }
        "md" | "markdown" => {
            let md = doc.to_markdown();
            std::fs::write(&output_path, md)?;
        }
        "png" | "jpg" | "jpeg" | "tif" | "tiff" => {
            let (format, extension) = parse_image_format(to, image.quality, image.transparent)?;
            let layout = doc.layout_deterministic()?;
            let selected = selected_zero_based_pages(layout.layout.pages.len(), image.pages)?;
            match format {
                RasterFormat::Tiff => {
                    let output = oxml_pdf::render_pages(
                        &layout.layout,
                        &selected,
                        RasterOptions {
                            dpi: dpi as f64,
                            format,
                        },
                    )?;
                    let RasterOutput::MultiPageTiff(tiff) = output else {
                        return Err("TIFF render did not produce one stream".into());
                    };
                    stage_and_publish(&[(output_path.clone(), tiff)])?;
                    println!("Written to {}", output_path.display());
                }
                RasterFormat::Png { .. } | RasterFormat::Jpeg { .. } => {
                    let output_paths =
                        convert_separate_output_paths(&output_path, extension, selected.len());
                    ensure_output_paths_available(&output_paths)?;
                    let mut staged = StagedOutputSet::new();
                    for (page_index, path) in selected.iter().zip(output_paths.iter()) {
                        let image = render_one_raster_page(
                            &layout.layout,
                            *page_index,
                            RasterOptions {
                                dpi: dpi as f64,
                                format,
                            },
                        )?;
                        staged.stage_bytes(path, &image)?;
                    }
                    staged.publish()?;
                    if output_paths.len() == 1 {
                        println!("Written to {}", output_path.display());
                    } else {
                        let stem = output_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy();
                        let parent = output_path.parent().unwrap_or(Path::new("."));
                        println!(
                            "Written {} pages to {}/{stem}_NNN.{extension}",
                            output_paths.len(),
                            parent.display()
                        );
                    }
                }
            }
            return Ok(());
        }
        _ => unreachable!(),
    }

    println!("Written to {}", output_path.display());
    Ok(())
}

/// Structural diff between two DOCX files.
pub fn diff(file_a: &Path, file_b: &Path) -> Result<()> {
    let doc_a = Document::open(file_a)?;
    let doc_b = Document::open(file_b)?;

    let paras_a: Vec<String> = doc_a.paragraphs().iter().map(|p| p.text()).collect();
    let paras_b: Vec<String> = doc_b.paragraphs().iter().map(|p| p.text()).collect();

    println!(
        "--- {} ({} paragraphs, {} tables)",
        file_a.display(),
        doc_a.paragraph_count(),
        doc_a.table_count()
    );
    println!(
        "+++ {} ({} paragraphs, {} tables)",
        file_b.display(),
        doc_b.paragraph_count(),
        doc_b.table_count()
    );
    println!();

    // Simple LCS-based diff on paragraph texts
    let lcs = compute_lcs(&paras_a, &paras_b);
    let mut i = 0;
    let mut j = 0;
    let mut k = 0;

    while k < lcs.len() {
        // Output removed lines before the match
        while i < paras_a.len() && paras_a[i] != lcs[k] {
            println!("- [{}] {}", i + 1, paras_a[i]);
            i += 1;
        }
        // Output added lines before the match
        while j < paras_b.len() && paras_b[j] != lcs[k] {
            println!("+ [{}] {}", j + 1, paras_b[j]);
            j += 1;
        }
        // Skip the common line
        i += 1;
        j += 1;
        k += 1;
    }

    // Remaining lines
    while i < paras_a.len() {
        println!("- [{}] {}", i + 1, paras_a[i]);
        i += 1;
    }
    while j < paras_b.len() {
        println!("+ [{}] {}", j + 1, paras_b[j]);
        j += 1;
    }

    let changes = paras_a.len() + paras_b.len() - 2 * lcs.len();
    if changes == 0 {
        println!("(no differences in paragraph text)");
    } else {
        println!("\n{changes} paragraph(s) differ.");
    }

    Ok(())
}

/// List Word comments in their package order.
pub fn comment_list(file: &Path, json_output: bool) -> Result<()> {
    let doc = Document::open(file)?;
    let comments = doc.comments();
    let records = comments
        .iter()
        .map(|comment| {
            json!({
                "id": comment.id(),
                "author": comment.author(),
                "initials": comment.initials(),
                "date": comment.date(),
                "text": comment.text(),
                "parent_id": comment.parent_id(),
                "resolved": comment.resolved(),
            })
        })
        .collect::<Vec<_>>();
    if json_output {
        print_json(json!({
            "scope": "main",
            "comments": records,
        }))?;
    } else if comments.is_empty() {
        println!("(no comments)");
    } else {
        for comment in comments {
            println!(
                "{}\t{}\t{}\t{}",
                comment.id(),
                comment.author().unwrap_or(""),
                if comment.resolved() {
                    "resolved"
                } else {
                    "open"
                },
                comment.text().replace('\n', " ")
            );
        }
    }
    Ok(())
}

/// Add one Word comment and publish the complete mutated document atomically.
pub fn comment_add(
    file: &Path,
    range: RunRange,
    author: &str,
    initials: Option<&str>,
    text: &str,
    output: &Path,
    json_output: bool,
) -> Result<()> {
    let mut doc = Document::open(file)?;
    let id = doc.add_comment(range, author, initials, text)?;
    publish_document(&mut doc, output)?;
    mutation_record(
        json_output,
        "main",
        "add",
        json!({ "comment_id": id }),
        output,
    )
}

/// Add one reply and publish the complete mutated document atomically.
pub fn comment_reply(
    file: &Path,
    parent_id: i32,
    author: &str,
    text: &str,
    output: &Path,
    json_output: bool,
) -> Result<()> {
    let mut doc = Document::open(file)?;
    let id = doc.reply_to(parent_id, author, text)?;
    publish_document(&mut doc, output)?;
    mutation_record(
        json_output,
        "main",
        "reply",
        json!({ "comment_id": id, "parent_id": parent_id }),
        output,
    )
}

/// Resolve one comment thread and publish the complete document atomically.
pub fn comment_resolve(file: &Path, id: i32, output: &Path, json_output: bool) -> Result<()> {
    let mut doc = Document::open(file)?;
    if !doc.resolve_comment(id, true)? {
        return Err(format!("comment id {id} does not exist").into());
    }
    publish_document(&mut doc, output)?;
    mutation_record(
        json_output,
        "main",
        "resolve",
        json!({ "comment_id": id }),
        output,
    )
}

/// Remove one comment thread and publish the complete document atomically.
pub fn comment_remove(file: &Path, id: i32, output: &Path, json_output: bool) -> Result<()> {
    let mut doc = Document::open(file)?;
    if !doc.remove_comment(id)? {
        return Err(format!("comment id {id} does not exist").into());
    }
    publish_document(&mut doc, output)?;
    mutation_record(
        json_output,
        "main",
        "remove",
        json!({ "comment_id": id }),
        output,
    )
}

/// List modeled revisions from the main story.
pub fn revision_list(file: &Path, json_output: bool) -> Result<()> {
    let doc = Document::open(file)?;
    let revisions = doc.revisions();
    let records = revisions
        .iter()
        .map(|revision| {
            json!({
                "id": revision.id(),
                "author": revision.author(),
                "timestamp": revision.timestamp(),
                "kind": revision_kind_label(revision.kind()),
            })
        })
        .collect::<Vec<_>>();
    if json_output {
        print_json(json!({
            "scope": "main",
            "revisions": records,
        }))?;
    } else if revisions.is_empty() {
        println!("(no revisions in main story)");
    } else {
        for revision in revisions {
            println!(
                "{}\t{}\t{}\t{}",
                revision.id(),
                revision.author(),
                revision.timestamp().unwrap_or(""),
                revision_kind_label(revision.kind())
            );
        }
    }
    Ok(())
}

/// Accept or reject a validated revision selection across every supported story.
pub fn resolve_revisions(
    file: &Path,
    action: RevisionAction,
    selector: RevisionSelector<'_>,
    output: &Path,
    json_output: bool,
) -> Result<()> {
    validate_revision_selector(&selector)?;
    let mut doc = Document::open(file)?;
    let count = match (
        action,
        selector.id,
        selector.author,
        selector.start_date,
        selector.end_date,
    ) {
        (RevisionAction::Accept, Some(id), None, None, None) => doc.accept_revision_id(id)?,
        (RevisionAction::Reject, Some(id), None, None, None) => doc.reject_revision_id(id)?,
        (RevisionAction::Accept, None, Some(author), None, None) => {
            doc.accept_revisions_by_author(author)?
        }
        (RevisionAction::Reject, None, Some(author), None, None) => {
            doc.reject_revisions_by_author(author)?
        }
        (RevisionAction::Accept, None, None, Some(start), Some(end)) => {
            doc.accept_revisions_in_date_range(start, end)?
        }
        (RevisionAction::Reject, None, None, Some(start), Some(end)) => {
            doc.reject_revisions_in_date_range(start, end)?
        }
        (RevisionAction::Accept, None, None, None, None) => doc.accept_all()?,
        (RevisionAction::Reject, None, None, None, None) => doc.reject_all()?,
        _ => return Err("revision selector is invalid".into()),
    };
    publish_document(&mut doc, output)?;
    let action_label = action.label();
    if json_output {
        print_json(json!({
            "scope": "all-supported-stories",
            "action": action_label,
            "selector": selector_json(&selector),
            "resolved": count,
            "output": output.display().to_string(),
        }))?;
    } else {
        println!("{action_label}: {count} revision element(s)");
        println!("Written to {}", output.display());
    }
    Ok(())
}

/// Create a tracked-changes document from an original and edited input.
pub fn compare(
    original: &Path,
    edited: &Path,
    author: &str,
    timestamp: &str,
    output: &Path,
    json_output: bool,
) -> Result<()> {
    let mut original_doc = Document::open(original)?;
    let edited_doc = Document::open(edited)?;
    let diagnostics = original_doc.compare(&edited_doc, author, timestamp)?;
    let revision_count = original_doc.revisions().len();
    let records = diagnostics
        .iter()
        .map(|diagnostic| {
            json!({
                "location": diagnostic.location,
                "message": diagnostic.message,
            })
        })
        .collect::<Vec<_>>();
    publish_document(&mut original_doc, output)?;
    if json_output {
        print_json(json!({
            "scope": "all-supported-stories",
            "main_story_revisions": revision_count,
            "diagnostics": records,
            "output": output.display().to_string(),
        }))?;
    } else {
        println!("Created {revision_count} main-story revision element(s)");
        println!("Diagnostics: {}", diagnostics.len());
        println!("Written to {}", output.display());
    }
    Ok(())
}

/// Rebuild supported existing table-of-contents fields.
pub fn toc_rebuild(file: &Path, output: &Path, json_output: bool) -> Result<()> {
    let mut doc = Document::open(file)?;
    let report = doc.rebuild_toc()?;
    publish_document(&mut doc, output)?;
    if json_output {
        print_json(json!({
            "scope": "main",
            "entry_count": report.entry_count,
            "bookmark_count": report.bookmark_count,
            "diagnostic_count": report.diagnostic_count(),
            "output": output.display().to_string(),
        }))?;
    } else {
        println!("Entries: {}", report.entry_count);
        println!("Bookmarks: {}", report.bookmark_count);
        println!("Diagnostics: {}", report.diagnostic_count());
        println!("Written to {}", output.display());
    }
    Ok(())
}

impl RevisionAction {
    fn label(self) -> &'static str {
        match self {
            Self::Accept => "accept",
            Self::Reject => "reject",
        }
    }
}

fn validate_revision_selector(selector: &RevisionSelector<'_>) -> Result<()> {
    let date_count =
        usize::from(selector.start_date.is_some()) + usize::from(selector.end_date.is_some());
    if date_count == 1 {
        return Err("--start-date and --end-date must be provided together".into());
    }
    let selector_count = usize::from(selector.id.is_some())
        + usize::from(selector.author.is_some())
        + usize::from(date_count == 2);
    if selector_count > 1 {
        return Err("--id, --author, and the date range are mutually exclusive".into());
    }
    Ok(())
}

fn selector_json(selector: &RevisionSelector<'_>) -> Value {
    if let Some(id) = selector.id {
        json!({ "kind": "id", "id": id })
    } else if let Some(author) = selector.author {
        json!({ "kind": "author", "author": author })
    } else if let (Some(start), Some(end)) = (selector.start_date, selector.end_date) {
        json!({ "kind": "date-range", "start": start, "end": end })
    } else {
        json!({ "kind": "all" })
    }
}

fn revision_kind_label(kind: RevisionKind) -> &'static str {
    match kind {
        RevisionKind::Insertion => "insertion",
        RevisionKind::Deletion => "deletion",
        RevisionKind::MoveFrom => "move-from",
        RevisionKind::MoveTo => "move-to",
        RevisionKind::RunPropertyChange => "run-property-change",
        RevisionKind::ParagraphPropertyChange => "paragraph-property-change",
        RevisionKind::TablePropertyChange => "table-property-change",
        RevisionKind::SectionPropertyChange => "section-property-change",
    }
}

fn publish_document(doc: &mut Document, output: &Path) -> Result<()> {
    let bytes = doc.to_bytes()?;
    stage_and_publish(&[(output.to_path_buf(), bytes)])
}

fn mutation_record(
    json_output: bool,
    scope: &str,
    action: &str,
    detail: Value,
    output: &Path,
) -> Result<()> {
    if json_output {
        let mut payload = json!({
            "scope": scope,
            "action": action,
            "output": output.display().to_string(),
        });
        let object = payload.as_object_mut().expect("literal is an object");
        let Value::Object(detail) = detail else {
            return Err("mutation detail must be an object".into());
        };
        object.extend(detail);
        print_json(payload)?;
    } else {
        println!("{action}");
        println!("Written to {}", output.display());
    }
    Ok(())
}

fn print_json(payload: Value) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json_envelope(payload)?)?
    );
    Ok(())
}

/// Compute the longest common subsequence of two string slices.
fn compute_lcs(a: &[String], b: &[String]) -> Vec<String> {
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0u32; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1] + 1
            } else {
                dp[i - 1][j].max(dp[i][j - 1])
            };
        }
    }

    // Backtrack
    let mut result = Vec::new();
    let mut i = m;
    let mut j = n;
    while i > 0 && j > 0 {
        if a[i - 1] == b[j - 1] {
            result.push(a[i - 1].clone());
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] >= dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    result.reverse();
    result
}

/// Replace a placeholder in a DOCX file and save to output.
pub fn replace(
    file: &Path,
    placeholder: &str,
    value: &str,
    expect: Option<usize>,
    output: &Path,
) -> Result<()> {
    let mut doc = Document::open(file)?;
    let count = doc.try_replace_text(placeholder, value)?;
    if let Some(expected) = expect
        && count != expected
    {
        return Err(format!(
            "expected {expected} replacement(s) of \"{placeholder}\", found {count}"
        )
        .into());
    }
    publish_document(&mut doc, output)?;
    println!("Replaced {count} occurrence(s) of \"{placeholder}\" -> \"{value}\"");
    println!("Written to {}", output.display());
    Ok(())
}

/// Render document pages to image files.
pub fn render(
    file: &Path,
    output_dir: Option<&Path>,
    dpi: f64,
    options: RenderOptions<'_>,
) -> Result<()> {
    validate_dpi(dpi)?;
    let doc = Document::open(file)?;
    let out_dir = output_dir.unwrap_or_else(|| Path::new("."));
    let (format, extension) =
        parse_image_format(options.format, options.quality, options.transparent)?;
    let layout = doc.layout_deterministic()?;
    let selected = selected_render_pages(layout.layout.pages.len(), options.page, options.pages)?;
    let stem = file.file_stem().unwrap_or_default().to_string_lossy();
    let legacy_single_page = options.page.is_some();

    // Writing into a directory the user named but has not created should not
    // fail with a bare "No such file or directory". Do it after validation and
    // encoding so invalid options leave no partial output.
    match format {
        RasterFormat::Tiff => {
            let output =
                oxml_pdf::render_pages(&layout.layout, &selected, RasterOptions { dpi, format })?;
            let RasterOutput::MultiPageTiff(tiff) = output else {
                return Err("TIFF render did not produce one stream".into());
            };
            let out_path = out_dir.join(format!("{stem}.tiff"));
            std::fs::create_dir_all(out_dir)?;
            let tiff_len = tiff.len();
            stage_and_publish(&[(out_path.clone(), tiff)])?;
            println!(
                "Pages {} -> {} ({} bytes)",
                selected
                    .iter()
                    .map(|page| (page + 1).to_string())
                    .collect::<Vec<_>>()
                    .join(","),
                out_path.display(),
                tiff_len
            );
        }
        RasterFormat::Png { .. } | RasterFormat::Jpeg { .. } => {
            let output_paths = selected
                .iter()
                .map(|page_index| {
                    let one_based = page_index + 1;
                    out_dir.join(format!("{stem}_page{one_based}.{extension}"))
                })
                .collect::<Vec<_>>();
            std::fs::create_dir_all(out_dir)?;
            ensure_output_paths_available(&output_paths)?;
            let mut staged = StagedOutputSet::new();
            let mut rendered = Vec::with_capacity(selected.len());
            for (page_index, out_path) in selected.iter().zip(output_paths.iter()) {
                let one_based = page_index + 1;
                let image = render_one_raster_page(
                    &layout.layout,
                    *page_index,
                    RasterOptions { dpi, format },
                )?;
                staged.stage_bytes(out_path, &image)?;
                rendered.push((one_based, out_path.clone(), image.len()));
            }
            staged.publish()?;
            for (one_based, out_path, len) in rendered {
                println!("Page {one_based} -> {} ({len} bytes)", out_path.display());
            }
            if !legacy_single_page {
                println!("Rendered {} page(s) at {dpi} DPI", selected.len());
            }
        }
    }

    Ok(())
}

fn parse_image_format(
    format: &str,
    quality: u8,
    transparent: bool,
) -> Result<(RasterFormat, &'static str)> {
    match format {
        "png" => Ok((
            RasterFormat::Png {
                transparent_background: transparent,
            },
            "png",
        )),
        "jpg" | "jpeg" => {
            if !(1..=100).contains(&quality) {
                return Err(format!("JPEG quality must be 1 through 100, got {quality}").into());
            }
            Ok((RasterFormat::Jpeg { quality }, "jpg"))
        }
        "tif" | "tiff" => Ok((RasterFormat::Tiff, "tiff")),
        other => Err(format!("Unknown image format: {other}. Supported: png, jpeg, tiff").into()),
    }
}

fn selected_zero_based_pages(page_count: usize, pages: Option<&str>) -> Result<Vec<usize>> {
    let one_based = match pages {
        Some(pages) => parse_range(pages)?,
        None => (1..=page_count).collect(),
    };
    if let Some(page) = one_based.iter().copied().find(|page| *page > page_count) {
        return Err(format!("page {page} is out of range for {page_count} pages").into());
    }
    Ok(one_based.into_iter().map(|page| page - 1).collect())
}

fn selected_render_pages(
    page_count: usize,
    page: Option<usize>,
    pages: Option<&str>,
) -> Result<Vec<usize>> {
    match (page, pages) {
        (Some(_), Some(_)) => Err("--page and --pages cannot be used together".into()),
        (Some(page), None) => {
            if page >= page_count {
                return Err(format!(
                    "zero-based page {page} is out of range for {page_count} pages"
                )
                .into());
            }
            Ok(vec![page])
        }
        (None, pages) => selected_zero_based_pages(page_count, pages),
    }
}

fn convert_separate_output_paths(
    output_path: &Path,
    extension: &str,
    page_count: usize,
) -> Vec<PathBuf> {
    if page_count == 1 {
        return vec![output_path.to_path_buf()];
    }
    let stem = output_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let parent = output_path.parent().unwrap_or(Path::new("."));
    (1..=page_count)
        .map(|index| parent.join(format!("{stem}_{index:03}.{extension}")))
        .collect()
}

fn render_one_raster_page(
    layout: &oxml_layout::LayoutResult,
    page_index: usize,
    options: RasterOptions,
) -> Result<Vec<u8>> {
    let output = oxml_pdf::render_pages(layout, &[page_index], options)?;
    let RasterOutput::SeparatePages(mut pages) = output else {
        return Err("single-page render did not produce a separate image".into());
    };
    if pages.len() != 1 {
        return Err("single-page render returned the wrong page count".into());
    }
    Ok(pages.remove(0))
}

fn stage_and_publish(outputs: &[(PathBuf, Vec<u8>)]) -> Result<()> {
    let paths = outputs
        .iter()
        .map(|(path, _)| path.clone())
        .collect::<Vec<_>>();
    ensure_output_paths_available(&paths)?;
    let mut staged = StagedOutputSet::new();
    for (path, bytes) in outputs {
        staged.stage_bytes(path, bytes)?;
    }
    staged.publish()?;
    Ok(())
}

fn validate_dpi(dpi: f64) -> Result<()> {
    if !dpi.is_finite() || dpi <= 0.0 {
        return Err("DPI must be a positive finite number".into());
    }
    Ok(())
}

/// Validate a DOCX file's structure.
///
/// Returns `Ok(false)` when a structural error was found, so the caller can
/// exit non-zero — a validator that always succeeds cannot gate anything in CI.
/// Advisory findings (empty paragraphs, missing metadata) are reported but do
/// not affect the exit status.
pub fn validate(file: &Path) -> Result<bool> {
    let doc = Document::open(file)?;

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    // --- Structural errors ---

    // Every relationship must point at a part that is actually in the package.
    let package = oxml_opc::OpcPackage::open(file)?;
    if let Some(doc_part) = package.main_document_part() {
        if package.get_part(&doc_part).is_none() {
            errors.push(format!("main document part {doc_part} is missing"));
        }
        if let Some(rels) = package.get_part_rels(&doc_part) {
            for rel in &rels.items {
                // External targets live outside the package by definition.
                if rel.target_mode.as_deref() == Some("External") {
                    continue;
                }
                let target = oxml_opc::OpcPackage::resolve_rel_target(&doc_part, &rel.target);
                if package.get_part(&target).is_none() {
                    errors.push(format!(
                        "relationship {} points at missing part {target}",
                        rel.id
                    ));
                }
            }
        }
    } else {
        errors.push("package declares no main document relationship".to_string());
    }

    // Every part needs a declared content type.
    for part_name in package.parts.keys() {
        if package.content_types.content_type_for(part_name).is_none() {
            errors.push(format!("part {part_name} has no declared content type"));
        }
    }

    // --- Advisory findings ---

    if doc.content_count() == 0 {
        warnings.push("Document has no content (no paragraphs or tables)".to_string());
    }

    let empty_count = doc
        .paragraphs()
        .iter()
        .filter(|p| p.text().trim().is_empty())
        .count();
    if empty_count > 0 {
        warnings.push(format!("{empty_count} empty paragraph(s) found"));
    }

    let mut prev_level: Option<u32> = None;
    for (level, _) in doc.headings() {
        if let Some(prev) = prev_level
            && level > prev + 1
        {
            warnings.push(format!(
                "Heading level gap: Heading{prev} -> Heading{level} (skipped level(s))"
            ));
        }
        prev_level = Some(level);
    }

    if doc.title().is_none() {
        warnings.push("Missing document title".to_string());
    }
    if doc.author().is_none() {
        warnings.push("Missing document author".to_string());
    }

    // --- Report ---

    if errors.is_empty() && warnings.is_empty() {
        println!("OK — no issues found in {}", file.display());
        return Ok(true);
    }

    if !errors.is_empty() {
        println!("{} error(s) in {}:", errors.len(), file.display());
        for (i, issue) in errors.iter().enumerate() {
            println!("  {}. {issue}", i + 1);
        }
    }
    if !warnings.is_empty() {
        println!("{} warning(s) in {}:", warnings.len(), file.display());
        for (i, issue) in warnings.iter().enumerate() {
            println!("  {}. {issue}", i + 1);
        }
    }

    Ok(errors.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_json_uses_the_shared_schema_one_envelope() {
        let document = Document::new();
        let styles = vec!["Heading1".to_owned(), "Normal".to_owned()];
        let value = inspect_json(Path::new("input.docx"), &document, styles.clone()).unwrap();

        assert_eq!(
            value,
            json!({
                "schema": 1,
                "file": "input.docx",
                "paragraphs": document.paragraph_count(),
                "tables": document.table_count(),
                "content_elements": document.content_count(),
                "metadata": {
                    "title": document.title(),
                    "author": document.author(),
                    "subject": document.subject(),
                    "keywords": document.keywords(),
                },
                "styles_used": styles,
            })
        );
    }

    #[test]
    fn convert_without_output_uses_the_default_extension_path() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let input = std::env::temp_dir().join(format!(
            "rdocx_cli_default_convert_{}_{unique}.docx",
            std::process::id()
        ));
        let expected_output = input.with_extension("md");
        let mut document = Document::new();
        document.add_paragraph("Default path regression");
        document.save(&input).unwrap();

        convert(
            &input,
            "md",
            None,
            96,
            None,
            ImageOptions {
                pages: None,
                quality: 90,
                transparent: false,
            },
        )
        .unwrap();

        let converted = std::fs::read_to_string(&expected_output);
        std::fs::remove_file(input).unwrap();
        std::fs::remove_file(expected_output).ok();
        assert_eq!(converted.unwrap(), "Default path regression\n\n");
    }
}
