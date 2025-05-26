//! Crate to quickly navigate to a crate's published links from the terminal.
//!
//! The links you can open your browser to are the homepage, documentation,
//! and repository links that show, when set, on crates.io pages.

#![deny(unsafe_code)]
#![deny(clippy::all)]

use anyhow::{Result, anyhow};
use fern::Dispatch;
use log::{LevelFilter, debug, error, info};
use serde::Deserialize;
use std::{env, fmt, io, process};
use structopt::{StructOpt, clap::arg_enum};

arg_enum! {
    /// Destination options.
    ///
    /// The single-letter options are provided as good
    /// UX shorthand for the CLI.
    #[derive(Debug)]
    enum Destination {
        C, Crate,
        H, Homepage,
        D, Documentation,
        R, Repository,
    }
}

/// CLI program for quickly navigating to crate links as found on crates.io.
///
/// Call with: cargo nav <crate-name> [destination]
///
/// The 'destination' argument is one of several options, shown below. The single-
/// letter versions are shorthand for less typing.
#[derive(Debug, StructOpt)]
#[structopt(name = "cargo-nav")]
struct Options {
    /// Enable debug logging.
    #[structopt(short, long)]
    debug: bool,

    /// Name of the crate to look up.
    crate_name: String,

    /// Type of link to open.
    #[structopt(possible_values = &Destination::variants(), case_insensitive = true, default_value = "c")]
    destination: Destination,
}

/// Crate info JSON struct.
#[derive(Clone, Debug, Deserialize)]
struct CrateInfo {
    name: String,
    homepage: Option<String>,
    documentation: Option<String>,
    repository: Option<String>,
}

impl fmt::Display for CrateInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.homepage.is_none() && self.documentation.is_none() && self.repository.is_none() {
            return write!(
                f,
                "no links found for crate '{name}'; check https://crates.io/crates/{name}",
                name = self.name,
            );
        }
        let pairs = [
            ("Homepage", &self.homepage),
            ("Documentation", &self.documentation),
            ("Repository", &self.repository),
        ];
        let buffer = pairs
            .iter()
            .filter(|(_, link)| link.is_some())
            .map(|(label, link)| format!("{label}: {}", link.as_ref().unwrap()))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{buffer}")
    }
}

/// Top-level crates.io API response data.
#[derive(Debug, Deserialize)]
struct CrateInfoWrapper {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

/// Set up logging based on whether or not the user wants debug logging.
fn setup_logging(debug: bool) -> Result<()> {
    let base_config = if debug {
        Dispatch::new()
            .level(LevelFilter::Debug)
            .level_for("hyper::proto", LevelFilter::Info)
    } else {
        Dispatch::new().level(LevelFilter::Info)
    };
    let stdout_config = Dispatch::new()
        .format(move |out, message, record| {
            if record.level() == LevelFilter::Info {
                out.finish(format_args!("{message}"))
            } else {
                out.finish(format_args!(
                    "[{}] {} {message}",
                    record.target(),
                    record.level(),
                ))
            }
        })
        .chain(io::stdout());
    base_config.chain(stdout_config).apply()?;
    Ok(())
}

fn get_api_url() -> String {
    #[cfg(not(test))]
    return String::from("https://crates.io/api/v1/crates");
    #[cfg(test)]
    return mockito::server_url();
}

/// Get info from a crate from the crates.io API.
fn get_crate_info(crate_name: &str) -> Result<CrateInfo> {
    debug!("Requesting crate info from crates.io API");
    let client = reqwest::blocking::Client::builder()
        .user_agent("cargo-nav (https://github.com/celeo/cargo-nav)")
        .build()?;
    let resp = client
        .get(format!("{}/{crate_name}", get_api_url()))
        .send()?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "Got bad status {} from crates.io API",
            resp.status()
        ));
    }
    let data: CrateInfoWrapper = resp.json()?;
    Ok(data.crate_info)
}

