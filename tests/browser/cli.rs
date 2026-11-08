use super::*;

/// `--font system` measures text with the system's fonts through
/// FreeType; skipped when none of the known font files is there.
#[test]
fn layout_system_font() {
    let page = "browser/tests/layout/nested.html";
    let out = run(&["dump", "--layout", "--font", "system", page]);
    if text(&out.stderr).contains("no system font found") {
        println!("skipped: no system font");
        return;
    }
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    assert!(
        got.starts_with("viewport 800x600\n  block <html> (0, 0) 800x"),
        "{got}"
    );
    assert!(got.contains("\"The Ore Times\""), "{got}");
    // Proportional glyphs: widths differ from the fixed-metric font's.
    let fixed = text(&run(&["dump", "--layout", page]).stdout);
    assert_ne!(got, fixed);
}

#[test]
fn aye_unit_tests() {
    // The tests of every module under `browser` (15 modules have tests:
    // the root and 14 submodules), in one run.
    let out = aye().args(["test", "browser"]).output().unwrap();
    let stdout = text(&out.stdout);
    assert!(
        out.status.success() && stdout.contains(" passed; 0 failed"),
        "`aye test browser` failed:\n{stdout}{}",
        text(&out.stderr)
    );
    let summary = stdout
        .lines()
        .rev()
        .find(|l| l.contains(" passed; "))
        .unwrap();
    let modules: usize = summary
        .split("; ")
        .find_map(|p| p.strip_suffix(" modules"))
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("no module count in `{summary}`"));
    assert!(modules >= 15, "only {modules} modules ran tests: {summary}");
}

#[test]
fn data_urls() {
    expect(
        &["dump", "--text", "data:text/html,<h1>Hi</h1><p>there%21"],
        0,
        "Hi\n\nthere!\n",
        "",
    );
    expect(
        &["dump", "--tree", "data:text/html;base64,PHA+Q2Fm6TwvcD4="],
        0,
        "#document\n  <html>\n    <head>\n    <body>\n      <p>\n        \"Café\"\n",
        "",
    );
    // Plain text is shown as it is.
    expect(
        &["dump", "--text", "data:,a  <b>%0A  c"],
        0,
        "a  <b>\n  c\n",
        "",
    );
}

#[test]
fn queries() {
    let page = "browser/tests/basic.html";
    expect(
        &["dump", "--tree", "--id", "main", page],
        0,
        "<div id=\"main\" class=\"content\">\n  \"\\n    \"\n  <p>\n    \"Second paragraph.\"\n  \"\\n  \"\n",
        "",
    );
    expect(
        &["dump", "--text", "--tag", "P", page],
        0,
        "This is a small page with a link.\nSecond paragraph.\n",
        "",
    );
    expect(&["dump", "--text", "--id", "none", page], 0, "", "");
}

#[test]
fn errors() {
    let usage = "error: usage: dweller dump --tree|--text|--style|--layout [--js [--no-scroll | --scroll-steps N]] [--width PX] [--dpr N] [--font fixed|system] [--id ID | --tag NAME] FILE|URL\n";
    expect(
        &[],
        2,
        "",
        "error: usage: dweller [open] FILE|URL | dump --tree|--text|--style|--layout|--a11y|--md ... FILE|URL | screenshot FILE|URL -o OUT.png | js [--print-bytecode|--test262] FILE | wpt FILE|URL [--timeout S] | gpu-smoke | fuzz TARGET [--seconds N] [--corpus DIR] [--seed S] | fuzz --list | agent | mcp | connect SOCKET\n",
    );
    expect(&["dump", "--tree"], 2, "", usage);
    expect(
        &[
            "dump",
            "--layout",
            "--font",
            "serif",
            "browser/tests/basic.html",
        ],
        2,
        "",
        usage,
    );
    expect(
        &["dump", "--tree", "browser/tests/missing.html"],
        1,
        "",
        &format!(
            "error: cannot fetch file://{}/browser/tests/missing.html: not found\n",
            root().display()
        ),
    );
    expect(
        &["dump", "--text", "ftp://example.com/"],
        1,
        "",
        "error: cannot fetch ftp://example.com/: this kind of URL is not supported yet\n",
    );
    expect(
        &["dump", "--text", "data:text/html;base64,%%%"],
        1,
        "",
        "error: cannot fetch data:text/html;base64,%%%: invalid data: URL\n",
    );
}

