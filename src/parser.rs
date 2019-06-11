// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::error::{ParserError, Result};
use crate::{Encoding, Header, Link, Param, UriRef, Value};
use percent_encoding::percent_decode;
pub use pest::{iterators::Pair, Parser};
use std::fmt::{self, Display};

#[derive(Parser)]
#[grammar = "rfc8288.pest"]
pub struct Rfc8288Parser;

impl Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

pub fn parse(input: &str) -> Result<Header> {
    let rule = Rfc8288Parser::parse(Rule::header, &input)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    collect_header(rule)
}

fn collect_header(pair: Pair<Rule>) -> Result<Header> {
    ensure!(
        pair.as_rule() == Rule::header,
        ParserError::InvalidRule(Rule::header.into(), pair.as_rule().into())
    );

    let mut links = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::link => {
                let link = collect_links(inner_pair)?;
                links.extend(link);
            }

            Rule::EOI => (),

            _ => unreachable!(),
        }
    }

    Ok(Header { links })
}

fn collect_links(pair: Pair<Rule>) -> Result<Vec<Link>> {
    ensure!(
        pair.as_rule() == Rule::link,
        ParserError::InvalidRule(Rule::link, pair.as_rule())
    );

    let mut target = String::new();
    let mut params = vec![];
    let mut relations = vec![];

    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::target => {
                target.push_str(inner_pair.as_str());
            }

            Rule::param => {
                let param = collect_param(inner_pair)?;

                if param.name() == "rel" && relations.is_empty() && param.value().is_some() {
                    let value: String = param.value().clone().unwrap().to_string();
                    let values: Vec<String> = value.split(" ").map(|s| s.into()).collect();
                    relations.extend(values);
                } else {
                    params.push(param);
                }
            }

            _ => unreachable!(),
        }
    }

    Ok(explode_links(target.clone().into(), relations, &params))
}

fn explode_links(target: UriRef, relations: Vec<String>, params: &[Param]) -> Vec<Link> {
    let mut result = vec![];

    if relations.is_empty() {
        return vec![Link {
            target,
            relation: None,
            params: params.to_vec(),
        }];
    }

    for rel in relations.into_iter() {
        result.push(Link {
            target: target.clone(),
            relation: Some(rel.into()),
            params: params.to_vec(),
        });
    }

    result
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
                relation: None,
                params: vec![],
            }],
        };

        let actual = parse(input).expect("Expect a valid header");

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
                    relation: Some("next".into()),
                    params: vec![],
                },
                Link {
                    target: "https://example.org/1".into(),
                    relation: Some("previous".into()),
                    params: vec![],
                },
            ],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_1() {
        let input =
            r#"<http://example.com/TheBook/chapter2>; rel="previous"; title="previous chapter""#;

        let expected = Header {
            links: vec![Link {
                target: "http://example.com/TheBook/chapter2".into(),
                relation: Some("previous".into()),
                params: vec![Param::new("title", Some("previous chapter".into()))],
            }],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_2() {
        let input = r#"</>; rel="http://example.net/foo""#;

        let expected = Header {
            links: vec![Link {
                target: "/".into(),
                relation: Some("http://example.net/foo".into()),
                params: vec![],
            }],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_3() {
        let input = "</terms>; rel=\"copyright\"; anchor=\"#foo\"";

        let expected = Header {
            links: vec![Link {
                target: "/terms".into(),
                relation: Some("copyright".into()),
                params: vec![Param::new("anchor", Some("#foo".into()))],
            }],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_4() {
        let input = r#"</TheBook/chapter2>; rel="previous"; title*=UTF-8'de'letztes%20Kapitel, </TheBook/chapter4>; rel="next"; title*=UTF-8'de'n%c3%a4chstes%20Kapitel"#;

        let expected = Header {
            links: vec![
                Link {
                    target: "/TheBook/chapter2".into(),
                    relation: Some("previous".into()),
                    params: vec![Param::new(
                        "title",
                        Some(Value::Compound {
                            value: "letztes Kapitel".into(),
                            encoding: Encoding::Utf8,
                            language: Some("de".into()),
                        }),
                    )],
                },
                Link {
                    target: "/TheBook/chapter4".into(),
                    relation: Some("next".into()),
                    params: vec![Param::new(
                        "title",
                        Some(Value::Compound {
                            value: "nächstes Kapitel".into(),
                            encoding: Encoding::Utf8,
                            language: Some("de".into()),
                        }),
                    )],
                },
            ],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn link_header_field_examples_5() {
        let input = r#"<http://example.org/>; rel="start http://example.net/relation/other""#;

        let expected = Header {
            links: vec![
                Link {
                    target: "http://example.org/".into(),
                    relation: Some("start".into()),
                    params: vec![],
                },
                Link {
                    target: "http://example.org/".into(),
                    relation: Some("http://example.net/relation/other".into()),
                    params: vec![],
                },
            ],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn tolerate_extra_rel() {
        let input = r#"<http://example.org/>; rel="next"; rel="wrong""#;

        let expected = Header {
            links: vec![Link {
                target: "http://example.org/".into(),
                relation: Some("next".into()),
                params: vec![Param::new("rel", Some("wrong".into()))],
            }],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }

    #[test]
    fn unicode_fest() {
        let input = "<http://example.org/\u{FE0F}>; rel=\"\u{1F383}\"";

        let expected = Header {
            links: vec![Link {
                target: "http://example.org/\u{FE0F}".into(),
                relation: Some("🎃".into()),
                params: vec![],
            }],
        };

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }
}
