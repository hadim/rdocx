//! rdocx CLI — "jq for DOCX"
//!
//! Inspect, convert, diff, and manipulate DOCX files from the command line.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use clap::{Args, Parser, Subcommand};

mod commands;

#[derive(Parser)]
#[command(name = "rdocx", version, about = "CLI tool for DOCX files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print document structure: paragraph/table count, styles, images, metadata
    Inspect {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Extract plain text from a DOCX file
    Text {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output accepted-view rich text as schema-1 JSON
        #[arg(long)]
        json: bool,
    },
    /// Inspect deterministic top-level body layout geometry
    Layout {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output point-space body fragments as schema-1 JSON
        #[arg(long)]
        json: bool,
    },
    /// Convert DOCX to another format (pdf, html, md, png, jpeg, tiff)
    Convert {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output format: pdf, html, md, png, jpeg, tiff
        #[arg(long, short = 't')]
        to: String,
        /// Output file path (defaults to input with new extension)
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Replace existing output files, but never the input file
        #[arg(long)]
        force: bool,
        /// DPI for image rendering (default: 150)
        #[arg(long, default_value = "150")]
        dpi: u32,
        /// Directory containing font files (.ttf/.otf) to use for PDF rendering
        #[arg(long)]
        font_dir: Option<PathBuf>,
        /// One-based page range for image output, such as 1,3-5
        #[arg(long)]
        pages: Option<String>,
        /// JPEG quality from 1 through 100
        #[arg(long, default_value = "90")]
        quality: u8,
        /// Preserve unpainted PNG pixels as transparent
        #[arg(long)]
        transparent: bool,
    },
    /// Structural diff between two DOCX files
    Diff {
        /// First DOCX file
        file_a: PathBuf,
        /// Second DOCX file
        file_b: PathBuf,
    },
    /// Replace placeholders in a DOCX file
    Replace {
        /// Path to the DOCX file
        file: PathBuf,
        /// Placeholder string
        #[arg(long, short = 'p')]
        placeholder: String,
        /// Replacement value
        #[arg(long, short = 'v')]
        value: String,
        /// Output file path
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Require exactly this many replacements before publishing output
        #[arg(long)]
        expect: Option<usize>,
    },
    /// Validate OOXML conformance
    Validate {
        /// Path to the DOCX file
        file: PathBuf,
    },
    /// Render pages to image files
    Render {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output directory (defaults to current directory)
        #[arg(long, short = 'o')]
        output_dir: Option<PathBuf>,
        /// Replace existing page images, but never the input file
        #[arg(long)]
        force: bool,
        /// DPI resolution (default: 150)
        #[arg(long, default_value = "150")]
        dpi: f64,
        /// Render only a specific page (0-based index)
        #[arg(long, conflicts_with = "pages")]
        page: Option<usize>,
        /// One-based page range, such as 1,3-5
        #[arg(long)]
        pages: Option<String>,
        /// Output format: png, jpeg, tiff
        #[arg(long, default_value = "png")]
        format: String,
        /// JPEG quality from 1 through 100
        #[arg(long, default_value = "90")]
        quality: u8,
        /// Preserve unpainted PNG pixels as transparent
        #[arg(long)]
        transparent: bool,
    },
    /// Inspect and mutate Word comment threads
    Comment {
        #[command(subcommand)]
        command: CommentCommand,
    },
    /// Inspect and resolve tracked Word revisions
    Revision {
        #[command(subcommand)]
        command: RevisionCommand,
    },
    /// Create a tracked-changes document from an original and edited file
    Compare {
        /// Original DOCX file
        original: PathBuf,
        /// Edited DOCX file
        edited: PathBuf,
        /// Revision author recorded in the comparison
        #[arg(long)]
        author: String,
        /// RFC 3339 revision timestamp
        #[arg(long)]
        timestamp: String,
        /// Unit of a text change: run, word, or character
        #[arg(
            long,
            value_name = "UNIT",
            default_value = "run",
            value_parser = commands::parse_comparison_granularity
        )]
        granularity: rdocx::ComparisonGranularity,
        /// Keep the original formatting and record no formatting change
        #[arg(long)]
        ignore_formatting: bool,
        /// Keep the original whitespace and record no whitespace-only change
        #[arg(long)]
        ignore_whitespace: bool,
        /// Keep the original field results and record no field change
        #[arg(long)]
        ignore_fields: bool,
        /// Keep the original comments and anchors, dropping the edited ones
        #[arg(long)]
        ignore_comments: bool,
        /// Keep one story of the original, repeatable: body, header, footer,
        /// comment, text_box, footnote, or endnote
        #[arg(
            long = "ignore-story",
            value_name = "KIND",
            value_parser = commands::parse_comparison_story
        )]
        ignore_stories: Vec<rdocx::ComparisonStoryKind>,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Rebuild supported fields
    Toc {
        #[command(subcommand)]
        command: TocCommand,
    },
}

