# Dweller terminal (V4 spike)

A terminal emulator on Dweller's text stack: a PTY, a VT/xterm parser, a
cell grid drawn with `paint.TextPainter` (FreeType, HarfBuzz, the glyph
cache) into an SDL window. `docs/vision.md` has the direction, the card is
`docs/plans/cards/V4.md`.

```
aye build apps/term -o target/term --mode fast
target/term                      # the user's shell ($SHELL)
target/term --exec "vim notes.txt"
```

- `vt/`: the parser (the state machine of vt100.net's DEC parser) and the
  screen model: two screens (primary and alternate), cells with a code
  point, colors and attributes, the scroll region, scrollback (10 000
  rows), the title, replies to reports, and a dirty flag per row.
  `vt.dump` prints the grid for tests.
- the files beside `main.aye`: the PTY (`forkpty`), the window loop, the
  drawing of dirty rows, the keyboard (`key-bytes.aye`).

Keys: Shift+PageUp and Shift+PageDown, or the wheel, scroll the
scrollback; typing goes back to the bottom.

For tests and for looking at a stream without a window:

```
target/term dump FILE.bytes                # the cell grid as text
target/term shot FILE.bytes OUT.png        # drawn with the fixed test font
target/term shot FILE.bytes OUT.png --font system
SDL_VIDEODRIVER=dummy target/term --exec "COMMAND" --type "keys\r" --dump --save OUT.png
```

`tests/*.bytes` are recorded with `tools/record.py` (vim, htop, less on
an 80x24 pty) or written by `tools/synthetic.py` (colors, modes);
`tests/*.grid` and `*.png` are the goldens of `cargo test --test browser
-- term_`. `tools/reference.py FILE.bytes` prints the screen text
according to pyte, another emulator, to check a golden before blessing.
