// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::error::{ParserError, Result};
use crate::{Encoding, Header, Link, Param, Value};
use percent_encoding::percent_decode;
pub use pest::{iterators::Pair, Parser};
use std::fmt::{self, Display};
use url;

#[derive(Parser)]
#[grammar = "rfc8288.pest"]
pub struct Rfc8288Parser;

impl Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

pub fn parse(input: &str, context: Option<url::Url>) -> Result<Header> {
    let rule = Rfc8288Parser::parse(Rule::header, &input)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    collect_header(rule, context)
}

fn collect_header(pair: Pair<Rule>, context: Option<url::Url>) -> Result<Header> {
    ensure!(
        pair.as_rule() == Rule::header,
        ParserError::InvalidRule(Rule::header.into(), pair.as_rule().into())
    );

    let mut links = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::link => {
                let link = collect_links(inner_pair, context.clone())?;
                links.extend(link);
            }

            Rule::EOI => (),

            _ => unreachable!(),
        }
    }

    Ok(Header { links })
}

/// Collects attributes and params for a set of links.
#[derive(Debug, Clone)]
pub struct LinkBuilder {
    target: String,
    context: Option<url::Url>,
    anchored_context: Option<url::Url>,
    relations: Vec<String>,
    title: Option<Value>,
    params: Vec<Param>,
}

impl LinkBuilder {
    pub fn new(context: Option<url::Url>) -> LinkBuilder {
        LinkBuilder {
            target: String::new(),
            context,
            anchored_context: None,
            title: None,
            params: vec![],
            relations: vec![],
        }
    }

    pub fn set_target(&mut self, target: &str) {
        self.target.push_str(target);
    }

    pub fn set_anchor(&mut self, value: Value) {
        if self.anchored_context.is_none() {
            self.anchored_context = compose_context(&self.context, &value.to_string());

            // We keep the anchor param if it is not composable with
            // the given context to preserve information.
            if self.anchored_context.is_none() {
                self.params.push(Param::new("anchor", Some(value)));
            }
        } else {
            self.params.push(Param::new("anchor", Some(value)));
        }
    }

    pub fn set_title(&mut self, value: Value) {
        match &self.title {
            Some(current_value) => {
                if current_value.is_simple() && value.is_compound() {
                    self.params
                        .push(Param::new("title", Some(current_value.clone())));
                    self.title = Some(value);
                } else {
                    self.params.push(Param::new("title", Some(value)));
                }
            }
            None => {
                self.title = Some(value);
            }
        }
    }

    /// Takes a rel value and either sets it as a list of rel tokens or keeps
    /// it as a parameter.
    pub fn set_rel(&mut self, value: Value) {
        if self.relations.is_empty() {
            let values: Vec<String> = value.to_string().split(" ").map(|s| s.into()).collect();

            self.relations.extend(values);
        } else {
            self.params.push(Param::new("rel", Some(value)));
        }
    }

    pub fn add_param(&mut self, param: Param) {
        self.params.push(param);
    }

    pub fn build(self) -> Vec<Link> {
        let mut result = vec![];
        let context = self.anchored_context.or(self.context);

        if self.relations.is_empty() {
            return vec![Link {
                target: self.target.into(),
                context: context,
                relation: None,
                title: self.title,
                params: self.params,
            }];
        }

        for rel in self.relations.into_iter() {
            result.push(Link {
                target: self.target.clone().into(),
                context: context.clone(),
                relation: Some(rel.into()),
                title: self.title.clone(),
                params: self.params.to_vec(),
            });
        }

        result
    }
}

