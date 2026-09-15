use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use rdocx::Document;
use serde_json::json;

static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(label: &str) -> Self {
        let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("rdocx-cli-{label}-{}-{id}", std::process::id()));
        fs::create_dir(&path).expect("create temporary workspace");
        Self { path }
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rdocx"))
        .args(args)
        .output()
        .expect("run rdocx CLI")
}

fn assert_success(output: &Output, command: &str) {
    assert!(
        output.status.success(),
        "{command} failed with {}: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{command} wrote stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn fixture_document(paragraphs: &[&str]) -> Document {
    let mut document = Document::new();
    document.set_title("CLI fixture");
    document.set_author("rdocx tests");
    document.set_subject("command integration");
    document.set_keywords("cli,test");
    for text in paragraphs {
        document.add_paragraph(text);
    }
    document
}

fn write_document(path: &Path, paragraphs: &[&str]) {
    fixture_document(paragraphs)
        .save(path)
        .expect("write DOCX fixture");
}

fn write_revision_fixture(path: &Path) {
    let mut document = fixture_document(&[]);
    let mut paragraph = document.add_paragraph("");
    paragraph.add_run("Alpha");
    paragraph.add_run("Bravo");
    paragraph.add_run("Charlie");
    document.save(path).unwrap();

    let mut package = OpcPackage::open(path).unwrap();
    let part = package.main_document_part().unwrap();
    let xml = String::from_utf8(package.get_part(&part).unwrap().to_vec()).unwrap();
    let revisions = [
        ("Alpha", 10, "Alice", "2026-01-01T00:00:00Z"),
        ("Bravo", 20, "Bob", "2026-01-02T00:00:00Z"),
        ("Charlie", 30, "Alice", "2026-01-03T00:00:00Z"),
    ];
    let mut xml = xml;
    for (text, id, author, timestamp) in revisions {
        let marker = format!(">{text}</w:t>");
        let text_position = xml.find(&marker).expect("fixture text is present");
        let run_start = xml[..text_position]
            .rfind("<w:r")
            .expect("fixture run starts before text");
        let run_end = text_position
            + xml[text_position..]
                .find("</w:r>")
                .expect("fixture run ends after text")
            + "</w:r>".len();
        let run = xml[run_start..run_end].to_owned();
        let revision = format!(
            "<w:ins w:id=\"{id}\" w:author=\"{author}\" w:date=\"{timestamp}\">{run}</w:ins>"
        );
        xml.replace_range(run_start..run_end, &revision);
    }
    package.set_part(&part, xml.into_bytes());
    package.save(path).unwrap();
    assert_eq!(Document::open(path).unwrap().revisions().len(), 3);
}

fn path_text(path: &Path) -> &str {
    path.to_str().expect("temporary path is UTF-8")
}

fn png_dimensions(bytes: &[u8]) -> (u32, u32) {
    assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
    assert_eq!(&bytes[12..16], b"IHDR");
    (
        u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
        u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
    )
}

#[test]
fn inspect_json_uses_the_shared_schema() {
    let temp = TempWorkspace::new("inspect");
    let input = temp.path.join("structure.docx");
    let mut document = fixture_document(&["First", "Second"]);
    {
        let mut table = document.add_table(1, 2);
        table.cell(0, 0).unwrap().set_text("Left");
        table.cell(0, 1).unwrap().set_text("Right");
    }
    document.save(&input).unwrap();

    let output = cli(&["inspect", path_text(&input), "--json"]);
    assert_success(&output, "inspect");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(
        value,
        json!({
            "schema": 1,
            "file": path_text(&input),
            "paragraphs": 2,
            "tables": 1,
            "content_elements": 3,
            "metadata": {
                "title": "CLI fixture",
                "author": "rdocx tests",
                "subject": "command integration",
                "keywords": "cli,test",
            },
            "styles_used": [],
        })
    );
}

#[test]
fn text_prints_body_and_table_content_in_document_order() {
    let temp = TempWorkspace::new("text");
    let input = temp.path.join("content.docx");
    let mut document = fixture_document(&["Body first"]);
    {
        let mut table = document.add_table(1, 2);
        table.cell(0, 0).unwrap().set_text("Left cell");
        table.cell(0, 1).unwrap().set_text("Right cell");
    }
    document.add_paragraph("Body second");
    document.save(&input).unwrap();

    let output = cli(&["text", path_text(&input)]);
    assert_success(&output, "text");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Body first\nLeft cell\tRight cell\t\nBody second\n"
    );
}

