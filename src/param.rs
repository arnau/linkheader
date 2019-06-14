// Copyright 2019 Arnau Siches
//
// Licensed under the MIT license <LICENSE or http://opensource.org/licenses/MIT>.
// This file may not be copied, modified, or distributed except
// according to those terms.

use percent_encoding::{utf8_percent_encode, DEFAULT_ENCODE_SET};
use std::fmt::{self, Display};

/// A link param pair.
///
/// A param has three types of value: token, quoted text or compound (RFC8187).
///
/// The first two are represented by `Value::Simple` and the latter by `Value::Compound`.
///
/// A "token param", for example, `rel=next` is represented as:
///
/// ```norun
/// Param {
///     name: "rel".into(),
///     value: Some(Value::Simple("next".into()))
/// };
/// ```
///
/// A "star param", for example, `title*=utf-8'ca'%C3%A0bac` is represented as:
///
/// ```norun
/// Param {
///     name: "title".to_string(),
///     value: Value::Compound {
///         encoding: Encoding::Utf8,
///         language: Some("ca".to_string()),
///         value: "àbac".to_string()
///     }
/// };
/// ```
///
/// ## Examples
///
/// ```
/// use linkheader::param::{ Param, Value };
///
/// let param = Param::new("rel", Some("next".into()));
///
/// assert_eq!(param.name(), "rel");
/// assert_eq!(param.value(), &Some(Value::Simple("next".into())));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    name: String,
    value: Option<Value>,
}

impl Param {
    pub fn new(name: impl Into<String>, value: Option<Value>) -> Param {
        Param {
            name: name.into(),
            value,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &Option<Value> {
        &self.value
    }

    /// Consumes the param and returns its value.
    pub fn into_value(self) -> Option<Value> {
        self.value
    }

    /// A star param e.g. "title*" is a param marked to identify that its value
    /// is a compound value.
    pub fn is_star(&self) -> bool {
        match self.value {
            Some(Value::Compound { .. }) => true,
            _ => false,
        }
    }
}

/// The character encoding of a compound value.
///
/// RFC8187 Section 3.2.1 names it as "charset" and defines it as:
///
/// ```abnf
/// charset = "UTF-8" / mime-charset
/// ```
///
/// It also says:
///
/// > Producers MUST use the "UTF-8" ([RFC3629]) character encoding.
/// > Extension character encodings (mime-charset) are reserved for future
/// > use.
#[derive(Clone, Debug, PartialEq)]
pub enum Encoding {
    Utf8,
    Extension(String),
}

impl From<&str> for Encoding {
    fn from(s: &str) -> Encoding {
        let sl = s.to_lowercase();

        match &sl[..] {
            "utf-8" => Encoding::Utf8,
            _ => Encoding::Extension(sl),
        }
    }
}

impl Display for Encoding {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Encoding::Utf8 => write!(formatter, "UTF-8"),
            Encoding::Extension(ext) => write!(formatter, "{}", ext),
        }
    }
}

/// A value, either a simple text or a compound of text, character encoding and
/// optionally a language tag.
///
/// Note that RFC8187 names a compound value as "extended value".
///
/// When the encoding of a compound value is not UTF-8, the value will be kept
/// untouched, that is percent-encoded.
///
/// ```
/// use linkheader::param::Value;
///
/// let value = Value::Simple("next".into());
///
/// assert_eq!(value.to_string(), "next".to_string());
/// ```
///
/// ```
/// use linkheader::param::{Value, Encoding};
///
/// let value = Value::Compound {
///     encoding: Encoding::Utf8,
///     language: Some("en".into()),
///     value: "GBP (£)".into(),
/// };
///
/// assert_eq!(value.to_string(), "UTF-8'en'GBP%20(%C2%A3)".to_string());
/// ```
///
/// ```
/// use linkheader::param::{Value, Encoding};
///
/// let value = Value::Compound {
///     encoding: Encoding::Extension("GIB".into()),
///     language: None,
///     value: "%C0%FF%EE".into(),
/// };
///
/// assert_eq!(value.to_string(), "GIB''%C0%FF%EE".to_string());
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Simple(String),
    Compound {
        encoding: Encoding,
        language: Option<String>,
        value: String,
    },
}

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value::Simple(s.into())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Value {
        Value::Simple(s)
    }
}

impl Value {
    /// Returns the text value from either simple or compound values.
    pub fn text(&self) -> &str {
        match self {
            Value::Simple(value) => &value,
            Value::Compound { value, .. } => &value,
        }
    }

    pub fn is_compound(&self) -> bool {
        match self {
            Value::Compound { .. } => true,
            _ => false,
        }
    }

    pub fn is_simple(&self) -> bool {
        match self {
            Value::Simple(_) => true,
            _ => false,
        }
    }
}

impl Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Simple(val) => write!(formatter, "{}", val),
            Value::Compound {
                encoding,
                language,
                value,
            } => {
                let val = match encoding {
                    Encoding::Utf8 => utf8_percent_encode(value, DEFAULT_ENCODE_SET).to_string(),
                    _ => value.to_string(),
                };

                write!(
                    formatter,
                    "{}'{}'{}",
                    encoding,
                    language.clone().unwrap_or("".into()),
                    val
                )
            }
        }
    }
}
