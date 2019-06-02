# Link Header parser

This parser tokenises a string conforming the serialisation rules described in
[RFC 8288](https://tools.ietf.org/html/rfc8288).


## Non-goals

* No semantic processing. Rules regarding `rel` param names are not applied.
* Values are untouched. For example, there is no validation that a value
  conforms to the [RFC 8172](https://tools.ietf.org/html/rfc8187).