#[test]
fn convert_writes_valid_formats_and_uses_the_shared_default_output() {
    let temp = TempWorkspace::new("convert");
    let input = temp.path.join("source.docx");
    write_document(&input, &["Converted content"]);

    let markdown = input.with_extension("md");
    let output = cli(&["convert", path_text(&input), "--to", "md"]);
    assert_success(&output, "convert markdown");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", markdown.display())
    );
    assert_eq!(
        fs::read_to_string(&markdown).unwrap(),
        "Converted content\n\n"
    );

    let html = temp.path.join("converted.html");
    let output = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "html",
        "--output",
        path_text(&html),
    ]);
    assert_success(&output, "convert HTML");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", html.display())
    );
    let html_text = fs::read_to_string(html).unwrap();
    assert!(html_text.starts_with("<!DOCTYPE html>\n<html>"));
    assert!(html_text.contains("<p>Converted content</p>"));

    let pdf = temp.path.join("converted.pdf");
    let output = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "pdf",
        "--output",
        path_text(&pdf),
    ]);
    assert_success(&output, "convert PDF");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", pdf.display())
    );
    let pdf_bytes = fs::read(pdf).unwrap();
    assert!(pdf_bytes.starts_with(b"%PDF-"));
    assert!(pdf_bytes.ends_with(b"%%EOF"));

    let png = temp.path.join("converted.png");
    let output = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "png",
        "--dpi",
        "24",
        "--output",
        path_text(&png),
    ]);
    assert_success(&output, "convert PNG");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("Written to {}\n", png.display())
    );
    let dimensions = png_dimensions(&fs::read(png).unwrap());
    assert!(dimensions.0 > 0 && dimensions.1 > 0);
}

#[test]
fn diff_reports_changed_paragraphs_without_using_exit_status_as_a_verdict() {
    let temp = TempWorkspace::new("diff");
    let before = temp.path.join("before.docx");
    let after = temp.path.join("after.docx");
    write_document(&before, &["Same", "Old text"]);
    write_document(&after, &["Same", "New text"]);

    let output = cli(&["diff", path_text(&before), path_text(&after)]);
    assert_success(&output, "diff");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "--- {} (2 paragraphs, 0 tables)\n+++ {} (2 paragraphs, 0 tables)\n\n- [2] Old text\n+ [2] New text\n\n2 paragraph(s) differ.\n",
            before.display(),
            after.display()
        )
    );
}

#[test]
fn replace_writes_a_reopenable_document_and_reports_the_exact_count() {
    let temp = TempWorkspace::new("replace");
    let input = temp.path.join("template.docx");
    let replaced = temp.path.join("replaced.docx");
    write_document(&input, &["Hello {{name}}, {{name}} again"]);

    let output = cli(&[
        "replace",
        path_text(&input),
        "--placeholder",
        "{{name}}",
        "--value",
        "Reader",
        "--output",
        path_text(&replaced),
    ]);
    assert_success(&output, "replace");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "Replaced 2 occurrence(s) of \"{{{{name}}}}\" -> \"Reader\"\nWritten to {}\n",
            replaced.display()
        )
    );

    let reopened = Document::open(&replaced).unwrap();
    assert_eq!(
        reopened.paragraph(0).unwrap().text(),
        "Hello Reader, Reader again"
    );
    assert_eq!(
        Document::open(input).unwrap().paragraph(0).unwrap().text(),
        "Hello {{name}}, {{name}} again"
    );
}