/// `text-overflow: ellipsis` is only drawn: the Markdown keeps the whole
/// text (the layout fixture `text_overflow.html` checks the ellipsis).
#[test]
fn text_overflow_markdown() {
    let page = "data:text/html,<div style='width:100px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis'>A very long title</div>";
    expect(&["dump", "--md", page], 0, "A very long title\n", "");
}

// ---- B8: HTML and CSS

/// `noscript` follows scripting: the dumps without `--js` show it (the
/// agent fixture `noscript`), with `--js` they leave it out.
#[test]
fn noscript_with_scripts() {
    let page = "browser/tests/agent/noscript.html";
    expect(
        &["dump", "--text", "--js", page],
        0,
        "Scripts\n\nAlways here.\n\nAn icon and a formula .\n",
        "",
    );
    expect(
        &["dump", "--md", "--js", page],
        0,
        "# Scripts\n\nAlways here.\n\nAn icon and a formula .\n",
        "",
    );
    expect(
        &["dump", "--a11y", "--js", page],
        0,
        "- heading \"Scripts\" [level=1] [e1]\n- paragraph: Always here.\n- paragraph: An icon and a formula .\n",
        "",
    );
}

/// `dump --js` scrolls through the page (`engine.scrollThrough`), so what
/// scripts load when it comes into view is in the dump: sections an
/// IntersectionObserver fills, and an infinite list or a page that never
/// stops growing until two steps made the document taller (what a reader
/// sees near the top, not a whole feed). `--no-scroll` turns it off;
/// `--scroll-steps N` caps it. The goldens of these fixtures are in
/// `browser/tests/jsdom/` too, with those of the pages it does not step
/// through (`unwatched_page`), stops after one step (`quiet_page`) or
/// takes to the bottom after a step that changed nothing (`lively_page`).
#[test]
fn dump_scrolls_through() {
    let page = |name: &str| format!("browser/tests/jsdom/{name}.html");
    let md = |flags: &[&str], name: &str| -> String {
        let p = page(name);
        let mut all = vec!["dump", "--md", "--js"];
        all.extend_from_slice(flags);
        all.push(&p);
        let out = run(&all);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{all:?}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    let sections = md(&[], "lazy_sections");
    assert!(
        sections.contains("Fourth section, at the bottom.")
            && sections.contains("scroll events: 3; image loaded at"),
        "{sections}"
    );
    // The native lazy image loads once the scroll brings it near.
    let sections = md(&["--no-scroll"], "lazy_sections");
    assert!(
        !sections.contains("First section")
            && sections.contains("scroll events: 0; image loading"),
        "{sections}"
    );
    let list = md(&[], "infinite_list");
    assert!(
        list.contains("30. Item 30\n\nLoading more...\n") && !list.contains("Item 31"),
        "{list}"
    );
    let list = md(&["--scroll-steps", "3"], "infinite_list");
    assert!(
        list.contains("30. Item 30\n\nLoading more...\n") && !list.contains("Item 31"),
        "{list}"
    );
    assert_eq!(
        md(&["--no-scroll"], "infinite_list"),
        "# Infinite list\n\nLoading more...\n"
    );
    assert_eq!(md(&[], "endless_page"), "Chunk 1\n\nChunk 2\n\nChunk 3\n");
    // Five results at load and five after each scroll: two growth rounds.
    let feed = md(&[], "endless_feed");
    assert!(
        feed.ends_with("\n15. Result 15\n") && feed.contains("1. Result 1\n"),
        "{feed}"
    );
    assert_eq!(
        md(&["--scroll-steps", "2"], "endless_page"),
        "Chunk 1\n\nChunk 2\n\nChunk 3\n"
    );
    expect(
        &[
            "dump",
            "--text",
            "--js",
            "--scroll-steps",
            "2",
            &page("endless_page"),
        ],
        0,
        "Endless page\n\nChunk 1\nChunk 2\nChunk 3\n",
        "",
    );
    expect(
        &[
            "dump",
            "--text",
            "--js",
            "--no-scroll",
            &page("endless_page"),
        ],
        0,
        "Endless page\n\nChunk 1\n",
        "",
    );
    let usage = "error: usage: dweller dump --a11y [--boxes] [--json] | --md [--refs] [--js] [--width PX] [--no-scroll | --scroll-steps N] FILE|URL\n";
    expect(
        &[
            "dump",
            "--md",
            "--js",
            "--scroll-steps",
            "-1",
            &page("endless_page"),
        ],
        2,
        "",
        usage,
    );
}

/// `browser/tests/fonts/bold.html`: bold text from a face of one weight
/// gets synthetic bold, and from a variable face its `wght` axis, so the
/// bold rows have more ink than the regular ones (and the variable bold
/// row is wider: its advances grow). Skipped without a system font or
/// python3.
#[test]
fn bold_weights() {
    let out = scratch().join("font_bold.png");
    let res = run(&[
        "screenshot",
        "browser/tests/fonts/bold.html",
        "-o",
        out.to_str().unwrap(),
        "--font",
        "system",
        "--width",
        "400",
        "--height",
        "240",
    ]);
    if text(&res.stderr).contains("no system font found") {
        println!("skipped: no system font");
        return;
    }
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}\n{}",
        res.status,
        text(&res.stderr)
    );
    // Ink (255 - red, summed) and the rightmost inked column of each
    // 60 px row; the PNG encoder uses filter 0.
    let script = r#"
