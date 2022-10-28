use std::path::PathBuf;

use fossick::Fossicker;
use futures::future::join_all;
pub use issues::SitefixIssue;
pub use options::{FixOptions, SitefixInboundConfig};
use wax::{Glob, WalkEntry};

mod fossick;
#[macro_use]
mod logging;
mod issues;
mod options;

pub struct FixState {
    pub options: FixOptions,
}

#[derive(Debug, Default)]
pub struct Globals {
    pub urls: Vec<String>,
    pub paths: Vec<PathBuf>,
}

impl FixState {
    pub fn new(options: FixOptions) -> Self {
        Self { options }
    }

    pub async fn walk_for_files(&mut self) -> Vec<Fossicker> {
        let log = &self.options.logger;

        log.status("[Walking source directory]");
        if let Ok(glob) = Glob::new(&self.options.glob) {
            glob.walk(&self.options.source)
                .filter_map(Result::ok)
                .map(WalkEntry::into_path)
                .map(|e| Fossicker::new(e, &self.options))
                .collect()
        } else {
            log.error(format!(
                "Error: Provided glob \"{}\" did not parse as a valid glob.",
                self.options.glob
            ));
            std::process::exit(1);
        }
    }

    pub async fn run(&mut self) {
        let log = &self.options.logger;
        log.status(&format!("Running Sitefix v{}", self.options.version));
        log.v_info("Running in verbose mode");

        log.info(format!(
            "Running from: {:?}",
            self.options.working_directory
        ));
        log.info(format!("Source:       {:?}", self.options.source));

        let files = self.walk_for_files().await;
        let log = &self.options.logger;

        log.info(format!(
            "Found {} file{} matching {}",
            files.len(),
            plural!(files.len()),
            self.options.glob
        ));
        log.status("[Parsing files]");

        let globals = Globals {
            urls: files.iter().flat_map(|f| f.urls.clone()).collect(),
            paths: files.iter().map(|f| f.file_path.clone()).collect(),
        };

        let results: Vec<_> = files
            .into_iter()
            .map(|f| f.fossick(&globals, &self.options))
            .collect();
        let all_pages = join_all(results).await;

        if self.options.root_selector == "html" {
            let pages_without_html = all_pages
                .iter()
                .flatten()
                .filter(|p| !p.has_html_element)
                .map(|p| format!("  * {:?} has no <html> element", p.file_path))
                .collect::<Vec<_>>();
            if !pages_without_html.is_empty() {
                log.warn(format!(
                    "{} page{} found without an <html> element. \n\
                    Pages without an outer <html> element will not be processed by default. \n\
                    If adding this element is not possible, use the root selector config to target a different root element.",
                    pages_without_html.len(),
                    plural!(pages_without_html.len())
                ));
                log.v_warn(pages_without_html.join("\n"));
            }
        }

        log.info(format!(
            "Checked {} file{}",
            all_pages.len(),
            plural!(all_pages.len()),
        ));

        let issues: Vec<String> = all_pages
            .into_iter()
            .flatten()
            .flat_map(|page| {
                let path = page.file_path.to_str().unwrap_or("[unknown path]");
                page.issues
                    .into_iter()
                    .map(|issue| format!("* {}: {}", path, issue))
                    .collect::<Vec<_>>()
            })
            .collect();

        if issues.is_empty() {
            log.info("All ok!");
        } else {
            log.error(format!("{} issues{}:", issues.len(), plural!(issues.len())));

            for issue in issues {
                log.error(issue);
            }
        }
    }
}
