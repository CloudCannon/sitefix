use lazy_static::lazy_static;
use lol_html::{element, HtmlRewriter, Settings};
use regex::Regex;
use std::cell::RefCell;
use std::default::Default;
use std::rc::Rc;
use urlencoding::decode;

use crate::FixOptions;
use crate::Globals;
use crate::SitefixIssue;

lazy_static! {
    static ref EXTERNAL_URL: Regex = Regex::new("^(https?:)?//").unwrap();
}
lazy_static! {
    static ref PAGE_LINK_SELECTORS: Vec<&'static str> = vec!("a");
}

// We aren't transforming HTML, just parsing, so we dump the output.
#[derive(Default)]
struct EmptySink;
impl lol_html::OutputSink for EmptySink {
    fn handle_chunk(&mut self, _: &[u8]) {}
}

/// Houses the HTML parsing instance and the internal data while parsing
pub struct DomParser<'a> {
    rewriter: HtmlRewriter<'a, EmptySink>,
    data: Rc<RefCell<DomParserData>>,
}

// The internal state while parsing,
// with a reference to the deepest HTML element
// that we're currently reading
#[derive(Default, Debug)]
struct DomParserData {
    current_node: Rc<RefCell<DomParsingNode>>,
    has_html_element: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum NodeStatus {
    Fixing,
    // Our content & children should not be reviewed
    Ignored,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Fixing
    }
}

// A single HTML element that we're reading into.
// Contains a reference to the parent element,
// and since we collapse this tree upwards while we parse,
// we don't need to store tree structure.
#[derive(Default, Debug)]
struct DomParsingNode {
    issues: Vec<SitefixIssue>,
    parent: Option<Rc<RefCell<DomParsingNode>>>,
    status: NodeStatus,
}

/// The fix-relevant data that was retrieved from the given input
#[derive(Debug)]
pub struct DomParserResult {
    pub issues: Vec<SitefixIssue>,
    pub has_html_element: bool,
}

// Some shorthand to clean up our use of Rc<RefCell<*>> in the lol_html macros
// From https://github.com/rust-lang/rfcs/issues/2407#issuecomment-385291238
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

