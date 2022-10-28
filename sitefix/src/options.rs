use anyhow::{bail, Result};
use clap::Parser;
use std::{env, path::PathBuf};
use twelf::config;

use crate::logging::{LogLevel, Logger};

#[config]
#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct SitefixInboundConfig {
    #[clap(long, short, help = "The location of your built static website")]
    #[clap(required = false)]
    #[serde(default)] // This is actually required, but we validate that later
    pub source: String,

    #[clap(
        long,
        help = "The file glob Sitefix uses to find HTML files. Defaults to \"**/*.{html}\""
    )]
    #[clap(required = false)]
    #[serde(default = "defaults::default_glob")]
    pub glob: String,

    #[clap(
        long,
        help = "The element Sitefix should treat as the root of the document."
    )]
    #[clap(required = false)]
    #[serde(default = "defaults::default_root_selector")]
    pub root_selector: String,

    #[clap(long, short, help = "Print verbose logging while reviewing the site.")]
    #[clap(required = false)]
    #[serde(default = "defaults::default_false")]
    pub verbose: bool,
}

mod defaults {
    pub fn default_glob() -> String {
        "**/*.{html}".into()
    }
    pub fn default_root_selector() -> String {
        "html".into()
    }
    pub fn default_false() -> bool {
        false
    }
}

// The configuration object used internally
#[derive(Debug)]
pub struct FixOptions {
    pub working_directory: PathBuf,
    pub source: PathBuf,
    pub root_selector: String,
    pub glob: String,
    pub version: &'static str,
    pub logger: Logger,
}

impl FixOptions {
    pub fn load(config: SitefixInboundConfig) -> Result<Self> {
        if config.source.is_empty() {
            eprintln!("Required argument source not supplied. Sitefix needs to know the root of your built static site.");
            eprintln!("Provide a --source flag, a SITEFIX_SOURCE environment variable, or a source key in a Sitefix configuration file.");
            bail!("Missing argument: source");
        } else {
            let log_level = if config.verbose {
                LogLevel::Verbose
            } else {
                LogLevel::Standard
            };

            Ok(Self {
                working_directory: env::current_dir().unwrap(),
                source: PathBuf::from(config.source),
                root_selector: config.root_selector,
                glob: config.glob,
                version: env!("CARGO_PKG_VERSION"),
                logger: Logger::new(log_level),
            })
        }
    }
}
