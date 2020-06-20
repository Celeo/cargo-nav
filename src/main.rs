#![deny(clippy::all)]

use anyhow::{anyhow, Result};
use log::{debug, error, info, LevelFilter};
use serde::Deserialize;
use std::{fmt, io, process};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Options {
    #[structopt(short, long)]
    debug: bool,
    crate_name: String,
}

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

#[derive(Debug, Deserialize)]
struct CrateInfoWrapper {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

fn setup_logging(debug: bool) -> Result<()> {
    let base_config = if debug {
        fern::Dispatch::new().level(LevelFilter::Debug)
    } else {
        fern::Dispatch::new().level(LevelFilter::Info)
    };
    let stdout_config = fern::Dispatch::new()
        .format(|out, message, record| {
            if record.level() == LevelFilter::Info {
                out.finish(format_args!("{}", message))
            } else {
                out.finish(format_args!("[{}] {}", record.level(), message))
            }
        })
        .chain(io::stdout());
    base_config.chain(stdout_config).apply()?;
    Ok(())
}

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

fn main() {
    let opt = Options::from_args();
    if let Err(e) = setup_logging(opt.debug) {
        eprintln!("Error setting up: {}", e);
        process::exit(1)
    }
    let info = match get_crate_info(&opt.crate_name) {
        Ok(i) => {
            debug!("API info: {:?}", i);
            i
        }
        Err(e) => {
            debug!("{}", e);
            error!(
                r#"Could not find crate information for "{}""#,
                opt.crate_name
            );
            process::exit(1);
        }
    };

    info!("{}", info);
}
