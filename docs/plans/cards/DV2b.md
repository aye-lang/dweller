# DV2b: the macOS-only wait in the slowest tests

- goal: the tests that take 70 to 140 s on macOS and under 10 s on Ubuntu (DV2's card) take on macOS about what they take on Ubuntu, and the macOS shards fall toward DV2's 9 minutes.
- what the times say (`tests/browser-times.txt`, 375 tests): 22 tests take 50 s or more on macOS and under 15 s on Ubuntu. Every one starts the Python test server: one server costs about 36 s on macOS (`net::hsts` 0 / 36, `websocket::websocket_echo` 0 / 36, `security::csp_script_src_events` 1 / 36), two about 71 s (`csp_script_src`, `link_sheets_shared`, `module_cors`, `script_events`: `server(), server()`), and `link_events::link_events` 141 s (two pages of `dump_link_page`, two servers each). So the wait is per server start, not in the browser.
- cause: `http.server.HTTPServer.server_bind` calls `socket.getfqdn(host)`, a reverse DNS lookup of `127.0.0.1` and of the names it finds; on the macOS runners that lookup stalls (a resolver timeout of about 35 s), and the test waits for the port line, which the server prints after the bind. Ubuntu answers it from `/etc/hosts` at once.
- fix: `browser/tests/server.py`, `hsts_server.py` and `tls_server.py` bind with `QuickServer`, a `ThreadingHTTPServer` whose `server_bind` is `TCPServer.server_bind` plus the name and port from the address: no lookup. `server_name` was only used for CGI's `SERVER_NAME`, which nothing here reads.
- files: `browser/tests/{server,hsts_server,tls_server}.py`, `docs/plans/cards/DV2b.md` (new).
- verify:
  - on this machine (Linux): the three servers start and print their ports; the whole suite passes (Q2.14's verify).
  - the macOS verify is the next merge-queue run: the 22 tests' macOS times (`browser/tools/test_times.py RUN_ID`) under 15 s, and the two macOS shards' walls. The integrator then refreshes `tests/browser-times.txt` (DV2).
