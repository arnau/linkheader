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
pub mod param;
pub mod parser;
