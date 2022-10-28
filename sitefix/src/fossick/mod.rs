use std::io::Error;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::time::{sleep, Duration};

use crate::{FixOptions, Globals, SitefixIssue};
use parser::DomParser;

use self::parser::DomParserResult;

mod parser;

#[derive(Debug)]
pub struct FossickedData {
    pub file_path: PathBuf,
    pub issues: Vec<SitefixIssue>,
    pub has_html_element: bool,
}

#[derive(Debug)]
pub struct Fossicker {
    pub file_path: PathBuf,
    pub urls: Vec<String>,
    data: Option<DomParserResult>,
}

impl Fossicker {
    pub fn new(file_path: PathBuf, options: &FixOptions) -> Self {
        Self {
            urls: vec![build_url(&file_path, options)],
            file_path,
            data: None,
        }
    }

    async fn read_file(&mut self, globals: &Globals, options: &FixOptions) -> Result<(), Error> {
        let file = File::open(&self.file_path).await?;

        let mut rewriter = DomParser::new(globals, options);

        let mut br = BufReader::new(file);
        let mut buf = [0; 20000];
        while let Ok(read) = br.read(&mut buf).await {
            if read == 0 {
                break;
            }
            if let Err(error) = rewriter.write(&buf[..read]) {
                println!(
                    "Failed to parse file {} — skipping this file. Error:\n{error}",
                    self.file_path.to_str().unwrap_or("[unknown file]")
                );
                return Ok(());
            }
        }

        self.data = Some(rewriter.wrap());

        Ok(())
    }

    pub async fn fossick(
        mut self,
        globals: &Globals,
        options: &FixOptions,
    ) -> Result<FossickedData, ()> {
        while self.read_file(globals, options).await.is_err() {
            sleep(Duration::from_millis(1)).await;
        }

        if self.data.is_none() {
            return Err(());
        }

        let data = self.data.unwrap();
        Ok(FossickedData {
            file_path: self.file_path,
            has_html_element: data.has_html_element,
            issues: data.issues,
        })
    }
}

fn build_url(page_url: &PathBuf, options: &FixOptions) -> String {
    let url = page_url
        .strip_prefix(&options.source)
        .expect("File was found that does not start with the source directory");

    format!(
        "/{}",
        url.to_str().unwrap().to_owned().replace("index.html", "")
    )
}
