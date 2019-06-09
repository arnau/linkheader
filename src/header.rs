// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::error::{ParserError, Result};
use crate::link::Link;
use crate::parser::Rule;
use pest::iterators::Pair;

#[derive(Debug, PartialEq)]
pub struct Header {
    pub links: Vec<Link>,
}

impl Header {
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
    use crate::parser::{Parser, Rfc8288Parser, Rule};

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
}
