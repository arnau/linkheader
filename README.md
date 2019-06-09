# Link Header parser

This parser tokenises a string conforming the serialisation rules described in
[RFC8288](https://tools.ietf.org/html/rfc8288).


## Non-goals

* Language tags [RFC5646](https://tools.ietf.org/html/rfc5646) are not decoded.
* Percent-decoding other than UTF-8.
