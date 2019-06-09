// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

//! Error types and utilities.

use crate::parser::Rule;
pub use failure::Error;
use failure::*;

/// Either `Ok(T)` or `Err(failure::Error)`.
pub type Result<T> = ::std::result::Result<T, failure::Error>;

/// A parser error.
#[derive(Clone, Eq, PartialEq, Debug, Fail)]
pub enum ParserError {
    /// Given invalid `Rule` variant to `from_rule`
    #[fail(display = "Expected a rule of type {} but given {} instead", _0, _1)]
    InvalidRule(Rule, Rule),
}