import struct, sys, zlib
data = open(sys.argv[1], 'rb').read()
w, h = struct.unpack('>II', data[16:24])
chunks, offset = [], 8
while offset < len(data):
    n = int.from_bytes(data[offset:offset+4], 'big')
    if data[offset+4:offset+8] == b'IDAT':
        chunks.append(data[offset+8:offset+8+n])
    offset += n + 12
raw = zlib.decompress(b''.join(chunks))
stride = w * 4 + 1
for band in range(4):
    ink, right = 0, 0
    for y in range(band * 60, band * 60 + 60):
        for x in range(w):
            v = 255 - raw[y * stride + 1 + x * 4]
            if v > 64:
                right = max(right, x)
            ink += v
    print(ink, right)
"#;
    let Ok(py) = Command::new("python3")
        .args(["-c", script, out.to_str().unwrap()])
        .output()
    else {
        println!("skipped: no python3");
        return;
    };
    assert!(py.status.success(), "{}", text(&py.stderr));
    let rows: Vec<(u64, u64)> = text(&py.stdout)
        .lines()
        .map(|l| {
            let mut f = l.split(' ').map(|n| n.parse::<u64>().unwrap());
            (f.next().unwrap(), f.next().unwrap())
        })
        .collect();
    assert_eq!(rows.len(), 4, "{rows:?}");
    assert!(rows[0].0 > 0 && rows[2].0 > 0, "{rows:?}");
    assert!(
        rows[1].0 * 10 > rows[0].0 * 11,
        "synthetic bold adds no ink: {rows:?}"
    );
    assert!(
        rows[3].0 * 10 > rows[2].0 * 15,
        "the variable face is not drawn bold: {rows:?}"
    );
    assert!(
        rows[3].1 > rows[2].1 + 5,
        "the variable face's bold advances do not grow: {rows:?}"
    );
}

/// `browser/tests/fonts/`: a tiny font as TrueType, WOFF and WOFF 2
/// (`@font-face`, system fonts): the three pages must look the same, and
/// unlike the page without the font. WOFF 2 is checked where
/// libbrotlidec is installed; skipped without a system font.
#[test]
fn web_font_files() {
    let shot = |page: &str| -> Option<Vec<u8>> {
        let out = scratch().join(format!("font_{page}.png"));
        let path = format!("browser/tests/fonts/{page}.html");
        let res = run(&[
            "screenshot",
            &path,
            "-o",
            out.to_str().unwrap(),
            "--font",
            "system",
            "--width",
            "400",
            "--height",
            "80",
        ]);
        if text(&res.stderr).contains("no system font found") {
            return None;
        }
        assert!(
            res.status.success() && res.stderr.is_empty(),
            "{page}: {}\n{}",
            res.status,
            text(&res.stderr)
        );
        Some(std::fs::read(&out).unwrap())
    };
    let Some(ttf) = shot("ttf") else {
        println!("skipped: no system font");
        return;
    };
    assert_ne!(
        ttf,
        shot("none").unwrap(),
        "the TrueType web font is not used"
    );
    assert_eq!(ttf, shot("woff").unwrap(), "WOFF renders unlike TrueType");
    let brotli = Command::new("pkg-config")
        .args(["--exists", "libbrotlidec"])
        .status()
        .is_ok_and(|s| s.success());
    if brotli {
        assert_eq!(
            ttf,
            shot("woff2").unwrap(),
            "WOFF 2 renders unlike TrueType"
        );
    } else {
        println!("WOFF 2 skipped: no libbrotlidec");
    }
}