impl<'a> DomParser<'a> {
    pub fn new(globals: &'a Globals, options: &'a FixOptions) -> Self {
        let data = Rc::new(RefCell::new(DomParserData::default()));
        let root = format!("{}, {} *", options.root_selector, options.root_selector);

        let rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![
                    enclose! { (data) element!("html", move |_el| {
                        let mut data = data.borrow_mut();
                        data.has_html_element = true;
                        Ok(())
                    })},
                    enclose! { (data) element!(root, move |el| {
                        let mut issues = vec![];
                        let status = if el.has_attribute("data-sitefix-ignore") {
                            NodeStatus::Ignored
                        } else {
                            NodeStatus::Fixing
                        };

                        let tag_name = el.tag_name();
                        if PAGE_LINK_SELECTORS.contains(&tag_name.as_str()) {
                            match el.get_attribute("href") {
                                Some(url) => {
                                    let decoded_url = decode(&url).expect("UTF-8");
                                    if decoded_url.starts_with('#') {
                                        // TODO: add page-level test category
                                    } else if EXTERNAL_URL.is_match(&decoded_url) {
                                        // TODO: add external test category
                                    } else {
                                        if let Some((main_url, _hash)) = decoded_url.split_once('#') {
                                            if !globals.urls.contains(&main_url.to_string()) {
                                                issues.push(SitefixIssue::DeadLink(format!("<{tag_name}> links to {decoded_url}, but that page does not exist")))
                                            }
                                            // TODO: Add site-level hash tester
                                        } else {
                                            if !globals.urls.contains(&decoded_url.to_string()) {
                                                issues.push(SitefixIssue::DeadLink(format!("<{tag_name}> links to {decoded_url}, but that page does not exist")))
                                            }
                                        }
                                    }
                                },
                                None => issues.push(SitefixIssue::MissingLink(format!("<{tag_name}> has no href"))),
                            }
                        }

                        let node = {
                            let mut data = data.borrow_mut();
                            let parent_status = data.current_node.borrow().status;

                            let node = Rc::new(RefCell::new(DomParsingNode{
                                parent: Some(Rc::clone(&data.current_node)),
                                status: match parent_status {
                                    NodeStatus::Ignored => parent_status,
                                    _ => status,
                                },
                                issues,
                                ..DomParsingNode::default()
                            }));

                            data.current_node = Rc::clone(&node);
                            node
                        };

                        let can_have_content = el.on_end_tag(enclose! { (data, node) move |_end| {
                            let mut data = data.borrow_mut();
                            let node = node.borrow_mut();

                            // When we reach an end tag, we need to
                            // make sure to move focus back to the parent node.
                            if let Some(parent) = &node.parent {
                                data.current_node = Rc::clone(parent);
                            }

                            // For ignored elements, we want to bail
                            if node.status == NodeStatus::Ignored {
                                return Ok(());
                            }

                            let mut parent = data.current_node.borrow_mut();

                            match node.status {
                                NodeStatus::Ignored => {},
                                NodeStatus::Fixing => {
                                    parent.issues.extend(node.issues.clone());
                                }
                            };

                            Ok(())
                        }});

                        // Try to handle tags like <img /> which have no end tag,
                        // and thus will never hit the logic to reset the current node.
                        // TODO: This could still be missed for tags with implied ends?
                        if can_have_content.is_err() {
                            let mut data = data.borrow_mut();
                            let node = node.borrow();
                            if let Some(parent) = &node.parent {
                                data.current_node = Rc::clone(parent);
                            }

                            // For ignored elements, we want to bail
                            if node.status == NodeStatus::Ignored {
                                return Ok(());
                            }


                            let mut parent = data.current_node.borrow_mut();

                            match node.status {
                                NodeStatus::Ignored => {},
                                NodeStatus::Fixing => {
                                    parent.issues.extend(node.issues.clone());
                                }
                            };
                        }
                        Ok(())
                    })},
                ],
                ..Settings::default()
            },
            EmptySink::default(),
        );

        Self { rewriter, data }
    }

    /// Writes a chunk of data to the underlying HTML parser
    pub fn write(&mut self, data: &[u8]) -> Result<(), lol_html::errors::RewritingError> {
        self.rewriter.write(data)
    }

    /// Performs any post-processing and returns the summated search results
    pub fn wrap(self) -> DomParserResult {
        drop(self.rewriter); // Clears the extra Rcs on and within data
        let data = Rc::try_unwrap(self.data).unwrap().into_inner();
        let mut node = data.current_node;

        // Fallback: If we are left with a tree, collapse it up into the parents
        // until we get to the root node.
        while node.borrow().parent.is_some() {
            {
                let node = node.borrow();
                let mut parent = node.parent.as_ref().unwrap().borrow_mut();
                match node.status {
                    NodeStatus::Ignored => {}
                    NodeStatus::Fixing => {
                        parent.issues.extend(node.issues.clone());
                    }
                };
            }
            let old_node = node.borrow();
            let new_node = Rc::clone(old_node.parent.as_ref().unwrap());
            drop(old_node);
            node = new_node;
        }

        let node = node.borrow();

        DomParserResult {
            issues: node.issues.clone(),
            has_html_element: data.has_html_element,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_raw_parse(input: Vec<&'static str>) -> DomParserResult {
        let config_args = vec![twelf::Layer::Clap(
            <crate::SitefixInboundConfig as clap::IntoApp>::command().get_matches_from(vec![
                "sitefix",
                "--source",
                "not_important",
            ]),
        )];
        let config =
            FixOptions::load(crate::SitefixInboundConfig::with_layers(&config_args).unwrap())
                .unwrap();
        let g = Globals::default();
        let mut rewriter = DomParser::new(&g, &config);
        for line in input {
            let _ = rewriter.write(line.as_bytes());
        }
        rewriter.wrap()
    }

    fn test_parse(mut input: Vec<&'static str>) -> DomParserResult {
        input.insert(0, "<html><body>");
        input.push("</body></html>");
        test_raw_parse(input)
    }

    #[test]
    fn ignored_elements() {
        let data = test_parse(vec![
            "<div data-sitefix-ignore>",
            "<a href='/nowhere'>This should not return an error</a>",
            "</div>",
        ]);

        assert!(data.issues.is_empty());
    }
}
