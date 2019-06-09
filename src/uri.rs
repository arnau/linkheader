// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

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