#[test]
fn cli_replace_reports_namespace_preflight_errors_without_panicking() {
    let temp = TempWorkspace::new("replace-namespace-error");
    let input = temp.path.join("used-default.docx");
    let output_path = temp.path.join("must-not-exist.docx");
    write_document(&input, &["before"]);

    let mut package =
        OpcPackage::from_reader(std::io::Cursor::new(fs::read(&input).unwrap())).unwrap();
    let xml = std::str::from_utf8(package.get_part("/word/document.xml").unwrap())
        .unwrap()
        .replacen("<w:document", r#"<w:document xmlns="urn:used-default""#, 1)
        .replacen("<w:body>", "<w:body><producer/>", 1);
    package.set_part("/word/document.xml", xml.into_bytes());
    let mut file = fs::File::create(&input).unwrap();
    package.write_to(&mut file).unwrap();

    let output = cli(&[
        "replace",
        path_text(&input),
        "--placeholder",
        "before",
        "--value",
        "after",
        "--output",
        path_text(&output_path),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("shadowed `default` namespace"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!output_path.exists());
}

#[test]
fn validate_exit_status_is_a_verdict() {
    let temp = TempWorkspace::new("validate");
    let valid = temp.path.join("valid.docx");
    let corrupt = temp.path.join("corrupt.docx");
    write_document(&valid, &["Valid content"]);

    let output = cli(&["validate", path_text(&valid)]);
    assert_success(&output, "validate valid document");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("OK — no issues found in {}\n", valid.display())
    );

    let mut package = OpcPackage::open(&valid).unwrap();
    let document_part = package.main_document_part().unwrap();
    let relationship_id = package
        .get_or_create_part_rels(&document_part)
        .add(rel_types::IMAGE, "media/missing.png");
    package.save(&corrupt).unwrap();

    let output = cli(&["validate", path_text(&corrupt)]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "1 error(s) in {}:\n  1. relationship {relationship_id} points at missing part /word/media/missing.png\n",
            corrupt.display()
        )
    );
}

#[test]
fn render_uses_the_bundled_font_deterministic_path() {
    let temp = TempWorkspace::new("render");
    let input = temp.path.join("visible.docx");
    let first_dir = temp.path.join("first");
    let second_dir = temp.path.join("second");
    let mut document = fixture_document(&[]);
    document
        .add_paragraph("")
        .add_run("Visible deterministic text")
        .set_font("Helvetica Neue");
    document.save(&input).unwrap();
    let expected = Document::open(&input)
        .unwrap()
        .render_page_to_png_deterministic(0, 24.0)
        .unwrap()
        .unwrap();

    let selected_output = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&first_dir),
        "--dpi",
        "24",
        "--page",
        "0",
    ]);
    assert_success(&selected_output, "render selected page");
    let selected_png = first_dir.join("visible_page1.png");
    let selected_bytes = fs::read(&selected_png).unwrap();
    assert_eq!(
        String::from_utf8(selected_output.stdout).unwrap(),
        format!(
            "Page 1 -> {} ({} bytes)\n",
            selected_png.display(),
            selected_bytes.len()
        )
    );
    let dimensions = png_dimensions(&selected_bytes);
    assert!(dimensions.0 > 0 && dimensions.1 > 0);
    assert_eq!(selected_bytes, expected);

    let all_output = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&second_dir),
        "--dpi",
        "24",
    ]);
    assert_success(&all_output, "render all pages");
    let all_png = second_dir.join("visible_page1.png");
    let all_bytes = fs::read(&all_png).unwrap();
    assert_eq!(
        String::from_utf8(all_output.stdout).unwrap(),
        format!(
            "Page 1 -> {} ({} bytes)\nRendered 1 page(s) at 24 DPI\n",
            all_png.display(),
            all_bytes.len()
        )
    );
    assert_eq!(all_bytes, expected);

    assert_eq!(fs::read(selected_png).unwrap(), fs::read(all_png).unwrap());
}

