// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::error::{ParserError, Result};
use crate::param::Param;
use pest::{iterators::Pair, Parser};
use std::fmt::{self, Display};

#[derive(Parser)]
#[grammar = "rfc8288.pest"]
pub struct Rfc8288Parser;

#[derive(Debug, PartialEq)]
pub struct UriRef(String);

impl From<String> for UriRef {
    fn from(s: String) -> Self {
        UriRef(s)
    }
}

impl From<&str> for UriRef {
    fn from(s: &str) -> Self {
        UriRef(s.to_string())
    }
}

impl Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{:?}", self)
    }
}

/// Primitive representation of a link without Context or handling rules for
/// "rel", "anchor", "hreflang", etc.
#[derive(Debug, PartialEq)]
pub struct Link {
    target: UriRef,
    params: Vec<Param>,
}

impl Link {
    pub fn from_rule(pair: Pair<Rule>) -> Result<Link> {
        ensure!(
            pair.as_rule() == Rule::link,
            ParserError::InvalidRule(Rule::link, pair.as_rule())
        );

        let mut target = String::new();
        let mut params = vec![];

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::target => {
                    target.push_str(inner_pair.as_str());
                }

                Rule::param => {
                    let param = Param::from_rule(inner_pair)?;
                    params.push(param);
                }

                _ => unreachable!(),
            }
        }

        let link = Link {
            target: target.into(),
            params,
        };

        Ok(link)
    }
}

#[derive(Debug, PartialEq)]
pub struct Header {
    links: Vec<Link>,
}

impl Header {
    pub fn parse(input: &str) -> Result<Header> {
        let rule = Rfc8288Parser::parse(Rule::header, &input)
            .expect("unsuccessful parse")
            .next()
            .unwrap();

        Header::from_rule(rule)
    }

    pub fn from_rule(pair: Pair<Rule>) -> Result<Header> {
        ensure!(
            pair.as_rule() == Rule::header,
            ParserError::InvalidRule(Rule::header.into(), pair.as_rule().into())
        );

        let mut links = vec![];

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::link => {
                    let link = Link::from_rule(inner_pair)?;
                    links.push(link);
                }

                Rule::EOI => (),

                _ => unreachable!(),
            }
        }

        Ok(Header { links })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod header {
        use super::*;
        use crate::param::*;

        #[test]
        fn single_link() {
            let input = r#"<https://example.org>"#;
            let expected = Header {
                links: vec![Link {
                    target: "https://example.org".into(),
                    params: vec![],
                }],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

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
                        params: vec![Param::new("rel", Some("next".into()))],
                    },
                    Link {
                        target: "https://example.org/1".into(),
                        params: vec![Param::new("rel", Some("previous".into()))],
                    },
                ],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_1() {
            let input = r#"<http://example.com/TheBook/chapter2>; rel="previous"; title="previous chapter""#;

            let expected = Header {
                links: vec![Link {
                    target: "http://example.com/TheBook/chapter2".into(),
                    params: vec![
                        Param::new("rel", Some("previous".into())),
                        Param::new("title", Some("previous chapter".into())),
                    ],
                }],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_2() {
            let input = r#"</>; rel="http://example.net/foo""#;

            let expected = Header {
                links: vec![Link {
                    target: "/".into(),
                    params: vec![Param::new("rel", Some("http://example.net/foo".into()))],
                }],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_3() {
            let input = "</terms>; rel=\"copyright\"; anchor=\"#foo\"";

            let expected = Header {
                links: vec![Link {
                    target: "/terms".into(),
                    params: vec![
                        Param::new("rel", Some("copyright".into())),
                        Param::new("anchor", Some("#foo".into())),
                    ],
                }],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_4() {
            let input = r#"</TheBook/chapter2>; rel="previous"; title*=UTF-8'de'letztes%20Kapitel, </TheBook/chapter4>; rel="next"; title*=UTF-8'de'n%c3%a4chstes%20Kapitel"#;

            let expected = Header {
                links: vec![
                    Link {
                        target: "/TheBook/chapter2".into(),
                        params: vec![
                            Param::new("rel", Some("previous".into())),
                            Param::new(
                                "title",
                                Some(Value::Compound {
                                    value: "letztes Kapitel".into(),
                                    encoding: Encoding::Utf8,
                                    language: Some("de".into()),
                                }),
                            ),
                        ],
                    },
                    Link {
                        target: "/TheBook/chapter4".into(),
                        params: vec![
                            Param::new("rel", Some("next".into())),
                            Param::new(
                                "title",
                                Some(Value::Compound {
                                    value: "nächstes Kapitel".into(),
                                    encoding: Encoding::Utf8,
                                    language: Some("de".into()),
                                }),
                            ),
                        ],
                    },
                ],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_5() {
            let input = r#"<http://example.org/>; rel="start http://example.net/relation/other""#;

            let expected = Header {
                links: vec![Link {
                    target: "http://example.org/".into(),
                    params: vec![Param::new(
                        "rel",
                        Some("start http://example.net/relation/other".into()),
                    )],
                }],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }

        #[test]
        fn unicode_fest() {
            let input = "<http://example.org/\u{FE0F}>; rel=\"\u{1F383}\"";

            let expected = Header {
                links: vec![Link {
                    target: "http://example.org/\u{FE0F}".into(),
                    params: vec![Param::new("rel", Some("🎃".into()))],
                }],
            };

            let actual = Header::parse(input).expect("Expect a valid header");

            assert_eq!(actual, expected);
        }
    }

    mod link {
        use super::*;
        use crate::param::*;

        #[test]
        fn wrong_rule_type() {
            let input = r#"<https://example.org>"#;

            let rule = Rfc8288Parser::parse(Rule::link, &input)
                .expect("unsuccessful parse")
                .next()
                .unwrap();

            let actual = Header::from_rule(rule).is_err();

            assert!(actual);
        }

        #[test]
        fn target() {
            let input = r#"<https://example.org>"#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![],
            };

            let rule = Rfc8288Parser::parse(Rule::link, &input)
                .expect("unsuccessful parse")
                .next()
                .unwrap();

            let actual = Link::from_rule(rule).expect("Expect a valid link");

            assert_eq!(actual, expected);
        }

        #[test]
        fn target_param_no_value() {
            let input = r#"<https://example.org>; foo"#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![Param::new("foo", None)],
            };

            let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

            let actual = Link::from_rule(rule.next().unwrap()).expect("Expect a valid link");

            assert_eq!(actual, expected);
        }

        #[test]
        fn target_single_param() {
            let input = r#"<https://example.org>; rel=next"#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![Param::new("rel", Some(Value::Simple("next".into())))],
            };

            let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

            let actual = Link::from_rule(rule.next().unwrap()).expect("Expect a valid link");

            assert_eq!(actual, expected);
        }

        #[test]
        fn target_single_param_quoted_value() {
            let input = r#"<https://example.org>; rel="next""#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![Param::new("rel", Some(Value::Simple("next".into())))],
            };

            let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

            let actual = Link::from_rule(rule.next().unwrap()).expect("Expect a valid link");

            assert_eq!(actual, expected);
        }
    }
}
