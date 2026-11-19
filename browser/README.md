# Dweller

A web browser written in Aye, for people and AI agents at once. The
design, the milestones and their state are in `docs/browser.md`; the
agent protocol is in `docs/agent-protocol.md`.

## Build and run

From the repository root, with SDL2, FreeType and libcurl installed
(`pkg-config` must find them); HarfBuzz (text shaping: Arabic, Indic
scripts, icon font ligatures) and libbrotlidec (WOFF 2 web fonts) are
used when they are installed. The first build also needs libclang, for the
`@cimport`s (`pthread.h` in `js/js.aye`, the vendored `wasmtime.h` and
`angle.h`; macOS has it with the Command Line Tools; later builds read
the declarations from `~/.cache/aye`). Two libraries are optional and
fetched by a script: libwasmtime (`browser/wasm/fetch_wasmtime.sh`;
without it `WebAssembly` is undefined) and ANGLE
(`browser/gpu/fetch_angle.sh`; without it there is no WebGL).

The compiler is the `aye` of the revision in `AYE_REV` (the root
`README.md` has the install line), on the PATH or named by `AYE`;
`browser/aye.toml` names the executable:

```
aye build browser                               # writes ./dweller
AYE=/path/to/aye browser/build.sh               # safe mode, target/dweller
browser/build.sh [OUT] [--mode safe|fast|debug] [--coverage]
```

Run it:

```
./dweller https://example.com/                 # a window
./dweller dump --md https://example.com/       # the page as Markdown
./dweller dump --a11y https://example.com/     # the accessibility tree
./dweller agent                                # JSON lines on stdio
./dweller mcp                                  # an MCP server on stdio
```

`AYE=$(command -v aye) cargo test --release --test browser` runs the
browser's tests (headless, offline; `DWELLER_BLESS=1` rewrites the
goldens), and `aye test browser` the unit tests.

`DWELLER_PROCESS=tab` runs each tab's page in a sandboxed renderer
process and `DWELLER_PROCESS=site` also each cross-site iframe
(`docs/process-model.md`); unset, pages run in the window's process.
`DWELLER_PROFILE` names the profile directory (`~/.dweller` by
default) and `DWELLER_DOWNLOADS` the download directory (`~/Downloads`
by default).

Before coding, read [`docs/practices/browser.md`](../docs/practices/browser.md);
when you learn a lesson that will come up again, add an entry there.

## Use with Claude Code / MCP clients

Two ways, depending on whether a person should see the page.

**Without a window.** `dweller mcp` is an MCP server on stdio: the
agent gets its own invisible browser.

```
claude mcp add dweller -- /path/to/dweller mcp
```

**In your window.** Open a window that listens for agents, then let the
client connect to it through `dweller connect`, a bridge between stdio
and the window's socket:

```
/path/to/dweller --agent-socket ~/.dweller/agent.sock https://example.com/ &
claude mcp add dweller -- /path/to/dweller connect ~/.dweller/agent.sock --mcp
```

The socket's directory must be yours alone (`0700`; the window makes it
when it is missing), and every connection starts with the token the
window writes next to the socket (`agent.sock.token`); `dweller connect`
sends it for you (docs/agent-protocol.md, "A shared window").

The agent then works in a Space: tabs of its own, out of your tab
strip. It sees what you do through `browser_user_context`, and you see
what it does:

- a chip per agent (its Space) shows in the toolbar with its state, and
  the status bar tells its last action; a click on the chip (or
  Cmd+Shift+1..8) watches its tabs, "You" (or Cmd+Shift+0) goes back to
  yours;
- **Pause** in the toolbar, or Cmd+Shift+P, stops it: its actions fail
  with "paused by user" until you resume (it can still look);
- Cmd+Shift+K points it at the element under the mouse (or the focused
  one) and your selection: it gets a `user_pointed` notification, and
  `browser_user_context` tells it what you select, hover, focus and
  pointed at;
- Cmd+T, Cmd+W and Cmd+1..9 open, close and show tabs.

Any MCP client that starts stdio servers works the same way: the command
is `dweller` with the arguments `connect ~/.dweller/agent.sock --mcp`. Without
`--mcp`, the connection speaks the JSON lines of `dweller agent`.
Socket paths are limited to about 100 bytes.
