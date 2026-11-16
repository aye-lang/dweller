# Native apps on Dweller's engine

`counter/` is the V2 spike of `docs/vision.md`: an Aye program that opens
a window and runs a small app on the browser's engine. The document is
built from code (`dom.Document` is the scene graph), styled by a CSS
string, laid out and painted by the browser's own `css/`, `layout/` and
`paint/`. No HTML parser runs, and `js/`, `engine/` and `net/` are not in
the binary.

```
aye build examples/native/counter -o target/counter --mode fast
target/counter
```

The app is `counter/main.aye` and `counter/counter.css`. Its `aye.toml`
names the engine as a path dependency, and the app imports
`dweller.shell` (the window, `browser/shell/`) and `dweller.dom`.
`shell.run(alloc, css, build)` opens the window: `build` fills
`app.body` (`app.element`, `app.setAttr`, `app.setText`) and names the
click handlers (`app.onClick(element, handler)`). A handler changes the
document; the next frame styles and lays out only what changed.

`counter/counter.html` is the same app as a web page, for comparing the
two (`dweller counter.html`).

Headless runs, as the browser's window has them:

```
SDL_VIDEODRIVER=dummy target/counter --font fixed --frames 8 \
    --events "click:36,134 frame text:hello wheel:-3" \
    --save out.png --dump --timing
```

`--events` takes `click:X,Y`, `wheel:Y`, `text:STRING` (`_` is a space),
`key:NAME` (backspace, delete, left, right, home, end) and `frame`.
`--dump` prints the document, `--timing` the time to the first frame, the
frame times and how many layouts were full or partial, and
`--full-layout` turns the partial layout off (the test compares the two).
