// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

#[macro_use]
extern crate failure;

extern crate pest;
#[macro_use]
extern crate pest_derive;

pub mod error;
pub mod header;
pub mod link;
pub mod param;
pub mod parser;
pub mod uri;

pub use parser::parse;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::Header;
    use crate::link::Link;
    use crate::param::{Encoding, Param, Value};

    #[test]
    fn single_link() {
        let input = r#"<https://example.org>"#;
        let expected = Header {
            links: vec![Link {
                target: "https://example.org".into(),
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
                    params: vec![Param::new("rel", Some("next".into()))],
                },
                Link {
                    target: "https://example.org/1".into(),
                    params: vec![Param::new("rel", Some("previous".into()))],
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
                params: vec![
                    Param::new("rel", Some("previous".into())),
                    Param::new("title", Some("previous chapter".into())),
                ],
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
                params: vec![Param::new("rel", Some("http://example.net/foo".into()))],
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
                params: vec![
                    Param::new("rel", Some("copyright".into())),
                    Param::new("anchor", Some("#foo".into())),
                ],
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

        let actual = parse(input).expect("Expect a valid header");

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

        let actual = parse(input).expect("Expect a valid header");

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

        let actual = parse(input).expect("Expect a valid header");

        assert_eq!(actual, expected);
    }
}