#[test]
fn render_page_and_pages_flags_keep_legacy_and_range_indexing_separate() {
    let temp = TempWorkspace::new("render-page-pages");
    let input = temp.path.join("pages.docx");
    let page_dir = temp.path.join("page");
    let pages_dir = temp.path.join("pages");
    let mut document = fixture_document(&[]);
    document.add_paragraph("page one");
    document.add_paragraph("page two").page_break_before(true);
    document.save(&input).unwrap();

    let expected = Document::open(&input)
        .unwrap()
        .render_page_to_png_deterministic(1, 36.0)
        .unwrap()
        .unwrap();

    let legacy = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&page_dir),
        "--dpi",
        "36",
        "--page",
        "1",
    ]);
    assert_success(&legacy, "render zero-based legacy page");
    let legacy_png = page_dir.join("pages_page2.png");
    assert_eq!(fs::read(&legacy_png).unwrap(), expected);
    assert_eq!(
        String::from_utf8(legacy.stdout).unwrap(),
        format!(
            "Page 2 -> {} ({} bytes)\n",
            legacy_png.display(),
            expected.len()
        )
    );

    let ranged = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&pages_dir),
        "--dpi",
        "36",
        "--pages",
        "2",
    ]);
    assert_success(&ranged, "render one-based page range");
    let ranged_png = pages_dir.join("pages_page2.png");
    assert_eq!(fs::read(&ranged_png).unwrap(), expected);
    assert_eq!(
        String::from_utf8(ranged.stdout).unwrap(),
        format!(
            "Page 2 -> {} ({} bytes)\nRendered 1 page(s) at 36 DPI\n",
            ranged_png.display(),
            expected.len()
        )
    );

    let conflict_dir = temp.path.join("conflict");
    let conflict = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&conflict_dir),
        "--page",
        "0",
        "--pages",
        "1",
    ]);
    assert!(!conflict.status.success());
    assert!(
        String::from_utf8_lossy(&conflict.stderr).contains("cannot be used with"),
        "unexpected conflict stderr: {}",
        String::from_utf8_lossy(&conflict.stderr)
    );
    assert!(!conflict_dir.exists());

    let bad_page_dir = temp.path.join("bad-page");
    let rejected = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&bad_page_dir),
        "--page",
        "2",
    ]);
    assert!(!rejected.status.success());
    assert!(!bad_page_dir.exists());
}

#[test]
fn image_export_ranges_share_declared_index_conventions() {
    let temp = TempWorkspace::new("image-options");
    let input = temp.path.join("pages.docx");
    let mut document = fixture_document(&[]);
    document.add_paragraph("page one");
    document.add_paragraph("page two").page_break_before(true);
    document.add_paragraph("page three").page_break_before(true);
    document.save(&input).unwrap();

    let converted = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "jpeg",
        "--dpi",
        "72",
        "--quality",
        "80",
        "--pages",
        "2",
    ]);
    assert_success(&converted, "convert selected JPEG");
    let jpeg = input.with_extension("jpg");
    let expected = Document::open(&input)
        .unwrap()
        .render_pages_deterministic(
            &[1],
            rdocx::RasterOptions {
                dpi: 72.0,
                format: rdocx::RasterFormat::Jpeg { quality: 80 },
            },
        )
        .unwrap();
    let rdocx::RasterOutput::SeparatePages(expected) = expected else {
        panic!("JPEG output should be separate pages");
    };
    assert_eq!(fs::read(&jpeg).unwrap(), expected[0]);
    assert!(!temp.path.join("pages_001.jpg").exists());

    let default_all = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "png",
        "--dpi",
        "72",
        "--output",
        path_text(&temp.path.join("all.png")),
    ]);
    assert_success(&default_all, "convert all deterministic PNG pages");
    for one_based in 1..=3 {
        assert!(
            temp.path.join(format!("all_{one_based:03}.png")).exists(),
            "missing deterministic default page {one_based}"
        );
    }

    let rendered = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&temp.path.join("tiff-out")),
        "--format",
        "tiff",
        "--dpi",
        "72",
        "--pages",
        "1,3",
    ]);
    assert_success(&rendered, "render selected TIFF");
    assert!(
        fs::read(temp.path.join("tiff-out/pages.tiff"))
            .unwrap()
            .starts_with(b"II*\0")
    );

    let bad_quality = temp.path.join("bad.jpg");
    let rejected = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "jpeg",
        "--output",
        path_text(&bad_quality),
        "--quality",
        "0",
        "--pages",
        "1",
    ]);
    assert!(!rejected.status.success());
    assert!(!bad_quality.exists());

    let bad_dir = temp.path.join("bad-range");
    let rejected = cli(&[
        "render",
        path_text(&input),
        "--output-dir",
        path_text(&bad_dir),
        "--pages",
        "4",
    ]);
    assert!(!rejected.status.success());
    assert!(!bad_dir.exists());

    let commands = include_str!("../src/commands.rs");
    assert!(
        !commands.contains("doc.layout()"),
        "Word CLI image selection must use the deterministic render snapshot, not ambient layout"
    );
}

