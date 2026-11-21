# DV2b: the macOS-only wait in the slowest tests

- goal: the tests that take 70 to 140 s on macOS and under 10 s on Ubuntu (DV2's card) take on macOS about what they take on Ubuntu, and the macOS shards fall toward DV2's 9 minutes.
- what the times say (`tests/browser-times.txt`, 375 tests): 22 tests take 50 s or more on macOS and under 15 s on Ubuntu. Every one starts the Python test server: one server costs about 36 s on macOS (`net::hsts` 0 / 36, `websocket::websocket_echo` 0 / 36, `security::csp_script_src_events` 1 / 36), two about 71 s (`csp_script_src`, `link_sheets_shared`, `module_cors`, `script_events`: `server(), server()`), and `link_events::link_events` 141 s (two pages of `dump_link_page`, two servers each). So the wait is per server start, not in the browser.
- cause: `http.server.HTTPServer.server_bind` calls `socket.getfqdn(host)`, a reverse DNS lookup of `127.0.0.1` and of the names it finds; on the macOS runners that lookup stalls (a resolver timeout of about 35 s), and the test waits for the port line, which the server prints after the bind. Ubuntu answers it from `/etc/hosts` at once.
- fix: `browser/tests/server.py`, `hsts_server.py` and `tls_server.py` bind with `QuickServer`, a `ThreadingHTTPServer` whose `server_bind` is `TCPServer.server_bind` plus the name and port from the address: no lookup. `server_name` was only used for CGI's `SERVER_NAME`, which nothing here reads.
- files: `browser/tests/{server,hsts_server,tls_server}.py`, `docs/plans/cards/DV2b.md` (new).
- verify:
  - on this machine (Linux): the three servers start and print their ports; the whole suite passes (Q2.14's verify).
  - the macOS verify is the next merge-queue run: the 22 tests' macOS times (`browser/tools/test_times.py RUN_ID`) under 15 s, and the two macOS shards' walls. The integrator then refreshes `tests/browser-times.txt` (DV2).

- measured (PR #378's run 44955222897 with the `macos` label, macOS, seconds before (`tests/browser-times.txt` of DV2) and after): `link_events::link_events` 141 -> 1.9, `net::coop_popups` 106 -> 2.2, `net::fetch_metadata` 82 -> 17.1, `net::cert_bypass` 78 -> 7.5, `media::window_lazy_images` 71 -> 1.8, `security::csp_script_src` 71 -> 0.6, `net::hsts` 36 -> 0.4, `websocket::websocket_echo` 36 -> 1.3. `browser/tools/test_times.py --logs` on that run's seven shard logs: the macOS shards' summed seconds fell from 2924 to 1113 each (379 tests measured on both systems); `tests/browser-times.txt` refreshed from it.

## Status: done, 2026-11-21

Left:
- `downloads::downloads_saved_and_listed` takes 88 s on macOS and 1 s on Ubuntu (its `_in_renderers` twin 3 s): another macOS-only wait, not looked at.
- DV2's 9 minutes: the slowest macOS shard is now set by `jsdom::jsdom_gc_stress` alone (537 s on macOS, 305 s on Ubuntu) and `window_renderer_limits_find_in_frames` (300 s, 190 s); slow on both systems, so DV2's own card.
