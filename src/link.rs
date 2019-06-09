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

/// A link relation type.
///
/// RFC8288 requires a link to have a direct relation type. Reverse relations
/// are kept as link params but not handled as relation types.
#[derive(Debug, Clone, PartialEq)]
pub struct Relation(String);

impl From<&str> for Relation {
    fn from(s: &str) -> Relation {
        Relation(s.into())
    }
}

impl From<String> for Relation {
    fn from(s: String) -> Relation {
        Relation(s)
    }
}

/// A link to a target resource. The context is implicit so rules around
/// "anchor" are not applied.
#[derive(Debug, PartialEq)]
pub struct Link {
    pub target: UriRef,
    pub relation: Option<Relation>,
    pub params: Vec<Param>,
}

impl Link {
    pub fn from_rule(pair: Pair<Rule>) -> Result<Vec<Link>> {
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
                    let param = Param::from_rule(inner_pair)?;

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

        Ok(explode(target.clone().into(), relations, &params))
    }
}

fn explode(target: UriRef, relations: Vec<String>, params: &[Param]) -> Vec<Link> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::param::*;
    use crate::parser::{Parser, Rfc8288Parser, Rule};

    #[test]
    fn target() {
        let input = r#"<https://example.org>"#;
        let expected = vec![Link {
            target: "https://example.org".into(),
            relation: None,
            params: vec![],
        }];

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
        let expected = vec![Link {
            target: "https://example.org".into(),
            relation: None,
            params: vec![Param::new("foo", None)],
        }];

        let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

        let actual = Link::from_rule(rule.next().unwrap()).expect("Expect a valid link");

        assert_eq!(actual, expected);
    }

    #[test]
    fn target_single_param() {
        let input = r#"<https://example.org>; rel=next"#;
        let expected = vec![Link {
            target: "https://example.org".into(),
            relation: Some(Relation("next".into())),
            params: vec![],
        }];

        let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

        let actual = Link::from_rule(rule.next().unwrap()).expect("Expect a valid link");

        assert_eq!(actual, expected);
    }

    #[test]
    fn target_single_param_quoted_value() {
        let input = r#"<https://example.org>; rel="next""#;
        let expected = vec![Link {
            target: "https://example.org".into(),
            relation: Some("next".into()),
            params: vec![],
        }];

        let mut rule = Rfc8288Parser::parse(Rule::link, &input).expect("unsuccessful parse");

        let actual = Link::from_rule(rule.next().unwrap()).expect("Expect a valid link");

        assert_eq!(actual, expected);
    }
}
