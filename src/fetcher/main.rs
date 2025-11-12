// This is free and unencumbered software released into the public domain.

#[cfg(not(feature = "std"))]
compile_error!("asimov-http-fetcher requires the 'std' feature");

use asimov_module::SysexitsError::{self, *};
use clap::Parser;
use clientele::StandardOptions;
use know::traits::ToJsonLd;
use std::{error::Error, io::Write as _};

/// asimov-http-fetcher
#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
struct Options {
    #[clap(flatten)]
    flags: StandardOptions,

    /// The output format.
    #[arg(value_name = "FORMAT", short = 'o', long, default_value_t, value_enum)]
    output: OutputFormat,

    /// The `http:` or `https:` URLs to fetch
    urls: Vec<String>,
}

#[derive(Clone, Debug, Default, clap::ValueEnum)]
enum OutputFormat {
    #[default]
    Cli,
    Jsonl,
    Jsonld,
    Json,
}

fn main() -> Result<SysexitsError, Box<dyn Error>> {
    // Load environment variables from `.env`:
    asimov_module::dotenv().ok();

    // Expand wildcards and @argfiles:
    let args = asimov_module::args_os()?;

    // Parse command-line options:
    let options = Options::parse_from(args);

    // Handle the `--version` flag:
    if options.flags.version {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(EX_OK);
    }

    // Handle the `--license` flag:
    if options.flags.license {
        print!("{}", include_str!("../../UNLICENSE"));
        return Ok(EX_OK);
    }

    // Configure logging & tracing:
    #[cfg(feature = "tracing")]
    asimov_module::init_tracing_subscriber(&options.flags).expect("failed to initialize logging");

    let mut output = std::io::stdout().lock();

    for url in options.urls {
        let mut input = asimov_http_module::open(&url)?;

        match options.output {
            OutputFormat::Jsonl | OutputFormat::Jsonld | OutputFormat::Json => {
                let mut data = Vec::new();
                input.read_to_end(&mut data)?;
                let file = know::classes::File {
                    id: Some(url),
                    name: None,
                    size: data.len() as _,
                    data,
                };
                writeln!(&mut output, "{}", file.to_jsonld()?)?;
            },
            OutputFormat::Cli => {
                std::io::copy(&mut input, &mut output)?;
            },
        }
    }

    Ok(EX_OK)
}