#[derive(Subcommand)]
enum CommentCommand {
    /// List comments in package order
    List {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Add a comment over a zero-based half-open body run range
    Add {
        /// Path to the DOCX file
        file: PathBuf,
        #[command(flatten)]
        range: CommentRangeArgs,
        /// Comment author
        #[arg(long)]
        author: String,
        /// Optional comment author initials
        #[arg(long)]
        initials: Option<String>,
        /// Comment text
        #[arg(long)]
        text: String,
        /// RFC 3339 comment timestamp, omitted from the comment when absent
        #[arg(long)]
        date: Option<String>,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reply to an existing comment
    Reply {
        /// Path to the DOCX file
        file: PathBuf,
        /// Parent comment id
        #[arg(long)]
        id: i32,
        /// Reply author
        #[arg(long)]
        author: String,
        /// Reply text
        #[arg(long)]
        text: String,
        /// RFC 3339 reply timestamp, omitted from the reply when absent
        #[arg(long)]
        date: Option<String>,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mark one comment thread resolved
    Resolve {
        /// Path to the DOCX file
        file: PathBuf,
        /// Comment id
        #[arg(long)]
        id: i32,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove one comment and its replies
    Remove {
        /// Path to the DOCX file
        file: PathBuf,
        /// Comment id
        #[arg(long)]
        id: i32,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct CommentRangeArgs {
    /// Zero-based body paragraph index at the inclusive start
    #[arg(long)]
    start_paragraph: usize,
    /// Zero-based run boundary at the inclusive start, counting the runs that
    /// `text --json` lists
    #[arg(long)]
    start_run: usize,
    /// Zero-based body paragraph index at the exclusive end
    #[arg(long)]
    end_paragraph: usize,
    /// Zero-based run boundary at the exclusive end, counting the runs that
    /// `text --json` lists
    #[arg(long)]
    end_run: usize,
}

#[derive(Subcommand)]
enum RevisionCommand {
    /// List modeled revisions from every supported story
    List {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Accept revisions from every supported story
    Accept {
        /// Path to the DOCX file
        file: PathBuf,
        #[command(flatten)]
        selector: RevisionSelectorArgs,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reject revisions from every supported story
    Reject {
        /// Path to the DOCX file
        file: PathBuf,
        #[command(flatten)]
        selector: RevisionSelectorArgs,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Args)]
struct RevisionSelectorArgs {
    /// Select one shared revision id
    #[arg(long, conflicts_with_all = ["author", "start_date", "end_date"])]
    id: Option<i32>,
    /// Select revisions by exact case-sensitive author
    #[arg(long, conflicts_with_all = ["id", "start_date", "end_date"])]
    author: Option<String>,
    /// Inclusive RFC 3339 lower date bound
    #[arg(long, requires = "end_date", conflicts_with_all = ["id", "author"])]
    start_date: Option<String>,
    /// Inclusive RFC 3339 upper date bound
    #[arg(long, requires = "start_date", conflicts_with_all = ["id", "author"])]
    end_date: Option<String>,
}

#[derive(Subcommand)]
enum TocCommand {
    /// Rebuild supported existing table-of-contents fields
    Rebuild {
        /// Path to the DOCX file
        file: PathBuf,
        /// Output DOCX file
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    // `validate` is the one command whose exit status carries a verdict, so it
    // is dispatched separately from the commands that only report errors.
    if let Command::Validate { file } = &cli.command {
        match commands::validate(file) {
            Ok(true) => return,
            Ok(false) => process::exit(1),
            Err(e) => {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        }
    }

    let result = match cli.command {
        Command::Inspect { file, json } => commands::inspect(&file, json),
        Command::Text { file, json } => commands::text(&file, json),
        Command::Layout { file, json } => commands::layout(&file, json),
        Command::Convert {
            file,
            to,
            output,
            force,
            dpi,
            font_dir,
            pages,
            quality,
            transparent,
        } => commands::convert(
            &file,
            &to,
            output.as_deref(),
            force,
            dpi,
            font_dir.as_deref(),
            commands::ImageOptions {
                pages: pages.as_deref(),
                quality,
                transparent,
            },
        ),
        Command::Diff { file_a, file_b } => commands::diff(&file_a, &file_b),
        Command::Replace {
            file,
            placeholder,
            value,
            output,
            expect,
        } => commands::replace(&file, &placeholder, &value, expect, &output),
        // Handled above so its exit code can reflect the verdict.
        Command::Validate { .. } => unreachable!(),
        Command::Render {
            file,
            output_dir,
            force,
            dpi,
            page,
            pages,
            format,
            quality,
            transparent,
        } => commands::render(
            &file,
            output_dir.as_deref(),
            force,
            dpi,
            commands::RenderOptions {
                page,
                pages: pages.as_deref(),
                format: &format,
                quality,
                transparent,
            },
        ),
        Command::Comment { command } => match command {
            CommentCommand::List { file, json } => commands::comment_list(&file, json),
            CommentCommand::Add {
                file,
                range,
                author,
                initials,
                text,
                date,
                output,
                json,
            } => commands::comment_add(
                &file,
                rdocx::RunRange {
                    start: rdocx::RunPosition {
                        body_index: range.start_paragraph,
                        run_index: range.start_run,
                    },
                    end: rdocx::RunPosition {
                        body_index: range.end_paragraph,
                        run_index: range.end_run,
                    },
                },
                &author,
                initials.as_deref(),
                &text,
                date.as_deref(),
                &output,
                json,
            ),
            CommentCommand::Reply {
                file,
                id,
                author,
                text,
                date,
                output,
                json,
            } => commands::comment_reply(&file, id, &author, &text, date.as_deref(), &output, json),
            CommentCommand::Resolve {
                file,
                id,
                output,
                json,
            } => commands::comment_resolve(&file, id, &output, json),
            CommentCommand::Remove {
                file,
                id,
                output,
                json,
            } => commands::comment_remove(&file, id, &output, json),
        },
        Command::Revision { command } => match command {
            RevisionCommand::List { file, json } => commands::revision_list(&file, json),
            RevisionCommand::Accept {
                file,
                selector,
                output,
                json,
            } => commands::resolve_revisions(
                &file,
                commands::RevisionAction::Accept,
                commands::RevisionSelector {
                    id: selector.id,
                    author: selector.author.as_deref(),
                    start_date: selector.start_date.as_deref(),
                    end_date: selector.end_date.as_deref(),
                },
                &output,
                json,
            ),
            RevisionCommand::Reject {
                file,
                selector,
                output,
                json,
            } => commands::resolve_revisions(
                &file,
                commands::RevisionAction::Reject,
                commands::RevisionSelector {
                    id: selector.id,
                    author: selector.author.as_deref(),
                    start_date: selector.start_date.as_deref(),
                    end_date: selector.end_date.as_deref(),
                },
                &output,
                json,
            ),
        },
        Command::Compare {
            original,
            edited,
            author,
            timestamp,
            granularity,
            ignore_formatting,
            ignore_whitespace,
            ignore_fields,
            ignore_comments,
            ignore_stories,
            output,
            json,
        } => commands::compare(
            &original,
            &edited,
            &author,
            &timestamp,
            &rdocx::ComparisonOptions {
                granularity,
                ignore_formatting,
                ignore_whitespace,
                ignore_fields,
                ignore_comments,
                ignored_stories: ignore_stories,
            },
            &output,
            json,
        ),
        Command::Toc { command } => match command {
            TocCommand::Rebuild { file, output, json } => {
                commands::toc_rebuild(&file, &output, json)
            }
        },
    };

    // Standard output is line buffered, so a last line without a newline is
    // only written, and can only fail, when it is flushed.
    let result = result.and_then(|()| io::stdout().flush().map_err(Into::into));
    if let Err(e) = result {
        // A reader that closes standard output early, as `| head` does, ends
        // the output. That is not a failure of the command.
        let closed_stdout = e
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::BrokenPipe);
        if !closed_stdout {
            eprintln!("Error: {e}");
            process::exit(1);
        }
    }
}
