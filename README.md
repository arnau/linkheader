# Link Header parser

`linkheader` is a parser for the [RFC8288](https://tools.ietf.org/html/rfc8288).

## Goals

* [x] Forgiving parser.
  * [x] A link with no "rel" param is not an error.
  * [x] A link with many "rel" params is not an error.
  * [x] UTF-8 characters outside the US-ASCII range are not an error.
  * [x] A link with many "anchor" params is not an error.
* [x] Quoted and unquoted param values are equivalent. E.g. `hreflang=en` is
      the same as `hreflang="en"`.
* [x] Collect the link relation type when available.
* [x] Explode to individual links when given a multi-token "rel".
* [x] Change the link context when a valid "anchor" is present.
* [x] Star params (e.g. `title*=UTF-8'en'foo%20bar`) expect a valid
      [RFC8187](https://tools.ietf.org/html/rfc8187) value.
* [ ] Compose relative targets with the given context.
* [x] Collect the "title" param prioritising `title*` when present.
* [x] Collect the "hreflang" param.
* [x] Collect the "type" param.
* [x] Collect the "media" param.

## Non-goals

* "rev" is not treated specially. It is just another param.
* "rel" values are not validated against the [IANA registry](https://www.iana.org/assignments/link-relations/link-relations.xhtml).
* Language tags [RFC5646](https://tools.ietf.org/html/rfc5646) are not parsed.
* [RFC8187](https://tools.ietf.org/html/rfc8187) values not in UTF-8 are not decoded.
* Media types [RFC2046](https://tools.ietf.org/html/rfc2046) are not parsed.
* The special (HTML) "rel" `alternate stylesheet` is not handled. Any
  multi-token "rel" expands to individual links with a different relation
  type.


## Licence

linkheader is licensed under the [MIT License](./LICENSE).
