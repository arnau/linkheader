// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::error::Result;
use crate::header::Header;
pub use pest::Parser;
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

    Header::from_rule(rule)
}
