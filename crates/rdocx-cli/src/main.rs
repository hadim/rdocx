//! rdocx CLI — "jq for DOCX"
//!
//! Inspect, convert, diff, and manipulate DOCX files from the command line.

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
    /// Zero-based run boundary at the inclusive start
    #[arg(long)]
    start_run: usize,
    /// Zero-based body paragraph index at the exclusive end
    #[arg(long)]
    end_paragraph: usize,
    /// Zero-based run boundary at the exclusive end
    #[arg(long)]
    end_run: usize,
}

#[derive(Subcommand)]
enum RevisionCommand {
    /// List modeled revisions from the main story
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
        Command::Text { file } => commands::text(&file),
        Command::Convert {
            file,
            to,
            output,
            dpi,
            font_dir,
            pages,
            quality,
            transparent,
        } => commands::convert(
            &file,
            &to,
            output.as_deref(),
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
        } => commands::replace(&file, &placeholder, &value, &output),
        // Handled above so its exit code can reflect the verdict.
        Command::Validate { .. } => unreachable!(),
        Command::Render {
            file,
            output_dir,
            dpi,
            page,
            pages,
            format,
            quality,
            transparent,
        } => commands::render(
            &file,
            output_dir.as_deref(),
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
                &output,
                json,
            ),
            CommentCommand::Reply {
                file,
                id,
                author,
                text,
                output,
                json,
            } => commands::comment_reply(&file, id, &author, &text, &output, json),
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
            output,
            json,
        } => commands::compare(&original, &edited, &author, &timestamp, &output, json),
        Command::Toc { command } => match command {
            TocCommand::Rebuild { file, output, json } => {
                commands::toc_rebuild(&file, &output, json)
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        process::exit(1);
    }
}
