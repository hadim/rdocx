use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use oxml_opc::OpcPackage;
use oxml_opc::relationship::rel_types;
use rdocx::{Document, WordPackageClass};
use serde_json::{Value, json};

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

/// Runs the CLI while its reader takes a short prefix of standard output and
/// then closes it, as `| head -1` does.
fn cli_with_closed_stdout(args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rdocx"))
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn rdocx CLI");
    let mut stdout = child.stdout.take().expect("piped stdout");
    stdout
        .read_exact(&mut [0; 64])
        .expect("read an output prefix");
    drop(stdout);
    child.wait_with_output().expect("wait for rdocx CLI")
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

/// Asserts that `args` refuses to replace an existing `output` and leaves it
/// byte-identical, and that the same command with `--force` replaces it.
fn assert_existing_output_needs_force(args: &[&str], output: &Path) {
    fs::write(output, b"keep me").unwrap();
    let refused = cli(args);
    assert_eq!(refused.status.code(), Some(1), "{args:?} was not refused");
    assert_eq!(
        String::from_utf8_lossy(&refused.stderr),
        format!(
            "Error: output already exists: {} (pass --force to replace it)\n",
            output.display()
        )
    );
    assert_eq!(fs::read(output).unwrap(), b"keep me");

    let forced = [args, &["--force"]].concat();
    assert_success(&cli(&forced), &forced.join(" "));
    assert_ne!(fs::read(output).unwrap(), b"keep me");
    assert!(
        fs::read_dir(output.parent().unwrap())
            .unwrap()
            .all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".tmp")
            })
    );
}

/// Asserts that `args`, whose output is `input` under some spelling, leaves
/// the input byte-identical with and without `--force`.
fn assert_input_is_never_replaced(args: &[&str], input: &Path, output: &str) {
    let before = fs::read(input).unwrap();
    for force in [&[][..], &["--force"]] {
        let refused = cli(&[args, force].concat());
        assert_eq!(refused.status.code(), Some(1), "{args:?} {force:?}");
        assert_eq!(
            String::from_utf8_lossy(&refused.stderr),
            format!("Error: output is the input file: {output}\n")
        );
        assert_eq!(fs::read(input).unwrap(), before, "{args:?} {force:?}");
    }
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

    let revisions = [
        ("Alpha", 10, "Alice", "2026-01-01T00:00:00Z"),
        ("Bravo", 20, "Bob", "2026-01-02T00:00:00Z"),
        ("Charlie", 30, "Alice", "2026-01-03T00:00:00Z"),
    ];
    for (text, id, author, timestamp) in revisions {
        rewrite_run(path, text, |run| {
            format!(
                "<w:ins w:id=\"{id}\" w:author=\"{author}\" w:date=\"{timestamp}\">{run}</w:ins>"
            )
        });
    }
    assert_eq!(Document::open(path).unwrap().revisions().len(), 3);
}

/// Replace the main-story run whose text is exactly `text` with `rewrite(run)`.
fn rewrite_run(path: &Path, text: &str, rewrite: impl FnOnce(&str) -> String) {
    let mut package = OpcPackage::open(path).unwrap();
    let part = package.main_document_part().unwrap();
    let mut xml = String::from_utf8(package.get_part(&part).unwrap().to_vec()).unwrap();
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
    let replacement = rewrite(&xml[run_start..run_end]);
    xml.replace_range(run_start..run_end, &replacement);
    package.set_part(&part, xml.into_bytes());
    package.save(path).unwrap();
}

fn package_part_text(path: &Path, part: &str) -> String {
    let package = OpcPackage::open(path).unwrap();
    String::from_utf8(package.get_part(part).expect("part is present").to_vec()).unwrap()
}

