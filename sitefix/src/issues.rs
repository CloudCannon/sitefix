use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum SitefixIssue {
    MissingLink(String),
    DeadLink(String),
}

impl Display for SitefixIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SitefixIssue::MissingLink(msg) => write!(f, "Missing Link: {msg}"),
            SitefixIssue::DeadLink(msg) => write!(f, "Dead Link: {msg}"),
        }
    }
}