#[test]
fn multi_file_image_export_preserves_existing_outputs_and_streams_separate_pages() {
    let temp = TempWorkspace::new("image-streaming");
    let input = temp.path.join("pages.docx");
    let output = temp.path.join("export.png");
    let mut document = fixture_document(&[]);
    document.add_paragraph("page one");
    document.add_paragraph("page two").page_break_before(true);
    document.save(&input).unwrap();

    let preexisting = temp.path.join("export_002.png");
    fs::write(&preexisting, b"keep me").unwrap();
    let rejected = cli(&[
        "convert",
        path_text(&input),
        "--to",
        "png",
        "--output",
        path_text(&output),
        "--dpi",
        "72",
    ]);

    assert!(!rejected.status.success());
    assert!(!temp.path.join("export_001.png").exists());
    assert_eq!(fs::read(preexisting).unwrap(), b"keep me");

    let commands = include_str!("../src/commands.rs");
    assert!(commands.contains("render_one_raster_page"));
    assert!(
        !commands.contains("RasterOutput::SeparatePages(images)"),
        "separate PNG and JPEG export must not branch on an all-pages image Vec"
    );
    assert!(
        !commands.contains("zip(images.iter())"),
        "separate PNG and JPEG export must not retain every encoded page"
    );
}

#[test]
fn cli_collaboration_commands_are_schema_stable_and_atomic() {
    let temp = TempWorkspace::new("collaboration-schema");
    let input = temp.path.join("input.docx");
    write_document(&input, &["Comment target"]);

    let listed = cli(&["comment", "list", path_text(&input), "--json"]);
    assert_success(&listed, "comment list");
    let value: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "comments": [],
        })
    );

    let invalid_output = temp.path.join("invalid-comment.docx");
    let invalid = cli(&[
        "comment",
        "add",
        path_text(&input),
        "--start-paragraph",
        "99",
        "--start-run",
        "0",
        "--end-paragraph",
        "99",
        "--end-run",
        "1",
        "--author",
        "Alice",
        "--text",
        "Invalid",
        "--output",
        path_text(&invalid_output),
        "--json",
    ]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stdout.is_empty());
    assert!(!invalid_output.exists());

    let edited = temp.path.join("edited.docx");
    let redline = temp.path.join("redline.docx");
    write_document(&edited, &["Edited target"]);
    let compared = cli(&[
        "compare",
        path_text(&input),
        path_text(&edited),
        "--author",
        "Alice",
        "--timestamp",
        "2026-09-13T12:00:00Z",
        "--output",
        path_text(&redline),
        "--json",
    ]);
    assert_success(&compared, "compare JSON");
    let value: serde_json::Value = serde_json::from_slice(&compared.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "all-supported-stories",
            "main_story_revisions": 2,
            "diagnostics": [],
            "output": path_text(&redline),
        })
    );

    let revisions = cli(&["revision", "list", path_text(&redline), "--json"]);
    assert_success(&revisions, "revision list JSON");
    let value: serde_json::Value = serde_json::from_slice(&revisions.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "revisions": [
                {
                    "id": 0,
                    "author": "Alice",
                    "timestamp": "2026-09-13T12:00:00Z",
                    "kind": "deletion",
                },
                {
                    "id": 1,
                    "author": "Alice",
                    "timestamp": "2026-09-13T12:00:00Z",
                    "kind": "insertion",
                },
            ],
        })
    );

    let toc_output = temp.path.join("toc.docx");
    let rebuilt = cli(&[
        "toc",
        "rebuild",
        path_text(&input),
        "--output",
        path_text(&toc_output),
        "--json",
    ]);
    assert_success(&rebuilt, "toc rebuild JSON");
    let value: serde_json::Value = serde_json::from_slice(&rebuilt.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "entry_count": 0,
            "bookmark_count": 0,
            "diagnostic_count": 0,
            "output": path_text(&toc_output),
        })
    );
    assert_eq!(
        Document::open(toc_output).unwrap().text(),
        "Comment target\n"
    );
}