fn header_part_name(path: &Path) -> String {
    let package = OpcPackage::open(path).unwrap();
    let mut headers = package
        .parts
        .keys()
        .filter(|name| name.starts_with("/word/header"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(headers.len(), 1, "fixture has one header part");
    headers.remove(0)
}

/// Concatenate the text of every `w:delText` element, like the acceptance
/// suite's `<w:delText[^>]*>([^<]*)</w:delText>` search.
fn deleted_text(xml: &str) -> String {
    let mut deleted = String::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<w:delText") {
        rest = &rest[start..];
        let tag_end = rest.find('>').expect("delText start tag ends");
        if rest[..tag_end].ends_with('/') {
            rest = &rest[tag_end..];
            continue;
        }
        let content_end = rest.find("</w:delText>").expect("delText element ends");
        deleted.push_str(&rest[tag_end + 1..content_end]);
        rest = &rest[content_end..];
    }
    deleted
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
fn printing_commands_end_cleanly_when_the_reader_closes_stdout() {
    let temp = TempWorkspace::new("closed-stdout");
    let input = temp.path.join("long.docx");
    // Several hundred kilobytes outgrow a default pipe buffer, so the CLI
    // is still writing when its reader goes away.
    let lines = (0..20_000)
        .map(|index| format!("Line {index}, lorem ipsum dolor sit amet."))
        .collect::<Vec<_>>();
    write_document(
        &input,
        &lines.iter().map(String::as_str).collect::<Vec<_>>(),
    );

    for args in [
        vec!["text", path_text(&input)],
        vec!["text", path_text(&input), "--json"],
    ] {
        let output = cli_with_closed_stdout(&args);
        assert_success(&output, &args.join(" "));
    }
}

#[test]
fn validate_keeps_its_verdict_when_the_reader_closes_stdout() {
    let temp = TempWorkspace::new("validate-closed-stdout");
    let valid = temp.path.join("valid.docx");
    let corrupt = temp.path.join("corrupt.docx");
    write_document(&valid, &["Valid content"]);
    // Thousands of undeclared parts make a report that outgrows a default pipe
    // buffer, so the CLI is still writing when its reader goes away.
    let mut package = OpcPackage::open(&valid).unwrap();
    for index in 0..3_000 {
        package.set_part(&format!("/word/undeclared{index}.dat"), Vec::new());
    }
    package.save(&corrupt).unwrap();

    let output = cli_with_closed_stdout(&["validate", path_text(&corrupt)]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_failing_stdout_other_than_a_closed_pipe_is_an_error() {
    let temp = TempWorkspace::new("full-stdout");
    let input = temp.path.join("short.docx");
    write_document(&input, &["No space left"]);
    let full = fs::OpenOptions::new()
        .write(true)
        .open("/dev/full")
        .expect("open /dev/full");

    let output = Command::new(env!("CARGO_BIN_EXE_rdocx"))
        .args(["text", path_text(&input)])
        .stdout(full)
        .output()
        .expect("run rdocx CLI");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("Error: "), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

#[test]
fn text_prints_paragraphs_wrapped_by_a_body_content_control() {
    let temp = TempWorkspace::new("text-content-control");
    let input = temp.path.join("control.docx");
    write_document(&input, &["Body text"]);

    let mut package =
        OpcPackage::from_reader(std::io::Cursor::new(fs::read(&input).unwrap())).unwrap();
    let xml = std::str::from_utf8(package.get_part("/word/document.xml").unwrap())
        .unwrap()
        .replacen(
            "<w:body>",
            r#"<w:body><w:sdt><w:sdtPr><w:tag w:val="goog_rdk_1"/></w:sdtPr><w:sdtContent><w:p><w:r><w:t>Wrapped paragraph</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
            1,
        );
    package.set_part("/word/document.xml", xml.into_bytes());
    let mut file = fs::File::create(&input).unwrap();
    package.write_to(&mut file).unwrap();

    let output = cli(&["text", path_text(&input)]);
    assert_success(&output, "text");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "Wrapped paragraph\nBody text\n"
    );
}

#[test]
fn plain_text_prints_the_accepted_view_of_tracked_changes() {
    let temp = TempWorkspace::new("text-tracked");
    let input = temp.path.join("tracked.docx");
    let mut document = fixture_document(&[]);
    {
        let mut paragraph = document.add_paragraph("");
        paragraph.add_run("Tracked: ");
        paragraph.add_run("ins NEEDLE");
        paragraph.add_run("gone");
    }
    {
        let mut table = document.add_table(1, 1);
        table.cell(0, 0).unwrap().set_text("cell NEEDLE");
    }
    document.save(&input).unwrap();
    rewrite_run(&input, "ins NEEDLE", |run| {
        format!(r#"<w:ins w:id="1" w:author="Ada" w:date="2026-09-29T08:00:00Z">{run}</w:ins>"#)
    });
    rewrite_run(&input, "gone", |run| {
        let run = run
            .replace("<w:t", "<w:delText")
            .replace("</w:t>", "</w:delText>");
        format!(r#"<w:del w:id="2" w:author="Ada" w:date="2026-09-29T08:00:00Z">{run}</w:del>"#)
    });
    rewrite_run(&input, "cell NEEDLE", |run| {
        format!(r#"<w:ins w:id="3" w:author="Ada" w:date="2026-09-29T08:00:00Z">{run}</w:ins>"#)
    });
    assert_eq!(Document::open(&input).unwrap().revisions().len(), 3);

    let plain = cli(&["text", path_text(&input)]);
    assert_success(&plain, "plain text");
    assert_eq!(
        String::from_utf8(plain.stdout).unwrap(),
        "Tracked: ins NEEDLE\ncell NEEDLE\t\n"
    );

    let structured = cli(&["text", path_text(&input), "--json"]);
    assert_success(&structured, "structured text");
    let value: serde_json::Value = serde_json::from_slice(&structured.stdout).unwrap();
    let texts = value["paragraphs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|paragraph| paragraph["text"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["Tracked: ins NEEDLE", "cell NEEDLE"]);
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
fn convert_replaces_an_existing_output_only_with_force_and_never_its_input() {
    let temp = TempWorkspace::new("convert-output-policy");
    let input = temp.path.join("source.docx");
    write_document(&input, &["Output policy"]);
    let source = path_text(&input);
    let spelled = temp.path.join("pages/../source.docx");
    fs::create_dir(temp.path.join("pages")).unwrap();

    let default_pdf = temp.path.join("source.pdf");
    assert_existing_output_needs_force(&["convert", source, "--to", "pdf"], &default_pdf);
    for to in ["pdf", "md", "html", "png", "tiff"] {
        let output = temp.path.join(format!("converted.{to}"));
        assert_existing_output_needs_force(
            &[
                "convert",
                source,
                "--to",
                to,
                "--dpi",
                "24",
                "-o",
                path_text(&output),
            ],
            &output,
        );
        for spelling in [source, path_text(&spelled)] {
            assert_input_is_never_replaced(
                &["convert", source, "--to", to, "--dpi", "24", "-o", spelling],
                &input,
                spelling,
            );
        }
    }
    #[cfg(unix)]
    {
        let refused = cli(&[
            "convert",
            source,
            "--to",
            "md",
            "-o",
            "/dev/null",
            "--force",
        ]);
        assert_eq!(refused.status.code(), Some(1));
        assert_eq!(
            String::from_utf8_lossy(&refused.stderr),
            "Error: output is not a regular file: /dev/null\n"
        );
    }
}

#[test]
fn render_replaces_existing_pages_only_with_force_and_never_its_input() {
    let temp = TempWorkspace::new("render-output-policy");
    let input = temp.path.join("source.docx");
    let pages = temp.path.join("pages");
    write_document(&input, &["Output policy"]);
    fs::create_dir(&pages).unwrap();
    let render = [
        "render",
        path_text(&input),
        "-o",
        path_text(&pages),
        "--dpi",
        "24",
    ];

    assert_existing_output_needs_force(&render, &pages.join("source_page1.png"));
    assert_existing_output_needs_force(
        &[render.as_slice(), &["--format", "tiff"]].concat(),
        &pages.join("source.tiff"),
    );

    // A document named like its own TIFF output, rendered into its folder.
    let named = temp.path.join("named.tiff");
    fs::copy(&input, &named).unwrap();
    assert_input_is_never_replaced(
        &[
            "render",
            path_text(&named),
            "-o",
            path_text(&temp.path),
            "--format",
            "tiff",
            "--dpi",
            "24",
        ],
        &named,
        path_text(&named),
    );
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
fn replace_output_extension_selects_the_package_class() {
    let temp = TempWorkspace::new("replace-class");
    let input = temp.path.join("template.dotx");
    let replaced = temp.path.join("report.docx");
    write_document(&input, &["Hello {{name}}"]);
    assert_eq!(
        Document::open(&input).unwrap().package_class().unwrap(),
        WordPackageClass::Template
    );

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
        Document::open(&replaced).unwrap().package_class().unwrap(),
        WordPackageClass::Document
    );
}

#[test]
fn replace_refuses_a_macro_free_output_for_a_vba_input() {
    let temp = TempWorkspace::new("replace-vba");
    let input = temp.path.join("macros.docm");
    let replaced = temp.path.join("report.docx");
    let mut package = OpcPackage::from_reader(std::io::Cursor::new(
        fixture_document(&["Hello {{name}}"]).to_bytes().unwrap(),
    ))
    .unwrap();
    package
        .get_or_create_part_rels("/word/document.xml")
        .add(rel_types::VBA_PROJECT, "vbaProject.bin");
    package.set_part("/word/vbaProject.bin", b"vba-project".to_vec());
    package.content_types.add_override(
        "/word/vbaProject.bin",
        "application/vnd.ms-office.vbaProject",
    );
    package.content_types.add_override(
        "/word/document.xml",
        "application/vnd.ms-word.document.macroEnabled.main+xml",
    );
    package.save(&input).unwrap();

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
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("VBA project"));
    let entries: Vec<_> = fs::read_dir(&temp.path)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(entries, [input.file_name().unwrap()]);
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

/// Word writes a text box twice, as DrawingML in `mc:Choice` and as VML in
/// `mc:Fallback`. `rdocx replace --expect 1` failed on it with `found 2`.
#[test]
fn replace_with_expect_counts_a_word_text_box_once() {
    let temp = TempWorkspace::new("replace-word-text-box");
    let input = temp.path.join("text-box.docx");
    let replaced = temp.path.join("replaced.docx");
    write_document(&input, &["seed"]);

    let copy = r#"<w:txbxContent><w:p><w:r><w:t>Box NEEDLE</w:t></w:r></w:p></w:txbxContent>"#;
    let text_box = format!(
        r#"<w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wp:anchor><a:graphic><a:graphicData><wps:wsp><wps:txbx>{copy}</wps:txbx></wps:wsp></a:graphicData></a:graphic></wp:anchor></w:drawing></mc:Choice><mc:Fallback><w:pict><v:shape><v:textbox>{copy}</v:textbox></v:shape></w:pict></mc:Fallback></mc:AlternateContent></w:r>"#
    );
    let mut package =
        OpcPackage::from_reader(std::io::Cursor::new(fs::read(&input).unwrap())).unwrap();
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wp="http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape" xmlns:v="urn:schemas-microsoft-com:vml"><w:body><w:p><w:r><w:t>Anchor paragraph</w:t></w:r>{text_box}</w:p></w:body></w:document>"#
        )
        .into_bytes(),
    );
    package
        .write_to(&mut fs::File::create(&input).unwrap())
        .unwrap();

    let output = cli(&[
        "replace",
        path_text(&input),
        "--placeholder",
        "NEEDLE",
        "--value",
        "X",
        "--expect",
        "1",
        "--output",
        path_text(&replaced),
    ]);
    assert_success(&output, "replace");

    let package = OpcPackage::open(&replaced).unwrap();
    let saved =
        String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
    assert!(!saved.contains("NEEDLE"), "{saved}");
    assert_eq!(saved.matches(">Box X<").count(), 2, "{saved}");
}

/// `rdocx replace --expect 1` found none of the text that Google Docs and
/// Word keep in content controls: a run wrapped inside its paragraph, a
/// paragraph wrapped at body level, a control in a table cell, nested
/// controls, a control in a text box, and the table and controls of a
/// header or footer.
#[test]
fn replace_with_expect_counts_the_text_of_content_controls_everywhere() {
    let temp = TempWorkspace::new("replace-content-controls");
    let input = temp.path.join("controls.docx");
    write_document(&input, &["seed"]);

    let control = |tag: &str, content: &str| {
        format!(
            r#"<w:sdt><w:sdtPr><w:tag w:val="{tag}"/></w:sdtPr><w:sdtContent>{content}</w:sdtContent></w:sdt>"#
        )
    };
    let run = |text: &str| format!("<w:r><w:t>{text}</w:t></w:r>");
    let paragraph = |content: &str| format!("<w:p>{content}</w:p>");
    let table = |cell: &str| {
        format!(
            r#"<w:tbl><w:tblGrid><w:gridCol w:w="4000"/></w:tblGrid><w:tr><w:tc>{cell}</w:tc></w:tr></w:tbl>"#
        )
    };
    let text_box = format!(
        r#"<w:r><w:pict><v:shape style="width:216pt;height:72pt"><v:textbox><w:txbxContent>{}</w:txbxContent></v:textbox></v:shape></w:pict></w:r>"#,
        control("box", &paragraph(&run("{{box}}")))
    );
    let body = [
        paragraph(&[run("Body "), control("goog_rdk_0", &run("{{inline}}"))].concat()),
        control("goog_rdk_1", &paragraph(&run("{{block}}"))),
        table(&control("cell", &paragraph(&run("{{cell}}")))),
        control("outer", &paragraph(&control("inner", &run("{{nested}}")))),
        paragraph(&[run("Host"), text_box].concat()),
    ]
    .concat();
    let word = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let header = format!(
        r#"<w:hdr xmlns:w="{word}">{}{}</w:hdr>"#,
        table(&paragraph(&run("{{header_table}}"))),
        control("header", &paragraph(&run("{{header_control}}")))
    );
    let footer = format!(
        r#"<w:ftr xmlns:w="{word}">{}{}</w:ftr>"#,
        control("page", &paragraph(&run("{{footer_control}}"))),
        paragraph(&run("Confidential"))
    );

    let mut package =
        OpcPackage::from_reader(std::io::Cursor::new(fs::read(&input).unwrap())).unwrap();
    let mut references = String::new();
    for (kind, xml, rel_type) in [
        ("header", header, rel_types::HEADER),
        ("footer", footer, rel_types::FOOTER),
    ] {
        let part = format!("/word/{kind}1.xml");
        package.set_part(&part, xml.into_bytes());
        package.content_types.add_override(
            &part,
            &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"),
        );
        let id = package
            .get_or_create_part_rels("/word/document.xml")
            .add(rel_type, &format!("{kind}1.xml"));
        references.push_str(&format!(
            r#"<w:{kind}Reference w:type="default" r:id="{id}"/>"#
        ));
    }
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<w:document xmlns:w="{word}" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:v="urn:schemas-microsoft-com:vml"><w:body>{body}<w:sectPr>{references}</w:sectPr></w:body></w:document>"#
        )
        .into_bytes(),
    );
    package
        .write_to(&mut fs::File::create(&input).unwrap())
        .unwrap();

    for name in [
        "inline",
        "block",
        "cell",
        "nested",
        "box",
        "header_table",
        "header_control",
        "footer_control",
    ] {
        let tag = format!("{{{{{name}}}}}");
        let replaced = temp.path.join(format!("{name}.docx"));
        let output = cli(&[
            "replace",
            path_text(&input),
            "--placeholder",
            &tag,
            "--value",
            "done",
            "--expect",
            "1",
            "--output",
            path_text(&replaced),
        ]);
        assert_success(&output, name);

        let package = OpcPackage::open(&replaced).unwrap();
        let saved = ["document", "header1", "footer1"]
            .map(|part| {
                let xml = package.get_part(&format!("/word/{part}.xml")).unwrap();
                String::from_utf8(xml.to_vec()).unwrap()
            })
            .concat();
        assert!(!saved.contains(&tag), "{name}: {saved}");
        assert_eq!(saved.matches(">done<").count(), 1, "{name}: {saved}");
    }
}

/// `rdocx replace --expect` counts what a reader sees in a footnote, an
/// endnote and a tracked insertion, and neither the separators of the notes
/// parts nor deleted text.
#[test]
fn replace_with_expect_counts_notes_and_tracked_insertions() {
    let temp = TempWorkspace::new("replace-notes");
    let input = temp.path.join("notes.docx");
    let replaced = temp.path.join("replaced.docx");
    write_document(&input, &["seed"]);

    let word = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    let notes = |kind: &str| {
        format!(
            r#"<w:{kind}s xmlns:w="{word}"><w:{kind} w:type="separator" w:id="-1"><w:p><w:r><w:t>NEEDLE</w:t></w:r></w:p></w:{kind}><w:{kind} w:type="continuationSeparator" w:id="0"><w:p><w:r><w:continuationSeparator/></w:r></w:p></w:{kind}><w:{kind} w:id="1"><w:p><w:r><w:{kind}Ref/></w:r><w:r><w:t xml:space="preserve"> Note NEEDLE.</w:t></w:r></w:p></w:{kind}></w:{kind}s>"#
        )
    };
    let mut package =
        OpcPackage::from_reader(std::io::Cursor::new(fs::read(&input).unwrap())).unwrap();
    for (kind, rel_type) in [
        ("footnote", rel_types::FOOTNOTES),
        ("endnote", rel_types::ENDNOTES),
    ] {
        let part = format!("/word/{kind}s.xml");
        package.set_part(&part, notes(kind).into_bytes());
        package.content_types.add_override(
            &part,
            &format!("application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}s+xml"),
        );
        package
            .get_or_create_part_rels("/word/document.xml")
            .add(rel_type, &format!("{kind}s.xml"));
    }
    package.set_part(
        "/word/document.xml",
        format!(
            r#"<w:document xmlns:w="{word}"><w:body><w:p><w:r><w:t xml:space="preserve">Tracked: </w:t></w:r><w:ins w:id="901" w:author="Editor" w:date="2026-01-01T00:00:00Z"><w:r><w:t>ins NEEDLE</w:t></w:r></w:ins><w:del w:id="902" w:author="Editor" w:date="2026-01-01T00:00:00Z"><w:r><w:delText xml:space="preserve">del NEEDLE</w:delText></w:r></w:del><w:r><w:footnoteReference w:id="1"/></w:r><w:r><w:endnoteReference w:id="1"/></w:r></w:p><w:sectPr/></w:body></w:document>"#
        )
        .into_bytes(),
    );
    package
        .write_to(&mut fs::File::create(&input).unwrap())
        .unwrap();

    let replace = |expect: &str| {
        cli(&[
            "replace",
            path_text(&input),
            "--placeholder",
            "NEEDLE",
            "--value",
            "X",
            "--expect",
            expect,
            "--output",
            path_text(&replaced),
        ])
    };
    let output = replace("4");
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("found 3"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );

    let output = replace("3");
    assert_success(&output, "replace --expect 3");
    let package = OpcPackage::open(&replaced).unwrap();
    let part = |name: &str| String::from_utf8(package.get_part(name).unwrap().to_vec()).unwrap();
    let body = part("/word/document.xml");
    assert!(body.contains(">ins X</w:t></w:r></w:ins>"), "{body}");
    assert!(body.contains(">del NEEDLE</w:delText>"), "{body}");
    for kind in ["footnote", "endnote"] {
        let xml = part(&format!("/word/{kind}s.xml"));
        assert_eq!(xml, notes(kind).replace("Note NEEDLE", "Note X"), "{kind}");
    }
}

/// `rdocx text --json` shows the text inside a simple field, a smart tag and
/// an inline custom XML element, and `rdocx replace --expect` counts it.
#[test]
fn text_and_replace_read_simple_fields_smart_tags_and_custom_xml() {
    let temp = TempWorkspace::new("wrapped-text");
    let word = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
    for (name, start, end) in [
        (
            "field",
            r#"<w:fldSimple w:instr=" DOCPROPERTY Title ">"#,
            "</w:fldSimple>",
        ),
        (
            "smart-tag",
            r#"<w:smartTag w:element="place">"#,
            "</w:smartTag>",
        ),
        (
            "custom-xml",
            r#"<w:customXml w:element="item">"#,
            "</w:customXml>",
        ),
    ] {
        let input = temp.path.join(format!("{name}.docx"));
        let replaced = temp.path.join(format!("{name}-replaced.docx"));
        write_document(&input, &["seed"]);
        let mut package =
            OpcPackage::from_reader(std::io::Cursor::new(fs::read(&input).unwrap())).unwrap();
        package.set_part(
            "/word/document.xml",
            format!(
                r#"<w:document xmlns:w="{word}"><w:body><w:p><w:r><w:t xml:space="preserve">before </w:t></w:r>{start}<w:r><w:t>MID</w:t></w:r>{end}<w:r><w:t xml:space="preserve"> after</w:t></w:r></w:p><w:sectPr/></w:body></w:document>"#
            )
            .into_bytes(),
        );
        package
            .write_to(&mut fs::File::create(&input).unwrap())
            .unwrap();

        let output = cli(&["text", path_text(&input), "--json"]);
        assert_success(&output, "text --json");
        let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["paragraphs"][0]["text"], "before MID after", "{name}");

        let output = cli(&[
            "replace",
            path_text(&input),
            "--placeholder",
            "MID",
            "--value",
            "X",
            "--expect",
            "1",
            "--output",
            path_text(&replaced),
        ]);
        assert_success(&output, "replace --expect 1");
        let package = OpcPackage::open(&replaced).unwrap();
        let body =
            String::from_utf8(package.get_part("/word/document.xml").unwrap().to_vec()).unwrap();
        assert!(
            body.contains(&format!("{start}<w:r><w:t>X</w:t></w:r>{end}")),
            "{body}"
        );
    }
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

/// #160: rdocx 0.14 rewrote an empty comments root without its `w14`
/// declaration while `mc:Ignorable` still listed `w14`, and validate passed
/// the part.
#[test]
fn validate_reports_an_undeclared_ignorable_prefix() {
    let temp = TempWorkspace::new("validate-compatibility");
    let valid = temp.path.join("valid.docx");
    let broken = temp.path.join("broken.docx");
    write_document(&valid, &["Valid content"]);

    let mut package = OpcPackage::open(&valid).unwrap();
    package.set_part(
        "/word/comments.xml",
        br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:comments xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:w15="http://schemas.microsoft.com/office/word/2012/wordml" mc:Ignorable="w14 w15"></w:comments>"#.to_vec(),
    );
    package.content_types.add_override(
        "/word/comments.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.comments+xml",
    );
    let document_part = package.main_document_part().unwrap();
    package
        .get_or_create_part_rels(&document_part)
        .add(rel_types::COMMENTS, "comments.xml");
    package.save(&broken).unwrap();

    let output = cli(&["validate", path_text(&broken)]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!(
            "1 error(s) in {}:\n  1. part /word/comments.xml lists undeclared prefix `w14` in mc:Ignorable\n",
            broken.display()
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
            "options": {
                "granularity": "run",
                "ignore_formatting": false,
                "ignore_whitespace": false,
                "ignore_fields": false,
                "ignore_comments": false,
                "ignored_stories": [],
            },
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

/// GitHub issue #172: `comment add` counts runs the way `text --json` lists
/// them, including the runs of an inline content control.
#[test]
fn comment_add_counts_the_runs_that_text_json_lists() {
    let temp = TempWorkspace::new("comment-inline-control");
    let input = temp.path.join("input.docx");
    let mut document = fixture_document(&[]);
    let mut paragraph = document.add_paragraph("");
    for text in ["before ", "TAR", "GET", " after"] {
        paragraph.add_run(text);
    }
    document.save(&input).unwrap();
    let mut package = OpcPackage::open(&input).unwrap();
    let part = package.main_document_part().unwrap();
    let xml = String::from_utf8(package.get_part(&part).unwrap().to_vec()).unwrap();
    let start = xml[..xml.find(">TAR<").unwrap()].rfind("<w:r>").unwrap();
    let end =
        xml.find(">GET<").unwrap() + xml[xml.find(">GET<").unwrap()..].find("</w:r>").unwrap();
    let xml = format!(
        "{}<w:sdt><w:sdtPr/><w:sdtContent>{}</w:r></w:sdtContent></w:sdt>{}",
        &xml[..start],
        &xml[start..end],
        &xml[end + "</w:r>".len()..]
    );
    package.set_part(&part, xml.into_bytes());
    package.save(&input).unwrap();

    let text = cli(&["text", path_text(&input), "--json"]);
    assert_success(&text, "text --json");
    let value: serde_json::Value = serde_json::from_slice(&text.stdout).unwrap();
    let runs = value["paragraphs"][0]["runs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|run| run["text"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(runs, ["before ", "TAR", "GET", " after"]);

    let add = |start_run: &str, end_run: &str, output: &Path| {
        cli(&[
            "comment",
            "add",
            path_text(&input),
            "--start-paragraph",
            "0",
            "--start-run",
            start_run,
            "--end-paragraph",
            "0",
            "--end-run",
            end_run,
            "--author",
            "Alice",
            "--text",
            "Here",
            "--output",
            path_text(output),
        ])
    };
    let added = temp.path.join("added.docx");
    assert_success(&add("1", "3", &added), "comment add");
    let package = OpcPackage::open(&added).unwrap();
    let xml = String::from_utf8(package.get_part(&part).unwrap().to_vec()).unwrap();
    let anchored =
        &xml[xml.find("<w:commentRangeStart").unwrap()..xml.find("<w:commentRangeEnd").unwrap()];
    let anchored_text = anchored
        .split("<w:t>")
        .skip(1)
        .map(|text| &text[..text.find("</w:t>").unwrap()])
        .collect::<String>();
    assert_eq!(anchored_text, "TARGET");

    // From inside the control to after it cannot be anchored exactly.
    let refused = temp.path.join("refused.docx");
    let output = add("2", "4", &refused);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("crosses the edge of an inline content control"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!refused.exists());
}

#[test]
fn comment_add_and_reply_write_the_given_rfc3339_date() {
    const ADDED: &str = "2026-09-29T08:30:00Z";
    const REPLIED: &str = "2026-09-29T10:45:00+02:00";
    let temp = TempWorkspace::new("comment-dates");
    let input = temp.path.join("input.docx");
    let added_path = temp.path.join("added.docx");
    let replied_path = temp.path.join("replied.docx");
    write_document(&input, &["Comment target"]);
    let add = |date: &str, output: &Path| {
        cli(&[
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
            "--text",
            "Review this",
            "--date",
            date,
            "--output",
            path_text(output),
        ])
    };

    assert_success(&add(ADDED, &added_path), "dated comment add");
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
        "--date",
        REPLIED,
        "--output",
        path_text(&replied_path),
    ]);
    assert_success(&replied, "dated comment reply");

    let document = Document::open(&replied_path).unwrap();
    let dates = document
        .comments()
        .iter()
        .map(|comment| comment.date().map(str::to_owned))
        .collect::<Vec<_>>();
    assert_eq!(dates, [Some(ADDED.to_owned()), Some(REPLIED.to_owned())]);
    let comments_xml = package_part_text(&replied_path, "/word/comments.xml");
    assert!(comments_xml.contains(&format!(r#"w:date="{ADDED}""#)));
    assert!(comments_xml.contains(&format!(r#"w:date="{REPLIED}""#)));

    for invalid in ["2026-02-30T00:00:00Z", "2026-09-29", "yesterday"] {
        let rejected_path = temp.path.join("rejected.docx");
        let rejected = add(invalid, &rejected_path);
        assert_eq!(rejected.status.code(), Some(1), "{invalid}");
        assert!(rejected.stdout.is_empty());
        assert_eq!(
            String::from_utf8(rejected.stderr).unwrap(),
            format!("Error: invalid RFC 3339 comment timestamp: {invalid}\n")
        );
        assert!(!rejected_path.exists());
    }
    let rejected_reply_path = temp.path.join("rejected-reply.docx");
    let rejected_reply = cli(&[
        "comment",
        "reply",
        path_text(&added_path),
        "--id",
        "0",
        "--author",
        "Bob",
        "--text",
        "Agreed",
        "--date",
        "2026-09-29T25:00:00Z",
        "--output",
        path_text(&rejected_reply_path),
    ]);
    assert_eq!(rejected_reply.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&rejected_reply.stderr)
            .contains("invalid RFC 3339 comment timestamp: 2026-09-29T25:00:00Z")
    );
    assert!(!rejected_reply_path.exists());
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

const COMPARE_TIMESTAMP: &str = "2026-09-29T12:00:00Z";

/// Run `rdocx compare --json` and return its record.
fn compare_record(original: &Path, edited: &Path, redline: &Path, options: &[&str]) -> Value {
    let mut args = vec![
        "compare",
        path_text(original),
        path_text(edited),
        "--author",
        "Reviewer",
        "--timestamp",
        COMPARE_TIMESTAMP,
        "--output",
        path_text(redline),
        "--json",
    ];
    args.extend_from_slice(options);
    let output = cli(&args);
    assert_success(&output, &format!("compare {options:?}"));
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn compare_defaults_to_whole_runs_and_word_marks_only_the_changed_word() {
    const LOREM: &str =
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor.";
    let edited_text = LOREM.replace("dolor", "DOLOR");
    let temp = TempWorkspace::new("compare-granularity");
    let original = temp.path.join("original.docx");
    let edited = temp.path.join("edited.docx");
    write_document(&original, &[LOREM, "Second paragraph"]);
    write_document(&edited, &[&edited_text, "Second paragraph"]);

    for (granularity, recorded, deleted) in [
        (None, "run", LOREM),
        (Some("run"), "run", LOREM),
        (Some("word"), "word", "dolor"),
        (Some("character"), "character", "dolor"),
    ] {
        let redline = temp
            .path
            .join(format!("redline-{recorded}-{}.docx", granularity.is_some()));
        let options = granularity
            .map(|granularity| vec!["--granularity", granularity])
            .unwrap_or_default();
        let record = compare_record(&original, &edited, &redline, &options);
        assert_eq!(record["options"]["granularity"], recorded);
        assert_eq!(record["main_story_revisions"], 2, "{granularity:?}");
        assert_eq!(
            deleted_text(&package_part_text(&redline, "/word/document.xml")).trim(),
            deleted,
            "{granularity:?}"
        );
        let mut accepted = Document::open(&redline).unwrap();
        accepted.accept_all().unwrap();
        assert_eq!(
            accepted.text(),
            format!("{edited_text}\nSecond paragraph\n")
        );
        let mut rejected = Document::open(&redline).unwrap();
        rejected.reject_all().unwrap();
        assert_eq!(rejected.text(), format!("{LOREM}\nSecond paragraph\n"));
    }
}

#[test]
fn compare_ignore_flags_keep_the_original_side() {
    let temp = TempWorkspace::new("compare-ignore");
    let path = |name: &str| temp.path.join(name);

    write_document(&path("plain.docx"), &["plain"]);
    let mut bold = fixture_document(&[]);
    bold.add_paragraph("").add_run("plain").set_bold(true);
    bold.save(path("bold.docx")).unwrap();

    write_document(&path("spaced.docx"), &["old  tail"]);
    write_document(&path("single.docx"), &["old tail"]);

    for (name, result) in [("page-one.docx", "1"), ("page-two.docx", "2")] {
        write_document(&path(name), &["FIELD"]);
        rewrite_run(&path(name), "FIELD", |_| {
            format!(
                r#"<w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> PAGE </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r><w:r><w:t>{result}</w:t></w:r><w:r><w:fldChar w:fldCharType="end"/></w:r>"#
            )
        });
    }

    for (original, edited, flag, key) in [
        (
            "plain.docx",
            "bold.docx",
            "--ignore-formatting",
            "ignore_formatting",
        ),
        (
            "spaced.docx",
            "single.docx",
            "--ignore-whitespace",
            "ignore_whitespace",
        ),
        (
            "page-one.docx",
            "page-two.docx",
            "--ignore-fields",
            "ignore_fields",
        ),
    ] {
        let tracked = compare_record(
            &path(original),
            &path(edited),
            &path(&format!("tracked-{key}.docx")),
            &[],
        );
        assert_ne!(tracked["main_story_revisions"], 0, "{flag}");
        let ignored_path = path(&format!("ignored-{key}.docx"));
        let ignored = compare_record(&path(original), &path(edited), &ignored_path, &[flag]);
        assert_eq!(ignored["options"][key], true, "{flag}");
        assert_eq!(ignored["main_story_revisions"], 0, "{flag}");
        assert_eq!(
            Document::open(&ignored_path).unwrap().text(),
            Document::open(path(original)).unwrap().text(),
            "{flag}"
        );
    }

    // The edited side carries a comment added by the CLI and one changed word.
    write_document(&path("reviewed.docx"), &["Review this", "old ending"]);
    let commented = cli(&[
        "comment",
        "add",
        path_text(&path("reviewed.docx")),
        "--start-paragraph",
        "0",
        "--start-run",
        "0",
        "--end-paragraph",
        "0",
        "--end-run",
        "1",
        "--author",
        "Bob",
        "--text",
        "edited side note",
        "--output",
        path_text(&path("commented.docx")),
    ]);
    assert_success(&commented, "comment add on the edited copy");
    let replaced = cli(&[
        "replace",
        path_text(&path("commented.docx")),
        "--placeholder",
        "old",
        "--value",
        "new",
        "--expect",
        "1",
        "--output",
        path_text(&path("rewritten.docx")),
    ]);
    assert_success(&replaced, "replace on the edited copy");
    let redline = path("comments-ignored.docx");
    let record = compare_record(
        &path("reviewed.docx"),
        &path("rewritten.docx"),
        &redline,
        &["--ignore-comments"],
    );
    assert_eq!(record["options"]["ignore_comments"], true);
    assert_eq!(record["main_story_revisions"], 2);
    let document = Document::open(&redline).unwrap();
    assert!(document.comments().is_empty());
    assert_eq!(
        deleted_text(&package_part_text(&redline, "/word/document.xml")),
        "old ending"
    );
}

#[test]
fn compare_ignore_story_names_follow_the_python_story_kinds() {
    let temp = TempWorkspace::new("compare-stories");
    let original = temp.path.join("original.docx");
    let edited = temp.path.join("edited.docx");
    for (path, word) in [(&original, "old"), (&edited, "new")] {
        let mut document = fixture_document(&[&format!("{word} body")]);
        document.set_header(&format!("{word} header"));
        document.save(path).unwrap();
    }
    let header = header_part_name(&original);
    let both_tracked = ("old body".to_owned(), "old header".to_owned());
    let tracked_stories = |redline: &Path| {
        (
            deleted_text(&package_part_text(redline, "/word/document.xml")),
            deleted_text(&package_part_text(redline, &header)),
        )
    };

    let redline = temp.path.join("default.docx");
    let record = compare_record(&original, &edited, &redline, &[]);
    assert_eq!(record["options"]["ignored_stories"], json!([]));
    assert_eq!(tracked_stories(&redline), both_tracked);

    let redline = temp.path.join("header-ignored.docx");
    let record = compare_record(&original, &edited, &redline, &["--ignore-story", "header"]);
    assert_eq!(record["options"]["ignored_stories"], json!(["header"]));
    assert_eq!(
        tracked_stories(&redline),
        ("old body".to_owned(), String::new())
    );
    assert!(package_part_text(&redline, &header).contains("old header"));

    let redline = temp.path.join("body-ignored.docx");
    let record = compare_record(&original, &edited, &redline, &["--ignore-story", "body"]);
    assert_eq!(record["options"]["ignored_stories"], json!(["body"]));
    assert_eq!(record["main_story_revisions"], 0);
    assert_eq!(
        tracked_stories(&redline),
        (String::new(), "old header".to_owned())
    );

    let others = ["footer", "comment", "text_box", "footnote", "endnote"];
    let options = others
        .iter()
        .flat_map(|story| ["--ignore-story", story])
        .collect::<Vec<_>>();
    let redline = temp.path.join("others-ignored.docx");
    let record = compare_record(&original, &edited, &redline, &options);
    assert_eq!(record["options"]["ignored_stories"], json!(others));
    assert_eq!(tracked_stories(&redline), both_tracked);
}

#[test]
fn compare_rejects_unknown_and_duplicate_options_before_writing() {
    let temp = TempWorkspace::new("compare-invalid-options");
    let original = temp.path.join("original.docx");
    let edited = temp.path.join("edited.docx");
    let redline = temp.path.join("redline.docx");
    write_document(&original, &["before"]);
    write_document(&edited, &["after"]);

    for (options, status, message) in [
        (
            &["--granularity", "words"][..],
            2,
            r#"unknown comparison granularity "words", expected run, word, or character"#,
        ),
        (
            &["--ignore-story", "main"][..],
            2,
            r#"unknown comparison story "main", expected body, header, footer, comment, text_box, footnote, or endnote"#,
        ),
        (
            &["--ignore-story", "table_cell"][..],
            2,
            r#"unknown comparison story "table_cell""#,
        ),
        (
            &["--ignore-story", "header", "--ignore-story", "header"][..],
            1,
            "comparison options contain a duplicate ignored story",
        ),
    ] {
        let mut args = vec![
            "compare",
            path_text(&original),
            path_text(&edited),
            "--author",
            "Reviewer",
            "--timestamp",
            COMPARE_TIMESTAMP,
            "--output",
            path_text(&redline),
        ];
        args.extend_from_slice(options);
        let rejected = cli(&args);
        assert_eq!(rejected.status.code(), Some(status), "{options:?}");
        assert!(rejected.stdout.is_empty(), "{options:?}");
        let stderr = String::from_utf8(rejected.stderr).unwrap();
        assert!(stderr.contains(message), "{options:?}: {stderr}");
        assert!(!redline.exists(), "{options:?}");
    }
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
