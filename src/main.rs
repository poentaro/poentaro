use clap::{Parser, Subcommand};
use poentaro::cli::generate::{
    generate_mei, generate_midi, generate_mp3, generate_pdf, generate_svg, generate_wav,
};
use poentaro::cli::{check, GenerateInput};
use poentaro::{self as jg, error_reporter};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "poentaro", about = "Generate numbered musical notation files")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Generate {
        #[command(subcommand)]
        format: GenerateFormat,
    },
    Check {
        input: PathBuf,
    },
    /// Format each file in place: anyfix-expr's layout first, then the score.
    Format {
        paths: Vec<PathBuf>,
    },
}

#[derive(Subcommand)]
enum GenerateFormat {
    Pdf {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Export with this preset's metadata and visible parts (--tracks overrides its parts)"
        )]
        preset: Option<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Svg {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Export with this preset's metadata and visible parts (--tracks overrides its parts)"
        )]
        preset: Option<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    /// Experimental: MEI for engraving as western staff notation with Verovio.
    Mei {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Export with this preset's metadata and visible parts (--tracks overrides its parts)"
        )]
        preset: Option<String>,
    },
    Midi {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Export with this preset's metadata and visible parts (--tracks overrides its parts)"
        )]
        preset: Option<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Wav {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Export with this preset's metadata and visible parts (--tracks overrides its parts)"
        )]
        preset: Option<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
    Mp3 {
        input: PathBuf,
        #[arg(long, help = "Output file stem (extension is added automatically)")]
        output: Option<PathBuf>,
        #[arg(long, value_delimiter = ',', num_args = 0.., help = "Comma-separated list of track names to include (e.g. --tracks S1,S2)")]
        tracks: Vec<String>,
        #[arg(
            long,
            help = "Export with this preset's metadata and visible parts (--tracks overrides its parts)"
        )]
        preset: Option<String>,
        #[arg(
            long,
            help = "Generate one file per track instead of a single combined file"
        )]
        split_tracks: bool,
    },
}

fn main() -> ExitCode {
    let args = Args::parse();

    let result = match args.command {
        Commands::Generate { format } => run_generate(format).map(|()| true),
        Commands::Check { input } => run_check(&input),
        Commands::Format { paths } => paths
            .iter()
            .try_for_each(|path| poentaro::cli::format_file(path))
            .map(|()| true),
    };

    match result {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(e) => {
            error_reporter::render(&e);
            ExitCode::FAILURE
        }
    }
}

/// Returns `Ok(true)` when the file parses with no errors, `Ok(false)` when it
/// parses but has recoverable errors.
fn run_check(input: &std::path::Path) -> Result<bool, jg::error::IrrecoverableError> {
    let outcome = check(input)?;
    for diagnostic in &outcome.diagnostics {
        eprintln!("{}: {}", input.display(), diagnostic.message());
    }
    if outcome.ok {
        println!("{input:?}: ok");
    }
    Ok(outcome.ok)
}

fn run_generate(format: GenerateFormat) -> Result<(), jg::error::IrrecoverableError> {
    match format {
        GenerateFormat::Pdf {
            input,
            output,
            tracks,
            preset,
            split_tracks,
        } => generate_pdf(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
            preset,
        }),
        GenerateFormat::Svg {
            input,
            output,
            tracks,
            preset,
            split_tracks,
        } => generate_svg(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
            preset,
        }),
        GenerateFormat::Mei {
            input,
            output,
            tracks,
            preset,
        } => generate_mei(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks: false,
            preset,
        }),
        GenerateFormat::Midi {
            input,
            output,
            tracks,
            preset,
            split_tracks,
        } => generate_midi(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
            preset,
        }),
        GenerateFormat::Wav {
            input,
            output,
            tracks,
            preset,
            split_tracks,
        } => generate_wav(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
            preset,
        }),
        GenerateFormat::Mp3 {
            input,
            output,
            tracks,
            preset,
            split_tracks,
        } => generate_mp3(&GenerateInput {
            input,
            output,
            tracks,
            split_tracks,
            preset,
        }),
    }
}
