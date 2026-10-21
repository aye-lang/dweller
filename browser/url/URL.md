# url

The basic URL parser of the [WHATWG URL standard](https://url.spec.whatwg.org/)
as a state machine: one file per state (`*-state.aye`) over a `Parser`
(`parser.aye`), run by `runParser`, and serialized into a `Url` by
`serializeUrl`. `parse`, `resolve` and `resolveIn` call `basicParse`.

Supported: special and non-special schemes, user info, domain, IPv4, IPv6
and opaque hosts, ports, paths with `.` and `..` segments, Windows drive
letters in file URLs, opaque paths, queries (in a document's encoding with
`resolveIn`), fragments, and the percent-encode sets. The parser works on
UTF-8 bytes; every byte at or above 0x80 is in every percent-encode set, so
byte-wise encoding equals the spec's per-code-point encoding. Not yet:
IDNA (`uts46ToAscii` only lower-cases ASCII) and the state override for the
`URL` setters (`Parser.override` exists but nothing sets it).

`Url.href` is the serialized URL, owned by the `Url` (`deinit` frees it);
the other slices point into it:

- `scheme`: lower case, without the `:`.
- `host`: the serialized host (lower case; IPv6 in `[]`); empty when there
  is none (or for `file:///`).
- `port`: digits; empty for none or for the scheme's default port.
- `path`: starts with `/` for special URLs; anything for an opaque path.
- `query`, `fragment`: without the `?` and `#`; `has_query` and
  `has_fragment` tell an empty one from none.
- `opaque`: the URL has an opaque path (`data:...`, `mailto:...`); such a
  URL is a base only for a fragment.

These field names predate the code-taste rules (`scheme_lower`, `?[]u8`
for the query); renaming them touches every caller, so it is its own card
(T1b in `docs/plans/taste.md`).