#[test]
fn comment_commands_round_trip_one_resolved_thread() {
    let temp = TempWorkspace::new("comment-round-trip");
    let input = temp.path.join("input.docx");
    let added_path = temp.path.join("added.docx");
    let replied_path = temp.path.join("replied.docx");
    let resolved_path = temp.path.join("resolved.docx");
    let removed_path = temp.path.join("removed.docx");
    write_document(&input, &["Comment target"]);

    let added = cli(&[
        "comment",
        "add",
        path_text(&input),
        "--start-paragraph",
        "0",
        "--start-run",
        "0",
        "--end-paragraph",
        "0",
        "--end-run",
        "1",
        "--author",
        "Alice",
        "--initials",
        "AL",
        "--text",
        "Review this",
        "--output",
        path_text(&added_path),
        "--json",
    ]);
    assert_success(&added, "comment add");
    let value: serde_json::Value = serde_json::from_slice(&added.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "action": "add",
            "comment_id": 0,
            "output": path_text(&added_path),
        })
    );

    let replied = cli(&[
        "comment",
        "reply",
        path_text(&added_path),
        "--id",
        "0",
        "--author",
        "Bob",
        "--text",
        "Agreed",
        "--output",
        path_text(&replied_path),
        "--json",
    ]);
    assert_success(&replied, "comment reply");
    let value: serde_json::Value = serde_json::from_slice(&replied.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "action": "reply",
            "comment_id": 1,
            "parent_id": 0,
            "output": path_text(&replied_path),
        })
    );

    let resolved = cli(&[
        "comment",
        "resolve",
        path_text(&replied_path),
        "--id",
        "0",
        "--output",
        path_text(&resolved_path),
        "--json",
    ]);
    assert_success(&resolved, "comment resolve");
    let value: serde_json::Value = serde_json::from_slice(&resolved.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "action": "resolve",
            "comment_id": 0,
            "output": path_text(&resolved_path),
        })
    );
    let listed = cli(&["comment", "list", path_text(&resolved_path), "--json"]);
    assert_success(&listed, "comment list resolved thread");
    let value: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "comments": [
                {
                    "id": 0,
                    "author": "Alice",
                    "initials": "AL",
                    "date": null,
                    "text": "Review this",
                    "parent_id": null,
                    "resolved": true,
                },
                {
                    "id": 1,
                    "author": "Bob",
                    "initials": null,
                    "date": null,
                    "text": "Agreed",
                    "parent_id": 0,
                    "resolved": false,
                },
            ],
        })
    );

    let removed = cli(&[
        "comment",
        "remove",
        path_text(&resolved_path),
        "--id",
        "0",
        "--output",
        path_text(&removed_path),
        "--json",
    ]);
    assert_success(&removed, "comment remove");
    let value: serde_json::Value = serde_json::from_slice(&removed.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "action": "remove",
            "comment_id": 0,
            "output": path_text(&removed_path),
        })
    );
    assert!(Document::open(&removed_path).unwrap().comments().is_empty());
    assert_eq!(Document::open(&input).unwrap().comments().len(), 0);
}

