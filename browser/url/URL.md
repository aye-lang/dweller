# url

URL parsing and relative resolution, a subset of the
[WHATWG URL standard](https://url.spec.whatwg.org/).

Supported: schemes, the special schemes (http, https, ws, wss, ftp, file)
with their default ports and `\` as `/`, user info, ASCII hosts
(lower-cased; no IDNA and no IP address checks), ports, paths with `.` and
`..` segments removed, queries, fragments, opaque paths (`data:`,
`mailto:`), and percent-encoding of the bytes that need it. Windows drive
letters in file URLs are not treated specially.

`Url.href` is the serialized URL, owned by the `Url` (`deinit` frees it);
the other slices point into it:

- `scheme`: lower case, without the `:`.
- `host`: lower case; empty when there is none (or for `file:///`).
- `port`: digits; empty for none or for the scheme's default port.
- `path`: starts with `/` for special URLs; anything for an opaque path.
- `query`, `fragment`: without the `?` and `#`; `has_query` and
  `has_fragment` tell an empty one from none.
- `opaque`: an opaque path (`data:...`, `mailto:...`); such a URL is a
  base only for a fragment.

These field names predate the code-taste rules (`scheme_lower`, `?[]u8`
for the query); renaming them touches every caller, so it is its own card.
