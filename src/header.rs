// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use crate::link::Link;

/// A collection of links.
#[derive(Debug, PartialEq)]
pub struct Header {
    pub links: Vec<Link>,
}
