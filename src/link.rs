// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::param::{Param, Value};
use crate::uri::UriRef;
use url;

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

/// A link to a target resource.
#[derive(Debug, PartialEq)]
pub struct Link {
    pub target: UriRef,
    pub context: Option<url::Url>,
    pub relation: Option<Relation>,
    pub title: Option<Value>,
    pub lang: Option<Value>,
    pub media: Option<Value>,
    pub content_type: Option<Value>,
    pub params: Vec<Param>,
}
