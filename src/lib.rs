extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::{iterators::Pair, Parser};

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

#[derive(Debug, PartialEq)]
pub struct Param {
    name: String,
    value: Option<String>,
}

impl Param {
    pub fn from_rule(pair: Pair<Rule>) -> Param {
        if pair.as_rule() != Rule::param {
            println!("abort!");
        }

        let mut name = String::new();
        let mut value = None;

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::name => name.push_str(inner_pair.as_str()),

                Rule::value => value = Some(inner_pair.as_str().into()),

                Rule::quoted_value => value = Some(inner_pair.as_str().into()),

                _ => unreachable!(),
            }
        }

        Param {
            name: name.into(),
            value: value.clone(),
        }
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
    pub fn from_rule(pair: Pair<Rule>) -> Link {
        if pair.as_rule() != Rule::link {
            println!("abort!");
        }

        let mut target = String::new();
        let mut params = vec![];

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::target => {
                    target.push_str(inner_pair.as_str());
                }

                Rule::param => {
                    params.push(Param::from_rule(inner_pair));
                }

                _ => unreachable!(),
            }
        }

        let link = Link {
            target: target.into(),
            params,
        };

        link
    }
}

#[derive(Debug, PartialEq)]
pub struct Header {
    links: Vec<Link>,
}

impl Header {
    pub fn parse(input: &str) -> Header {
        let rule = Rfc8288Parser::parse(Rule::header, &input)
            .expect("unsuccessful parse")
            .next()
            .unwrap();

        Header::from_rule(rule)
    }

    pub fn from_rule(pair: Pair<Rule>) -> Header {
        if pair.as_rule() != Rule::header {
            println!("abort!");
        }

        let mut links = vec![];

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::link => {
                    links.push(Link::from_rule(inner_pair));
                }

                Rule::EOI => (),

                _ => unreachable!(),
            }
        }

        Header { links }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod header {
        use super::*;

        #[test]
        fn single_link() {
            let input = r#"<https://example.org>"#;
            let expected = Header {
                links: vec![Link {
                    target: "https://example.org".into(),
                    params: vec![],
                }],
            };

            let actual = Header::parse(input);

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
                        params: vec![Param {
                            name: "rel".into(),
                            value: Some("next".into()),
                        }],
                    },
                    Link {
                        target: "https://example.org/1".into(),
                        params: vec![Param {
                            name: "rel".into(),
                            value: Some("previous".into()),
                        }],
                    },
                ],
            };

            let actual = Header::parse(input);

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_1() {
            let input = r#"<http://example.com/TheBook/chapter2>; rel="previous"; title="previous chapter""#;

            let expected = Header {
                links: vec![Link {
                    target: "http://example.com/TheBook/chapter2".into(),
                    params: vec![
                        Param {
                            name: "rel".into(),
                            value: Some("previous".into()),
                        },
                        Param {
                            name: "title".into(),
                            value: Some("previous chapter".into()),
                        },
                    ],
                }],
            };

            let actual = Header::parse(input);

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_2() {
            let input = r#"</>; rel="http://example.net/foo""#;

            let expected = Header {
                links: vec![Link {
                    target: "/".into(),
                    params: vec![Param {
                        name: "rel".into(),
                        value: Some("http://example.net/foo".into()),
                    }],
                }],
            };

            let actual = Header::parse(input);

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_3() {
            let input = "</terms>; rel=\"copyright\"; anchor=\"#foo\"";

            let expected = Header {
                links: vec![Link {
                    target: "/terms".into(),
                    params: vec![
                        Param {
                            name: "rel".into(),
                            value: Some("copyright".into()),
                        },
                        Param {
                            name: "anchor".into(),
                            value: Some("#foo".into()),
                        },
                    ],
                }],
            };

            let actual = Header::parse(input);

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
                            Param {
                                name: "rel".into(),
                                value: Some("previous".into()),
                            },
                            Param {
                                name: "title*".into(),
                                value: Some("UTF-8'de'letztes%20Kapitel".into()),
                            },
                        ],
                    },
                    Link {
                        target: "/TheBook/chapter4".into(),
                        params: vec![
                            Param {
                                name: "rel".into(),
                                value: Some("next".into()),
                            },
                            Param {
                                name: "title*".into(),
                                value: Some("UTF-8'de'n%c3%a4chstes%20Kapitel".into()),
                            },
                        ],
                    },
                ],
            };

            let actual = Header::parse(input);

            assert_eq!(actual, expected);
        }

        #[test]
        fn link_header_field_examples_5() {
            let input = r#"<http://example.org/>; rel="start http://example.net/relation/other""#;

            let expected = Header {
                links: vec![Link {
                    target: "http://example.org/".into(),
                    params: vec![Param {
                        name: "rel".into(),
                        value: Some("start http://example.net/relation/other".into()),
                    }],
                }],
            };

            let actual = Header::parse(input);

            assert_eq!(actual, expected);
        }

    }

    mod link {
        use super::*;

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

            let actual = Link::from_rule(rule);

            assert_eq!(actual, expected);
        }

        #[test]
        fn target_param_no_value() {
            let input = r#"<https://example.org>; foo"#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![Param {
                    name: "foo".into(),
                    value: None,
                }],
            };

            let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

            let actual = Link::from_rule(rule.next().unwrap());

            assert_eq!(actual, expected);
        }

        #[test]
        fn target_single_param() {
            let input = r#"<https://example.org>; rel=next"#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![Param {
                    name: "rel".into(),
                    value: Some("next".into()),
                }],
            };

            let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

            let actual = Link::from_rule(rule.next().unwrap());

            assert_eq!(actual, expected);
        }

        #[test]
        fn target_single_param_quoted_value() {
            let input = r#"<https://example.org>; rel="next""#;
            let expected = Link {
                target: "https://example.org".into(),
                params: vec![Param {
                    name: "rel".into(),
                    value: Some("next".into()),
                }],
            };

            let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

            let actual = Link::from_rule(rule.next().unwrap());

            assert_eq!(actual, expected);
        }
    }
}