#[test]
fn revision_filters_change_only_matching_revisions() {
    let temp = TempWorkspace::new("revision-filters");
    let input = temp.path.join("input.docx");
    let by_id = temp.path.join("by-id.docx");
    let by_author = temp.path.join("by-author.docx");
    let by_date = temp.path.join("by-date.docx");
    write_revision_fixture(&input);

    let accepted = cli(&[
        "revision",
        "accept",
        path_text(&input),
        "--id",
        "10",
        "--output",
        path_text(&by_id),
        "--json",
    ]);
    assert_success(&accepted, "revision accept");
    let value: serde_json::Value = serde_json::from_slice(&accepted.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "all-supported-stories",
            "action": "accept",
            "selector": { "kind": "id", "id": 10 },
            "resolved": 1,
            "output": path_text(&by_id),
        })
    );
    assert_eq!(
        Document::open(&by_id)
            .unwrap()
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        vec![20, 30]
    );

    let rejected = cli(&[
        "revision",
        "reject",
        path_text(&input),
        "--author",
        "Alice",
        "--output",
        path_text(&by_author),
    ]);
    assert_success(&rejected, "revision reject by author");
    assert_eq!(
        Document::open(&by_author)
            .unwrap()
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        vec![20]
    );

    let dated = cli(&[
        "revision",
        "accept",
        path_text(&input),
        "--start-date",
        "2026-01-02T00:00:00Z",
        "--end-date",
        "2026-01-02T00:00:00Z",
        "--output",
        path_text(&by_date),
    ]);
    assert_success(&dated, "revision accept by date");
    assert_eq!(
        Document::open(&by_date)
            .unwrap()
            .revisions()
            .iter()
            .map(|revision| revision.id())
            .collect::<Vec<_>>(),
        vec![10, 30]
    );

    let invalid_output = temp.path.join("invalid.docx");
    let invalid = cli(&[
        "revision",
        "accept",
        path_text(&input),
        "--start-date",
        "2026-01-01T00:00:00Z",
        "--output",
        path_text(&invalid_output),
    ]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(!invalid_output.exists());

    let conflicting_output = temp.path.join("conflicting.docx");
    let conflicting = cli(&[
        "revision",
        "reject",
        path_text(&input),
        "--id",
        "10",
        "--author",
        "Alice",
        "--output",
        path_text(&conflicting_output),
    ]);
    assert_eq!(conflicting.status.code(), Some(2));
    assert!(!conflicting_output.exists());
}

#[test]
fn compare_accept_and_reject_reproduce_each_input() {
    let temp = TempWorkspace::new("compare-round-trip");
    let original = temp.path.join("original.docx");
    let edited = temp.path.join("edited.docx");
    let redline = temp.path.join("redline.docx");
    let accepted_path = temp.path.join("accepted.docx");
    let rejected_path = temp.path.join("rejected.docx");
    write_document(&original, &["Original"]);
    write_document(&edited, &["Edited"]);

    let compared = cli(&[
        "compare",
        path_text(&original),
        path_text(&edited),
        "--author",
        "Alice",
        "--timestamp",
        "2026-09-13T12:00:00Z",
        "--output",
        path_text(&redline),
    ]);
    assert_success(&compared, "compare");
    let mut accepted = Document::open(&redline).unwrap();
    assert_eq!(accepted.accept_all().unwrap(), 2);
    accepted.save(&accepted_path).unwrap();
    let mut rejected = Document::open(&redline).unwrap();
    assert_eq!(rejected.reject_all().unwrap(), 2);
    rejected.save(&rejected_path).unwrap();

    assert_eq!(
        Document::open(accepted_path).unwrap().text(),
        Document::open(edited).unwrap().text()
    );
    assert_eq!(
        Document::open(rejected_path).unwrap().text(),
        Document::open(original).unwrap().text()
    );
}