/// Determine which URL to open.
fn determine_link(info: &CrateInfo, destination: &Destination) -> Result<String> {
    let crate_url = Some(format!("https://crates.io/crates/{}", info.name));
    let pair = match destination {
        Destination::C | Destination::Crate => ("crate", &crate_url),
        Destination::H | Destination::Homepage => ("homepage", &info.homepage),
        Destination::D | Destination::Documentation => ("documentation", &info.documentation),
        Destination::R | Destination::Repository => ("repository", &info.repository),
    };
    match pair.1 {
        Some(u) => Ok(u.to_owned()),
        None => Err(anyhow!("The {} link isn't set for that crate", pair.0)),
    }
}

/// Entrypoint.
fn main() {
    // conditionally skip 1 to provide running through both 'cargo nav' and 'cargo-nav'
    let args: Vec<_> = env::args().collect();
    let args = if args.len() > 1 && args[1] == "nav" {
        args.iter().skip(1).cloned().collect::<Vec<_>>()
    } else {
        args
    };

    let opt = Options::from_iter(args.iter());
    if let Err(e) = setup_logging(opt.debug) {
        eprintln!("Error setting up: {e}");
        process::exit(1);
    }
    let info = match get_crate_info(&opt.crate_name) {
        Ok(i) => {
            debug!("API info: {i:?}");
            i
        }
        Err(e) => {
            debug!("Error getting crate info: {e}");
            error!(
                r#"Could not find crate information for "{}""#,
                opt.crate_name
            );
            process::exit(1);
        }
    };
    let url = match determine_link(&info, &opt.destination) {
        Ok(u) => u,
        Err(e) => {
            error!("Error determining link: {e}");
            info!("Here is the info that was found: {info}");
            process::exit(1);
        }
    };
    debug!("URL to open: {url}");
    if let Err(e) = webbrowser::open(&url) {
        debug!("Error opening link: {e}");
        error!("Could not open the link");
        process::exit(1);
    };
}

#[cfg(test)]
mod tests {
    use super::{CrateInfo, Destination, determine_link, get_crate_info};
    use mockito::mock;

    fn crate_info() -> CrateInfo {
        CrateInfo {
            name: "a".to_owned(),
            homepage: Some("b".to_owned()),
            documentation: Some("c".to_owned()),
            repository: None,
        }
    }

    #[test]
    fn test_determine_link_short() {
        let url = determine_link(&crate_info(), &Destination::D).unwrap();
        assert_eq!(url, "c");
    }

    #[test]
    fn test_determine_link_long() {
        let url = determine_link(&crate_info(), &Destination::Homepage).unwrap();
        assert_eq!(url, "b");
    }

    #[test]
    fn determine_link_missing() {
        let result = determine_link(&crate_info(), &Destination::Repository);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_crate_info_just_name() {
        let m = mock("GET", "/a")
            .with_body(r#"{"crate":{"name":"a"}}"#)
            .create();
        let info = get_crate_info("a").unwrap();
        assert_eq!(info.name, "a");
        assert_eq!(info.homepage, None);
        assert_eq!(info.documentation, None);
        assert_eq!(info.repository, None);
        m.assert();
    }

    #[test]
    fn test_get_crate_info_all() {
        let m = mock("GET", "/a")
            .with_body(
                r#"{"crate":{"name":"a","homepage":"b","documentation":"c","repository":"d","other":"info"}}"#,
            )
            .create();
        let info = get_crate_info("a").unwrap();
        assert_eq!(info.name, "a");
        assert_eq!(info.homepage, Some("b".to_owned()));
        assert_eq!(info.documentation, Some("c".to_owned()));
        assert_eq!(info.repository, Some("d".to_owned()));
        m.assert();
    }

    #[test]
    fn test_get_crate_info_not_found() {
        let result = get_crate_info("b");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_crate_info_some() {
        let s = format!("{}", crate_info());
        assert_eq!(s, "Homepage: b, Documentation: c");
    }

    #[test]
    fn test_get_crate_info_none() {
        let info = CrateInfo {
            name: "a".to_owned(),
            homepage: None,
            documentation: None,
            repository: None,
        };
        let s = format!("{}", info);
        assert_eq!(
            s,
            "no links found for crate 'a'; check https://crates.io/crates/a"
        );
    }
}
