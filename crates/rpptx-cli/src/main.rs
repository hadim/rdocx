//! Command-line access to the public `rpptx` facade.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand, ValueEnum};

mod commands;

#[derive(Parser)]
#[command(name = "rpptx", version, about = "CLI tool for PPTX files")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print presentation structure and metadata
    Inspect {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Extract slide text in presentation order
    Text {
        file: PathBuf,
        /// Output paragraphs, runs, and speaker notes as schema-1 JSON
        #[arg(long)]
        json: bool,
        /// Print speaker notes after each slide's text (JSON always includes them)
        #[arg(long)]
        notes: bool,
    },
    /// Convert a presentation to deterministic PDF or image output
    Convert {
        file: PathBuf,
        #[arg(long, short = 't')]
        to: String,
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Replace existing output files, but never the input file
        #[arg(long)]
        force: bool,
        #[arg(long, default_value = "150")]
        dpi: f64,
        /// One-based slide range for image output, such as 1,3-5
        #[arg(long)]
        slides: Option<String>,
        /// JPEG quality from 1 through 100
        #[arg(long, default_value = "90")]
        quality: u8,
        /// Preserve unpainted PNG pixels as transparent
        #[arg(long)]
        transparent: bool,
    },
    /// Compare slide text using a longest-common-subsequence diff
    Diff { file_a: PathBuf, file_b: PathBuf },
    /// Replace literal presentation text while retaining run formatting
    Replace {
        file: PathBuf,
        #[arg(long, short = 'p')]
        placeholder: String,
        #[arg(long, short = 'v')]
        value: String,
        /// Require exactly this many slide and speaker-note replacements
        #[arg(long)]
        expect: Option<usize>,
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
    /// Validate package and PresentationML invariants
    Validate { file: PathBuf },
    /// Render selected slides to deterministic image files
    Render {
        file: PathBuf,
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Replace existing slide images, but never the input file
        #[arg(long)]
        force: bool,
        #[arg(long, default_value = "150")]
        dpi: f64,
        #[arg(long)]
        slide: Option<String>,
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
    /// Render slide one as a proportional 320-pixel-wide PNG
    Thumbnail {
        file: PathBuf,
        #[arg(long, short = 'o')]
        output: Option<PathBuf>,
        /// Replace an existing output file, but never the input file
        #[arg(long)]
        force: bool,
    },
    /// Print each slide title and recursive paragraph outline
    Outline {
        file: PathBuf,
        /// Output titles, outline items, and speaker notes as schema-1 JSON
        #[arg(long)]
        json: bool,
        /// Print speaker notes after each slide's outline (JSON always includes them)
        #[arg(long)]
        notes: bool,
    },
    /// Inspect and mutate modern PowerPoint comment threads
    Comment {
        #[command(subcommand)]
        command: CommentCommand,
    },
    /// List, embed, and remove embedded fonts
    Font {
        #[command(subcommand)]
        command: FontCommand,
    },
}

#[derive(Subcommand)]
enum FontCommand {
    /// List embedded fonts and their faces
    List {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Embed one face of a TrueType or OpenType font file
    ///
    /// The embedded font entry is created when it does not exist, and the
    /// presentation's embedTrueTypeFonts setting is turned on, which
    /// PowerPoint requires before it uses embedded fonts.
    Embed {
        file: PathBuf,
        /// TrueType or OpenType file holding one face
        font_file: PathBuf,
        /// Typeface name slides use, normally the font's family name
        #[arg(long)]
        name: String,
        /// Face to embed: regular, bold, italic, or bold_italic
        #[arg(long, value_name = "FACE", value_enum, default_value = "regular")]
        face: FontFace,
        /// Confirm that the font license allows embedding it in the presentation
        #[arg(long)]
        license_authorized: bool,
        /// Exact license identity, such as "OFL-1.1: Example Sans"
        #[arg(long)]
        license_identity: String,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove one embedded face, or the whole embedded font without --face
    Remove {
        file: PathBuf,
        /// Typeface name of the embedded font
        #[arg(long)]
        name: String,
        /// Embedded face to remove: regular, bold, italic, or bold_italic
        #[arg(long, value_name = "FACE", value_enum)]
        face: Option<FontFace>,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Set the font embedding settings of the presentation
    Settings {
        file: PathBuf,
        /// Use and keep the embedded TrueType fonts: true or false
        #[arg(long, value_name = "BOOL")]
        embed_true_type_fonts: Option<bool>,
        /// Embed only the characters in use when PowerPoint saves: true or false
        #[arg(long, value_name = "BOOL")]
        save_subset_fonts: Option<bool>,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum FontFace {
    Regular,
    Bold,
    Italic,
    #[value(name = "bold_italic")]
    BoldItalic,
}

impl From<FontFace> for rpptx::EmbeddedFontKind {
    fn from(face: FontFace) -> Self {
        match face {
            FontFace::Regular => Self::Regular,
            FontFace::Bold => Self::Bold,
            FontFace::Italic => Self::Italic,
            FontFace::BoldItalic => Self::BoldItalic,
        }
    }
}

#[derive(Subcommand)]
enum CommentCommand {
    /// List modern comments and their replies in slide order
    List {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Add a comment to one slide
    Add {
        file: PathBuf,
        /// One-based slide number
        #[arg(long)]
        slide: usize,
        /// Comment author, reused by name or added to the author list
        #[arg(long)]
        author: String,
        /// Initials recorded when the author is added
        #[arg(long)]
        initials: Option<String>,
        #[arg(long)]
        text: String,
        /// RFC 3339 creation timestamp
        #[arg(long)]
        date: String,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Reply to an existing comment thread
    Reply {
        file: PathBuf,
        /// Comment id of the thread
        #[arg(long)]
        id: String,
        /// Reply author, reused by name or added to the author list
        #[arg(long)]
        author: String,
        #[arg(long)]
        text: String,
        /// RFC 3339 creation timestamp
        #[arg(long)]
        date: String,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mark one comment thread resolved
    Resolve {
        file: PathBuf,
        /// Comment id of the thread
        #[arg(long)]
        id: String,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove one comment with its replies, or one reply
    Remove {
        file: PathBuf,
        /// Comment or reply id
        #[arg(long)]
        id: String,
        #[arg(long, short = 'o')]
        output: PathBuf,
        /// Output the operation record as JSON
        #[arg(long)]
        json: bool,
    },
}

fn main() {
    #[cfg(windows)]
    {
        // A full validation can exceed the one MiB Windows main-thread stack.
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(run_cli)
            .expect("start rpptx CLI thread")
            .join()
            .expect("rpptx CLI thread panicked");
    }
    #[cfg(not(windows))]
    run_cli();
}

fn run_cli() {
    let cli = Cli::parse();
    if let Command::Validate { file } = &cli.command {
        match commands::validate(file) {
            Ok(true) => return,
            Ok(false) => process::exit(1),
            Err(error) => {
                eprintln!("Error: {error}");
                process::exit(1);
            }
        }
    }

    let result = match cli.command {
        Command::Inspect { file, json } => commands::inspect(&file, json),
        Command::Text { file, json, notes } => commands::text(&file, json, notes),
        Command::Convert {
            file,
            to,
            output,
            force,
            dpi,
            slides,
            quality,
            transparent,
        } => commands::convert(
            &file,
            &to,
            output.as_deref(),
            force,
            dpi,
            commands::ImageOptions {
                slides: slides.as_deref(),
                quality,
                transparent,
            },
        ),
        Command::Diff { file_a, file_b } => commands::diff(&file_a, &file_b),
        Command::Replace {
            file,
            placeholder,
            value,
            expect,
            output,
        } => commands::replace(&file, &placeholder, &value, expect, &output),
        Command::Validate { .. } => unreachable!("validate is dispatched above"),
        Command::Render {
            file,
            output,
            force,
            dpi,
            slide,
            format,
            quality,
            transparent,
        } => commands::render(
            &file,
            output.as_deref(),
            force,
            dpi,
            &format,
            commands::ImageOptions {
                slides: slide.as_deref(),
                quality,
                transparent,
            },
        ),
        Command::Thumbnail {
            file,
            output,
            force,
        } => commands::thumbnail(&file, output.as_deref(), force),
        Command::Outline { file, json, notes } => commands::outline(&file, json, notes),
        Command::Comment { command } => match command {
            CommentCommand::List { file, json } => commands::comment_list(&file, json),
            CommentCommand::Add {
                file,
                slide,
                author,
                initials,
                text,
                date,
                output,
                json,
            } => commands::comment_add(
                &file,
                slide,
                commands::CommentInput {
                    author: &author,
                    initials: initials.as_deref(),
                    text: &text,
                    date: &date,
                },
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
            } => commands::comment_reply(
                &file,
                &id,
                commands::CommentInput {
                    author: &author,
                    initials: None,
                    text: &text,
                    date: &date,
                },
                &output,
                json,
            ),
            CommentCommand::Resolve {
                file,
                id,
                output,
                json,
            } => commands::comment_resolve(&file, &id, &output, json),
            CommentCommand::Remove {
                file,
                id,
                output,
                json,
            } => commands::comment_remove(&file, &id, &output, json),
        },
        Command::Font { command } => match command {
            FontCommand::List { file, json } => commands::font_list(&file, json),
            FontCommand::Embed {
                file,
                font_file,
                name,
                face,
                license_authorized,
                license_identity,
                output,
                json,
            } => commands::font_embed(
                &file,
                commands::FontInput {
                    font_file: &font_file,
                    name: &name,
                    kind: face.into(),
                    license_identity: &license_identity,
                    license_authorized,
                },
                &output,
                json,
            ),
            FontCommand::Remove {
                file,
                name,
                face,
                output,
                json,
            } => commands::font_remove(&file, &name, face.map(Into::into), &output, json),
            FontCommand::Settings {
                file,
                embed_true_type_fonts,
                save_subset_fonts,
                output,
                json,
            } => commands::font_settings(
                &file,
                embed_true_type_fonts,
                save_subset_fonts,
                &output,
                json,
            ),
        },
    };
    // Standard output is line buffered, so a last line without a newline is
    // only written, and can only fail, when it is flushed.
    let result = result.and_then(|()| io::stdout().flush().map_err(Into::into));
    if let Err(error) = result {
        // A reader that closes standard output early, as `| head` does, ends
        // the output. That is not a failure of the command.
        let closed_stdout = error
            .downcast_ref::<io::Error>()
            .is_some_and(|error| error.kind() == io::ErrorKind::BrokenPipe);
        if !closed_stdout {
            eprintln!("Error: {error}");
            process::exit(1);
        }
    }
}
