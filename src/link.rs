// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::error::{ParserError, Result};
use crate::param::Param;
use crate::parser::Rule;
use crate::uri::UriRef;
use pest::iterators::Pair;

/// Primitive representation of a link without Context or handling rules for
/// "rel", "anchor", "hreflang", etc.
#[derive(Debug, PartialEq)]
pub struct Link {
    pub target: UriRef,
    pub params: Vec<Param>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::param::*;
    use crate::parser::{Parser, Rfc8288Parser, Rule};

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
