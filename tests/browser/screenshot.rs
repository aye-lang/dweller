use super::*;

/// `browser/tests/paint/X.html`: `dweller screenshot --font fixed` in a
/// 400x300 viewport must write exactly `X.png`; `full.html` is also taken
/// with `--full` into `full.full.png`.
#[test]
fn screenshot_fixtures() {
    let dir = root().join("browser").join("tests").join("paint");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 10,
        "expected at least 10 paint fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(String, Vec<&str>, PathBuf)> = htmls
        .iter()
        .map(|h| {
            (
                h.strip_prefix(root())
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                vec![],
                h.with_extension("png"),
            )
        })
        .collect();
    runs.push((
        "browser/tests/paint/full.html".into(),
        vec!["--full"],
        dir.join("full.full.png"),
    ));
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let out_dir = scratch();
    let mut failures = Vec::new();
    for (i, (page, extra, golden)) in runs.iter().enumerate() {
        let out = out_dir.join(format!("shot{i}.png"));
        let mut args = vec![
            "screenshot",
            page.as_str(),
            "-o",
            out.to_str().unwrap(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
        ];
        args.extend(extra.iter());
        let res = run(&args);
        if !res.status.success() || !res.stderr.is_empty() {
            failures.push(format!(
                "{page} {extra:?}: {}\n{}",
                res.status,
                text(&res.stderr)
            ));
            continue;
        }
        let got = std::fs::read(&out).unwrap();
        if bless {
            std::fs::write(golden, &got).unwrap();
            continue;
        }
        let want = std::fs::read(golden).unwrap_or_default();
        if got != want {
            failures.push(format!(
                "{page} {extra:?}: the image differs from {} (new one: {})",
                golden.display(),
                out.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} screenshots failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}

/// Q1.10d2: a JPEG whose decode passes the 256 MiB cap decodes at 1/2,
/// but lays out, reports and draws at the size in its header.
#[test]
fn downscaled_jpeg_natural_size() {
    let page = "browser/tests/image/cap_natural.html";
    let out = run(&["dump", "--text", "--js", page]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let got = text(&out.stdout);
    assert!(
        got.contains("natural 6324x6324 width 6324x6324 box 6324x6324 drawn 128,128,128,255"),
        "{got}"
    );
    let layout = text(&run(&["dump", "--layout", page]).stdout);
    assert!(layout.contains("replaced <img id=\"i\" src=\"cap_large.jpg\"> (0, 0) 6324x6324"), "{layout}");
}

#[test]
fn png_validity() {
    let out = scratch().join("valid.png");
    let res = run(&[
        "screenshot",
        "browser/tests/article.html",
        "-o",
        out.to_str().unwrap(),
        "--font",
        "fixed",
        "--width",
        "320",
        "--height",
        "200",
    ]);
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}",
        text(&res.stderr)
    );
    assert_eq!(check_png(&out), (320, 200));
    let full = scratch().join("valid_full.png");
    let res = run(&[
        "screenshot",
        "browser/tests/paint/full.html",
        "-o",
        full.to_str().unwrap(),
        "--font",
        "fixed",
        "--width",
        "300",
        "--height",
        "100",
        "--full",
    ]);
    assert!(res.status.success(), "{}", text(&res.stderr));
    assert_eq!(check_png(&full), (300, 416));
    for golden in std::fs::read_dir(root().join("browser/tests/paint")).unwrap() {
        let p = golden.unwrap().path();
        if p.extension().is_some_and(|e| e == "png") {
            check_png(&p);
        }
    }
}

/// `--font system` draws FreeType glyphs; skipped without a system font.
#[test]
fn screenshot_system_font() {
    let out = scratch().join("system.png");
    let res = run(&[
        "screenshot",
        "browser/tests/article.html",
        "-o",
        out.to_str().unwrap(),
        "--font",
        "system",
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
    assert_eq!(check_png(&out), (800, 600));
    let fixed = scratch().join("system_fixed.png");
    run(&[
        "screenshot",
        "browser/tests/article.html",
        "-o",
        fixed.to_str().unwrap(),
        "--font",
        "fixed",
    ]);
    assert_ne!(std::fs::read(&out).unwrap(), std::fs::read(&fixed).unwrap());
}

/// The page's scripts see the screenshot's viewport: a lazy loader loads
/// an image 700 px down in a 900 px tall screenshot (P3.45).
#[test]
fn screenshot_viewport_lazy_image() {
    let page = "browser/tests/screenshot/lazy_below.html";
    let lazy = scratch().join("lazy_below.png");
    let eager = scratch().join("lazy_below_eager.png");
    let res = run(&[
        "screenshot",
        page,
        "-o",
        lazy.to_str().unwrap(),
        "--font",
        "fixed",
        "--width",
        "200",
        "--height",
        "900",
    ]);
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}",
        text(&res.stderr)
    );
    let eager_page = format!("{page}#eager");
    let res = run(&[
        "screenshot",
        &eager_page,
        "-o",
        eager.to_str().unwrap(),
        "--font",
        "fixed",
        "--width",
        "200",
        "--height",
        "900",
    ]);
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}",
        text(&res.stderr)
    );
    assert!(
        std::fs::read(&lazy).unwrap() == std::fs::read(&eager).unwrap(),
        "the lazy image is missing: {}",
        lazy.display()
    );
}

#[test]
fn screenshot_errors() {
    let usage = "error: usage: dweller screenshot FILE|URL -o OUT.png [--width W] [--height H] [--font fixed|system] [--full] [--wait-class NAME]\n";
    expect(&["screenshot", "browser/tests/basic.html"], 2, "", usage);
    expect(&["screenshot", "-o", "x.png"], 2, "", usage);
    expect(
        &[
            "screenshot",
            "browser/tests/basic.html",
            "-o",
            "x.png",
            "--width",
            "0",
        ],
        2,
        "",
        usage,
    );
    expect(
        &[
            "screenshot",
            "browser/tests/missing.html",
            "-o",
            "x.png",
            "--font",
            "fixed",
        ],
        1,
        "",
        &format!(
            "error: cannot fetch file://{}/browser/tests/missing.html: not found\n",
            root().display()
        ),
    );
}
