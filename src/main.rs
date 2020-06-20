//! Crate to quickly navigate to a crate's published links from the terminal.
//!
//! The links you can open your browser to are the homepage, documentation,
//! and repository links that show, when set, on crates.io pages.

#![deny(clippy::all)]

use anyhow::{anyhow, Result};
use fern::{
    colors::{Color, ColoredLevelConfig},
    Dispatch,
};
use log::{debug, error, info, LevelFilter};
use serde::Deserialize;
use std::{fmt, io, process};
use structopt::{clap::arg_enum, StructOpt};

arg_enum! {
    /// Destination options.
    ///
    /// The single-letter options are provided as good
    /// UX shorthand for the CLI.
    #[derive(Debug)]
    enum Destination {
        H, Homepage,
        D, Documentation,
        R, Repository,
    }
}

/// TODO docs
#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    debug: bool,

    crate_name: String,

    #[structopt(possible_values = &Destination::variants(), case_insensitive = true, default_value = "h")]
    destination: Destination,
}

/// Crate info JSON struct.
#[derive(Debug, Deserialize)]
struct CrateInfo {
    name: String,
    homepage: Option<String>,
    documentation: Option<String>,
    repository: Option<String>,
}

impl fmt::Display for CrateInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let pairs = vec![
            ("Homepage", &self.homepage),
            ("Documentation", &self.documentation),
            ("Repository", &self.repository),
        ];
        write!(f, "Crate {}", self.name)?;
        let trimmed: Vec<_> = pairs
            .iter()
            .filter(|(_, link)| link.is_some())
            .map(|(label, link_opt)| (label, link_opt.as_ref().unwrap()))
            .collect();
        for (label, link) in trimmed {
            write!(f, ", {}: {}", label, link)?;
        }
        if self.homepage.is_none() && self.documentation.is_none() && self.repository.is_none() {
            write!(f, ", No links provided")?;
        }
        Ok(())
    }
}

/// Top-level crates.io API response data.
#[derive(Debug, Deserialize)]
struct CrateInfoWrapper {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

/// Set up logging based on whether or not the user wants to see debug logging.
fn setup_logging(debug: bool) -> Result<()> {
    let base_config = if debug {
        Dispatch::new().level(LevelFilter::Debug)
    } else {
        Dispatch::new().level(LevelFilter::Info)
    };
    let colors = ColoredLevelConfig::new().error(Color::Red);
    let stdout_config = Dispatch::new()
        .format(move |out, message, record| {
            if record.level() == LevelFilter::Info {
                out.finish(format_args!("{}", message))
            } else {
                out.finish(format_args!("{} {}", colors.color(record.level()), message))
            }
        })
        .chain(io::stdout());
    base_config.chain(stdout_config).apply()?;
    Ok(())
}

/// Get info from a crate from the crates.io API.
fn get_crate_info(crate_name: &str) -> Result<CrateInfo> {
    let resp = reqwest::blocking::get(&format!("https://crates.io/api/v1/crates/{}", crate_name))?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "Got bad status {} from crates.io API",
            resp.status()
        ));
    }
    let data: CrateInfoWrapper = resp.json()?;
    Ok(data.crate_info)
}

/// Open the requested link from the crate info, as long as it's set.
fn open_link(info: &CrateInfo, destination: &Destination) -> Result<()> {
    let pair = match destination {
        Destination::H | Destination::Homepage => ("homepage", &info.homepage),
        Destination::D | Destination::Documentation => ("documentation", &info.documentation),
        Destination::R | Destination::Repository => ("repository", &info.repository),
    };
    let url = match pair.1 {
        Some(u) => u,
        None => {
            error!("The {} link isn't set for that crate", pair.0);
            info!("Here is the info that was found: {}", info);
            process::exit(1);
        }
    };
    webbrowser::open(url)?;
    Ok(())
}

/// Entrypoint.
fn main() {
    let opt = Options::from_args();
    if let Err(e) = setup_logging(opt.debug) {
        eprintln!("Error setting up: {}", e);
        process::exit(1)
    }
    debug!("CLI options: {:?}", opt);
    let info = match get_crate_info(&opt.crate_name) {
        Ok(i) => {
            debug!("API info: {:?}", i);
            i
        }
        Err(e) => {
            debug!("Error getting crate info: {}", e);
            error!(
                r#"Could not find crate information for "{}""#,
                opt.crate_name
            );
            process::exit(1);
        }
    };
    if let Err(e) = open_link(&info, &opt.destination) {
        debug!("Error opening link: {}", e);
        error!("Could not open the link");
        process::exit(1);
    };
}