#[test]
fn cli_structured_text_layout_and_guarded_replace_preserve_exact_contracts() {
    let temp = TempWorkspace::new("structured-automation");
    let input = temp.path.join("structured.docx");
    let mismatch = temp.path.join("mismatch.docx");
    let replaced = temp.path.join("replaced.docx");
    let mut document = fixture_document(&["Alpha TOKEN"]);
    {
        let mut table = document.add_table(1, 1);
        table.cell(0, 0).unwrap().set_text("Nested TOKEN");
    }
    document.save(&input).unwrap();

    let text = cli(&["text", path_text(&input), "--json"]);
    assert_success(&text, "structured text");
    let text: serde_json::Value = serde_json::from_slice(&text.stdout).unwrap();
    assert_eq!(text["schema"], 1);
    assert_eq!(text["scope"], "main");
    assert_eq!(text["revision_view"], "accepted");
    assert_eq!(text["paragraphs"].as_array().unwrap().len(), 2);
    assert_eq!(text["paragraphs"][0]["body_index"], 0);
    assert_eq!(text["paragraphs"][0]["path"], json!([]));
    assert_eq!(
        text["paragraphs"][1]["path"],
        json!([
            {"kind": "row", "index": 0},
            {"kind": "cell", "index": 0},
            {"kind": "paragraph", "index": 0}
        ])
    );

    let layout = cli(&["layout", path_text(&input), "--json"]);
    assert_success(&layout, "structured layout");
    let layout: serde_json::Value = serde_json::from_slice(&layout.stdout).unwrap();
    assert_eq!(layout["schema"], 1);
    assert_eq!(layout["scope"], "main");
    assert_eq!(layout["units"], "points");
    let body_items = layout["body_items"].as_array().unwrap();
    assert_eq!(body_items.len(), 2);
    for (body_index, item) in body_items.iter().enumerate() {
        assert_eq!(item["body_index"], body_index);
        let fragments = item["fragments"].as_array().unwrap();
        assert!(!fragments.is_empty());
        for fragment in fragments {
            assert!(fragment["physical_page"].as_u64().unwrap() >= 1);
            assert!(fragment["displayed_page"].as_u64().unwrap() >= 1);
            assert!(fragment["width"].as_f64().unwrap() > 0.0);
            assert!(fragment["height"].as_f64().unwrap() > 0.0);
        }
    }

    let guarded = cli(&[
        "replace",
        path_text(&input),
        "--placeholder",
        "TOKEN",
        "--value",
        "done",
        "--expect",
        "3",
        "--output",
        path_text(&mismatch),
    ]);
    assert!(!guarded.status.success());
    assert!(!mismatch.exists());

    let replaced_output = cli(&[
        "replace",
        path_text(&input),
        "--placeholder",
        "TOKEN",
        "--value",
        "done",
        "--expect",
        "2",
        "--output",
        path_text(&replaced),
    ]);
    assert_success(&replaced_output, "guarded replacement");
    assert_eq!(
        Document::open(replaced).unwrap().text(),
        "Alpha done\nNested done\t\n"
    );
}

#[test]
fn text_json_preserves_nested_paths_styles_numbering_and_run_formatting() {
    let temp = TempWorkspace::new("structured-text");
    let input = temp.path.join("text.docx");
    let mut document = fixture_document(&[]);
    {
        let mut paragraph = document.add_paragraph("");
        paragraph.set_style("Heading1");
        assert!(paragraph.set_numbering(7, 2));
        paragraph.add_run("plain");
        let mut formatted = paragraph.add_run("rich");
        formatted.set_bold(true);
        formatted.set_italic(false);
        formatted.set_font("Carlito");
        formatted.set_size(14.0);
        formatted.set_color("FF0000");
    }
    {
        let mut table = document.add_table(1, 1);
        table.cell(0, 0).unwrap().set_text("nested");
    }
    document.save(&input).unwrap();

    let output = cli(&["text", path_text(&input), "--json"]);
    assert_success(&output, "structured text formatting");
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value,
        json!({
            "schema": 1,
            "scope": "main",
            "revision_view": "accepted",
            "paragraphs": [
                {
                    "body_index": 0,
                    "path": [],
                    "style": "Heading1",
                    "numbering": {"num_id": 7, "level": 2},
                    "text": "plainrich",
                    "runs": [
                        {"index": 0, "text": "plain", "formatting": null},
                        {
                            "index": 1,
                            "text": "rich",
                            "formatting": {
                                "bold": true,
                                "italic": false,
                                "strike": null,
                                "underline": null,
                                "font": "Carlito",
                                "size_points": 14.0,
                                "color": "FF0000",
                                "highlight": null,
                                "language": null,
                                "style": null
                            }
                        }
                    ]
                },
                {
                    "body_index": 1,
                    "path": [
                        {"kind": "row", "index": 0},
                        {"kind": "cell", "index": 0},
                        {"kind": "paragraph", "index": 0}
                    ],
                    "style": null,
                    "numbering": null,
                    "text": "nested",
                    "runs": [
                        {"index": 0, "text": "nested", "formatting": null}
                    ]
                }
            ]
        })
    );
}