fn collect_links(pair: Pair<Rule>, context: Option<url::Url>) -> Result<Vec<Link>> {
    ensure!(
        pair.as_rule() == Rule::link,
        ParserError::InvalidRule(Rule::link, pair.as_rule())
    );

    let mut link_builder = LinkBuilder::new(context.clone());

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::target => {
                link_builder.set_target(inner_pair.as_str());
            }

            Rule::param => {
                let param = collect_param(inner_pair)?;

                match (param.name(), param.value()) {
                    ("rel", Some(value)) => link_builder.set_rel(value.clone()),
                    ("anchor", Some(value)) => link_builder.set_anchor(value.clone()),
                    ("title", Some(value)) => link_builder.set_title(value.clone()),
                    _ => link_builder.add_param(param),
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(link_builder.build())
}

/// Returns a context combined with the given anchor.
///
/// Note that when no context is present, if the anchor is not an absolute URL,
/// the result is no context.
fn compose_context(context: &Option<url::Url>, anchor: &str) -> Option<url::Url> {
    match context {
        Some(ctx) => ctx.join(anchor).ok(),

        None => url::Url::parse(anchor).ok(),
    }
}

fn collect_param(pair: Pair<Rule>) -> Result<Param> {
    ensure!(
        pair.as_rule() == Rule::param,
        ParserError::InvalidRule(Rule::param, pair.as_rule())
    );

    let mut name = String::new();
    let mut value = None;
    let mut encoding = None;
    let mut language = None;

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::name => name.push_str(inner_pair.as_str()),

            Rule::token_value => value = Some(inner_pair.as_str().into()),

            Rule::quoted_value => value = Some(inner_pair.as_str().into()),

            Rule::pct_value => match &encoding {
                Some(enc @ Encoding::Utf8) => {
                    let decoded_value =
                        percent_decode(inner_pair.as_str().as_bytes()).decode_utf8()?;

                    value = Some(Value::Compound {
                        value: decoded_value.into(),
                        encoding: enc.clone(),
                        language: language.clone(),
                    });
                }

                Some(enc) => {
                    value = Some(Value::Compound {
                        value: inner_pair.as_str().into(),
                        encoding: enc.clone(),
                        language: language.clone(),
                    });
                }

                _ => unreachable!(),
            },

            Rule::encoding => {
                let enc: Encoding = inner_pair.as_str().into();

                encoding = Some(enc);
            }

            Rule::language => language = Some(inner_pair.as_str().into()),

            _ => unreachable!(),
        }
    }

    Ok(Param::new(name, value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Encoding, Header, Link, Param, Value};

    #[test]
    fn single_link() {
        let input = r#"<https://example.org>"#;
        let expected = Header {
            links: vec![Link {
                target: "https://example.org".into(),
                context: None,
                relation: None,
                title: None,
                params: vec![],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn multiple_links() {
        let input =
            r#"<https://example.org/3>; rel="next", <https://example.org/1>; rel="previous""#;
        let expected = Header {
            links: vec![
                Link {
                    target: "https://example.org/3".into(),
                    context: None,
                    relation: Some("next".into()),
                    title: None,
                    params: vec![],
                },
                Link {
                    target: "https://example.org/1".into(),
                    context: None,
                    relation: Some("previous".into()),
                    title: None,
                    params: vec![],
                },
            ],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_1() {
        let input =
            r#"<http://example.com/TheBook/chapter2>; rel="previous"; title="previous chapter""#;

        let expected = Header {
            links: vec![Link {
                target: "http://example.com/TheBook/chapter2".into(),
                context: None,
                relation: Some("previous".into()),
                title: Some("previous chapter".into()),
                params: vec![],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_2() {
        let input = r#"</>; rel="http://example.net/foo""#;

        let expected = Header {
            links: vec![Link {
                target: "/".into(),
                context: None,
                relation: Some("http://example.net/foo".into()),
                title: None,
                params: vec![],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_3() {
        let input = "</terms>; rel=\"copyright\"; anchor=\"#foo\"";

        let context = url::Url::parse("https://www.example.org/").ok();
        let expected_context = url::Url::parse("https://www.example.org/#foo").ok();

        let expected = Header {
            links: vec![Link {
                target: "/terms".into(),
                context: expected_context,
                relation: Some("copyright".into()),
                title: None,
                params: vec![],
            }],
        };

        let actual = parse(input, context).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_4() {
        let input = r#"</TheBook/chapter2>; rel="previous"; title*=UTF-8'de'letztes%20Kapitel, </TheBook/chapter4>; rel="next"; title*=UTF-8'de'n%c3%a4chstes%20Kapitel"#;

        let expected = Header {
            links: vec![
                Link {
                    target: "/TheBook/chapter2".into(),
                    context: None,
                    relation: Some("previous".into()),
                    title: Some(Value::Compound {
                        value: "letztes Kapitel".into(),
                        encoding: Encoding::Utf8,
                        language: Some("de".into()),
                    }),
                    params: vec![],
                },
                Link {
                    target: "/TheBook/chapter4".into(),
                    context: None,
                    relation: Some("next".into()),
                    title: Some(Value::Compound {
                        value: "nächstes Kapitel".into(),
                        encoding: Encoding::Utf8,
                        language: Some("de".into()),
                    }),
                    params: vec![],
                },
            ],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_5() {
        let input = r#"<http://example.org/>; rel="start http://example.net/relation/other""#;

        let expected = Header {
            links: vec![
                Link {
                    target: "http://example.org/".into(),
                    context: None,
                    relation: Some("start".into()),
                    title: None,
                    params: vec![],
                },
                Link {
                    target: "http://example.org/".into(),
                    context: None,
                    relation: Some("http://example.net/relation/other".into()),
                    title: None,
                    params: vec![],
                },
            ],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn prefer_star_title() {
        let input = r#"</TheBook/chapter2>; rel="previous"; title="letztes Kapitel"; title*=UTF-8'de'letztes%20Kapitel"#;

        let expected = Header {
            links: vec![Link {
                target: "/TheBook/chapter2".into(),
                context: None,
                relation: Some("previous".into()),
                title: Some(Value::Compound {
                    value: "letztes Kapitel".into(),
                    encoding: Encoding::Utf8,
                    language: Some("de".into()),
                }),
                params: vec![Param::new("title", Some("letztes Kapitel".into()))],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn tolerate_extra_rel() {
        let input = r#"<http://example.org/>; rel="next"; rel="wrong""#;

        let expected = Header {
            links: vec![Link {
                target: "http://example.org/".into(),
                context: None,
                relation: Some("next".into()),
                title: None,
                params: vec![Param::new("rel", Some("wrong".into()))],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn preserve_context() {
        let input = r#"<http://example.org/>; rel="next""#;

        let context = url::Url::parse("https://www.foobar.com").ok();

        let expected = Header {
            links: vec![Link {
                target: "http://example.org/".into(),
                context: context.clone(),
                relation: Some("next".into()),
                title: None,
                params: vec![],
            }],
        };

        let actual = parse(input, context).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn ignore_anchor_with_no_context() {
        let input = r##"<http://example.org/>; rel="next"; anchor="#foo""##;

        let expected = Header {
            links: vec![Link {
                target: "http://example.org/".into(),
                context: None,
                relation: Some("next".into()),
                title: None,
                params: vec![Param::new("anchor", Some("#foo".into()))],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn tolerate_extra_anchor() {
        let input = "</terms>; rel=\"copyright\"; anchor=\"#foo\"; anchor=\"#bar\"";

        let context = url::Url::parse("https://www.example.org/").ok();
        let expected_context = url::Url::parse("https://www.example.org/#foo").ok();

        let expected = Header {
            links: vec![Link {
                target: "/terms".into(),
                context: expected_context,
                relation: Some("copyright".into()),
                title: None,
                params: vec![Param::new("anchor", Some("#bar".into()))],
            }],
        };

        let actual = parse(input, context).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn unicode_fest() {
        let input = "<http://example.org/\u{FE0F}>; rel=\"\u{1F383}\"";

        let expected = Header {
            links: vec![Link {
                target: "http://example.org/\u{FE0F}".into(),
                context: None,
                relation: Some("🎃".into()),
                title: None,
                params: vec![],
            }],
        };

        let actual = parse(input, None).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }
}
