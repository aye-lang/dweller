//! Tests of the Dweller browser (`browser/`, written in Aye).
//!
//! - `browser/tests/X.html`: `dweller dump --tree` must print `X.tree` and
//!   `dweller dump --text` must print `X.txt`, with an empty stderr (the
//!   browser is built in debug mode, so a leak would show there).
//! - `aye test` on the browser's root module and on its standalone modules
//!   (`url`, `dom`, `encoding`) must pass.
//! - A few command-line cases: `data:` URLs, `--id`/`--tag`, errors.
//! - `browser/tests/js/X.js`: `dweller js` must print `X.out` (Node's
//!   output) and exit 0 with an empty stderr, or, when `X.err` exists,
//!   print it on stderr and exit 1 (an uncaught exception).
//! - `browser/tests/js/scopes/X.js` (or `.mjs`): `dweller js
//!   --print-scopes` must print `X.scopes` (the resolver's register and
//!   context allocation, P4.62).
//!
//! To accept new output after a deliberate change, run
//! `DWELLER_BLESS=1 cargo test --test browser` and review the diff.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

mod common;

#[path = "browser/perf.rs"]
mod perf;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn aye() -> Command {
    let mut c = Command::new(std::env::var_os("AYE").unwrap_or("aye".into()));
    c.current_dir(root());
    // libwasmtime from browser/wasm/fetch_wasmtime.sh, when fetched: its
    // pkg-config file links it with an rpath (P11.17).
    if let Some(pc) = wasmtime_pkgconfig() {
        let mut path = std::ffi::OsString::from(&pc);
        if let Some(old) = std::env::var_os("PKG_CONFIG_PATH") {
            path.push(":");
            path.push(old);
        }
        c.env("PKG_CONFIG_PATH", path);
    }
    c
}

/// target/wasmtime/<version>/lib/pkgconfig, the version of
/// browser/wasm/wasmtime.lock, when browser/wasm/fetch_wasmtime.sh made it.
fn wasmtime_pkgconfig() -> Option<PathBuf> {
    let lock = std::fs::read_to_string(root().join("browser/wasm/wasmtime.lock")).ok()?;
    let version = lock.lines().find_map(|l| l.strip_prefix("version "))?;
    let dir = root()
        .join("target/wasmtime")
        .join(version.trim())
        .join("lib/pkgconfig");
    dir.join("wasmtime.pc").exists().then_some(dir)
}

/// A child whose stdout and stderr are read on threads while it runs
/// (`SpawnDrained`), so that a window's verbose output (DWELLER_TIMING)
/// cannot fill a pipe and block it while the test waits for it to exit.
struct Drained {
    child: std::process::Child,
    out: std::thread::JoinHandle<Vec<u8>>,
    err: std::thread::JoinHandle<Vec<u8>>,
}

impl Drained {
    fn id(&self) -> u32 {
        self.child.id()
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    /// Waits for the child alone (what it started may keep its pipes).
    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait()
    }

    /// Waits for the child to exit and for its pipes to close.
    fn wait_with_output(mut self) -> std::io::Result<Output> {
        let status = self.child.wait()?;
        let stdout = self.out.join().unwrap_or_default();
        let stderr = self.err.join().unwrap_or_default();
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }
}

trait SpawnDrained {
    /// Spawns the command with its stdout and stderr piped and read on
    /// threads (`Drained`).
    fn spawn_drained(&mut self) -> Drained;
}

impl SpawnDrained for Command {
    fn spawn_drained(&mut self) -> Drained {
        use std::io::Read;
        let mut child = self
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = stdout.read_to_end(&mut b);
            b
        });
        let err = std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = stderr.read_to_end(&mut b);
            b
        });
        Drained { child, out, err }
    }
}

/// The browser, built once for all tests.
fn dweller() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("dweller-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("dweller");
        let out = aye()
            .arg("build")
            .arg("browser")
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "building the browser failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        exe
    })
}

fn run(args: &[&str]) -> Output {
    Command::new(dweller())
        .current_dir(root())
        .args(args)
        .output()
        .unwrap()
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

#[test]
fn fixtures() {
    let dir = root().join("browser").join("tests");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 20,
        "expected at least 20 fixtures, found {}",
        htmls.len()
    );
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for html in &htmls {
        let rel = html
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for (flag, ext) in [("--tree", "tree"), ("--text", "txt")] {
            let out = run(&["dump", flag, &rel]);
            let got = text(&out.stdout);
            let golden = html.with_extension(ext);
            if !out.status.success() || !out.stderr.is_empty() {
                failures.push(format!(
                    "{rel} {flag}: {}\n{}",
                    out.status,
                    text(&out.stderr)
                ));
                continue;
            }
            if bless {
                std::fs::write(&golden, &got).unwrap();
                continue;
            }
            let want = std::fs::read_to_string(&golden).unwrap_or_default();
            if got != want {
                failures.push(format!(
                    "{rel} {flag}: output differs from {}\n--- expected\n{want}--- got\n{got}",
                    golden.display()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} fixture runs failed:\n\n{}",
        failures.len(),
        htmls.len() * 2,
        failures.join("\n")
    );
}

/// `browser/tests/style/X.html`: `dweller dump --style` must print
/// `X.style`; `media.html` is also dumped 400px wide into
/// `media.w400.style`.
#[test]
fn style_fixtures() {
    let dir = root().join("browser").join("tests").join("style");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 20,
        "expected at least 20 style fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(Vec<String>, PathBuf)> = htmls
        .iter()
        .map(|h| {
            (
                vec![
                    h.strip_prefix(root())
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ],
                h.with_extension("style"),
            )
        })
        .collect();
    runs.push((
        vec![
            "--width".into(),
            "400".into(),
            "browser/tests/style/media.html".into(),
        ],
        dir.join("media.w400.style"),
    ));
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for (args, golden) in &runs {
        let mut all = vec!["dump", "--style"];
        all.extend(args.iter().map(|s| s.as_str()));
        let out = run(&all);
        let got = text(&out.stdout);
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{args:?}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        if bless {
            std::fs::write(golden, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(golden).unwrap_or_default();
        if got != want {
            failures.push(format!(
                "{args:?}: output differs from {}\n--- expected\n{want}--- got\n{got}",
                golden.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} style runs failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}

/// `browser/tests/layout/X.html`: `dweller dump --layout` (fixed-metric
/// font, 800x600 viewport) must print `X.layout`; `percentages.html` is
/// also laid out 400px wide into `percentages.w400.layout`.
#[test]
fn layout_fixtures() {
    let dir = root().join("browser").join("tests").join("layout");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 25,
        "expected at least 25 layout fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(Vec<String>, PathBuf)> = htmls
        .iter()
        .map(|h| {
            (
                vec![
                    h.strip_prefix(root())
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ],
                h.with_extension("layout"),
            )
        })
        .collect();
    runs.push((
        vec![
            "--width".into(),
            "400".into(),
            "browser/tests/layout/percentages.html".into(),
        ],
        dir.join("percentages.w400.layout"),
    ));
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for (args, golden) in &runs {
        let mut all = vec!["dump", "--layout"];
        all.extend(args.iter().map(|s| s.as_str()));
        let out = run(&all);
        let got = text(&out.stdout);
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{args:?}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        if bless {
            std::fs::write(golden, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(golden).unwrap_or_default();
        if got != want {
            failures.push(format!(
                "{args:?}: output differs from {}\n--- expected\n{want}--- got\n{got}",
                golden.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} layout runs failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}

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

/// `dweller gpu-smoke` on SwiftShader (P11.2, docs/webgl.md): a 64x64
/// framebuffer cleared to red reads back `ff0000ff`. ANGLE comes from
/// `DWELLER_ANGLE` or `target/angle/<rev>/lib` (browser/gpu/fetch_angle.sh).
/// Until browser/gpu/angle.lock has hashes, CI has no ANGLE: then the test
/// checks the "WebGL unavailable" path and skips the pixels.
#[test]
fn gpu_smoke() {
    let rev = std::fs::read_to_string(root().join("browser/gpu/angle.rev")).unwrap();
    let lib = root().join("target/angle").join(rev.trim()).join("lib");
    let present = std::env::var_os("DWELLER_ANGLE").is_some()
        || ["libEGL.dylib", "libEGL.so"]
            .iter()
            .any(|f| lib.join(f).exists());
    let out = Command::new(dweller())
        .current_dir(root())
        .arg("gpu-smoke")
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    let got = text(&out.stdout);
    assert!(
        out.status.success(),
        "{}\n{got}\n{}",
        out.status,
        text(&out.stderr)
    );
    if !present {
        let lock = std::fs::read_to_string(root().join("browser/gpu/angle.lock")).unwrap();
        let pinned = lock
            .lines()
            .any(|l| !l.starts_with('#') && !l.trim().is_empty());
        assert!(
            !pinned,
            "ANGLE not found in {}: run browser/gpu/fetch_angle.sh",
            lib.display()
        );
        assert!(got.contains("WebGL unavailable: ANGLE not found"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    assert!(got.starts_with("backend swiftshader\n"), "{got}");
    assert!(got.contains("SwiftShader"), "{got}");
    assert!(got.ends_with("ff0000ff\nok\n"), "{got}");
}

/// Whether ANGLE is there for the WebGL tests: `DWELLER_ANGLE` or
/// `target/angle/<rev>/lib`. Without it, the WebGL tests check the
/// unavailable path, and only while browser/gpu/angle.lock has no hashes
/// (CI fetches ANGLE once it has them).
fn angle_present() -> bool {
    let rev = std::fs::read_to_string(root().join("browser/gpu/angle.rev")).unwrap();
    let lib = root().join("target/angle").join(rev.trim()).join("lib");
    let present = std::env::var_os("DWELLER_ANGLE").is_some()
        || ["libEGL.dylib", "libEGL.so"]
            .iter()
            .any(|f| lib.join(f).exists());
    if !present {
        let lock = std::fs::read_to_string(root().join("browser/gpu/angle.lock")).unwrap();
        let pinned = lock
            .lines()
            .any(|l| !l.starts_with('#') && !l.trim().is_empty());
        assert!(
            !pinned,
            "ANGLE not found in {}: run browser/gpu/fetch_angle.sh",
            lib.display()
        );
    }
    present
}

/// P11.4 (docs/webgl.md §2, §3): `dump --text --js` of
/// `browser/tests/webgl/context.html` on SwiftShader prints its `.txt`:
/// the context attributes, drawing buffer size, limits, `getError`
/// sequences, cleared pixels, and the drawing buffer cleared after a
/// frame. Without ANGLE, `getContext('webgl2')` gives null.
#[test]
fn webgl_context() {
    let page = "browser/tests/webgl/context.html";
    let out = Command::new(dweller())
        .current_dir(root())
        .args(["dump", "--text", "--js", page])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(
            got.contains("webgl2 null\n2d after a failed webgl2 true"),
            "{got}"
        );
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/context.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.4: `dweller screenshot` of `browser/tests/webgl/clear.html` on
/// SwiftShader shows its WebGL canvases (scissored clears, premultiplied
/// alpha over the page, a frame drawn in requestAnimationFrame) exactly
/// as `clear.png`. Without ANGLE the canvases stay blank.
#[test]
fn webgl_screenshot() {
    let out_png = scratch().join("webgl_clear.png");
    let res = Command::new(dweller())
        .current_dir(root())
        .args([
            "screenshot",
            "browser/tests/webgl/clear.html",
            "-o",
            out_png.to_str().unwrap(),
            "--font",
            "fixed",
            "--width",
            "200",
            "--height",
            "180",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}\n{}",
        res.status,
        text(&res.stderr)
    );
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let got = std::fs::read(&out_png).unwrap();
    let golden = root().join("browser/tests/webgl/clear.png");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    assert!(
        got == std::fs::read(&golden).unwrap(),
        "the image differs from {} (new one: {})",
        golden.display(),
        out_png.display()
    );
}

/// P11.5 (docs/webgl.md §3, §4): `dweller screenshot` on SwiftShader of
/// the pages in `browser/tests/webgl` that draw: a triangle with
/// per-vertex colors (`triangle.html`), an indexed quad with a uniform
/// color and matrix (`quad.html`), and the info logs of a shader that
/// does not compile and a program that does not link
/// (`shader_errors.html`); each is its `.png`. Without ANGLE the canvases
/// stay blank.
#[test]
fn webgl_triangle() {
    for page in ["triangle", "quad", "shader_errors"] {
        let out_png = scratch().join(format!("webgl_{page}.png"));
        let res = Command::new(dweller())
            .current_dir(root())
            .args([
                "screenshot",
                &format!("browser/tests/webgl/{page}.html"),
                "-o",
                out_png.to_str().unwrap(),
                "--font",
                "fixed",
                "--width",
                "200",
                "--height",
                "160",
            ])
            .env("DWELLER_GL", "swiftshader")
            .output()
            .unwrap();
        assert!(
            res.status.success() && res.stderr.is_empty(),
            "{page}: {}\n{}",
            res.status,
            text(&res.stderr)
        );
        if !angle_present() {
            println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
            return;
        }
        let got = std::fs::read(&out_png).unwrap();
        let golden = root().join(format!("browser/tests/webgl/{page}.png"));
        if std::env::var_os("DWELLER_BLESS").is_some() {
            std::fs::write(&golden, &got).unwrap();
            continue;
        }
        assert!(
            got == std::fs::read(&golden).unwrap(),
            "the image differs from {} (new one: {})",
            golden.display(),
            out_png.display()
        );
    }
}

/// P11.5: `dump --text --js` of `browser/tests/webgl/objects.html` on
/// SwiftShader prints `objects.txt`: the error of each WebGL object rule
/// (another context, deleted objects, buffer targets, uniform locations
/// after a relink, draws), active attributes and uniforms, and the pixels
/// of a drawn triangle.
#[test]
fn webgl_objects() {
    let out = Command::new(dweller())
        .current_dir(root())
        .args(["dump", "--text", "--js", "browser/tests/webgl/objects.html"])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/objects.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.6 (docs/webgl.md §3): `dump --text --js` of
/// `browser/tests/webgl/textures.html` on SwiftShader, served over HTTP
/// (the page loads an image from a second origin), prints
/// `textures.txt`: a textured quad from a PNG, a 2D canvas, an
/// OffscreenCanvas, an ImageData, ImageBitmaps and typed arrays read back
/// with and without UNPACK_FLIP_Y_WEBGL and UNPACK_PREMULTIPLY_ALPHA_WEBGL,
/// the object and pixel store rules, and the SecurityError of a
/// cross-origin image. It samples with NEAREST at texel centers, so the
/// pixels do not depend on the backend.
#[test]
fn webgl_textures() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = Command::new(dweller())
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            &format!("{}/webgl/textures.html", srv.base),
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/textures.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.8 (docs/webgl.md §3): `dump --text --js` of
/// `browser/tests/webgl/webgl2_objects.html` on SwiftShader prints
/// `webgl2_objects.txt`: VAOs, instanced draws, drawRangeElements,
/// vertexAttribIPointer, a float MRT framebuffer sampled afterwards,
/// renderbuffers with a multisample blit and a depth test, uniform
/// buffers, samplers, 2D-array and 3D textures and float and integer
/// formats, each drawn and read back.
#[test]
fn webgl2_objects() {
    let out = Command::new(dweller())
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            "browser/tests/webgl/webgl2_objects.html",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/webgl2_objects.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.6b: `dump --text --js` of `browser/tests/webgl/imagebitmap.html`
/// on SwiftShader, served over HTTP (the page fetches a PNG), prints
/// `imagebitmap.txt`: createImageBitmap from a Blob, an ImageData and the
/// canvas sources, the crop rectangle (negative sizes, outside the
/// source, RangeError), imageOrientation, premultiplyAlpha, the resize
/// options, close(), and a Blob bitmap uploaded with texImage2D and read
/// back. The pixels of the bitmaps come through a 2D canvas.
#[test]
fn imagebitmap_sources() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = Command::new(dweller())
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            &format!("{}/webgl/imagebitmap.html", srv.base),
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/imagebitmap.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// `browser/tests/wasm/X.html` (P11.17, the WebAssembly JS API on
/// wasmtime): `dweller dump --text --js` must print `X.txt` with an empty
/// stderr. Without libwasmtime (browser/wasm/fetch_wasmtime.sh not run)
/// `WebAssembly` is undefined and each page prints "no WebAssembly"; CI
/// fetches it, so there it must be present.
#[test]
fn wasm_api() {
    let present = wasmtime_pkgconfig().is_some();
    assert!(
        present || std::env::var_os("CI").is_none(),
        "CI must run browser/wasm/fetch_wasmtime.sh"
    );
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for name in ["api"] {
        let page = format!("browser/tests/wasm/{name}.html");
        let golden = root().join(format!("browser/tests/wasm/{name}.txt"));
        let out = run(&["dump", "--text", "--js", &page]);
        let got = text(&out.stdout);
        if bless && present {
            std::fs::write(&golden, &got).unwrap();
            continue;
        }
        let want = if present {
            std::fs::read_to_string(&golden).unwrap()
        } else {
            "no WebAssembly\n".to_string()
        };
        if !out.status.success() || got != want || !out.stderr.is_empty() {
            failures.push(format!(
                "{page}: {}\n--- expected\n{want}--- got\n{got}--- stderr\n{}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `browser/tests/audio/core.html` (P11.19, Web Audio core): an
/// OfflineAudioContext renders a looped buffer through a gain with
/// linear, exponential and setTarget automation. Its `samples` line must
/// match the values computed here within 1e-4; the other lines must be
/// `core.txt`, with an empty stderr. No audio device is used.
#[test]
fn webaudio_core() {
    let page = "browser/tests/audio/core.html";
    let golden = root().join("browser/tests/audio/core.txt");
    let out = run(&["dump", "--text", "--js", page]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{page}: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let samples = got
        .lines()
        .find_map(|l| l.strip_prefix("samples "))
        .unwrap_or_else(|| panic!("no samples line:\n{got}"));
    let rest: String = got
        .lines()
        .filter(|l| !l.starts_with("samples "))
        .map(|l| format!("{l}\n"))
        .collect();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &rest).unwrap();
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(rest, want, "differs from {}", golden.display());

    // 8000 Hz; the 10-frame buffer loops from frame 80 (0.01 s) and stops
    // at 0.19 s; the gain is 0 -> 1 linear to 0.05 s, exponential to 0.25
    // at 0.1 s, then setTarget(0, 0.1, 0.01).
    let data: Vec<f32> = (0..10)
        .map(|i| ((i + 1) as f64 / 10.0 - 0.5) as f32)
        .collect();
    let got: Vec<f64> = samples.split(',').map(|v| v.parse().unwrap()).collect();
    assert_eq!(got.len(), 1600);
    let mut worst = (0.0f64, 0usize);
    for (f, &g) in got.iter().enumerate() {
        let t = f as f64 / 8000.0;
        let src = if (80..1520).contains(&f) {
            data[(f - 80) % 10] as f64
        } else {
            0.0
        };
        let gain = if t < 0.05 {
            t / 0.05
        } else if t < 0.1 {
            0.25f64.powf((t - 0.05) / 0.05)
        } else {
            0.25 * (-(t - 0.1) / 0.01).exp()
        };
        let err = (g - src * gain).abs();
        if err > worst.0 {
            worst = (err, f);
        }
    }
    assert!(
        worst.0 < 1e-4,
        "frame {} is off by {} (got {})",
        worst.1,
        worst.0,
        got[worst.1]
    );
}

/// `browser/tests/audio/effects.html` (P11.20, Web Audio effects):
/// OfflineAudioContext renders through BiquadFilterNode (8 types, with the
/// frequency responses), StereoPannerNode, DynamicsCompressorNode and
/// ConvolverNode. Each `ref NAME v,v,..` line of its output must match
/// `effects.ref` (the page's output in Chrome) within 1e-3; the other
/// lines must be `effects.txt` (Chrome's too). No audio device is used.
#[test]
fn webaudio_effects() {
    let page = "browser/tests/audio/effects.html";
    let golden = root().join("browser/tests/audio/effects.txt");
    let out = run(&["dump", "--text", "--js", page]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{page}: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut refs = std::collections::HashMap::new();
    let mut rest = String::new();
    for l in got.lines() {
        match l.strip_prefix("ref ") {
            Some(r) => {
                let (name, values) = r.split_once(' ').unwrap_or((r, ""));
                refs.insert(name.to_string(), values.to_string());
            }
            None => rest.push_str(&format!("{l}\n")),
        }
    }
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &rest).unwrap();
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(rest, want, "differs from {}", golden.display());

    let reference =
        std::fs::read_to_string(root().join("browser/tests/audio/effects.ref")).unwrap();
    let mut failures = Vec::new();
    let mut checked = 0;
    for l in reference.lines() {
        let (name, values) = l.split_once(' ').unwrap();
        checked += 1;
        let Some(have) = refs.get(name) else {
            failures.push(format!("{name}: missing"));
            continue;
        };
        let want: Vec<f64> = values.split(',').map(|v| v.parse().unwrap()).collect();
        let have: Vec<f64> = have
            .split(',')
            .map(|v| v.parse().unwrap_or(f64::INFINITY))
            .collect();
        if want.len() != have.len() {
            failures.push(format!(
                "{name}: {} samples, want {}",
                have.len(),
                want.len()
            ));
            continue;
        }
        // The worst difference (NaN must be NaN).
        let mut worst = (0.0f64, 0usize);
        for (i, (w, h)) in want.iter().zip(&have).enumerate() {
            let d = if w.is_nan() && h.is_nan() {
                0.0
            } else if w.is_nan() || h.is_nan() {
                f64::INFINITY
            } else {
                (w - h).abs()
            };
            if d > worst.0 {
                worst = (d, i);
            }
        }
        if worst.0 >= 1e-3 {
            failures.push(format!(
                "{name}: sample {} is {} (want {})",
                worst.1, have[worst.1], want[worst.1]
            ));
        }
    }
    assert!(checked >= 60, "only {checked} reference lines");
    assert!(
        failures.is_empty(),
        "{} of {checked} differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// loop.wasm (browser/tests/wasm/loop.rs, built from Rust) in Dweller: the
/// checksum must be node's; the median time of 20M steps is printed (with
/// node's when `node` is installed) for comparison, not checked.
#[test]
fn wasm_loop_bench() {
    if wasmtime_pkgconfig().is_none() {
        println!("skipped: no libwasmtime (browser/wasm/fetch_wasmtime.sh)");
        return;
    }
    let out = run(&["dump", "--text", "--js", "browser/tests/wasm/bench.html"]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{got}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(got.starts_with("checksum 486553404.1931757\n"), "{got}");
    println!("dweller (wasmtime): {}", got.lines().nth(1).unwrap_or(""));
    let script = r#"
        const b = require('fs').readFileSync(process.argv[1]);
        WebAssembly.instantiate(b).then(({ instance }) => {
          instance.exports.run(1000);
          const t = [];
          let r;
          for (let k = 0; k < 5; k++) { const t0 = Date.now(); r = instance.exports.run(20000000); t.push(Date.now() - t0); }
          t.sort((a, b) => a - b);
          console.log('checksum ' + r + ' median ms ' + t[2].toFixed(1));
        });
    "#;
    if let Ok(node) = Command::new("node")
        .current_dir(root())
        .args(["-e", script, "browser/tests/wasm/loop.wasm"])
        .output()
    {
        println!("node (V8): {}", text(&node.stdout).trim());
    }
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

/// `dweller args...` must exit with `status` and print exactly `stdout`
/// (and `stderr`).
fn expect(args: &[&str], status: i32, stdout: &str, stderr: &str) {
    let out = run(args);
    assert_eq!(text(&out.stdout), stdout, "stdout of dweller {args:?}");
    assert_eq!(text(&out.stderr), stderr, "stderr of dweller {args:?}");
    assert_eq!(
        out.status.code(),
        Some(status),
        "status of dweller {args:?}"
    );
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
    let usage = "error: usage: dweller dump --tree|--text|--style|--layout [--js [--no-scroll | --scroll-steps N]] [--width PX] [--font fixed|system] [--id ID | --tag NAME] FILE|URL\n";
    expect(
        &[],
        2,
        "",
        "error: usage: dweller [open] FILE|URL | dump --tree|--text|--style|--layout|--a11y|--md ... FILE|URL | screenshot FILE|URL -o OUT.png | js [--print-bytecode|--test262] FILE | wpt FILE|URL [--timeout S] | gpu-smoke | agent | mcp | connect SOCKET\n",
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

/// The local test server (`browser/tests/server.py`) serving
/// `browser/tests`; killed when dropped. `None` without python3.
struct Server {
    child: std::process::Child,
    base: String,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn server() -> Option<Server> {
    use std::io::BufRead;
    let tests = root().join("browser").join("tests");
    let mut child = Command::new("python3")
        .arg("-I")
        .arg(tests.join("server.py"))
        .arg(&tests)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .ok()?;
    let port: u16 = line.trim().parse().ok()?;
    Some(Server {
        child,
        base: format!("http://127.0.0.1:{port}"),
    })
}

#[test]
fn http() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let basic = std::fs::read_to_string(root().join("browser/tests/basic.txt")).unwrap();
    // A page over HTTP reads as from a file, also after a redirect.
    expect(
        &["dump", "--text", &format!("{base}/basic.html")],
        0,
        &basic,
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/redirect")],
        0,
        &basic,
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/agent")],
        0,
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Dweller/0.1\n",
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/gzip")],
        0,
        &format!("{}\n", ["compressed"; 50].join(" ")),
        "",
    );
    // Style sheets linked from a page over HTTP are fetched relative to
    // the URL after redirects.
    let link_style =
        std::fs::read_to_string(root().join("browser/tests/style/link.style")).unwrap();
    expect(
        &["dump", "--style", &format!("{base}/redirect-style")],
        0,
        &link_style,
        "",
    );
    // The charset of the Content-Type header decodes the body.
    expect(
        &["dump", "--text", &format!("{base}/latin1")],
        0,
        "Café à la crème\n",
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/gbk")],
        0,
        "中文网页\n",
        "",
    );
    // An error status still has a page.
    let out = run(&["dump", "--text", &format!("{base}/missing.html")]);
    assert_eq!(out.status.code(), Some(0));
    assert!(text(&out.stdout).contains("404"), "{}", text(&out.stdout));
    expect(
        &["dump", "--text", &format!("{base}/redirect-loop")],
        1,
        "",
        &format!("error: cannot fetch {base}/redirect-loop: too many redirects\n"),
    );
    // Nothing listens on port 1.
    expect(
        &["dump", "--text", "http://127.0.0.1:1/"],
        1,
        "",
        "error: cannot fetch http://127.0.0.1:1/: cannot connect\n",
    );
}

/// The img requests that scripts observe (load state) are fetched
/// together: twelve images that each take 0.3 s must not be requested
/// one after another (they were, from c40a1ea: lobste.rs took 8 s).
#[test]
fn http_images_concurrent() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{base}/net/slow_images.html"),
    ]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let out = run(&["dump", "--text", &format!("{base}/slow-peak")]);
    let peak: u32 = text(&out.stdout).trim().parse().unwrap();
    assert!(peak > 1, "the slow images were fetched one at a time");
}

/// The log of `browser/tests/net/bg_images.html` on the virtual clock of a
/// dump: the load events in the order of the requests. The second
/// script's fetch settles in a networking task that does not delay the
/// load event, so on the virtual clock it comes after it (as a
/// `setTimeout(f, 0)` would).
const BG_IMAGES_LOG: &str = "first script: complete=false,false,false,false
second script: complete=false,false,false,false naturalWidth=0
DOMContentLoaded: complete=false,false,false,false
load a a 64x48 complete=true readyState=interactive
load b b 64x48 complete=true readyState=interactive
load c c 64x48 complete=true readyState=interactive
load d d 64x48 complete=true readyState=interactive
load e e2 64x48 complete=true readyState=interactive
window load: true/64 true/64 true/64 true/64 true/64
animation frames before it: 0
images in flight during the second script: some
";

/// Script-visible image requests load in the background: the scripts
/// after them run while the server still holds them, each request's task
/// fires its load event (in the order of the requests on the virtual
/// clock) before the window's load event, which waits for them, and a
/// request replaced while it loads never fires. The same under GC stress.
#[test]
fn http_images_background() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/bg_images.html", srv.base);
    for stress in [false, true] {
        let mut c = Command::new(dweller());
        c.current_dir(root())
            .args(["dump", "--text", "--js", &page]);
        if stress {
            c.env("DWELLER_JS_GC", "stress");
        }
        let out = c.output().unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        let got = text(&out.stdout);
        let log = got.split_once("\n\n").map(|(_, l)| l).unwrap_or("");
        assert_eq!(log, BG_IMAGES_LOG, "GC stress: {stress}");
    }
}

/// On the window's clock the page's load event waits for its images as
/// they arrive, while the window keeps drawing (animation frames run
/// before it).
#[test]
fn window_images_load_in_background() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/bg_images.html", srv.base);
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 400 frames in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    // The loads end in any order on the real clock.
    let mut loads: Vec<&str> = got.lines().filter(|l| l.starts_with("load ")).collect();
    loads.sort();
    let want: Vec<&str> = BG_IMAGES_LOG
        .lines()
        .filter(|l| l.starts_with("load "))
        .collect();
    assert_eq!(loads, want, "{got}");
    // The window's load event comes after them.
    let window_load = got.find("window load: true/64 true/64 true/64 true/64 true/64\nanimation frames before it: several").expect(&got);
    assert!(
        want.iter()
            .all(|l| got.find(l).is_some_and(|at| at < window_load)),
        "{got}"
    );
}

/// The events of `browser/tests/net/bg_image_events.html` on the virtual
/// clock of a dump, in the order of the requests. The observer reports
/// only at the dump's animation frames, after the load event.
const BG_IMAGE_EVENTS_LOG: &str = "before: wide complete=false naturalWidth=0 height=0 box height=0
load wide: complete=true natural=64x48 box by ratio: true below still under it: true
error broken: complete=true naturalWidth=0
error missing: complete=true naturalWidth=0
load late (onload set after src): 64x48
load late (listener added after src)
window load: wide true/64, late true/64
image loads captured by the window: 0, by the document: 1
below visible: false
below visible: true
";

/// Background image loads end in a task: the image's size is known then
/// (the page lays out again, and its box gets the image's ratio), a load
/// that fails (an undecodable body, a 404) fires error, handlers set after
/// `src` see the event, and the window's capturing listeners do not see
/// the images' load events. The same under GC stress.
#[test]
fn http_image_events_background() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/bg_image_events.html", srv.base);
    for stress in [false, true] {
        let mut c = Command::new(dweller());
        c.current_dir(root())
            .args(["dump", "--text", "--js", &page]);
        if stress {
            c.env("DWELLER_JS_GC", "stress");
        }
        let out = c.output().unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        let got = text(&out.stdout);
        let log = got
            .split_once("\n\n")
            .and_then(|(_, l)| l.split_once("\n\n"))
            .map(|(_, l)| l)
            .unwrap_or("");
        assert_eq!(log, BG_IMAGE_EVENTS_LOG, "GC stress: {stress}");
    }
}

/// A background image load that failed is remembered for the page
/// (`browser/tests/net/failed_image_once.html`): its two img elements
/// drop their requests after it failed and ask for the URL again, and
/// the page lays out, but the server sees one request, and each img
/// fires error. The same under GC stress.
#[test]
fn http_failed_image_fetched_once() {
    let page_log = "laid out: true
inserted again: complete=false,false
error a: complete=true naturalWidth=0
error b: complete=true naturalWidth=0
";
    for stress in [false, true] {
        let Some(srv) = server() else {
            println!("skipped: cannot start python3 for the test server");
            return;
        };
        let mut c = Command::new(dweller());
        c.current_dir(root()).args([
            "dump",
            "--text",
            "--js",
            &format!("{}/net/failed_image_once.html", srv.base),
        ]);
        if stress {
            c.env("DWELLER_JS_GC", "stress");
        }
        let out = c.output().unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        let got = text(&out.stdout);
        let log = got.split_once("\n\n").map(|(_, l)| l).unwrap_or("");
        assert_eq!(log, page_log, "GC stress: {stress}");
        let requests = text(&run(&["dump", "--text", &format!("{}/log", srv.base)]).stdout);
        assert_eq!(requests, "GET /fail/once.png\n", "GC stress: {stress}");
    }
}

/// In the window the page lays out again when an image arrives: an
/// observer sees the paragraph below it visible first and pushed out of
/// the viewport after the image's load event.
#[test]
fn window_relayout_after_image_load() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/window_relayout.html", srv.base);
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 400 frames in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    let at = |line: &str| {
        got.find(&format!("\n{line}"))
            .unwrap_or_else(|| panic!("no {line:?} in\n{got}"))
    };
    // The loads end in any order on the real clock; each event comes
    // before the window's load event.
    let window_load = at(
        "window load: wide true/64, late true/64\nimage loads captured by the window: 0, by the document: 1\n",
    );
    for line in BG_IMAGE_EVENTS_LOG
        .lines()
        .filter(|l| l.starts_with("load ") || l.starts_with("error "))
    {
        assert!(
            at(line) < window_load,
            "{line:?} after the window's load event:\n{got}"
        );
    }
    let loaded = at("load wide: ");
    assert!(at("below visible: true\n") < loaded, "{got}");
    assert!(at("below visible: false\n") > loaded, "{got}");
}

// ---- B4: paint and window

/// A scratch directory for the files the browser writes.
fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dweller-out-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

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

/// A bitmap mutation from requestAnimationFrame must reach the window's
/// display list. Compare the animated final frame to an immediate draw
/// of the identical page, including chrome and CSS bitmap placement.
#[test]
fn canvas_animation() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let animated = scratch().join("canvas-animated.png");
    let immediate = scratch().join("canvas-immediate.png");
    let page = "browser/tests/ui/canvas_animate.html";
    let out = run_window(&[
        page,
        "--frames",
        "60",
        "--font",
        "fixed",
        "--text",
        "--save",
        animated.to_str().unwrap(),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains("three frames drawn"),
        "{}",
        text(&out.stdout)
    );
    // The fragment also changes the address bar; compare only the page
    // area when checking the animated and immediate bitmap pixels.
    let static_page = format!("{page}#static");
    let out = run_window(&[
        &static_page,
        "--frames",
        "1",
        "--font",
        "fixed",
        "--text",
        "--save",
        immediate.to_str().unwrap(),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains("three frames drawn"),
        "{}",
        text(&out.stdout)
    );
    assert_eq!(check_png(&animated), check_png(&immediate));
    // The PNG encoder uses filter 0, so compare the decoded page area.
    let script = r#"
import struct, sys, zlib
def pixels(path):
    data = open(path, 'rb').read()
    w, h = struct.unpack('>II', data[16:24])
    chunks = []
    offset = 8
    while offset < len(data):
        n = int.from_bytes(data[offset:offset+4], 'big')
        if data[offset+4:offset+8] == b'IDAT':
            chunks.append(data[offset+8:offset+8+n])
        offset += n + 12
    raw = zlib.decompress(b''.join(chunks))
    stride = w*4+1
    assert all(raw[y*stride] == 0 for y in range(h))
    return b''.join(raw[y*stride+1:(y+1)*stride] for y in range(40, h))
assert pixels(sys.argv[1]) == pixels(sys.argv[2]), 'canvas repaint did not match immediate drawing'
"#;
    if let Ok(out) = Command::new("python3")
        .args([
            "-c",
            script,
            animated.to_str().unwrap(),
            immediate.to_str().unwrap(),
        ])
        .output()
    {
        assert!(out.status.success(), "{}", text(&out.stderr));
    }
}

/// Cross-origin pixel reads are refused, even when a page sets a
/// crossorigin attribute without a CORS-validated image response.
#[test]
fn canvas_origin_clean() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    expect(
        &[
            "dump",
            "--text",
            "--js",
            &format!("{}/canvas/origin.html", srv.base),
        ],
        0,
        "canvas origin-clean checks passed\n",
        "",
    );
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Checks a PNG: signature, chunk CRCs, IHDR (8-bit RGBA), and, when
/// python3 is there, that zlib inflates the IDAT data to the image's
/// size. Returns (width, height).
fn check_png(path: &Path) -> (u32, u32) {
    let d = std::fs::read(path).unwrap();
    assert_eq!(&d[..8], b"\x89PNG\r\n\x1a\n", "{}", path.display());
    let be = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    let (mut i, mut size, mut kinds) = (8, (0, 0), Vec::new());
    while i < d.len() {
        let n = be(&d[i..]) as usize;
        let kind = &d[i + 4..i + 8];
        assert_eq!(
            crc32(&d[i + 4..i + 8 + n]),
            be(&d[i + 8 + n..]),
            "CRC of {} in {}",
            String::from_utf8_lossy(kind),
            path.display()
        );
        if kind == b"IHDR" {
            size = (be(&d[i + 8..]), be(&d[i + 12..]));
            assert_eq!(
                &d[i + 16..i + 21],
                &[8, 6, 0, 0, 0],
                "bit depth, color type and methods"
            );
        }
        kinds.push(String::from_utf8_lossy(kind).into_owned());
        i += 12 + n;
    }
    assert_eq!(kinds, ["IHDR", "IDAT", "IEND"], "{}", path.display());
    let script = "import sys, zlib, struct\nd = open(sys.argv[1], 'rb').read()\nn = struct.unpack('>I', d[33:37])[0]\nprint(len(zlib.decompress(d[41:41 + n])))";
    if let Ok(out) = Command::new("python3")
        .args(["-I", "-c", script])
        .arg(path)
        .output()
    {
        if out.status.success() {
            let len: u64 = text(&out.stdout).trim().parse().unwrap();
            assert_eq!(
                len,
                size.1 as u64 * (size.0 as u64 * 4 + 1),
                "inflated size of {}",
                path.display()
            );
        }
    }
    size
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
    let usage = "error: usage: dweller screenshot FILE|URL -o OUT.png [--width W] [--height H] [--font fixed|system] [--full]\n";
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

const WINDOW_USAGE: &str = "usage: dweller [open] FILE|URL [--width W] [--height H] [--font fixed|system] [--no-js] [--agent-socket PATH [--spaces] [--exit-when-idle]] [--frames N [--events SCRIPT] [--state] [--text] [--save OUT.png]]";

fn has_sdl() -> bool {
    Command::new("pkg-config")
        .args(["--exists", "sdl2"])
        .status()
        .is_ok_and(|s| s.success())
}

/// `dweller args...` with SDL's dummy video driver (no display).
fn run_window(args: &[&str]) -> Output {
    Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .args(args)
        .output()
        .unwrap()
}

/// The window, headless: a frame or two, then exit 0 with nothing on
/// stderr (no leaks). Skipped when SDL2 is not installed.
#[test]
fn window_frames() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    for args in [
        vec!["--frames", "1", "browser/tests/article.html"],
        vec![
            "open",
            "browser/tests/links.html",
            "--frames",
            "2",
            "--font",
            "fixed",
        ],
    ] {
        let out = run_window(&args);
        assert!(
            out.status.success(),
            "dweller {args:?}: {}\n{}{}",
            out.status,
            text(&out.stdout),
            text(&out.stderr)
        );
        assert!(
            out.stderr.is_empty(),
            "dweller {args:?}: {}",
            text(&out.stderr)
        );
    }
    // The last frame, saved: the window's size, chrome included.
    let png = scratch().join("window.png");
    let out = run_window(&[
        "browser/tests/paint/text.html",
        "--font",
        "fixed",
        "--width",
        "500",
        "--height",
        "300",
        "--frames",
        "1",
        "--save",
        png.to_str().unwrap(),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(check_png(&png), (500, 300));
    expect(
        &["--frames", "1", "--bogus", "x.html"],
        2,
        "",
        &format!("error: {WINDOW_USAGE}\n"),
    );
}

/// Scripted input (`--events`, ui/script.aye) through the window's event
/// handler: scrolling, links, fragments, history, the address bar, find.
#[test]
fn window_events() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let ui = format!("file://{}/browser/tests/ui/", root().display());
    // A 400x340 window: the page is 1260 px tall in a 300 px viewport.
    let cases: &[(&str, &str)] = &[
        (
            "key:down wheel:-1",
            "url long.html\nscroll 80\nfocus page\naddress long.html\nhistory 1 of 1\nmatches 0\nstatus \n",
        ),
        (
            "key:end key:pageup key:space key:up",
            "url long.html\nscroll 920\nfocus page\naddress long.html\nhistory 1 of 1\nmatches 0\nstatus \n",
        ),
        (
            "key:end key:home wheel:3",
            "url long.html\nscroll 0\nfocus page\naddress long.html\nhistory 1 of 1\nmatches 0\nstatus \n",
        ),
        // A link to a fragment scrolls to its element (y 958); the link
        // keeps the focus.
        (
            "click:30,58",
            "url long.html#end\nscroll 958\nfocus page\naddress long.html#end\nhistory 2 of 2\nmatches 0\nstatus \nelement a\n",
        ),
        // Back and forward restore scroll offsets; they stay in the document
        // (a fragment of it), so the link keeps the focus.
        (
            "key:pagedown key:home click:30,58 key:cmd+[ key:cmd+]",
            "url long.html#end\nscroll 958\nfocus page\naddress long.html#end\nhistory 2 of 2\nmatches 0\nstatus \nelement a\n",
        ),
        (
            "click:130,58 key:cmd+[",
            "url long.html\nscroll 0\nfocus page\naddress long.html\nhistory 1 of 2\nmatches 0\nstatus \n",
        ),
        (
            "move:130,58",
            "url long.html\nscroll 0\nfocus page\naddress long.html\nhistory 1 of 1\nmatches 0\nstatus other.html\n",
        ),
        // The back button, the forward button, reload.
        (
            "click:130,58 click:20,20 click:50,20 click:85,20",
            "url other.html\nscroll 0\nfocus page\naddress other.html\nhistory 2 of 2\nmatches 0\nstatus \n",
        ),
        // Typing in the address bar (Cmd+L selects everything first).
        (
            "key:cmd+l text:other.htm key:backspace text:ml key:escape",
            "url long.html\nscroll 0\nfocus page\naddress long.html\nhistory 1 of 1\nmatches 0\nstatus \n",
        ),
        (
            "key:cmd+l text:browser/tests/ui/other.html",
            "url long.html\nscroll 0\nfocus address\naddress browser/tests/ui/other.html\nhistory 1 of 1\nmatches 0\nstatus \n",
        ),
        (
            "key:cmd+l text:browser/tests/ui/other.html key:return",
            "url other.html\nscroll 0\nfocus page\naddress other.html\nhistory 2 of 2\nmatches 0\nstatus \n",
        ),
        // Find in page: matches, Enter scrolls to the next one.
        // ("To the end", "Other", "The end"; the third is at y 962, shown a
        // third down the 268 px left above the find bar.)
        (
            "key:cmd+f text:the",
            "url long.html\nscroll 0\nfocus find\naddress long.html\nhistory 1 of 1\nmatches 3\nstatus \n",
        ),
        (
            "key:cmd+f text:the key:return key:return",
            "url long.html\nscroll 873\nfocus find\naddress long.html\nhistory 1 of 1\nmatches 3\nstatus \n",
        ),
        (
            "key:cmd+f text:the key:return key:return key:shift+return",
            "url long.html\nscroll 0\nfocus find\naddress long.html\nhistory 1 of 1\nmatches 3\nstatus \n",
        ),
        (
            "key:cmd+f text:the key:escape",
            "url long.html\nscroll 0\nfocus page\naddress long.html\nhistory 1 of 1\nmatches 0\nstatus \n",
        ),
    ];
    let mut failures = Vec::new();
    for (events, want) in cases {
        let out = run_window(&[
            "browser/tests/ui/long.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&ui, "");
        if !out.status.success() || !out.stderr.is_empty() || got != *want {
            failures.push(format!(
                "--events {events:?}: {}\n{}--- expected\n{want}--- got\n{got}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} scripts failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

/// Forms in the window (`browser/tests/ui/form.html`, 400x340): clicks
/// and Tab focus controls, typing edits a field, Enter submits (by GET
/// to `result.html`), Space toggles a checkbox, a select opens a
/// dropdown and takes arrow keys, labels and reset buttons work, the
/// clipboard copies and pastes. `--state` shows the focused element and
/// its value.
#[test]
fn window_forms() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let ui = format!("file://{}/browser/tests/ui/", root().display());
    let state = |url: &str, rest: &str| {
        format!(
            "url {url}\nscroll 0\nfocus page\naddress {url}\nhistory {}\nmatches 0\nstatus \n{rest}",
            if url == "form.html" {
                "1 of 1"
            } else {
                "2 of 2"
            }
        )
    };
    // The query field is at (60, 60), the Go button at (170, 60), the
    // label "All" at (45, 95), the select at (40, 125), the textarea at
    // (50, 165), the reset button at (30, 205), the link at (80, 205).
    let cases: Vec<(&str, String)> = vec![
        (
            "click:60,60 key:end text:_there key:return",
            state("result.html?q=hi+there&go=1&lang=en&note=&tags=b", ""),
        ),
        (
            "click:60,60 text:!",
            state("form.html", "element input name=q\nvalue hi!\n"),
        ),
        (
            "click:60,60 key:backspace key:left text:_ key:delete key:home text:o",
            state("form.html", "element input name=q\nvalue o \n"),
        ),
        (
            "click:60,60 key:shift+home text:yo",
            state("form.html", "element input name=q\nvalue yo\n"),
        ),
        (
            "key:tab key:tab key:tab key:space key:tab key:down key:down key:tab text:a key:return text:b",
            state("form.html", "element textarea name=note\nvalue a\nb\n"),
        ),
        (
            "key:tab key:tab key:tab key:space key:tab key:down key:down key:tab text:a key:return text:b click:170,60",
            state(
                "result.html?q=hi&go=1&all=yes&lang=ja&note=a%0D%0Ab&tags=b",
                "",
            ),
        ),
        (
            "click:45,95 click:170,60",
            state("result.html?q=hi&go=1&all=yes&lang=en&note=&tags=b", ""),
        ),
        (
            "click:40,125 click:40,170",
            state("form.html", "element select name=lang\nvalue fr\n"),
        ),
        // The disabled option is skipped by Enter (it stays on English)...
        (
            "click:40,125 key:down key:down key:return",
            state("form.html", "element select name=lang\nvalue en\n"),
        ),
        (
            "click:40,125 key:down key:down key:down key:return",
            state("form.html", "element select name=lang\nvalue ja\n"),
        ),
        // ...and by the arrow keys on the closed select.
        (
            "click:40,125 key:escape key:down key:down",
            state("form.html", "element select name=lang\nvalue ja\n"),
        ),
        (
            "click:40,125 key:escape click:170,60",
            state("result.html?q=hi&go=1&lang=en&note=&tags=b", ""),
        ),
        (
            "click:60,60 text:xyz click:30,205 click:170,60",
            state("result.html?q=hi&go=1&lang=en&note=&tags=b", ""),
        ),
        (
            "click:60,60 key:cmd+a key:cmd+c key:end key:cmd+v",
            state("form.html", "element input name=q\nvalue hihi\n"),
        ),
        (
            "click:60,60 key:left key:shift+left key:cmd+x key:end key:cmd+v",
            state("form.html", "element input name=q\nvalue ih\n"),
        ),
        ("click:60,60 key:escape", state("form.html", "")),
        ("click:60,60 click:300,300", state("form.html", "")),
        (
            "key:shift+tab key:shift+tab key:return",
            state("other.html", ""),
        ),
        // The list box (rows a, b, c at y 240, 257, 273): a click toggles
        // an option of a `multiple` select.
        (
            "click:20,273 click:20,240 click:170,60",
            state(
                "result.html?q=hi&go=1&lang=en&note=&tags=a&tags=b&tags=c",
                "",
            ),
        ),
        (
            "click:20,257 click:20,273",
            state("form.html", "element select name=tags\nvalue c\n"),
        ),
    ];
    let mut failures = Vec::new();
    for (events, want) in &cases {
        let out = run_window(&[
            "browser/tests/ui/form.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&ui, "");
        if !out.status.success() || !out.stderr.is_empty() || got != *want {
            failures.push(format!(
                "--events {events:?}: {}\n{}--- expected\n{want}--- got\n{got}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} scripts failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

/// HTML5 form keyboard input, dragging, validation and reset share the
/// real SDL handler. The transcript also checks scripts see drag change.
#[test]
fn window_forms_html5() {
    if !has_sdl() {
        return;
    }
    let cases = [
        "key:tab key:up key:up",
        "key:tab text:3 key:up",
        "key:tab key:tab key:right key:down",
        "click:40,95 move:160,95 release:160,95",
        "key:tab key:tab key:tab key:return",
        "key:tab text:4 key:tab key:tab key:return",
    ];
    // The address bar is part of the golden PNG, so the page is loaded
    // from a copy at a fixed path that does not depend on the checkout.
    let fixed_dir = "/tmp/dweller-forms-html5";
    let fixed = format!("{fixed_dir}/forms_html5.html");
    std::fs::create_dir_all(fixed_dir).unwrap();
    for name in ["forms_html5.html", "result.html"] {
        std::fs::write(
            format!("{fixed_dir}/{name}"),
            std::fs::read(root().join("browser/tests/ui").join(name)).unwrap(),
        )
        .unwrap();
    }
    let mut transcript = String::new();
    let image = scratch().join("validation.png");
    for (i, events) in cases.iter().enumerate() {
        let mut args = vec![
            fixed.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--text",
            "--events",
            events,
        ];
        if i == 4 {
            args.extend(["--save", image.to_str().unwrap()]);
        }
        let out = run_window(&args);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        transcript.push_str(&format!(
            "events: {events}\n{}\n",
            rooted(&text(&out.stdout)).replace(
                &format!("file://{fixed_dir}/"),
                "file://$ROOT/browser/tests/ui/"
            )
        ));
    }
    let golden = root().join("browser/tests/ui/forms_html5.validation.png");
    let got = std::fs::read(&image).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    } else {
        assert_eq!(
            got,
            std::fs::read(&golden).unwrap(),
            "validation tooltip differs: {}",
            image.display()
        );
    }
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/ui/forms_html5.out"),
        &transcript,
        "HTML5 window forms",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---- the agent interface (browser/agent, docs/agent-protocol.md)

/// `s` with the repository's path written as `$ROOT`, so that goldens
/// with absolute `file:` URLs do not depend on the checkout.
fn rooted(s: &str) -> String {
    s.replace(&root().display().to_string(), "$ROOT")
}

/// Compares `got` with the golden file (or writes it when blessing).
fn check_golden(golden: &Path, got: &str, what: &str, failures: &mut Vec<String>) {
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(golden, got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(golden).unwrap_or_default();
    if got != want {
        failures.push(format!(
            "{what}: output differs from {}\n--- expected\n{want}--- got\n{got}",
            golden.display()
        ));
    }
}

/// `browser/tests/agent/X.html`: `dweller dump --a11y` must print
/// `X.a11y` and `dweller dump --md` must print `X.md`; a few fixtures
/// also have `--boxes`, `--json` and `--refs` variants.
#[test]
fn agent_fixtures() {
    let dir = root().join("browser").join("tests").join("agent");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 15,
        "expected at least 15 agent fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(Vec<String>, PathBuf)> = Vec::new();
    for h in &htmls {
        let rel = h
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        runs.push((vec!["--a11y".into(), rel.clone()], h.with_extension("a11y")));
        runs.push((vec!["--md".into(), rel], h.with_extension("md")));
    }
    let page = |name: &str| format!("browser/tests/agent/{name}.html");
    runs.push((
        vec!["--a11y".into(), "--boxes".into(), page("long_page")],
        dir.join("long_page.boxes.a11y"),
    ));
    runs.push((
        vec!["--a11y".into(), "--json".into(), page("form_login")],
        dir.join("form_login.json"),
    ));
    runs.push((
        vec![
            "--a11y".into(),
            "--json".into(),
            "--boxes".into(),
            page("tables"),
        ],
        dir.join("tables.boxes.json"),
    ));
    runs.push((
        vec!["--md".into(), "--refs".into(), page("article")],
        dir.join("article.refs.md"),
    ));
    let mut failures = Vec::new();
    for (args, golden) in &runs {
        let mut all = vec!["dump"];
        all.extend(args.iter().map(|s| s.as_str()));
        let out = run(&all);
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{args:?}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        check_golden(
            golden,
            &rooted(&text(&out.stdout)),
            &format!("{args:?}"),
            &mut failures,
        );
    }
    assert!(
        failures.is_empty(),
        "{} of {} agent dumps failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}

/// `browser/tests/agent/sessions/X.jsonl`: requests fed to `dweller
/// agent` (or `dweller mcp` for `X.mcp.jsonl`) on stdin; the responses
/// must be `X.out` (`X.mcp.out`).
#[test]
fn agent_sessions() {
    use std::io::Write;
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("sessions");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 4,
        "expected at least 4 agent sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for input in &inputs {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        let mode = if name.ends_with(".mcp.jsonl") {
            "mcp"
        } else {
            "agent"
        };
        let mut child = Command::new(dweller())
            .current_dir(root())
            .arg(mode)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&std::fs::read(input).unwrap())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{name}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        check_golden(
            &input.with_extension("out"),
            &rooted(&text(&out.stdout)),
            &name,
            &mut failures,
        );
    }
    assert!(
        failures.is_empty(),
        "{} of {} agent sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

/// `text-overflow: ellipsis` is only drawn: the Markdown keeps the whole
/// text (the layout fixture `text_overflow.html` checks the ellipsis).
#[test]
fn text_overflow_markdown() {
    let page = "data:text/html,<div style='width:100px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis'>A very long title</div>";
    expect(&["dump", "--md", page], 0, "A very long title\n", "");
}

#[test]
fn agent_errors() {
    let usage = "error: usage: dweller dump --a11y [--boxes] [--json] | --md [--refs] [--js] [--width PX] [--no-scroll | --scroll-steps N] FILE|URL\n";
    expect(&["dump", "--a11y"], 2, "", usage);
    expect(
        &[
            "dump",
            "--md",
            "--width",
            "x",
            "browser/tests/agent/article.html",
        ],
        2,
        "",
        usage,
    );
    expect(
        &["dump", "--md", "browser/tests/agent/missing.html"],
        1,
        "",
        &format!(
            "error: cannot fetch file://{}/browser/tests/agent/missing.html: not found\n",
            root().display()
        ),
    );
    expect(
        &["agent", "--color"],
        2,
        "",
        "error: usage: dweller agent|mcp [--width PX] [--height PX] [--no-js]\n",
    );
    expect(
        &["dump", "--a11y", "data:text/html,<button>OK</button>"],
        0,
        "- button \"OK\" [e1]\n",
        "",
    );
    expect(
        &["dump", "--md", "data:text/html,<h2>Hi</h2><p>a <b>b</b>"],
        0,
        "## Hi\n\na **b**\n",
        "",
    );
}

fn base64_decode(s: &str) -> Vec<u8> {
    let digit = |c: u8| match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => panic!("not base64: {:?}", c as char),
    };
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for &c in s.as_bytes().iter().filter(|&&c| c != b'=') {
        acc = acc << 6 | digit(c) as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

/// The string value of `"key":"..."` in one JSON line (no escapes in it).
fn json_str<'a>(line: &'a str, key: &str) -> &'a str {
    let k = format!("\"{key}\":\"");
    let at = line
        .find(&k)
        .unwrap_or_else(|| panic!("no {key} in {line}"))
        + k.len();
    &line[at..at + line[at..].find('"').unwrap()]
}

/// `screenshot` in `dweller agent` and `browser_screenshot` in `dweller
/// mcp`: PNGs (base64) of the viewport, the whole page and one element.
#[test]
fn agent_screenshot() {
    use std::io::Write;
    let session = |mode: &str, input: &str| -> Vec<String> {
        let mut child = Command::new(dweller())
            .current_dir(root())
            .args([mode, "--width", "400", "--height", "300"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout).lines().map(String::from).collect()
    };
    let png_size = |b64: &str, name: &str| {
        let path = scratch().join(name);
        std::fs::write(&path, base64_decode(b64)).unwrap();
        check_png(&path)
    };
    let lines = session(
        "agent",
        concat!(
            "{\"id\":1,\"cmd\":\"screenshot\"}\n",
            "{\"id\":2,\"cmd\":\"navigate\",\"url\":\"browser/tests/agent/form_login.html\"}\n",
            "{\"id\":3,\"cmd\":\"screenshot\",\"ref\":\"e2\"}\n",
            "{\"id\":4,\"cmd\":\"screenshot\",\"ref\":\"e99\"}\n",
            "{\"id\":5,\"cmd\":\"snapshot\"}\n",
            "{\"id\":6,\"cmd\":\"navigate\",\"url\":\"browser/tests/paint/full.html\"}\n",
            "{\"id\":7,\"cmd\":\"screenshot\"}\n",
            "{\"id\":8,\"cmd\":\"screenshot\",\"full\":true}\n",
        ),
    );
    assert_eq!(lines.len(), 8);
    assert_eq!(
        lines[0],
        "{\"id\":1,\"ok\":false,\"error\":\"no page: navigate first\"}"
    );
    let (w, h) = png_size(json_str(&lines[2], "png"), "agent_ref.png");
    assert!(w < 400 && h < 300, "element {w}x{h}");
    assert!(
        lines[2].contains("\"result\":\"screenshot of textbox \\\"Email address\\\" [e2]"),
        "{}",
        lines[2]
    );
    assert!(
        lines[3].contains("\"ok\":false,\"error\":\"unknown ref e99"),
        "{}",
        lines[3]
    );
    // The page is laid out as before the screenshots: refs keep working.
    assert!(
        lines[4].contains("textbox \\\"Email address\\\" [e2]"),
        "{}",
        lines[4]
    );
    assert!(
        lines[6].contains("\"result\":\"screenshot of the viewport, 400x300 px"),
        "{}",
        lines[6]
    );
    assert_eq!(
        png_size(json_str(&lines[6], "png"), "agent_viewport.png"),
        (400, 300)
    );
    let (w, h) = png_size(json_str(&lines[7], "png"), "agent_full.png");
    assert!(w == 400 && h > 300, "whole page {w}x{h}");
    assert!(
        json_str(&lines[7], "result")
            .starts_with(&format!("screenshot of the whole page, 400x{h} px")),
        "{}",
        lines[7]
    );

    let lines = session(
        "mcp",
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"browser_screenshot\",\"arguments\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"browser_navigate\",\"arguments\":{\"url\":\"browser/tests/agent/links.html\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"browser_screenshot\",\"arguments\":{}}}\n",
        ),
    );
    assert_eq!(lines.len(), 3);
    assert!(
        lines[0].contains("\"isError\":true") && !lines[0].contains("\"image\""),
        "{}",
        lines[0]
    );
    assert!(
        lines[2].contains("\"type\":\"image\"")
            && lines[2].contains("\"mimeType\":\"image/png\"")
            && lines[2].contains("\"isError\":false"),
        "{}",
        &lines[2][..200.min(lines[2].len())]
    );
    assert_eq!(
        png_size(json_str(&lines[2], "data"), "mcp_viewport.png"),
        (400, 300)
    );
}

/// A window shared by its user and agents (`--agent-socket`,
/// ui/agents.aye): `browser/tests/agent/window/X.jsonl` is sent over the
/// window's Unix socket (from this test, or through `dweller connect
/// --mcp` for `X.mcp.jsonl`) while the window runs headless with the
/// scripted events of `X.events` (`agents:N` waits for N requests); the
/// responses, then the window's `--state`, must be `X.out`.
#[test]
fn window_agents() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("window");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 5,
        "expected at least 5 window agent sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        match window_agent_session(input, i, false) {
            Ok(got) => check_golden(
                &input.with_extension("out"),
                &rooted(&got),
                &name,
                &mut failures,
            ),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} window agent sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

// Runs one session of `window_agents` (or with `--spaces`, of
// `window_spaces`): what the agent got, then `--- window` and the
// window's state without `loop_max_ms` (it varies).
fn window_agent_session(input: &Path, n: usize, spaces: bool) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let name = input.file_name().unwrap().to_string_lossy().into_owned();
    // A short relative path: sockets' paths are limited to ~100 bytes.
    let kind = if spaces { "spaces" } else { "window" };
    let sock = format!("target/agent-{kind}-{}-{n}.sock", std::process::id());
    let _ = std::fs::remove_file(root().join(&sock));
    let mut args = vec![
        "--agent-socket".to_string(),
        sock.clone(),
        "--exit-when-idle".into(),
        "--font".into(),
        "fixed".into(),
    ];
    if spaces {
        args.push("--spaces".into());
    }
    args.extend(["--width", "800", "--height", "600", "--state"].map(String::from));
    if let Ok(events) = std::fs::read_to_string(input.with_extension("events")) {
        args.push("--events".into());
        args.push(events.trim().to_string());
    }
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .args(&args)
        .spawn_drained();
    let start = Instant::now();
    while !root().join(&sock).exists() {
        if start.elapsed() > Duration::from_secs(20) || window.try_wait().unwrap().is_some() {
            let _ = window.kill();
            let out = window.wait_with_output().unwrap();
            return Err(format!("the window does not listen: {}", text(&out.stderr)));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let requests = std::fs::read(input).unwrap();
    let mut got = String::new();
    if name.ends_with(".mcp.jsonl") {
        let mut connect = Command::new(dweller())
            .current_dir(root())
            .args(["connect", &sock, "--mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        connect.stdin.take().unwrap().write_all(&requests).unwrap();
        let out = connect.wait_with_output().unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            let _ = window.kill();
            return Err(format!(
                "dweller connect: {}\n{}",
                out.status,
                text(&out.stderr)
            ));
        }
        got.push_str(&text(&out.stdout));
    } else {
        // Relative to the test's directory (the root), to keep it short.
        let mut s = match std::os::unix::net::UnixStream::connect(&sock) {
            Ok(s) => s,
            Err(e) => {
                let _ = window.kill();
                return Err(format!("cannot connect to {sock}: {e}"));
            }
        };
        s.write_all(&requests).unwrap();
        s.shutdown(std::net::Shutdown::Write).unwrap();
        s.read_to_string(&mut got).unwrap();
    }
    let window_pid = window.id();
    // The window leaves once its last agent did.
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(20) {
            let _ = window.kill();
            return Err("the window did not exit after its agent left".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    if !out.status.success() || !out.stderr.is_empty() {
        return Err(format!("the window: {}\n{}", out.status, text(&out.stderr)));
    }
    if root().join(&sock).exists() {
        return Err("the window left its socket file".into());
    }
    if spaces {
        no_spaces_left(window_pid)?;
    }
    got.push_str("--- window\n");
    for line in text(&out.stdout).lines() {
        // With Spaces the status line shows the person's acts and the
        // Spaces' last actions in the order they reached the window,
        // which varies; the `space` lines have each Space's last action.
        if !line.starts_with("loop_max_ms ") && !(spaces && line.starts_with("activity ")) {
            got.push_str(line);
            got.push('\n');
        }
    }
    Ok(got)
}

/// The Spaces the window `pid` started and that still run (`pgrep` on
/// their sockets' directories, `/tmp/dweller-PID-*`).
fn spaces_of(pid: u32) -> Vec<u32> {
    let out = Command::new("pgrep")
        .args(["-f", &format!("/tmp/dweller-{pid}-")])
        .output()
        .unwrap();
    text(&out.stdout)
        .lines()
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

/// After the window `pid` exited: none of its Spaces runs (within 5 s),
/// and their sockets' directories are gone.
fn no_spaces_left(pid: u32) -> Result<(), String> {
    use std::time::{Duration, Instant};
    let start = Instant::now();
    loop {
        let left = spaces_of(pid);
        if left.is_empty() {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            return Err(format!("Spaces left running after the window: {left:?}"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let prefix = format!("dweller-{pid}-");
    let dirs: Vec<String> = std::fs::read_dir("/tmp")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(&prefix))
        .collect();
    if !dirs.is_empty() {
        return Err(format!("Spaces' directories left in /tmp: {dirs:?}"));
    }
    Ok(())
}

/// The window with `--spaces` (B9 R2, ui/spaces.aye): each connection
/// is served by a Space, a process of its own, and the window copies
/// the bytes. `browser/tests/agent/spaces/X.jsonl` runs as in
/// `window_agents` (copies of `isolation` and `frames`): the agent's
/// own work answers as in the window, the user's tab stays as it was,
/// and no Space is left after the window exits. P9.3: `pause` (Cmd+Shift+P
/// pauses the Space, `paused` and `resumed`, `stop:1` ends it with
/// `stopped`), `pointing` (the person's context and Cmd+Shift+K reach
/// the Space, named by the hello), `attach` (a hello that names no done
/// Space). `waited N ms` stands for the time a `wait` took.
#[test]
fn window_spaces() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("spaces");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 3,
        "expected at least 3 Space sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        match window_agent_session(input, i, true) {
            Ok(got) => check_golden(
                &input.with_extension("out"),
                &waited_n(&rooted(&got)),
                &name,
                &mut failures,
            ),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} Space sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

// `waited 512 ms` as `waited N ms`: a `wait` in real time takes a
// little longer each run.
fn waited_n(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(at) = rest.find("waited ") {
        out.push_str(&rest[..at + 7]);
        rest = &rest[at + 7..];
        let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits > 0 && rest[digits..].starts_with(" ms") {
            out.push('N');
            rest = &rest[digits..];
        }
    }
    out.push_str(rest);
    out
}

// A headless window listening at `sock` for the Space tests, its output
// read as it comes (`Drained`).
fn spaces_window(sock: &str, extra: &[&str]) -> Drained {
    spaces_window_env(sock, extra, &[])
}

fn spaces_window_env(sock: &str, extra: &[&str], env: &[(&str, &str)]) -> Drained {
    use std::time::{Duration, Instant};
    let _ = std::fs::remove_file(root().join(sock));
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .envs(env.iter().copied())
        .args(["--agent-socket", sock, "--exit-when-idle", "--font", "fixed"])
        .args(["--width", "800", "--height", "600"])
        .args(extra)
        .spawn_drained();
    let start = Instant::now();
    while !root().join(sock).exists() {
        if start.elapsed() > Duration::from_secs(20) || window.try_wait().unwrap().is_some() {
            let _ = window.kill();
            let out = window.wait_with_output().unwrap();
            panic!("the window does not listen: {}", text(&out.stderr));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    window
}

// A connection to the window's socket (relative to the root: short).
struct AgentConn {
    stream: std::os::unix::net::UnixStream,
    read: std::io::BufReader<std::os::unix::net::UnixStream>,
}

impl AgentConn {
    fn open(sock: &str) -> AgentConn {
        let path = root().join(sock);
        let rel = path.strip_prefix(std::env::current_dir().unwrap()).unwrap_or(&path);
        let stream = std::os::unix::net::UnixStream::connect(rel).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .unwrap();
        let read = std::io::BufReader::new(stream.try_clone().unwrap());
        AgentConn { stream, read }
    }

    fn send(&mut self, line: &str) {
        use std::io::Write;
        self.stream.write_all(line.as_bytes()).unwrap();
        self.stream.write_all(b"\n").unwrap();
    }

    // The next line ("" at the end).
    fn line(&mut self) -> String {
        use std::io::BufRead;
        let mut s = String::new();
        self.read.read_line(&mut s).unwrap();
        s
    }

    // A request and its answer, and how long it took in ms.
    fn ask(&mut self, line: &str) -> (String, u128) {
        let start = std::time::Instant::now();
        self.send(line);
        let got = self.line();
        (got, start.elapsed().as_millis())
    }
}

// What `window_spaces_responsive` measured in one window.
struct Responsive {
    loop_max_ms: u64,
    // C's `tabs` while A and B were busy, in ms.
    c_ms: u128,
    a: (String, u128),
    b: (String, u128),
    state: String,
    // The window's DWELLER_TIMING records, in order (its Spaces' own
    // `ui_loop` records left out).
    records: Vec<Record>,
}

// A `ui_loop` record of the window (ms), or the window relaying an
// agent's bytes to its Space or the Space's answer back (`ui_space_sent`
// and `ui_space_answer`, with the Space's name).
#[derive(Debug, PartialEq)]
enum Record {
    Loop(u64),
    Sent(String),
    Answer(String),
}

// The window's `Record`s in `err`, the stderr of a window with Spaces
// and DWELLER_TIMING, and the lines of `err` that are not records.
fn space_records(err: &str) -> (Vec<Record>, String) {
    let mut records = Vec::new();
    let mut rest = String::new();
    for line in err.lines() {
        let Some(record) = line.strip_prefix("info: DWELLER_TIMING\t") else {
            rest.push_str(line);
            rest.push('\n');
            continue;
        };
        let fields: Vec<&str> = record.split('\t').collect();
        assert_eq!(fields.len(), 3, "{line}");
        match (fields[0], fields[2]) {
            ("ui_loop", "") => {
                records.push(Record::Loop(fields[1].parse::<u64>().unwrap() / 1_000_000))
            }
            ("ui_space_sent", name) => records.push(Record::Sent(name.to_string())),
            ("ui_space_answer", name) => records.push(Record::Answer(name.to_string())),
            _ => {}
        }
    }
    (records, rest)
}

// Agent A runs a 1500 ms `eval` loop and B loads a page whose server
// waits 1500 ms; while both run, C asks for `tabs` and the user clicks
// into a text field and types (`agents:4`: after C's answer).
fn responsive_session(base: &str, spaces: bool) -> Responsive {
    let sock = format!("target/spaces-resp-{}-{spaces}.sock", std::process::id());
    let mut extra = vec![
        "--state",
        "--events",
        "agents:4 click:60,100 text:hello",
        "browser/tests/agent/site/notes.html",
    ];
    if spaces {
        extra.push("--spaces");
    }
    let window = spaces_window_env(&sock, &extra, &[("DWELLER_TIMING", "1")]);
    let window_pid = window.id();
    // The records are read as they come (`spaces_window`).
    let mut a = AgentConn::open(&sock);
    let mut b = AgentConn::open(&sock);
    let mut c = AgentConn::open(&sock);
    // Each one's Space is up.
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"data:text/html,<title>A</title>"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = b.ask(r#"{"id":1,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = c.ask(r#"{"id":1,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let start = std::time::Instant::now();
    a.send(r#"{"id":2,"cmd":"eval","expression":"var t = Date.now(); while (Date.now() - t < 1500) {} 'spun'"}"#);
    b.send(&format!(r#"{{"id":2,"cmd":"navigate","url":"{base}/sleep?ms=1500"}}"#));
    std::thread::sleep(std::time::Duration::from_millis(300));
    let (got, c_ms) = c.ask(r#"{"id":2,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let a_got = a.line();
    let a_ms = start.elapsed().as_millis();
    let b_got = b.line();
    let b_ms = start.elapsed().as_millis();
    drop(a);
    drop(b);
    drop(c);
    let out = window.wait_with_output().unwrap();
    let (records, rest) = space_records(&text(&out.stderr));
    assert!(
        out.status.success() && rest.is_empty(),
        "the window: {}\n{rest}",
        out.status
    );
    let state = text(&out.stdout);
    let loop_max_ms = state
        .lines()
        .find_map(|l| l.strip_prefix("loop_max_ms "))
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("no loop_max_ms in the state:\n{state}"));
    if spaces {
        no_spaces_left(window_pid).unwrap();
    }
    Responsive {
        loop_max_ms,
        c_ms,
        a: (a_got, a_ms),
        b: (b_got, b_ms),
        state,
        records,
    }
}

/// B9 R2: while agent A runs a 1500 ms `eval` loop and agent B loads a
/// page whose server waits 1500 ms, the window with `--spaces` never
/// waits for their Spaces: in the DWELLER_TIMING records, the window
/// relays A's `eval` to its Space and the answer back in different loop
/// iterations, with others between them, in which it also relays a third
/// agent's request and answer; it handles the user's click and typing.
/// What does not depend on the machine's load is asserted; the times are
/// bounded generously (a loaded machine stalls a loop iteration for tens
/// of ms, a wait for A's Space stalls one for over a second). Without
/// `--spaces` the same load holds the loop up for over a second (the
/// measure measures).
#[test]
fn window_spaces_responsive() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let r = responsive_session(&srv.base, true);
    assert!(
        r.a.0.contains("\"result\":\"spun\"") && r.a.1 >= 1400,
        "A: {} ms: {}",
        r.a.1,
        r.a.0
    );
    assert!(
        r.b.0.contains("\"title\":\"Slept\"") && r.b.1 >= 1400,
        "B: {} ms: {}",
        r.b.1,
        r.b.0
    );
    // The Spaces in the order their agents first asked: A, B, C.
    let mut names: Vec<&str> = Vec::new();
    for rec in &r.records {
        if let Record::Sent(name) = rec {
            if !names.contains(&name.as_str()) {
                names.push(name);
            }
        }
    }
    assert_eq!(names.len(), 3, "{:?}", r.records);
    // The index of the `n`th (from 0) record equal to `want`.
    let nth = |n: usize, want: Record| {
        r.records
            .iter()
            .enumerate()
            .filter(|(_, rec)| **rec == want)
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("{want:?} {n} not found: {:?}", r.records))
    };
    let (a_name, c_name) = (names[0].to_string(), names[2].to_string());
    let a_sent = nth(1, Record::Sent(a_name.clone()));
    let a_answer = nth(1, Record::Answer(a_name));
    let c_sent = nth(1, Record::Sent(c_name.clone()));
    let c_answer = nth(1, Record::Answer(c_name));
    let loops_between = r.records[a_sent..a_answer]
        .iter()
        .filter(|rec| matches!(rec, Record::Loop(_)))
        .count();
    assert!(
        a_sent < c_sent && c_answer < a_answer && loops_between >= 3,
        "A's eval out at {a_sent}, back at {a_answer}, {loops_between} loop iterations between; \
         C's tabs out at {c_sent}, back at {c_answer}: {:?}",
        r.records
    );
    let longest = r
        .records
        .iter()
        .filter_map(|rec| match rec {
            Record::Loop(ms) => Some(*ms),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    assert!(
        longest < 1000 && r.loop_max_ms < 1000,
        "longest loop {longest} ms\n{}",
        r.state
    );
    assert!(r.c_ms < 1000, "C waited {} ms", r.c_ms);
    assert!(r.state.contains("\nvalue hello\n"), "{}", r.state);
    let old = responsive_session(&srv.base, false);
    assert!(
        old.loop_max_ms >= 1000,
        "without --spaces: loop_max_ms {}\n{}",
        old.loop_max_ms,
        old.state
    );
    println!(
        "loop_max_ms {} with --spaces (C {} ms, {} loop iterations during A's eval), {} without (C {} ms)",
        r.loop_max_ms, r.c_ms, loops_between, old.loop_max_ms, old.c_ms
    );
}

/// A Space that crashes (SIGKILL) ends only its own agent's connection,
/// with `{"id":null,"ok":false,"error":"the Space crashed"}`; the window
/// and the other agents go on. A Space whose window is killed exits (the
/// end of file on its control connection) and removes its directory.
#[test]
fn window_spaces_crash() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let sock = format!("target/spaces-crash-{}.sock", std::process::id());
    let mut window = spaces_window(&sock, &["--spaces"]);
    let mut a = AgentConn::open(&sock);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let first = spaces_of(window.id());
    assert_eq!(first.len(), 1, "{first:?}");
    let mut b = AgentConn::open(&sock);
    let (got, _) = b.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(window.id()).len(), 2);
    let status = Command::new("kill")
        .args(["-9", &first[0].to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        a.line(),
        "{\"id\":null,\"ok\":false,\"error\":\"the Space crashed\"}\n"
    );
    assert_eq!(a.line(), "", "the connection ends");
    assert!(window.try_wait().unwrap().is_none(), "the window goes on");
    let (got, _) = b.ask(r#"{"id":2,"cmd":"snapshot"}"#);
    assert!(got.contains("\"title\":\"Site home\""), "{got}");
    b.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(b.line(), "");
    let window_pid = window.id();
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    no_spaces_left(window_pid).unwrap();

    // The window killed: its Space exits by itself.
    let sock = format!("target/spaces-kill-{}.sock", std::process::id());
    let mut window = spaces_window(&sock, &["--spaces"]);
    let mut a = AgentConn::open(&sock);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(window.id()).len(), 1);
    window.kill().unwrap();
    window.wait().unwrap();
    let _ = std::fs::remove_file(root().join(&sock));
    no_spaces_left(window.id()).unwrap();
}

// Waits (at most 5 s) until the window `pid` runs `n` Spaces.
fn wait_for_spaces(pid: u32, n: usize) {
    let start = std::time::Instant::now();
    while spaces_of(pid).len() != n {
        assert!(
            start.elapsed() < std::time::Duration::from_secs(5),
            "expected {n} Spaces, found {:?}",
            spaces_of(pid)
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// B9 R4, R6 (P9.3): the window controls each Space over its control
/// connection. Two agents, A and B, named by their hellos: the person
/// pauses A (the scripted word `pause:1`): A gets `paused` and its
/// actions fail with "paused by user" while B's work; `resume:1` lets A
/// go on; `stop:2` ends B (`stopped`, then the end of its connection,
/// and its Space's process exits) while A goes on. `spaces` lists the
/// Spaces with their names and states. A Space whose agent left stays
/// `done`, and a new connection attaches to it (`"space":3` in the
/// hello); attaching to a Space that has an agent, or to none, fails;
/// `space_close` ends the Space. No Space is left after the window.
#[test]
fn window_spaces_control() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let sock = format!("target/spaces-control-{}.sock", std::process::id());
    let window = spaces_window(
        &sock,
        &[
            "--spaces",
            "--state",
            "--events",
            "agents:2 pause:1 agents:5 resume:1 agents:7 stop:2",
        ],
    );
    let pid = window.id();
    let mut a = AgentConn::open(&sock);
    a.send(r#"{"name":"Alpha"}"#);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let mut b = AgentConn::open(&sock);
    b.send(r#"{"name":"Beta"}"#);
    let (got, _) = b.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(pid).len(), 2);
    // The person pauses A only.
    assert_eq!(a.line(), "{\"event\":\"paused\",\"space\":1}\n");
    let (got, _) = a.ask(r#"{"id":2,"cmd":"click","ref":"e4"}"#);
    assert_eq!(got, "{\"id\":2,\"ok\":false,\"error\":\"paused by user\"}\n");
    let (got, _) = b.ask(r#"{"id":2,"cmd":"click","ref":"e6"}"#);
    assert!(got.contains("\"result\":\"clicked button \\\"Subscribe\\\" [e6]\""), "{got}");
    assert_eq!(a.line(), "{\"event\":\"resumed\",\"space\":1}\n");
    let (got, _) = a.ask(r#"{"id":3,"cmd":"click","ref":"e4"}"#);
    assert!(got.contains("\"result\":\"checked checkbox"), "{got}");
    // The person stops B: its connection ends, its Space exits, A goes on.
    assert_eq!(b.line(), "{\"event\":\"stopped\",\"space\":2}\n");
    assert_eq!(b.line(), "", "B's connection ends");
    wait_for_spaces(pid, 1);
    let (got, _) = a.ask(r#"{"id":4,"cmd":"spaces"}"#);
    assert!(
        got.starts_with("{\"id\":4,\"ok\":true,\"result\":\"1 space\",\"spaces\":[{\"id\":1,\"name\":\"Alpha\",\"mine\":true,\"state\":\"working\",\"last\":\"click checkbox"),
        "{got}"
    );
    // C leaves: its Space stays, done.
    let mut c = AgentConn::open(&sock);
    c.send(r#"{"name":"Gamma"}"#);
    let (got, _) = c.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/docs.html"}"#);
    assert!(got.contains("\"title\":\"Docs\""), "{got}");
    c.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(c.line(), "");
    assert_eq!(spaces_of(pid).len(), 2, "a done Space stays");
    let (got, _) = a.ask(r#"{"id":5,"cmd":"spaces"}"#);
    assert!(got.contains("\"result\":\"2 spaces\""), "{got}");
    assert!(
        got.contains("{\"id\":3,\"name\":\"Gamma\",\"mine\":false,\"state\":\"done\",\"last\":\"navigate browser/tests/agent/site/docs.html\",\"tabs\":1,\"profile\":\"shared\",\"watched\":false}"),
        "{got}"
    );
    // D attaches to it and finds C's page.
    let mut d = AgentConn::open(&sock);
    d.send(r#"{"name":"Delta","space":3}"#);
    let (got, _) = d.ask(r#"{"id":1,"cmd":"snapshot"}"#);
    assert!(got.contains("\"title\":\"Docs\""), "{got}");
    let (got, _) = d.ask(r#"{"id":2,"cmd":"spaces"}"#);
    assert!(
        got.contains("[{\"id\":3,\"name\":\"Delta\",\"mine\":true,\"state\":\"working\""),
        "{got}"
    );
    assert!(got.contains("{\"id\":1,\"name\":\"Alpha\",\"mine\":false"), "{got}");
    // Only a done Space can be attached to; the profile is shared.
    for (hello, why) in [
        (r#"{"space":3}"#, "Space 3 has an agent: only a done Space can be attached to"),
        (r#"{"space":9}"#, "no Space 9: see \"spaces\""),
        (r#"{"profile":"isolated"}"#, "the profile \"isolated\" is not available yet: only \"shared\""),
    ] {
        let mut e = AgentConn::open(&sock);
        e.send(hello);
        assert_eq!(
            e.line(),
            format!("{{\"id\":null,\"ok\":false,\"error\":{}}}\n", serde_like(why)),
            "{hello}"
        );
        assert_eq!(e.line(), "", "{hello}");
    }
    // D closes its Space.
    let (got, _) = d.ask(r#"{"id":3,"cmd":"space_close"}"#);
    assert_eq!(got, "{\"id\":3,\"ok\":true,\"result\":\"closing Space 3\"}\n");
    assert_eq!(d.line(), "");
    wait_for_spaces(pid, 1);
    let (got, _) = a.ask(r#"{"id":6,"cmd":"spaces"}"#);
    assert!(got.contains("\"result\":\"1 space\""), "{got}");
    a.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(a.line(), "");
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let state = text(&out.stdout);
    assert!(
        state.contains("\nspace 1 \"Alpha\" done: spaces\n")
            || state.contains("\nspace 1 \"Alpha\" done: click checkbox"),
        "{state}"
    );
    no_spaces_left(pid).unwrap();
}

/// B9 R6 (P9.4): the toolbar's agent area shows "You" and a chip per
/// Space (a dot by its state, its name); the mouse over a chip shows the
/// Space's state and last action in the status line; a click on a chip
/// opens its menu: Pause (Resume) and Stop act on that Space only. Three
/// agents: the person pauses Alpha with its chip (Alpha's actions fail
/// while Beta's work), stops Gamma (it gets `stopped`, its Space exits,
/// its chip goes), then points at Beta's chip. The person's page keeps
/// its scroll and its height (the chips sit in the toolbar, the menu
/// over the page), and the window never waits for a Space (its
/// DWELLER_TIMING records; the times bounded generously). The last
/// frame, with two Spaces (Alpha paused, Beta idle) and Beta's status,
/// is `browser/tests/ui/spaces_chips.png` (`DWELLER_BLESS=1` rewrites it).
#[test]
fn window_spaces_chips() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    // The address bar is in the PNG: the person's page is a copy at a
    // fixed path.
    let fixed_dir = "/tmp/dweller-chips";
    let page = format!("{fixed_dir}/long.html");
    std::fs::create_dir_all(fixed_dir).unwrap();
    std::fs::write(&page, std::fs::read(root().join("browser/tests/ui/long.html")).unwrap()).unwrap();
    // The person's scroll after the same wheel, without agents.
    let alone = run_window(&[
        &page, "--font", "fixed", "--width", "800", "--height", "600", "--frames", "1", "--state",
        "--events", "wheel:-3",
    ]);
    let alone = text(&alone.stdout);
    let scroll = alone.lines().find(|l| l.starts_with("scroll ")).unwrap().to_string();
    assert_ne!(scroll, "scroll 0", "{alone}");
    // With --font fixed in 800 px, the chips are You 503-536, Alpha
    // 541-599, Beta 604-656, Gamma 661-719 (y 9-31); a menu's items are
    // 24 px from y 38, under its chip. Without Gamma, Beta is 667-719.
    let sock = format!("target/spaces-chips-{}.sock", std::process::id());
    let png = scratch().join("spaces_chips.png");
    let events = "wheel:-3 agents:3 click:570,20 click:560,50 \
                  agents:6 click:690,20 click:680,74 \
                  agents:8 move:690,20 key:cmd+q";
    let window = spaces_window_env(
        &sock,
        &["--spaces", "--state", "--save", png.to_str().unwrap(), "--events", events, &page],
        &[("DWELLER_TIMING", "1")],
    );
    let pid = window.id();
    // The records are read as they come (`spaces_window`).
    let mut agents = Vec::new();
    for (name, site) in [("Alpha", "form.html"), ("Beta", "index.html"), ("Gamma", "docs.html")] {
        let mut c = AgentConn::open(&sock);
        c.send(&format!("{{\"name\":\"{name}\"}}"));
        let (got, _) = c.ask(&format!(
            "{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"browser/tests/agent/site/{site}\"}}"
        ));
        assert!(got.contains("\"ok\":true"), "{name}: {got}");
        agents.push(c);
    }
    let mut gamma = agents.pop().unwrap();
    let mut beta = agents.pop().unwrap();
    let mut alpha = agents.pop().unwrap();
    // Pause on Alpha's chip: Alpha only.
    assert_eq!(alpha.line(), "{\"event\":\"paused\",\"space\":1}\n");
    let (got, _) = beta.ask(r#"{"id":2,"cmd":"click","ref":"e6"}"#);
    assert!(got.contains("\"result\":\"clicked button \\\"Subscribe\\\" [e6]\""), "{got}");
    let (got, _) = alpha.ask(r#"{"id":2,"cmd":"click","ref":"e4"}"#);
    assert_eq!(got, "{\"id\":2,\"ok\":false,\"error\":\"paused by user\"}\n");
    // Stop on Gamma's chip: Gamma ends, the others stay.
    assert_eq!(gamma.line(), "{\"event\":\"stopped\",\"space\":3}\n");
    assert_eq!(gamma.line(), "", "Gamma's connection ends");
    wait_for_spaces(pid, 2);
    let (got, _) = alpha.ask(r#"{"id":3,"cmd":"spaces"}"#);
    assert!(got.contains("\"result\":\"2 spaces\""), "{got}");
    assert!(got.contains("{\"id\":1,\"name\":\"Alpha\",\"mine\":true,\"state\":\"paused\""), "{got}");
    assert!(got.contains("{\"id\":2,\"name\":\"Beta\",\"mine\":false,\"state\":\"idle\""), "{got}");
    // The person points at Beta's chip and closes the window.
    let out = window.wait_with_output().unwrap();
    let (records, rest) = space_records(&text(&out.stderr));
    assert!(
        out.status.success() && rest.is_empty(),
        "the window: {}\n{rest}",
        out.status
    );
    let state = text(&out.stdout);
    for want in [
        format!("\n{scroll}\n"),
        "\nfocus page\n".to_string(),
        "\nviewport 800x532\n".to_string(),
        "\nstatus Beta (idle): click button \"Subscribe\" [e6]\n".to_string(),
        "\nspace 1 \"Alpha\" paused".to_string(),
        "\nspace 2 \"Beta\" idle: click button \"Subscribe\" [e6]\n".to_string(),
    ] {
        assert!(state.contains(&want), "{want:?} in:\n{state}");
    }
    assert!(!state.contains("\nspace 3 "), "Gamma's chip goes:\n{state}");
    assert!(state.contains("\nactivity Gamma stopped\n"), "{state}");
    // The window never waits for a Space: each one's first request (its
    // navigate, which opens a page in the Space) goes out and comes back
    // in different loop iterations, with others between them.
    for name in ["Alpha", "Beta", "Gamma"] {
        let sent = records
            .iter()
            .position(|r| *r == Record::Sent(name.to_string()));
        let answer = records
            .iter()
            .position(|r| *r == Record::Answer(name.to_string()));
        let loops = match (sent, answer) {
            (Some(s), Some(a)) if s < a => records[s..a]
                .iter()
                .filter(|r| matches!(r, Record::Loop(_)))
                .count(),
            _ => 0,
        };
        assert!(
            loops >= 3,
            "{name}'s first request: {loops} loop iterations before its answer: {records:?}"
        );
    }
    let loop_max: u64 = state
        .lines()
        .find_map(|l| l.strip_prefix("loop_max_ms "))
        .unwrap()
        .parse()
        .unwrap();
    // Generous: a few ms, tens on a loaded machine; a wait for a Space
    // opening a page holds an iteration for over 120 ms.
    assert!(loop_max < 120, "loop_max_ms {loop_max}");
    drop(alpha);
    drop(beta);
    no_spaces_left(pid).unwrap();
    let golden = root().join("browser/tests/ui/spaces_chips.png");
    let got = std::fs::read(&png).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    } else {
        assert!(
            got == std::fs::read(&golden).unwrap(),
            "the chips differ from {}: {}",
            golden.display(),
            png.display()
        );
    }
}

// `s` as a JSON string (quotes and backslashes escaped).
fn serde_like(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// `dweller connect` and `--agent-socket` errors.
#[test]
fn window_agents_errors() {
    expect(
        &["connect"],
        2,
        "",
        "error: usage: dweller connect SOCKET [--mcp] [--name NAME] [--space N]\n",
    );
    expect(
        &["connect", "target/no-such.sock"],
        1,
        "",
        "error: cannot connect to target/no-such.sock: is a window listening (dweller --agent-socket target/no-such.sock)?\n",
    );
    if has_sdl() {
        let out = run_window(&[
            "--agent-socket",
            "target/no-such-dir/x.sock",
            "--frames",
            "1",
        ]);
        assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
        assert_eq!(
            text(&out.stderr),
            "error: cannot listen on target/no-such-dir/x.sock\n"
        );
    }
    expect(
        &["--exit-when-idle", "browser/tests/basic.html"],
        2,
        "",
        &format!("error: {}\n", WINDOW_USAGE),
    );
}

/// The agent over HTTP: a link followed from a page served by the test
/// server.
#[test]
fn agent_http() {
    use std::io::Write;
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let mut child = Command::new(dweller())
        .current_dir(root())
        .arg("agent")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!(
        "{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"{base}/agent/site/index.html\"}}\n{{\"id\":2,\"cmd\":\"click\",\"ref\":\"e1\"}}\n"
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(lines.len(), 2, "{got}");
    assert!(lines[0].contains("\"title\":\"Site home\""), "{got}");
    assert!(
        lines[1].contains(&format!("\"url\":\"{base}/agent/site/docs.html\"")),
        "{got}"
    );
    assert!(
        lines[1].contains("heading \\\"Documentation\\\" [level=1] [e10]"),
        "{got}"
    );
}

/// Form submission over HTTP: `browser/tests/agent/http/X.jsonl` (with
/// `$BASE` for the test server's address) fed to `dweller agent` must
/// print `X.out`. The forms in `browser/tests/forms/` submit to the
/// server's `/echo`, whose page shows the request (method, path,
/// Content-Type, fields, raw body).
#[test]
fn agent_forms_http() {
    use std::io::Write;
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("http");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 2,
        "expected at least 2 HTTP sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for input in &inputs {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        let mut child = Command::new(dweller())
            .current_dir(root())
            .arg("agent")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let requests = std::fs::read_to_string(input)
            .unwrap()
            .replace("$BASE", &srv.base);
        child
            .stdin
            .take()
            .unwrap()
            .write_all(requests.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{name}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        let got = rooted(&text(&out.stdout)).replace(&srv.base, "$BASE");
        check_golden(&input.with_extension("out"), &got, &name, &mut failures);
    }
    assert!(
        failures.is_empty(),
        "{} of {} HTTP sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

/// One JS conformance script: None when it passes, else what went wrong.
fn js_case(js: &Path, bless: bool, env: &[(&str, &str)]) -> Option<String> {
    let rel = js
        .strip_prefix(root())
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let out = Command::new(dweller())
        .current_dir(root())
        .args(["js", &rel])
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let got = text(&out.stdout);
    let golden = js.with_extension("out");
    if bless {
        std::fs::write(&golden, &got).unwrap();
        return None;
    }
    let want = std::fs::read_to_string(&golden).unwrap_or_default();
    let mut problems = Vec::new();
    if got != want {
        problems.push(format!(
            "stdout differs from {}\n--- expected\n{want}--- got\n{got}",
            golden.display()
        ));
    }
    let stderr = text(&out.stderr);
    match std::fs::read_to_string(js.with_extension("err")) {
        Ok(want_err) => {
            if stderr != want_err || out.status.code() != Some(1) {
                problems.push(format!(
                    "expected status 1 and stderr {want_err:?}, got {} and {stderr:?}",
                    out.status
                ));
            }
        }
        Err(_) => {
            if !out.status.success() || !stderr.is_empty() {
                problems.push(format!(
                    "expected status 0 and no stderr, got {}:\n{stderr}",
                    out.status
                ));
            }
        }
    }
    if problems.is_empty() {
        None
    } else {
        Some(format!("{rel}: {}", problems.join("\n")))
    }
}

/// Every workload of `browser/tests/jsbench/` once at a small size
/// (`SMOKE=10`); its checksum must equal the golden `NAME.smoke.out`.
/// Timing is `browser/tools/jsbench.py` without `--smoke`.
#[test]
fn jsbench_smoke() {
    let out = match Command::new("python3")
        .current_dir(root())
        .args(["browser/tools/jsbench.py", "--smoke"])
        .env("BENCH_BIN", dweller())
        .env("BENCH_REF", "none")
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            println!("skipped: cannot start python3");
            return;
        }
    };
    assert!(
        out.status.success(),
        "jsbench smoke failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn js_conformance() {
    js_conformance_with(&[], &[]);
}

/// The same scripts and goldens with the bytecode VM on
/// (`DWELLER_JS_VM=1`, BYTECODE.md section 1), then with the VM under
/// `DWELLER_JS_GC=stress` too (the rooting rules of its section 4). The
/// stress pass leaves out the allocation and recursion workloads, which
/// collect millions of times (minutes each).
#[test]
fn js_conformance_vm() {
    js_conformance_with(&[("DWELLER_JS_VM", "1")], &[]);
    js_conformance_with(
        &[("DWELLER_JS_VM", "1"), ("DWELLER_JS_GC", "stress")],
        &[
            "_gc_",
            "_perf_",
            "recursion",
            "many_shapes",
            "map_set_hash",
            "coroutine_stacks",
        ],
    );
}

fn js_conformance_with(env: &[(&str, &str)], skip: &[&str]) {
    let dir = root().join("browser").join("tests").join("js");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "js"))
        .filter(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            !skip.iter().any(|s| name.contains(s))
        })
        .collect();
    scripts.sort();
    assert!(
        scripts.len() >= 150,
        "expected at least 150 JS scripts, found {}",
        scripts.len()
    );
    dweller();
    let bless = env.is_empty() && std::env::var_os("DWELLER_BLESS").is_some();
    let failures: Vec<String> = common::run_queue(&scripts, |js| js_case(js, bless, env))
        .into_iter()
        .flatten()
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} JS scripts failed:\n\n{}",
        failures.len(),
        scripts.len(),
        failures.join("\n\n")
    );
}

/// `browser/tests/js/scopes/X.js` and `X.mjs`: `dweller js --print-scopes`
/// prints `X.scopes`, exit 0 and an empty stderr.
#[test]
fn js_print_scopes() {
    let dir = root().join("browser/tests/js/scopes");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "js" || e == "mjs"))
        .collect();
    scripts.sort();
    assert!(scripts.len() >= 10, "expected at least 10 scope scripts");
    let mut failures = Vec::new();
    for js in &scripts {
        let rel = js.strip_prefix(root()).unwrap().display().to_string();
        let out = run(&["js", "--print-scopes", &rel]);
        if out.status.code() != Some(0) || !out.stderr.is_empty() {
            failures.push(format!(
                "{rel}: status {:?}, stderr:\n{}",
                out.status.code(),
                text(&out.stderr)
            ));
            continue;
        }
        check_golden(
            &js.with_extension("scopes"),
            &text(&out.stdout),
            &rel,
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn js_command_line() {
    expect(
        &["js", "browser/tests/js/missing.js"],
        1,
        "",
        "error: cannot read browser/tests/js/missing.js\n",
    );
    expect(&["js"], 2, "", "error: usage: dweller js FILE\n");
    expect(
        &["js", "browser/tests/js/modules/entry.mjs"],
        0,
        "entry 42 true\n",
        "",
    );
    expect(
        &["js", "browser/tests/js/modules/thrown.mjs"],
        1,
        "",
        "Uncaught 42\n",
    );
    let syntax = run(&["js", "browser/tests/js/modules/syntax.mjs"]);
    assert_eq!(syntax.status.code(), Some(1));
    assert!(
        text(&syntax.stderr).starts_with("Uncaught SyntaxError:"),
        "{}",
        text(&syntax.stderr)
    );
    // P4.61: no compiler yet, a hand-assembled function's disassembly
    let bytecode = run(&["js", "--print-bytecode", "x.js"]);
    assert_eq!(bytecode.status.code(), Some(0));
    assert_eq!(text(&bytecode.stderr), "");
    let out = text(&bytecode.stdout);
    assert!(
        out.contains("@   29 : 46 17          JumpLoop [23] (@ 6)\n")
            && out.contains("Constant pool (size = 1)\n"),
        "{out}"
    );
}

// ---- P10.1: WPT testharness.js pages (`dweller wpt`, browser/tests/wpt)

/// `dweller wpt` on each fixture of browser/tests/wpt (file URLs: the
/// pages load browser/tools/wpt/testharnessreport.js by a relative path)
/// prints its one JSON line on stdout, with an empty stderr.
#[test]
fn wpt_runner() {
    let ok = |name: &str| format!(r#"{{"name":"{name}","status":"PASS","message":null}}"#);
    let line = |status: &str, message: &str, subtests: &[String]| {
        format!(
            r#"{{"status":"{status}","message":{message},"subtests":[{}]}}"#,
            subtests.join(",")
        ) + "\n"
    };
    let cases: &[(&[&str], String)] = &[
        (
            &["browser/tests/wpt/pass.html"],
            line("OK", "null", &[ok("addition"), ok("a timer"), ok("a promise")]),
        ),
        (
            &["browser/tests/wpt/fail.html"],
            line(
                "OK",
                "null",
                &[
                    ok("passes"),
                    r#"{"name":"fails","status":"FAIL","message":"assert_equals: one is two expected 2 but got 1"}"#.into(),
                ],
            ),
        ),
        (
            &["browser/tests/wpt/error.html"],
            line(
                "ERROR",
                r#""Uncaught Error: outside any test""#,
                &[ok("before the error")],
            ),
        ),
        (
            &["browser/tests/wpt/timeout.html"],
            line(
                "TIMEOUT",
                "null",
                &[
                    ok("finishes"),
                    r#"{"name":"never finishes","status":"TIMEOUT","message":"Test timed out"}"#.into(),
                ],
            ),
        ),
        // 15 s of the page's (virtual) time: past the default 10 s.
        (
            &["browser/tests/wpt/slow.html"],
            line(
                "TIMEOUT",
                "null",
                &[r#"{"name":"after 15 s","status":"TIMEOUT","message":"Test timed out"}"#.into()],
            ),
        ),
        (
            &["browser/tests/wpt/slow.html", "--timeout", "60"],
            line("OK", "null", &[ok("after 15 s")]),
        ),
        (
            &["browser/tests/wpt/sum.any.html"],
            line("OK", "null", &[ok("reduce sums"), ok("split")]),
        ),
    ];
    for (args, stdout) in cases {
        let mut all = vec!["wpt"];
        all.extend_from_slice(args);
        expect(&all, 0, stdout, "");
    }
    // A page that does not load: CRASH, exit 1.
    let missing = root().join("browser/tests/wpt/missing.html");
    expect(
        &["wpt", "browser/tests/wpt/missing.html"],
        1,
        &format!(
            "{{\"status\":\"CRASH\",\"message\":\"cannot fetch file://{}: not found\",\"subtests\":[]}}\n",
            missing.display()
        ),
        "",
    );
    // A page without testharness.js never reports: TIMEOUT, no subtests.
    expect(
        &["wpt", "browser/tests/basic.html", "--timeout", "1"],
        0,
        "{\"status\":\"TIMEOUT\",\"message\":\"no result from testharnessreport.js\",\"subtests\":[]}\n",
        "",
    );
    let usage = "error: usage: dweller wpt FILE|URL [--timeout S]\n";
    expect(&["wpt"], 2, "", usage);
    expect(
        &["wpt", "browser/tests/wpt/pass.html", "--timeout", "0"],
        2,
        "",
        usage,
    );
}

// P10.3: `dweller js --test262` and browser/tools/test262.py over the
// fixtures in browser/tests/test262.
#[test]
fn test262_runner() {
    let hooks = root().join("target").join("t262-hooks.js");
    std::fs::create_dir_all(hooks.parent().unwrap()).unwrap();
    std::fs::write(
        &hooks,
        "print('p'); print($262.global === globalThis); \
         print($262.evalScript('var q = 4; q + 1'));\n\
         var ab = new ArrayBuffer(8); $262.detachArrayBuffer(ab); $262.gc(); print(ab.byteLength);\n",
    )
    .unwrap();
    let hooks = hooks.to_str().unwrap().to_string();
    expect(&["js", "--test262", &hooks], 0, "p\ntrue\n5\n0\n", "");
    // without the flag there is no print or $262
    let plain = run(&["js", &hooks]);
    assert_eq!(plain.status.code(), Some(1));
    assert!(
        text(&plain.stderr).starts_with("Uncaught ReferenceError"),
        "{}",
        text(&plain.stderr)
    );

    let tool = root().join("browser").join("tools").join("test262.py");
    let fixtures = root().join("browser").join("tests").join("test262");
    let out = root().join("target").join("t262-fixtures");
    let res = Command::new("python3")
        .arg(&tool)
        .args(["run", "test", "-j", "4", "--root"])
        .arg(&fixtures)
        .arg("--dweller")
        .arg(dweller())
        .arg("--out")
        .arg(&out)
        .output();
    let Ok(res) = res else {
        println!("skipped: cannot start python3");
        return;
    };
    assert!(res.status.success(), "{}", text(&res.stderr));
    let stdout = text(&res.stdout);
    assert!(
        stdout.contains("test262 test: 8/11 passed (72.7%), 1 skipped"),
        "{stdout}"
    );
    let json = std::fs::read_to_string(out.join("test.json")).unwrap();
    for (name, status) in [
        ("test/pass/basic.js", "pass"),
        ("test/pass/skipped.js", "skip"),
        ("test/fail/wrong.js", "fail"),
        ("test/negative/parse.js", "pass"),
        ("test/negative/runtime.js", "pass"),
        ("test/negative/missing.js", "fail"),
        ("test/async/promise.js", "pass"),
        ("test/async/never.js", "fail"),
        ("test/strict/only.js", "pass"),
        ("test/strict/both.js", "pass"),
        ("test/strict/nostrict.js", "pass"),
        ("test/module/basic.js", "pass"),
    ] {
        let at = json
            .find(&format!("\"{name}\""))
            .unwrap_or_else(|| panic!("{name}: {json}"));
        let rest = &json[at..];
        let line = &rest[..rest.find(']').unwrap()];
        assert!(line.contains(&format!("\"{status}\"")), "{name}: {line}");
    }
}

// ---- page scripts and the DOM bindings (browser/engine/script.aye)

/// The window, headless: a click and a key reach the page's listeners,
/// which change the text (`--text` prints the document's text at the
/// end); with `--no-js` nothing changes.
#[test]
fn window_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let cases: &[(&[&str], &str)] = &[
        (&["--events", "click:50,65"], "\nclicked\n"),
        (&["--events", "key:x"], "\nkey x\n"),
        (&["--events", "click:50,200"], "\nbefore\n"),
        (&["--events", "click:50,65", "--no-js"], "\nbefore\n"),
    ];
    for (extra, want) in cases {
        let mut args = vec![
            "browser/tests/ui/script.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "1",
            "--text",
        ];
        args.extend_from_slice(extra);
        let out = run_window(&args);
        let got = text(&out.stdout);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{args:?}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        assert!(
            got.contains(want),
            "{args:?}: expected {want:?} in the text:\n{got}"
        );
    }
}

/// The page's log (`<pre id=log>`) after `events` ran in the window, as
/// the lines of its text between the page's markup and its script.
fn window_log(page: &str, events: &str) -> Vec<String> {
    let out = run_window(&[
        page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "1", "--text",
        "--events", events,
    ]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{page} {events:?}: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    got.lines()
        .skip_while(|l| *l != "pad" && *l != "canvas")
        .skip(1)
        .take_while(|l| !l.is_empty())
        .map(str::to_string)
        .collect()
}

/// Trusted key and pointer events of the window
/// (`browser/tests/ui/input_codes.html` logs them): `code` from the
/// scancode, `repeat`, `location`, modifiers, keyup; pointer events
/// before the mouse events with pointerId 1; other buttons; leave and
/// cancel; `movementX/Y`; visibilitychange.
#[test]
fn window_input_codes() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/ui/input_codes.html";
    let keys = window_log(
        page,
        "key:a key:shift+a key:repeat+a keyup:a key:KeyW key:ShiftLeft key:ShiftRight \
         key:left key:Numpad5 key:F5 key:alt+x key:Escape keyup:ShiftLeft key:tab",
    );
    assert_eq!(
        keys,
        [
            "keydown a KeyA r0 l0  true",
            "keydown A KeyA r0 l0 S true",
            "keydown a KeyA r1 l0  true",
            "keyup a KeyA r0 l0  true",
            "keydown w KeyW r0 l0  true",
            "keydown Shift ShiftLeft r0 l1  true",
            "keydown Shift ShiftRight r0 l2  true",
            "keydown ArrowLeft ArrowLeft r0 l0  true",
            "keydown 5 Numpad5 r0 l3  true",
            "keydown F5 F5 r0 l0  true",
            "keydown x KeyX r0 l0 A true",
            "keydown Escape Escape r0 l0  true",
            "keyup Shift ShiftLeft r0 l1  true",
            "keydown Tab Tab r0 l0  true",
        ]
    );
    // The pad is the first 100 px under the 40 px toolbar.
    let pointer = window_log(
        page,
        "move:20,60 move:25,70 click:25,70 release:25,70 click:30,70,right \
         release:30,70,right move:40,80,3,-4 window:blur click:30,70 window:blur \
         window:leave window:hidden window:shown",
    );
    assert_eq!(
        pointer,
        [
            "pointermove 20,20 b-1/0 id1P mouse m0,0",
            "mousemove 20,20 b0/0 m0,0",
            "pointermove 25,30 b-1/0 id1P mouse m5,10",
            "mousemove 25,30 b0/0 m5,10",
            "pointerdown 25,30 b0/1 id1P mouse m0,0",
            "mousedown 25,30 b0/1 m0,0",
            "click 0,0 b0/0 m0,0",
            "pointerup 25,30 b0/0 id1P mouse m0,0",
            "mouseup 25,30 b0/0 m0,0",
            "pointerdown 30,30 b2/2 id1P mouse m0,0",
            "mousedown 30,30 b2/2 m0,0",
            "pointerup 30,30 b2/0 id1P mouse m0,0",
            "mouseup 30,30 b2/0 m0,0",
            "pointermove 40,40 b-1/0 id1P mouse m3,-4",
            "mousemove 40,40 b0/0 m3,-4",
            "pointerdown 30,30 b0/1 id1P mouse m0,0",
            "mousedown 30,30 b0/1 m0,0",
            "click 0,0 b0/0 m0,0",
            "pointercancel 30,30 b0/0 id1P mouse m0,0",
            "pointerleave 30,30 b0/0 id1P mouse m0,0",
            "vis hidden true",
            "vis visible false",
        ]
    );
}

/// Pointer Lock and Fullscreen in the window
/// (`browser/tests/ui/pointer_lock.html`): the requests are promises that
/// need a user activation (rejected with `pointerlockerror` /
/// `fullscreenerror` before one), `pointerLockElement` and
/// `fullscreenElement`, the change events, relative motion as
/// `movementX/Y` on the locked element, Escape and `exitPointerLock`.
/// (Under SDL's dummy driver the DOM side is what is checked.)
#[test]
fn window_pointer_lock() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/ui/pointer_lock.html";
    let log = window_log(page, "click:50,70 move:70,90,13,-7 click:60,80");
    assert_eq!(
        log,
        [
            "start locked=true fs=true",
            "promise true",
            "pointerlockerror",
            "lock rejected NotAllowedError",
            "fullscreenerror",
            "fs rejected TypeError",
            "click active=true",
            "pointerlockchange c",
            "locked true",
            "fullscreenchange c true",
            "fullscreen true",
            "pointermove m13,-7",
            "mousemove 50,30 m13,-7",
            "click active=true",
            "locked true",
            "fullscreen true",
        ]
    );
    // Escape leaves both and is not a key of the page.
    let esc = window_log(page, "click:50,70 key:escape key:x");
    assert_eq!(
        esc[esc.len() - 4..],
        [
            "pointerlockchange null",
            "fullscreenchange null true",
            "keydown x",
            "exit asked",
        ]
    );
    // exitPointerLock while locked.
    let exit = window_log(page, "click:50,70 key:x");
    assert_eq!(
        exit[exit.len() - 3..],
        ["keydown x", "exit asked", "pointerlockchange null"]
    );
}

/// The History API in the window (`browser/tests/agent/site/spa.html`):
/// buttons that call pushState change the address without loading, and
/// back moves in the document (popstate shows item 2 again).
#[test]
fn window_history_api() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let args = [
        "browser/tests/agent/site/spa.html",
        "--font",
        "fixed",
        "--frames",
        "5",
        "--state",
        "--text",
        "--events",
        "click:30,133 click:140,133 key:cmd+[",
    ];
    let out = run_window(&args);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let page = format!(
        "file://{}/browser/tests/agent/site/spa.html?item=2",
        root().display()
    );
    for want in [
        format!("url {page}\n"),
        format!("\naddress {page}\n"),
        "\nhistory 2 of 3\n".into(),
        "\nItem 2\n".into(),
        "\nhistory.length 3\n".into(),
    ] {
        assert!(got.contains(&want), "expected {want:?} in:\n{got}");
    }
}

/// Timers that never let the queue empty (animation callbacks that take
/// longer than a frame, a task that queues itself at 0 ms) must not
/// starve the window: each event-loop turn runs only the timers queued
/// before it, then the window draws, so `--frames N` ends.
#[test]
fn window_timers_yield_to_frames() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .args([
            "browser/tests/ui/raf_overload.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "10",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 10 frames in 60 s: timers starve it");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains("animating"),
        "{}",
        text(&out.stdout)
    );
}

/// The window lays out a script's changes again where they are, with the
/// layout the scripts read (`Page.tree_borrowed`): one full cascade for
/// the load, none for the frames after; and each layout that keeps boxes
/// matches a full one (`DWELLER_LAYOUT_CHECK`, styles and boxes).
#[test]
fn window_relayout_incremental() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run = |env: &str, without: &str| {
        let out = Command::new(dweller())
            .current_dir(root())
            .env("SDL_VIDEODRIVER", "dummy")
            .env(env, "1")
            .env_remove(without)
            .args([
                "browser/tests/ui/relayout_incremental.html",
                "--font",
                "fixed",
                "--width",
                "400",
                "--height",
                "300",
                "--frames",
                "40",
                "--text",
            ])
            .output()
            .unwrap();
        assert!(
            text(&out.stdout).contains("frame "),
            "{}",
            text(&out.stdout)
        );
        (out.status, text(&out.stderr))
    };
    let (status, stderr) = run("DWELLER_LAYOUT_CHECK", "DWELLER_TIMING");
    assert!(status.success() && stderr.is_empty(), "{status}\n{stderr}");
    // Without the check, whose full cascades would count.
    let (status, stderr) = run("DWELLER_TIMING", "DWELLER_LAYOUT_CHECK");
    let count = |phase: &str| {
        stderr
            .lines()
            .filter(|l| l.contains(&format!("DWELLER_TIMING\t{phase}\t")))
            .count()
    };
    assert!(status.success(), "{status}\n{stderr}");
    assert!(count("ui_relayout") >= 6, "too few relayouts:\n{stderr}");
    assert_eq!(count("cascade"), 1, "full cascades after the load:\n{stderr}");
    assert!(count("restyle") >= 5, "no subtree restyles:\n{stderr}");
}

/// The window draws only what changed (P4.35, ui/pagedraw.aye): with
/// `DWELLER_DAMAGE_CHECK` each frame is drawn whole too and compared; a
/// canvas animation draws part of the page, an unchanged frame nothing.
#[test]
fn window_damage() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/canvas_animate.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "40",
            "--events",
            "frame frame wheel:-3 frame wheel:3 frame",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("DAMAGE CHECK"), "{err}");
    let px: Vec<u64> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\tui_damage_px\t").nth(1))
        .map(|r| {
            r.split('\t')
                .nth(1)
                .unwrap()
                .split(' ')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    let full = 400 * 260;
    assert!(
        px.iter().any(|&p| p > 0 && p < full),
        "no partial frame: {px:?}"
    );
    assert!(px.contains(&0), "no frame without damage: {px:?}");
}

/// A trackpad scrolls by fractions of a device pixel (P4.36b): the window
/// draws the page at its scroll offset snapped to device pixels, so such a
/// frame moves the last frame's pixels and draws the band, the fixed
/// header and the sticky heading (or nothing), never the whole page, and
/// `DWELLER_DAMAGE_CHECK` finds each frame equal to a whole drawing.
#[test]
fn window_fractional_scroll() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut events = vec!["wheel:-0.03 frame"; 30];
    events.extend(vec!["wheel:-0.37 frame"; 20]);
    events.extend(vec!["wheel:0.11 frame"; 10]);
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/fractional_scroll.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "80",
            "--state",
            "--events",
            &events.join(" "),
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("DAMAGE CHECK"), "{err}");
    assert!(text(&out.stdout).contains("\nscroll 72\n"), "{}", text(&out.stdout));
    let px: Vec<u64> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\tui_damage_px\t").nth(1))
        .map(|r| {
            r.split('\t')
                .nth(1)
                .unwrap()
                .split(' ')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    let full = 400 * 260;
    let whole = px.iter().filter(|&&p| p >= full).count();
    let partial = px.iter().filter(|&&p| p > 0 && p < full).count();
    assert!(whole <= 2, "{whole} frames drawn whole: {px:?}");
    assert!(partial >= 30, "{partial} partial frames: {px:?}");
}

/// P11.7 (docs/webgl.md §5): the window shows a WebGL canvas through its
/// readback. `spin.html` turns a triangle on a 1280x720 canvas in 120
/// animation frames; each frame is read back at its end and mapped into
/// the canvas in the next, so the canvas changes in 120 frames (and the
/// cleared one before them), each damaging the canvas alone (with the
/// one-pixel margin below it the damage keeps for antialiasing), and
/// `DWELLER_DAMAGE_CHECK` finds every frame drawn right. Prints the
/// `webgl_readback` p50 and p95. Without ANGLE it is skipped.
#[test]
fn window_webgl() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .env("DWELLER_GL", "swiftshader")
        .args([
            "browser/tests/webgl/spin.html",
            "--font",
            "fixed",
            "--width",
            "1280",
            "--height",
            "800",
            "--frames",
            "300",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("DAMAGE CHECK"), "{err}");
    let record = |phase: &str| -> Vec<(u64, String)> {
        let key = format!("DWELLER_TIMING\t{phase}\t");
        err.lines()
            .filter_map(|l| l.split(key.as_str()).nth(1))
            .map(|r| {
                let mut f = r.split('\t');
                let ns = f.next().unwrap().parse().unwrap();
                (ns, f.next().unwrap_or("").to_string())
            })
            .collect()
    };
    let mut readback: Vec<u64> = record("webgl_readback")
        .into_iter()
        .map(|(ns, label)| {
            assert_eq!(label, "1280x720");
            ns
        })
        .collect();
    assert!(readback.len() >= 120, "{} readbacks:\n{err}", readback.len());
    // The page area is 1280x760 (the chrome takes 40 px).
    let canvas = 1280 * 721;
    let full = 1280 * 760;
    let px: Vec<u64> = record("ui_damage_px")
        .iter()
        .map(|(_, label)| label.split(' ').next().unwrap().parse().unwrap())
        .collect();
    assert!(
        px.iter().all(|&p| p == 0 || p == canvas || p == full),
        "damage outside the canvas: {px:?}"
    );
    let updates = px.iter().filter(|&&p| p == canvas).count();
    assert!(updates >= 120, "{updates} canvas updates: {px:?}");
    assert!(px.iter().filter(|&&p| p == full).count() <= 2, "{px:?}");
    readback.sort();
    let ms = |q: f64| readback[((readback.len() as f64 * q) as usize).min(readback.len() - 1)] as f64 / 1e6;
    println!("webgl_readback 1280x720: p50 {:.2} ms, p95 {:.2} ms", ms(0.5), ms(0.95));
}

/// The pixels of a PNG as `dweller screenshot` writes it
/// (`browser/paint/png.aye`: 8-bit RGBA, filter 0 on every row, a zlib
/// stream of stored or fixed-Huffman blocks): width, height, RGBA.
fn png_rgba(png: &[u8]) -> (usize, usize, Vec<u8>) {
    let be = |i: usize| u32::from_be_bytes(png[i..i + 4].try_into().unwrap()) as usize;
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&png[24..29], b"\x08\x06\x00\x00\x00", "not 8-bit RGBA");
    let (w, h) = (be(16), be(20));
    let mut z = Vec::new();
    let mut i = 8;
    while i + 8 <= png.len() {
        let len = be(i);
        if &png[i + 4..i + 8] == b"IDAT" {
            z.extend_from_slice(&png[i + 8..i + 8 + len]);
        }
        i += 12 + len;
    }
    let raw = inflate(&z[2..]);
    assert_eq!(raw.len(), h * (w * 4 + 1));
    let mut px = Vec::with_capacity(w * h * 4);
    for row in raw.chunks(w * 4 + 1) {
        assert_eq!(row[0], 0, "a row filter other than none");
        px.extend_from_slice(&row[1..]);
    }
    (w, h, px)
}

/// Raw deflate (RFC 1951) of stored and fixed-Huffman blocks only.
fn inflate(d: &[u8]) -> Vec<u8> {
    const LEN: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [usize; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    let mut pos = 0usize;
    // `n` bits, least significant first.
    let take = |n: usize, pos: &mut usize| -> usize {
        let mut v = 0;
        for k in 0..n {
            v |= ((d[*pos / 8] >> (*pos % 8)) as usize & 1) << k;
            *pos += 1;
        }
        v
    };
    let mut out = Vec::new();
    loop {
        let last = take(1, &mut pos);
        let kind = take(2, &mut pos);
        if kind == 0 {
            pos = pos.div_ceil(8) * 8;
            let n = take(16, &mut pos);
            take(16, &mut pos);
            out.extend_from_slice(&d[pos / 8..pos / 8 + n]);
            pos += n * 8;
        } else {
            assert_eq!(kind, 1, "a dynamic-Huffman block");
            loop {
                // Huffman codes come most significant bit first.
                let mut code = 0;
                for _ in 0..7 {
                    code = code << 1 | take(1, &mut pos);
                }
                let sym = if code <= 0x17 {
                    256 + code
                } else {
                    code = code << 1 | take(1, &mut pos);
                    if (0x30..=0xbf).contains(&code) {
                        code - 0x30
                    } else if (0xc0..=0xc7).contains(&code) {
                        280 + code - 0xc0
                    } else {
                        (code << 1 | take(1, &mut pos)) - 0x190 + 144
                    }
                };
                if sym < 256 {
                    out.push(sym as u8);
                    continue;
                }
                if sym == 256 {
                    break;
                }
                let s = sym - 257;
                let len = LEN[s] + take(LEN_EXTRA[s], &mut pos);
                let mut dc = 0;
                for _ in 0..5 {
                    dc = dc << 1 | take(1, &mut pos);
                }
                let dist = DIST[dc] + take(dc.saturating_sub(2) / 2, &mut pos);
                for _ in 0..len {
                    out.push(out[out.len() - dist]);
                }
            }
        }
        if last == 1 {
            return out;
        }
    }
}

/// P11.9: three.js r170 (`browser/tests/webgl/three/`) draws
/// `three_cube.html` on SwiftShader: a MeshStandardMaterial cube with a
/// PNG texture and one directional light, in a 60-frame
/// `setAnimationLoop`. `dump --js` logs its message and no error; `dweller screenshot`
/// (the last frame) is `three_cube.png` but for at most 1% of the pixels
/// off by more than 8 in a channel (SwiftShader on another CPU may round
/// otherwise); in the window, the 60 frames draw (60 `webgl_readback`
/// records of 240x180). Prints the JS time of the drawing frames' timers
/// (the rAF callback, three's render and the readback's start) p50/p95,
/// and the same with `#finish` (the page calls gl.finish() after each
/// render): their difference is the GPU's time. Without ANGLE it is
/// skipped.
#[test]
fn webgl_three_cube() {
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let page = "browser/tests/webgl/three_cube.html";
    let out = Command::new(dweller())
        .current_dir(root())
        .args(["dump", "--text", "--js", page])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    // The console (on stderr) has the page's one message and no error.
    assert!(out.status.success(), "{}", out.status);
    assert_eq!(text(&out.stderr), "[log] three cube: 60 frames\n");

    let out_png = scratch().join("three_cube.png");
    let res = Command::new(dweller())
        .current_dir(root())
        .args([
            "screenshot",
            page,
            "-o",
            out_png.to_str().unwrap(),
            "--font",
            "fixed",
            "--width",
            "240",
            "--height",
            "180",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}\n{}",
        res.status,
        text(&res.stderr)
    );
    let got = std::fs::read(&out_png).unwrap();
    let golden = root().join("browser/tests/webgl/three_cube.png");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    }
    let (w, h, a) = png_rgba(&got);
    let (gw, gh, b) = png_rgba(&std::fs::read(&golden).unwrap());
    assert_eq!((w, h), (gw, gh), "size differs from {}", golden.display());
    let off = a
        .chunks(4)
        .zip(b.chunks(4))
        .filter(|(p, q)| p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > 8))
        .count();
    assert!(
        off * 100 <= w * h,
        "{off} pixels differ from {} (new one: {})",
        golden.display(),
        out_png.display()
    );

    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let frames = |target: &str| -> Vec<u64> {
        let out = Command::new(dweller())
            .current_dir(root())
            .env("SDL_VIDEODRIVER", "dummy")
            .env("DWELLER_DAMAGE_CHECK", "1")
            .env("DWELLER_TIMING", "1")
            .env("DWELLER_GL", "swiftshader")
            .args([
                target, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "200",
            ])
            .output()
            .unwrap();
        let err = text(&out.stderr);
        assert!(out.status.success(), "{}\n{err}", out.status);
        assert!(!err.contains("DAMAGE CHECK"), "{err}");
        // A frame that drew: its timer, then its readback.
        let recs: Vec<(&str, u64, &str)> = err
            .lines()
            .filter_map(|l| l.split("DWELLER_TIMING\t").nth(1))
            .map(|r| {
                let f: Vec<&str> = r.splitn(3, '\t').collect();
                (f[0], f[1].parse().unwrap(), f.get(2).copied().unwrap_or(""))
            })
            .filter(|r| r.0 == "timers" || r.0 == "webgl_readback")
            .collect();
        let readbacks = recs.iter().filter(|r| r.0 == "webgl_readback").count();
        assert!(readbacks >= 60, "{readbacks} readbacks:\n{err}");
        assert!(
            recs.iter()
                .all(|r| r.0 != "webgl_readback" || r.2 == "240x180"),
            "{err}"
        );
        let mut js: Vec<u64> = recs
            .windows(2)
            .filter(|p| p[0].0 == "timers" && p[1].0 == "webgl_readback")
            .map(|p| p[0].1)
            .collect();
        assert!(js.len() >= 60, "{} drawn frames:\n{err}", js.len());
        js.sort();
        js
    };
    let ms = |v: &[u64], q: f64| v[((v.len() as f64 * q) as usize).min(v.len() - 1)] as f64 / 1e6;
    let js = frames(page);
    let fin = frames(&format!("{page}#finish"));
    println!(
        "three cube frame JS: p50 {:.2} ms, p95 {:.2} ms; with gl.finish(): p50 {:.2} ms, p95 {:.2} ms",
        ms(&js, 0.5),
        ms(&js, 0.95),
        ms(&fin, 0.5),
        ms(&fin, 0.95)
    );
}

/// A page shaped like GitHub's, with many container queries on nested
/// query containers, served over HTTP: after the load, the frames that
/// change a list (and the side column's width every 15th, which flips
/// its `@container` conditions) style again only the changed subtrees
/// and the containers whose size changed, never the whole document
/// (geometry.aye `settleContainers`); every such layout equals a full
/// one (P4.44).
#[test]
fn window_container_queries() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let url = format!("{}/ui/container_feed.html", srv.base);
    let run = |env: &str, without: &str| {
        let out = Command::new(dweller())
            .current_dir(root())
            .env("SDL_VIDEODRIVER", "dummy")
            .env(env, "1")
            .env_remove(without)
            .args([
                &url, "--font", "fixed", "--width", "800", "--height", "600", "--frames", "90",
                "--text",
            ])
            .output()
            .unwrap();
        assert!(
            text(&out.stdout).contains("event 79"),
            "{}",
            text(&out.stdout)
        );
        (out.status, text(&out.stderr))
    };
    let (status, stderr) = run("DWELLER_LAYOUT_CHECK", "DWELLER_TIMING");
    assert!(status.success() && stderr.is_empty(), "{status}\n{stderr}");
    let (status, stderr) = run("DWELLER_TIMING", "DWELLER_LAYOUT_CHECK");
    assert!(status.success(), "{status}\n{stderr}");
    let phases: Vec<&str> = stderr
        .lines()
        .filter_map(|l| l.split('\t').nth(1))
        .collect();
    let count = |phase: &str| phases.iter().filter(|p| **p == phase).count();
    // The first relayouts style the loaded page (container queries: up
    // to three cascades); none after.
    let loaded = phases
        .iter()
        .enumerate()
        .filter(|(_, p)| **p == "ui_relayout")
        .nth(1)
        .map(|(i, _)| i)
        .unwrap_or(phases.len());
    let late = phases[loaded..].iter().filter(|p| **p == "cascade").count();
    assert!(count("ui_relayout") >= 50, "too few relayouts:\n{stderr}");
    assert_eq!(late, 0, "full cascades after the load:\n{stderr}");
    assert!(
        count("container_restyle") >= 4,
        "no container restyles:\n{stderr}"
    );
}

/// A transform and opacity animation is composited (P4.56): a 60-frame
/// `requestAnimationFrame` carousel moving a 600x300 track 1 px a frame
/// (and fading a box) makes no relayout after the load and rasterizes
/// no layer once they are promoted (`ui_composite`, `ui_layers` of
/// DWELLER_TIMING); every frame equals a whole drawing and every kept
/// layout a full one (`DWELLER_DAMAGE_CHECK`, `DWELLER_LAYOUT_CHECK`).
#[test]
fn window_transform_composite() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_LAYOUT_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/carousel.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "400",
            "--frames",
            "200",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("CHECK"), "{err}");
    // The phases in order, with their labels.
    let recs: Vec<(&str, &str)> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\t").nth(1))
        .map(|r| {
            let f: Vec<&str> = r.split('\t').collect();
            (f[0], *f.get(2).unwrap_or(&""))
        })
        .collect();
    let composites = recs.iter().filter(|r| r.0 == "ui_composite").count();
    assert!(composites >= 50, "{composites} composited frames:\n{err}");
    // After the first composited frame: no relayout, and the layers
    // drawn come from the raster cache.
    let first = recs.iter().position(|r| r.0 == "ui_composite").unwrap();
    let after = &recs[first..];
    let relayouts = after.iter().filter(|r| r.0 == "ui_relayout").count();
    assert_eq!(relayouts, 0, "relayouts after the load:\n{err}");
    let mut kept = 0;
    for (_, label) in after.iter().filter(|r| r.0 == "ui_layers") {
        let mut n = label.split(' ').map(|x| x.parse::<u64>().unwrap());
        kept += n.next().unwrap();
        assert_eq!(n.next().unwrap(), 0, "a layer rasterized again:\n{err}");
    }
    assert!(kept >= 50, "{kept} layers from the cache:\n{err}");
}

/// mistral.ai's squares (P4.57): a rAF loop that changes the `width` of
/// 6 divs in the flex cells of a grid, on a page with images, lays out
/// again only the boxes around them: after the first frame no layout of
/// the whole document (`layout_full`, the page's `layout`), each frame a
/// relayout of the changed boxes (`layout_dirty`), the same boxes as a
/// full layout (DWELLER_LAYOUT_CHECK).
#[test]
fn window_width_animation() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_LAYOUT_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/squares.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "400",
            "--frames",
            "100",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("CHECK"), "{err}");
    let phases: Vec<&str> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\t").nth(1))
        .map(|r| r.split('\t').next().unwrap())
        .collect();
    let first = phases.iter().position(|p| *p == "ui_draw").unwrap();
    let after = &phases[first..];
    let count = |name: &str| after.iter().filter(|p| **p == name).count();
    assert!(
        count("ui_relayout") >= 50,
        "{} relayouts:\n{err}",
        count("ui_relayout")
    );
    assert_eq!(
        count("layout_full") + count("layout"),
        0,
        "full layouts after the first frame:\n{err}"
    );
    assert!(
        count("layout_dirty") >= 50,
        "{} dirty relayouts:\n{err}",
        count("layout_dirty")
    );
}

/// The window and the scripts agree on the scroll offset: a script's
/// `scrollTo` scrolls the window, and the wheel moves `scrollY`.
#[test]
fn window_scroll_follows_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = root().join("browser/tests/ui/scroll_sync.html");
    let script = format!("file://{}?script", page.display());
    let cases: &[(&str, &[&str], &str, &str)] = &[
        (&script, &[], "scroll 500\n", "y=500\n"),
        (
            "browser/tests/ui/scroll_sync.html",
            &["--events", "wheel:-3"],
            "scroll 120\n",
            "y=120\n",
        ),
    ];
    for (target, extra, state, text_line) in cases {
        let mut args = vec![
            *target, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "30",
            "--state", "--text",
        ];
        args.extend_from_slice(extra);
        let out = run_window(&args);
        let got = text(&out.stdout);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{args:?}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        assert!(
            got.contains(state) && got.contains(text_line),
            "{args:?}: expected {state:?} and {text_line:?} in:\n{got}"
        );
    }
}

/// macOS wheel and trackpad scrolling (window.aye `wheel`): a fractional
/// SDL delta is a precise device's (10 px a unit, summed exactly), a
/// whole one a wheel notch (40 px); SDL_MOUSEWHEEL_FLIPPED (natural
/// scrolling) is not flipped again; X scrolls sideways; and the wheel
/// events of one loop iteration make one `wheel` DOM event (deltaMode 0,
/// positive down and right), one scroll and no layout.
#[test]
fn window_wheel() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let repeat = |word: &str, n: usize| vec![word; n].join(" ");
    let run = |page: &str, events: &str, extra: &[&str]| {
        let mut args = vec![
            page, "--font", "fixed", "--width", "400", "--height", "340", "--frames", "30",
            "--state", "--events", events,
        ];
        args.extend_from_slice(extra);
        run_window(&args)
    };
    let long = "browser/tests/ui/long.html";
    // Events, then the scroll offset of long.html (300 px viewport).
    let cases = [
        (repeat("wheel:-0.05", 30), 15),
        (repeat("wheel:-0.25", 10), 25),
        ("wheel:-1".to_string(), 40),
        (repeat("wheel:0,-0.5,flipped", 4), 20),
        ("wheel:0,-1,flipped".to_string(), 40),
        // A whole delta in a trackpad's stream is the trackpad's.
        ("wheel:-0.5 wheel:-1".to_string(), 15),
        (format!("{} {}", repeat("wheel:-0.5", 40), repeat("wheel:0.25", 4)), 190),
    ];
    let mut failures = Vec::new();
    for (events, want) in &cases {
        let out = run(long, events, &[]);
        let got = text(&out.stdout);
        let line = format!("\nscroll {want}\n");
        if !out.status.success() || !out.stderr.is_empty() || !got.contains(&line) {
            failures.push(format!("{events}: expected scroll {want}:\n{got}{}", text(&out.stderr)));
        }
    }
    // Sideways: 4 x 95 px brings the far right link of wide.html under x
    // 250 (one frame, as in window_sideways_scroll).
    let events = format!("{} move:250,80", repeat("wheel:9.5,0", 4));
    let out = run_window(&[
        "browser/tests/ui/wide.html", "--font", "fixed", "--width", "400", "--height", "340", "--frames", "1",
        "--state", "--events", &events,
    ]);
    let got = text(&out.stdout);
    if !out.status.success() || !got.lines().any(|l| l.starts_with("status ") && l.ends_with("other.html")) {
        failures.push(format!("sideways: expected other.html under the mouse:\n{got}{}", text(&out.stderr)));
    }
    // One wheel event of the summed deltas for 8 SDL events; the window's
    // own listener is passive; ?cancel: the div's listener cancels it.
    let page = root().join("browser/tests/ui/wheel.html");
    let events = format!("move:100,200 {}", repeat("wheel:0.5,-0.25", 8));
    for (query, scroll, line) in [
        ("", 20, "wheel=1 dx=40 dy=20 mode=0 scrolls=1 y=20"),
        ("?cancel", 0, "wheel=1 dx=40 dy=20 mode=0 scrolls=0 y=0"),
    ] {
        let target = format!("file://{}{query}", page.display());
        let out = run(&target, &events, &["--text"]);
        let got = text(&out.stdout);
        let want = format!("\nscroll {scroll}\n");
        if !out.status.success() || !out.stderr.is_empty() || !got.contains(&want) || !got.contains(line) {
            failures.push(format!("wheel.html{query}: expected {want:?} and {line:?}:\n{got}{}", text(&out.stderr)));
        }
    }
    // Scrolling repaints: no layout, and no display list built again; in
    // a frame (frames.html), one list for all the events of an iteration.
    let phases = |page: &str, events: &str| {
        let out = Command::new(dweller())
            .current_dir(root())
            .env("SDL_VIDEODRIVER", "dummy")
            .env("DWELLER_TIMING", "1")
            .args([page, "--font", "fixed", "--width", "400", "--height", "340", "--frames", "3", "--events", events])
            .output()
            .unwrap();
        let err = text(&out.stderr);
        let count = |phase: &str| err.lines().filter(|l| l.contains(&format!("DWELLER_TIMING\t{phase}\t"))).count();
        (count("layout"), count("script_layout"), count("paint_build"), count("paint"))
    };
    let frames = "browser/tests/ui/frames.html";
    for (page, one, many) in [
        (long, "move:100,200".to_string(), format!("move:100,200 {}", repeat("wheel:-0.3", 50))),
        (frames, "move:200,100 wheel:-1".to_string(), format!("move:200,100 {}", repeat("wheel:-0.3", 50))),
    ] {
        let (a, b) = (phases(page, &one), phases(page, &many));
        if a != b {
            failures.push(format!("{page}: phases (layout, script_layout, paint_build, paint) {a:?} for {one:?}, {b:?} for 50 wheel events"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// One run of a DOM fixture: `args` then the page; stdout must be
/// `golden`, and stderr (console messages, uncaught exceptions) must be
/// `err_golden`, or empty when there is no such file.
fn jsdom_case(
    args: &[&str],
    page: &Path,
    golden: &Path,
    err_golden: Option<&Path>,
    bless: bool,
) -> Option<String> {
    let rel = page
        .strip_prefix(root())
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let mut all: Vec<&str> = args.to_vec();
    all.push(&rel);
    // Each forced layout that keeps boxes is compared with a full one; a
    // difference goes to stderr ("LAYOUT CHECK").
    let out = Command::new(dweller())
        .current_dir(root())
        .env("DWELLER_LAYOUT_CHECK", "1")
        .args(&all)
        .output()
        .unwrap();
    let got = rooted(&text(&out.stdout));
    let stderr = rooted(&text(&out.stderr));
    if bless {
        std::fs::write(golden, &got).unwrap();
        if let Some(e) = err_golden {
            if stderr.is_empty() {
                let _ = std::fs::remove_file(e);
            } else {
                std::fs::write(e, &stderr).unwrap();
            }
        }
        return None;
    }
    let mut problems = Vec::new();
    if !out.status.success() {
        problems.push(format!("status {}", out.status));
    }
    let want = std::fs::read_to_string(golden).unwrap_or_default();
    if got != want {
        problems.push(format!(
            "stdout differs from {}\n--- expected\n{want}--- got\n{got}",
            golden.display()
        ));
    }
    let want_err = err_golden
        .and_then(|e| std::fs::read_to_string(e).ok())
        .unwrap_or_default();
    if stderr != want_err {
        problems.push(format!(
            "stderr differs\n--- expected\n{want_err}--- got\n{stderr}"
        ));
    }
    if problems.is_empty() {
        None
    } else {
        Some(format!("{rel} {args:?}: {}", problems.join("\n")))
    }
}

/// `browser/tests/jsdom/X.html`: `dweller dump --tree --js` must print
/// `X.tree` (the document after its scripts and timers ran), with the
/// console on stderr equal to `X.err` (or empty). `X.style` and `X.a11y`,
/// when they exist, are `dump --style --js` and `dump --a11y --js`.
#[test]
fn jsdom_fixtures() {
    let dir = root().join("browser").join("tests").join("jsdom");
    let mut pages: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    pages.sort();
    assert!(
        pages.len() >= 25,
        "expected at least 25 DOM fixtures, found {}",
        pages.len()
    );
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut runs: Vec<(Vec<&str>, PathBuf, PathBuf, Option<PathBuf>)> = Vec::new();
    for p in &pages {
        runs.push((
            vec!["dump", "--tree", "--js"],
            p.clone(),
            p.with_extension("tree"),
            Some(p.with_extension("err")),
        ));
        for (flag, ext) in [("--style", "style"), ("--a11y", "a11y")] {
            let golden = p.with_extension(ext);
            if golden.exists() {
                runs.push((vec!["dump", flag, "--js"], p.clone(), golden, None));
            }
        }
    }
    dweller();
    let failures: Vec<String> = common::run_queue(&runs, |(args, page, golden, err)| {
        jsdom_case(args, page, golden, err.as_deref(), bless)
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(
        failures.is_empty(),
        "{} of {} DOM fixture runs failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n\n")
    );
}

/// Under `DWELLER_BENCHMARK=1`, `Date` is a fixed epoch plus the virtual
/// clock, which a task's steps move: a loop that waits for `Date.now()`
/// ends, after the same work on every run.
#[test]
fn benchmark_date_follows_virtual_clock() {
    let page = "browser/tests/benchmark/date_now.html";
    let dump = || {
        let out = Command::new(dweller())
            .current_dir(root())
            .env("DWELLER_BENCHMARK", "1")
            .args(["dump", "--text", "--js", page])
            .output()
            .unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    let got = dump();
    assert_eq!(
        got,
        "date loop of 5 ms: n=1666664\nepoch: true\nnew Date: true\ntimer 100 ms later: true\n"
    );
    assert_eq!(dump(), got);
}

/// The DOM fixtures again with a collection at every allocation
/// (`DWELLER_JS_GC=stress`): the bindings keep what they hold reachable.
/// `gc_wrappers` allocates too much for that.
#[test]
fn jsdom_gc_stress() {
    let dir = root().join("browser").join("tests").join("jsdom");
    let mut pages: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html") && !p.ends_with("gc_wrappers.html"))
        .collect();
    pages.sort();
    // The slowest pages first, so that they do not all run at the end.
    const SLOW: [&str; 4] = [
        "virtual_clock.html",
        "wrapper_identity.html",
        "weak_nodes.html",
        "layout_many_changes.html",
    ];
    pages.sort_by_key(|p| !SLOW.iter().any(|s| p.ends_with(s)));
    dweller();
    let failures: Vec<String> = common::run_queue(&pages, |page| {
        let rel = page
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let out = Command::new(dweller())
            .current_dir(root())
            .env("DWELLER_JS_GC", "stress")
            .args(["dump", "--tree", "--js", &rel])
            .output()
            .unwrap();
        let want = std::fs::read_to_string(page.with_extension("tree")).unwrap_or_default();
        if !out.status.success() || rooted(&text(&out.stdout)) != want {
            Some(format!("{rel}: {}\n{}", out.status, text(&out.stderr)))
        } else {
            None
        }
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(
        failures.is_empty(),
        "{} DOM fixtures failed under GC stress:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}

// ---- B8: net

/// Request headers, cookies kept in a file, size limits, TLS errors and
/// `<meta charset>` over HTTP. (Cookies, the Referer and the cache within
/// one session: `browser/tests/agent/http/net_cookies.jsonl`.)
#[test]
fn net_http() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    // A URL the user typed: no Referer, no cookies yet.
    let headers = "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\nAccept-Language: en-US,en;q=0.9\nReferer: \nCookie: \n";
    expect(
        &["dump", "--text", &format!("{base}/headers")],
        0,
        headers,
        "",
    );
    // DWELLER_COOKIE_FILE keeps the persistent cookies (p, Max-Age)
    // from one run to the next; session cookies (a, d) end with the run.
    let file = scratch().join("cookies.txt");
    let _ = std::fs::remove_file(&file);
    let with_file = |url: String| {
        Command::new(dweller())
            .current_dir(root())
            .env("DWELLER_COOKIE_FILE", &file)
            .args(["dump", "--text", &url])
            .output()
            .unwrap()
    };
    let first = with_file(format!("{base}/net/start"));
    assert!(
        first.status.success() && first.stderr.is_empty(),
        "{}",
        text(&first.stderr)
    );
    let saved = std::fs::read_to_string(&file).unwrap();
    assert!(
        saved.starts_with("127.0.0.1\t1\t/\t0\t0\t")
            && saved.ends_with("\tp\t5\n")
            && saved.lines().count() == 1,
        "{saved}"
    );
    let second = with_file(format!("{base}/headers"));
    assert!(
        text(&second.stdout).contains("\nCookie: p=5\n"),
        "{}",
        text(&second.stdout)
    );
    // Too large: 33 MiB, with and without a Content-Length.
    for big in ["big", "big?stream"] {
        expect(
            &["dump", "--text", &format!("{base}/{big}")],
            1,
            "",
            &format!("error: cannot fetch {base}/{big}: the response is too large\n"),
        );
    }
    // TLS to a server that speaks plain HTTP.
    let https = base.replace("http://", "https://");
    expect(
        &["dump", "--text", &format!("{https}/")],
        1,
        "",
        &format!("error: cannot fetch {https}/: the secure connection failed (TLS)\n"),
    );
    // No charset in the Content-Type: the <meta charset> decides.
    expect(
        &["dump", "--text", &format!("{base}/meta-charset")],
        0,
        "\u{201c}quoted\u{201d} caf\u{e9}\n",
        "",
    );
}

/// P10.21: SameSite (Lax by default), the third-party policy and
/// fetch/XHR `credentials`, on the test server's two sites (`127.0.0.1`
/// and `localhost` of one port; `/samesite/` in `server.py`), cookies
/// kept between runs in a profile. `localhost` sets strict, lax, dflt
/// (no SameSite) and none (`SameSite=None` without Secure: rejected).
/// A same-site request gets all three; a cross-site top-level GET
/// navigation (a redirect from the other site, a form) lax and dflt; a
/// cross-site POST navigation and every third-party request (an
/// iframe, a script, fetch with any credentials mode) none. fetch with
/// `omit` sends none and does not store its Set-Cookie. (The public
/// suffix list: the unit tests "net: public suffixes..." and "net:
/// SameSite, the public suffix list...".)
#[test]
fn net_samesite() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let site = srv.base.clone();
    let other = site.replace("127.0.0.1", "localhost");
    let dir = profile("samesite");
    let dump = |args: &[&str]| {
        let out = Command::new(dweller())
            .current_dir(root())
            .env("DWELLER_PROFILE", &dir)
            .arg("dump")
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{args:?}: {}",
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    assert_eq!(dump(&["--text", &format!("{other}/samesite/set")]), "set\n");
    let saved = std::fs::read_to_string(dir.join("cookies")).unwrap();
    assert_eq!(
        saved,
        "localhost\t1\t/\t0\t0\t0\tstrict\tstrict\t1\n\
         localhost\t1\t/\t0\t0\t0\tlax\tlax\t1\n\
         localhost\t1\t/\t0\t0\t0\tlax\tdflt\t1\n"
    );
    let all = "strict=1; lax=1; dflt=1";
    // A URL the user typed: same-site.
    assert_eq!(
        dump(&["--text", &format!("{other}/samesite/show")]),
        format!("GET cookies: {all}\n")
    );
    // Typed on 127.0.0.1, redirected to localhost: cross-site GET.
    assert_eq!(
        dump(&[
            "--text",
            &format!("{site}/samesite/go?to={other}/samesite/show")
        ]),
        "GET cookies: lax=1; dflt=1\n"
    );
    // Embedded in a page of the other site: nothing (third-party); in
    // a page of its own site: all, but for fetch's omit. The frames'
    // requests are in the server's log.
    assert_eq!(
        dump(&[
            "--text",
            "--js",
            &format!("{site}/samesite/embed?host=localhost")
        ]),
        "script cookies:\n\nomit:\n\nsame:\n\ninclude:\n\nxhr:\n\nxhrcred:\n\nafter:\n"
    );
    assert_eq!(
        dump(&[
            "--text",
            "--js",
            &format!("{other}/samesite/embed?host=localhost")
        ]),
        format!(
            "script cookies: {all}\n\nomit:\n\nsame: {all}\n\ninclude: {all}\n\n\
             xhr: {all}\n\nxhrcred: {all}\n\nafter: {all}\n"
        )
    );
    let log = dump(&["--text", &format!("{site}/log")]);
    let frames: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("samesite-frame"))
        .collect();
    assert!(
        frames.len() == 2
            && frames.iter().filter(|l| l.contains("| cookie:")).count() == 1
            && frames.iter().any(|l| l.contains(&format!(
                "referer: {other}/samesite/embed?host=localhost | cookie: {all}"
            ))),
        "{log}"
    );
    // fetch with omit stored nothing.
    assert_eq!(
        dump(&["--text", &format!("{other}/samesite/show")]),
        format!("GET cookies: {all}\n")
    );
    if !has_sdl() {
        println!("skipped the navigations: no SDL2");
        return;
    }
    // A form submitted by the page's script: by GET from the other site
    // Lax goes, by POST nothing; from its own site all.
    let all_get = format!("GET cookies: {all}");
    let all_post = format!("POST cookies: {all}");
    for (method, from, want) in [
        ("get", &site, "GET cookies: lax=1; dflt=1"),
        ("post", &site, "POST cookies:"),
        ("get", &other, all_get.as_str()),
        ("post", &other, all_post.as_str()),
    ] {
        let got = login_window(
            &format!("{from}/samesite/nav?method={method}&to={other}/samesite/show"),
            "frame frame",
            Some(&dir),
        );
        assert!(
            got.contains(&format!("url {other}/samesite/show")) && got.trim_end().ends_with(want),
            "{method} from {from}: want {want:?}, got:\n{got}"
        );
    }
}

/// `fetch` with each method, kind of body and request headers
/// (`browser/tests/net/fetch_methods.html`, through the server's
/// `/fetch-echo`) must print `fetch_methods.txt`: what the server
/// received, as Node 22's fetch sends it (Content-Type defaults, the
/// multipart body, redirects that keep or drop the method and body),
/// except that forbidden headers (Sec-*, Proxy-*, Content-Length) are
/// not sent.
#[test]
fn fetch_methods() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/fetch_methods.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/fetch_methods.txt"),
        &text(&out.stdout),
        "fetch_methods",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// XMLHttpRequest with each method, kind of body and request header
/// (`browser/tests/net/xhr_methods.html`, through `/fetch-echo`) must
/// print `xhr_methods.txt`: the bodies and Content-Type defaults of
/// `fetch_methods`, plus a Document body; GET and HEAD send no body;
/// setRequestHeader combines repeated names and drops forbidden ones;
/// redirects as fetch's; InvalidStateError, SyntaxError and SecurityError
/// where the XHR standard throws them.
#[test]
fn xhr_methods() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/xhr_methods.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/xhr_methods.txt"),
        &text(&out.stdout),
        "xhr_methods",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// An asynchronous XMLHttpRequest fires its events in the XHR standard's
/// order (`browser/tests/net/xhr_events.html` must print
/// `xhr_events.txt`): loadstart at send, then in a networking task
/// readystatechange 2 and 3, progress, readystatechange 4, load and
/// loadend; a network error ends with readystatechange 4, error and
/// loadend; abort() in flight fires readystatechange 4, abort and
/// loadend and nothing after; open() again drops the request in flight.
/// A synchronous one is done when send returns; fetch() settles in a
/// task (after the microtasks), and aborting its signal rejects it.
#[test]
fn xhr_events() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/xhr_events.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/xhr_events.txt"),
        &text(&out.stdout),
        "xhr_events",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---- the window's loop iterations, from its DWELLER_TIMING records

/// A phase that ended in a loop iteration of the window: a DWELLER_TIMING
/// record (`info: DWELLER_TIMING\tPHASE\tNS\tLABEL`), in ms.
#[derive(Debug)]
struct Phase {
    name: String,
    ms: u64,
    label: String,
}

/// A loop iteration of the window: its `ui_loop` time (ms, without its
/// wait for events) and the phases that ended in it, in order.
#[derive(Debug, Default)]
struct Iteration {
    ms: u64,
    phases: Vec<Phase>,
}

/// The phases in which the window's thread waits for the network: a
/// request made at once (one in the cache returns at once), a batch of
/// prefetches, a task's wait for its background load.
const NET_WAITS: [&str; 3] = ["requests", "prefetch", "image_wait"];

/// The longest a loop iteration may wait for the network (`NET_WAITS`,
/// summed). What arrived in the background is read from the cache in
/// well under 1 ms; the slow parts of the pages take 200 ms or more.
const NET_WAIT_MS: u64 = 100;

/// The longest a loop iteration may take besides running a page's
/// scripts (making its realm and running them is CPU work that a loaded
/// machine makes slow). Its other work takes a few ms, tens on a loaded
/// machine; a wait for the slow parts of the pages takes 1 s or more.
const LOOP_MS: u64 = 1000;

impl Iteration {
    /// The ms of its phases named in `names`.
    fn ms_in(&self, names: &[&str]) -> u64 {
        self.phases
            .iter()
            .filter(|p| names.contains(&p.name.as_str()))
            .map(|p| p.ms)
            .sum()
    }

    /// Its waits for the network (`NET_WAITS`), but for the request of
    /// the document `page` itself, which the navigation makes at once.
    fn net_wait_ms(&self, page: &str) -> u64 {
        self.phases
            .iter()
            .filter(|p| NET_WAITS.contains(&p.name.as_str()))
            .filter(|p| !(p.name == "requests" && p.label == page))
            .map(|p| p.ms)
            .sum()
    }

    /// Does a phase `name` whose label contains `label` end in it?
    fn has(&self, name: &str, label: &str) -> bool {
        self.phases
            .iter()
            .any(|p| p.name == name && p.label.contains(label))
    }
}

/// The window's loop iterations from `err`, its stderr, which must hold
/// only DWELLER_TIMING records. A record is written when its phase ends,
/// so before the `ui_loop` record of its iteration; the records before
/// the first `ui_loop` (opening the page) belong to the first iteration,
/// those after the last (the iteration that drew the last frame and
/// ended the window) make an iteration of 0 ms.
fn iterations(err: &str) -> Vec<Iteration> {
    let mut v = vec![Iteration::default()];
    for line in err.lines() {
        let fields: Vec<&str> = line.trim_start_matches("info: ").split('\t').collect();
        assert!(
            fields.len() == 4 && fields[0] == "DWELLER_TIMING",
            "stderr has more than timing records:\n{err}"
        );
        let ms = fields[2].parse::<u64>().unwrap() / 1_000_000;
        if fields[1] == "ui_loop" {
            v.last_mut().unwrap().ms = ms;
            v.push(Iteration::default());
        } else {
            v.last_mut().unwrap().phases.push(Phase {
                name: fields[1].to_string(),
                ms,
                label: fields[3].to_string(),
            });
        }
    }
    v
}

/// The index of the first iteration in which a phase `name` whose label
/// contains `label` ends.
fn iteration_of(loops: &[Iteration], name: &str, label: &str) -> usize {
    loops
        .iter()
        .position(|it| it.has(name, label))
        .unwrap_or_else(|| panic!("no {name} record of {label:?}: {loops:#?}"))
}

/// The window never waits for the network in a loop iteration: in each,
/// the waits are short (`NET_WAIT_MS`), and what is not a page's scripts
/// takes less than `LOOP_MS`. Panics with the iteration otherwise.
fn assert_no_net_wait(loops: &[Iteration], page: &str) {
    for (i, it) in loops.iter().enumerate() {
        let waited = it.net_wait_ms(page);
        let other = it.ms.saturating_sub(it.ms_in(&["scripts"]));
        assert!(
            waited < NET_WAIT_MS && other < LOOP_MS,
            "iteration {i} of {} waited {waited} ms for the network, {other} ms besides scripts: {it:#?}",
            loops.len()
        );
    }
}

/// Page fetch() and XMLHttpRequest run in the background of the window's
/// event loop (`browser/tests/net/window_fetch.html`): while the server
/// holds a fetch for 1.5 s, scripted input types into a field and
/// scrolls, the XHR completes, and the fetch resolves afterwards with
/// its body. In the DWELLER_TIMING records, their responses end
/// iterations after the one whose task sent them, and no iteration waits
/// for the network (`assert_no_net_wait`).
#[test]
fn window_fetch_in_background() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/window_fetch.html", srv.base);
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "300",
            "--events",
            "frame frame frame frame frame text:abc frame wheel:-3 frame wheel:-3 frame",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 300 frames in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    let loops = iterations(&err);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(loops.len() > 100, "{} loop iterations", loops.len());
    assert_no_net_wait(&loops, &page);
    // The page's first task sends the requests.
    let sent = iteration_of(&loops, "timers", &page);
    let xhr = iteration_of(&loops, "background_requests", "/slow/window.txt");
    let fetched = iteration_of(&loops, "background_requests", "/sleep?ms=1500");
    assert!(
        sent + 3 <= xhr && xhr + 3 <= fetched,
        "sent in iteration {sent}, the XHR's response in {xhr}, the fetch's in {fetched}"
    );
    let got = text(&out.stdout);
    let want = [
        "requests sent",
        "input while the fetch waits: true",
        "scroll while the fetch waits: true",
        "xhr: 200 4 bytes while the fetch waits: true",
        "fetched: true, field abc, scrolled true",
    ];
    let at: Vec<Option<usize>> = want.iter().map(|w| got.find(w)).collect();
    assert!(
        at.iter().all(|a| a.is_some()) && at.windows(2).all(|w| w[0] < w[1]),
        "{got}"
    );
}

/// The mouse moves over a link while the page's scripts run one per loop
/// iteration (`browser/tests/net/hover_scripts.html`): the window keeps
/// the layout from before them, whose boxes name the links the scripts
/// replace and a collection frees. The hit test must not reach a freed
/// node (it crashed in `Document.isElement` under `engine.linkAt`); once
/// the scripts ran, the status shows the last link.
#[test]
fn window_hover_during_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/hover_scripts.html", srv.base);
    let events: String = (0..200)
        .map(|i| format!("move:{},70 frame ", 20 + i % 30))
        .collect();
    let args = [
        page.as_str(),
        "--font",
        "fixed",
        "--width",
        "400",
        "--height",
        "300",
        "--frames",
        "210",
        "--state",
        "--events",
        events.trim_end(),
    ];
    let out = run_window(&args);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let want = format!("\nstatus {}/net/link6.html", srv.base);
    assert!(got.contains(&want), "expected {want:?} in:\n{got}");
}

/// P4.33: the window paints a page once its document and its sheet are
/// in (0.3 s), before its script and its image arrive (1.5 s), and the
/// click and the wheel during the load are handled before the script
/// runs. In the DWELLER_TIMING records, the first paint comes iterations
/// before the script runs and the image is read, and no iteration waits
/// for the network (`assert_no_net_wait`).
#[test]
fn window_paints_early() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/early_paint.html", srv.base);
    // The click waits for the first paint (`painted`), which comes
    // 0.3 s into the 1.5 s the script is delayed, not for a number of
    // frames (40 take longer than 1.5 s on a loaded runner).
    let events = "painted click:14,54 frame wheel:-3 frame";
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "300",
            "--events",
            events,
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 300 frames in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    let loops = iterations(&err);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(loops.len() > 100, "{} loop iterations", loops.len());
    assert_no_net_wait(&loops, &page);
    let painted = iteration_of(&loops, "ui_first_paint", &page);
    let script = iteration_of(&loops, "script", "/delay/1500/net/early_paint.js");
    let image = iteration_of(&loops, "requests", "/delay/1500/paint/img/badge.png");
    assert!(
        painted + 3 <= script && painted + 3 <= image,
        "first paint in iteration {painted}, the script runs in {script}, the image is read in {image}"
    );
    let got = text(&out.stdout);
    assert!(
        got.contains("script ran; checked before it: true, scrolled before it: true, color: rgb(0, 128, 0)"),
        "{got}"
    );
}

/// P4.43: twenty import()s whose modules take 200-390 ms, a module whose
/// twenty imports do the same, and a frame whose document takes 1 s load
/// in the background: each import() settles with its module, the imports
/// evaluate in their order, and the load event waits for the frame. In
/// the DWELLER_TIMING records, the modules are read in several
/// iterations, the frame's document ends iterations after the page's
/// script ran, and no iteration waits for the network
/// (`assert_no_net_wait`).
#[test]
fn window_dynamic_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/dynamic_scripts.html", srv.base);
    let events = format!("{}wheel:-3 frame", "frame ".repeat(40));
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--events",
            events.as_str(),
            "--text",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    let loops = iterations(&err);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(loops.len() > 300, "{} loop iterations", loops.len());
    assert_no_net_wait(&loops, &page);
    // The iterations in which a module is read (its task runs).
    let modules: Vec<usize> = (0..loops.len())
        .filter(|&i| {
            loops[i].has("requests", "/net/dynamic_module.js")
                || loops[i].has("requests", "/net/dynamic_root.js")
        })
        .collect();
    let ran = iteration_of(&loops, "script", "inline");
    let frame = iteration_of(
        &loops,
        "background_requests",
        "/delay/1000/net/dynamic_frame.html",
    );
    assert!(
        modules.len() >= 3 && ran + 3 <= frame,
        "modules read in iterations {modules:?}, the page's script ran in {ran}, the frame's document ended in {frame}"
    );
    let got = text(&out.stdout);
    let graph: Vec<String> = (0..20).map(|i| format!("g={i}")).collect();
    let want = format!(
        "imports settled: 20, each with its module: true; graph: root {}; frame loaded: true",
        graph.join(",")
    );
    assert!(got.contains(&want), "{got}");
}

/// P4.45: scripts that a script inserts (`net/inserted_scripts.html`):
/// an async one, twenty `async = false` ones whose loads end last first,
/// a webpack-style JSONP chunk, a missing script and one of another
/// origin served as HTML, in `dump` (twice:
/// the same output) and in the window, where no loop iteration waits
/// for their loads.
#[test]
fn inserted_scripts() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/inserted_scripts.html", srv.base);
    let ordered: Vec<String> = (0..20).map(|i| format!("o{i}")).collect();
    let want = format!(
        "async: true; ordered: {}; chunk: lazy module; missing: true; html of another origin: blocked; loads: 21",
        ordered.join(",")
    );
    let port = srv.base.rsplit(':').next().unwrap();
    let missing = format!(
        "[error] Failed to load script: {}/net/inserted_missing.js\n[error] Blocked a script of another origin served as HTML: http://localhost:{port}/log/inserted.html\n",
        srv.base
    );
    let mut dumps = Vec::new();
    for _ in 0..2 {
        let out = run(&["dump", "--text", "--js", &page]);
        assert!(out.status.success(), "{}", text(&out.stderr));
        assert_eq!(text(&out.stderr), missing);
        let got = text(&out.stdout);
        assert!(got.contains(&want), "{got}");
        dumps.push(got);
    }
    assert_eq!(dumps[0], dumps[1]);
    if !has_sdl() {
        println!("window part skipped: no SDL2");
        return;
    }
    let events = format!("{}wheel:-3 frame", "frame ".repeat(40));
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--events",
            events.as_str(),
            "--text",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    // The iterations' times, apart from those that run the page's script.
    let mut loops = Vec::new();
    let mut ran_script = false;
    for line in err.lines() {
        let fields: Vec<&str> = line.trim_start_matches("info: ").split('\t').collect();
        if fields.len() < 3 || fields[0] != "DWELLER_TIMING" {
            continue;
        }
        let ms = fields[2].parse::<u64>().unwrap() / 1_000_000;
        match fields[1] {
            "ui_loop" if ran_script => ran_script = false,
            "ui_loop" => loops.push(ms),
            "scripts" => ran_script = true,
            _ => {}
        }
    }
    let longest = loops.iter().copied().max().unwrap_or(0);
    assert!(
        loops.len() > 300 && longest < 100,
        "longest loop {longest} ms of {}",
        loops.len()
    );
    let got = text(&out.stdout);
    assert!(got.contains(&want), "{got}");
}

// ---- P10.20: logged-in browsing (the login fixture of server.py)

/// The `--events` words that type `s` (at most 31 bytes a word).
fn typed(s: &str) -> String {
    s.as_bytes()
        .chunks(31)
        .map(|c| format!("text:{}", std::str::from_utf8(c).unwrap()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The `--events` words that go to `url` through the address bar.
fn go_to(url: &str) -> String {
    format!("key:cmd+l {} key:return frame frame", typed(url))
}

/// Signing in at `/login`: the username, the password, Enter.
const SIGN_IN: &str = "key:tab text:alice key:tab text:wonderland key:return frame frame";

/// The window on `url` with `events`, headless, with the profile
/// directory `profile` when given (`DWELLER_PROFILE`, P4.50); its
/// `--state` and `--text`.
fn login_window(url: &str, events: &str, profile: Option<&Path>) -> String {
    let mut c = Command::new(dweller());
    c.current_dir(root()).env("SDL_VIDEODRIVER", "dummy");
    if let Some(p) = profile {
        c.env("DWELLER_PROFILE", p);
    }
    let out = c
        .args([
            url, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "40",
            "--state", "--text", "--events", events,
        ])
        .output()
        .unwrap();
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{url} --events {events:?}: {}\n{}\n{got}",
        out.status,
        text(&out.stderr)
    );
    got
}

/// A fresh profile directory for one test.
fn profile(name: &str) -> PathBuf {
    let dir = scratch().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Form login in one run of the window: `/account` sends a stranger to
/// the sign-in form, a wrong password shows it again, a stale CSRF
/// token is refused, the right password lands on `/account` (a POST, a
/// 303 and two cookies), signing out ends the session; a redirect chain
/// across two hosts keeps each host's cookies apart; `localStorage`
/// holds a token between pages of the run; `/basic` is 401 without
/// credentials and 200 with URL userinfo. Later cards turn on what
/// lasts across a restart (`window_login_cookies`, P4.50, and
/// `window_login_storage`, P4.51), HTTP Basic (`window_login_basic`,
/// P10.22) and popups (`window_login_popup`, P4.52).
#[test]
fn window_login() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let account = format!("{base}/account");
    let has = |got: &str, want: &[&str], not: &[&str], what: &str| {
        assert!(
            want.iter().all(|w| got.contains(w)) && !not.iter().any(|n| got.contains(n)),
            "{what}: want {want:?}, not {not:?}, got:\n{got}"
        );
    };
    let stranger = login_window(&account, "frame", None);
    has(
        &stranger,
        &[
            &format!("url {base}/login?next=/account\n"),
            "Not signed in",
        ],
        &["Signed in as"],
        "a stranger at /account",
    );
    let wrong = login_window(
        &account,
        "key:tab text:alice key:tab text:queen key:return frame frame",
        None,
    );
    has(
        &wrong,
        &[&format!("url {base}/login\n"), "Wrong password"],
        &["Signed in as"],
        "a wrong password",
    );
    let stale = login_window(&format!("{base}/login/stale.html"), SIGN_IN, None);
    has(
        &stale,
        &["bad CSRF token"],
        &["Signed in as"],
        "a stale CSRF token",
    );
    let signed_in = login_window(&account, SIGN_IN, None);
    has(
        &signed_in,
        &[
            &format!("url {account}\n"),
            "history 2 of 2\n",
            "Signed in as Alice Liddell",
            "tab cookie: yes",
        ],
        &[],
        "signing in",
    );
    // Sign out (the button after the fields), then /account again.
    let out = login_window(
        &account,
        &format!(
            "{SIGN_IN} key:tab key:return frame frame {}",
            go_to(&account)
        ),
        None,
    );
    has(
        &out,
        &[
            &format!("url {base}/login?next=/account\n"),
            "Not signed in",
        ],
        &["Signed in as"],
        "signing out",
    );
    // 127.0.0.1 -> localhost -> 127.0.0.1, a cookie at each hop: back on
    // 127.0.0.1 only its own cookie comes.
    let sso = login_window(&format!("{base}/sso/start"), "frame", None);
    has(
        &sso,
        &[
            &format!("url {base}/sso/done\n"),
            "SSO done on 127.0.0.1, cookies: sso1=127.0.0.1",
        ],
        &["sso2"],
        "the SSO redirect chain",
    );
    // HTTP Basic: the 401 page without credentials, 200 with URL
    // userinfo (libcurl sends it). The sheet and the cache are P10.22's.
    let basic = format!("{base}/basic");
    let refused = login_window(&basic, "frame", None);
    has(
        &refused,
        &["401: sign in to realm r"],
        &["signed in as alice"],
        "/basic without credentials",
    );
    let userinfo = login_window(
        &basic.replace("http://", "http://alice:wonderland@"),
        "frame",
        None,
    );
    has(
        &userinfo,
        &["Basic: signed in as alice"],
        &[],
        "/basic with URL userinfo",
    );
    // localStorage between pages of one run.
    let storage = login_window(
        &format!("{base}/login/store.html"),
        &format!("frame {}", go_to(&format!("{base}/login/read.html"))),
        None,
    );
    has(
        &storage,
        &["token: tok-alice-42"],
        &[],
        "localStorage in one run",
    );
}

/// P4.50: signed in at `/login`, the window is killed (no clean exit);
/// started again on the same profile it is still signed in, the
/// session-only cookie restored too (a browser that restores its
/// session).
#[test]
fn window_login_cookies() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let account = format!("{base}/account");
    let dir = profile("login-cookies");
    let mut window = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .env("DWELLER_PROFILE", &dir)
        .args([
            account.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "16384",
            "--events",
            SIGN_IN,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    // Cookies are saved within a second of a change (P4.50).
    let jar = dir.join("cookies");
    let start = Instant::now();
    while !std::fs::read_to_string(&jar).is_ok_and(|s| s.contains("sid")) {
        if start.elapsed() > Duration::from_secs(30) || window.try_wait().unwrap().is_some() {
            let _ = window.kill();
            panic!("no sid cookie in {} after signing in", jar.display());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    window.kill().unwrap();
    window.wait().unwrap();
    // Only the person can read the jar; no temporary file is left.
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&jar).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "{}", jar.display());
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["cookies", "lock", "telemetry"]);
    let again = login_window(&account, "frame", Some(&dir));
    assert!(
        again.contains(&format!("url {account}\n"))
            && again.contains("Signed in as Alice Liddell")
            && again.contains("tab cookie: yes"),
        "after the restart:\n{again}"
    );
    // Signing out is saved too: the next start is signed out.
    login_window(&account, "key:tab key:return frame frame", Some(&dir));
    let out = login_window(&account, "frame", Some(&dir));
    assert!(out.contains("Not signed in"), "after signing out:\n{out}");
}

/// P4.51: a token in `localStorage` is there after a restart on the same
/// profile; `sessionStorage` lasts across a navigation in the tab, not
/// across a restart.
#[test]
fn window_login_storage() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let read = format!("{base}/login/read.html");
    let dir = profile("login-storage");
    let first = login_window(
        &format!("{base}/login/store.html"),
        &format!("frame {}", go_to(&read)),
        Some(&dir),
    );
    assert!(
        first.contains("token: tok-alice-42") && first.contains("session: kept"),
        "in the first run:\n{first}"
    );
    let again = login_window(&read, "frame", Some(&dir));
    assert!(
        again.contains("token: tok-alice-42") && again.contains("session: null"),
        "after the restart:\n{again}"
    );
    // One file for the origin, mode 0600, no temporary file left.
    let files: Vec<_> = std::fs::read_dir(dir.join("storage"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1, "{files:?}");
    let saved = std::fs::read_to_string(&files[0]).unwrap();
    assert!(saved.contains("5 12\ntokentok-alice-42\n"), "{saved:?}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&files[0]).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "{:?}", files[0]);
    }
}

/// P4.51: each storage area holds 5 MB of keys and values; a setItem
/// past it throws a QuotaExceededError DOMException and changes nothing.
#[test]
fn storage_quota() {
    expect(
        &["dump", "--text", "--js", "browser/tests/storage/quota.html"],
        0,
        "6 MB: QuotaExceededError (DOMException) | 3 MB: ok | 3 MB more: QuotaExceededError (DOMException) | replace with 4 MB: ok | session 6 MB: QuotaExceededError (DOMException) | keys 1 a 4194304 | after removing, 5 MB: ok\n",
        "",
    );
}

/// P4.51: a tab sees the localStorage writes of the tabs it opened as
/// `storage` events; a tab opened with an opener starts with a copy of
/// its sessionStorage, one opened with `noopener` with none, and neither
/// child's sessionStorage reaches the opener.
#[test]
fn storage_between_tabs() {
    expect(
        &["dump", "--text", "--js", "browser/tests/storage/tabs.html"],
        0,
        "copy=from the opener (old null, local, from child.html?copy) | fresh=null (old null, local, from child.html?fresh) | own session child: null | own length 1\n",
        "",
    );
}

/// P10.22: credentials for `/basic` (here from URL userinfo) are cached
/// per origin and realm for the process: the same page without them, in
/// the same run, is 200 too.
#[test]
fn window_login_basic() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    let url = basic.replace("http://", "http://alice:wonderland@");
    let got = login_window(&url, &format!("frame {}", go_to(&basic)), None);
    assert!(
        got.contains(&format!("url {basic}\n")) && got.contains("Basic: signed in as alice"),
        "{got}"
    );
}

/// P10.22: the sign-in prompt of a 401 with `WWW-Authenticate: Basic` in
/// the person's window (ui/auth.aye): typing the username, Tab, the
/// password and Enter loads `/basic` again with the credentials; a wrong
/// password brings the prompt back with the username kept and the focus
/// on the password. The prompt is `browser/tests/ui/auth_prompt.png`
/// (`DWELLER_BLESS=1` rewrites it; the fixed font draws every digit of
/// the server's port alike).
#[test]
fn window_login_basic_prompt() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    let asked = login_window(&basic, "frame", None);
    assert!(
        asked.contains("focus auth_user\n")
            && asked.contains("auth r user \n")
            && asked.contains("401: sign in to realm r"),
        "{asked}"
    );
    let signed = login_window(
        &basic,
        "frame text:alice key:tab text:wonderland key:return frame frame",
        None,
    );
    assert!(
        signed.contains(&format!("url {basic}\nscroll 0\nfocus page\n"))
            && signed.contains("history 1 of 1\n")
            && signed.contains("Basic: signed in as alice")
            && !signed.contains("\nauth "),
        "{signed}"
    );
    let wrong = login_window(
        &basic,
        "frame text:alice key:tab text:queen key:return frame frame",
        None,
    );
    assert!(
        wrong.contains("focus auth_password\n")
            && wrong.contains("auth r user alice\n")
            && wrong.contains("401: sign in to realm r"),
        "{wrong}"
    );
    let again = login_window(
        &basic,
        "frame text:alice key:tab text:queen key:return frame frame \
         text:wonderland key:return frame frame",
        None,
    );
    assert!(again.contains("Basic: signed in as alice"), "{again}");
    // While the prompt is open a click on the page or a key for the
    // window leaves the focus in it.
    let held = login_window(&basic, "frame click:200,280 key:cmd+l text:al", None);
    assert!(
        held.contains("focus auth_user\n") && held.contains("auth r user al\n"),
        "{held}"
    );
    let png = scratch().join("auth_prompt.png");
    let out = Command::new(dweller())
        .current_dir(root())
        .env("SDL_VIDEODRIVER", "dummy")
        .args([
            &basic,
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "40",
            "--events",
            "frame text:alice key:tab text:wonder",
            "--save",
            png.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/ui/auth_prompt.png");
    let got = std::fs::read(&png).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    } else {
        assert!(
            got == std::fs::read(&golden).unwrap(),
            "the sign-in prompt differs from {}: {}",
            golden.display(),
            png.display()
        );
    }
}

/// P10.22: Escape or Cancel closes the sign-in prompt and leaves the 401
/// page shown, the focus on the page; Sign in (a click) signs in. With
/// URL userinfo the address bar leaves `user:pass@` out.
#[test]
fn window_login_basic_cancel() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    // In a 400 px window the sheet is x 30-370 from y 39; Cancel is at
    // x 206-278 and Sign in at 298-370, y 153-177.
    for events in [
        "frame text:alice key:escape frame",
        "frame text:alice click:240,165 frame",
    ] {
        let got = login_window(&basic, events, None);
        assert!(
            got.contains("focus page\n")
                && !got.contains("\nauth ")
                && got.contains("401: sign in to realm r"),
            "{events}: {got}"
        );
    }
    let clicked = login_window(
        &basic,
        "frame text:alice key:tab text:wonderland click:330,165 frame frame",
        None,
    );
    assert!(clicked.contains("Basic: signed in as alice"), "{clicked}");
    let userinfo = basic.replace("http://", "http://alice:wonderland@");
    let shown = login_window(&userinfo, "frame", None);
    assert!(
        shown.contains(&format!("url {userinfo}\n"))
            && shown.contains(&format!("address {basic}\n"))
            && shown.contains("Basic: signed in as alice"),
        "{shown}"
    );
}

/// P10.22: an agent never gets the sign-in prompt: its navigation to a
/// 401 with a Basic challenge shows the 401 page and the reply says
/// `auth_required` with the realm; it signs in with URL userinfo, and
/// the credentials are cached for the process. `dump` shows the 401
/// page.
#[test]
fn net_basic_auth() {
    use std::io::Write;
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    let userinfo = basic.replace("http://", "http://alice:wonderland@");
    let mut child = Command::new(dweller())
        .current_dir(root())
        .arg("agent")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!(
        "{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"{basic}\"}}\n\
         {{\"id\":2,\"cmd\":\"navigate\",\"url\":\"{userinfo}\"}}\n\
         {{\"id\":3,\"cmd\":\"navigate\",\"url\":\"{basic}\"}}\n"
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(lines.len(), 3, "{got}");
    assert!(
        lines[0].contains(&format!("\"result\":\"loaded {basic} (status 401)\""))
            && lines[0].contains("\"auth_required\":{\"scheme\":\"Basic\",\"realm\":\"r\"}"),
        "{got}"
    );
    for line in &lines[1..] {
        assert!(
            line.contains("Basic: signed in as alice") && !line.contains("auth_required"),
            "{got}"
        );
    }
    expect(
        &["dump", "--text", &basic],
        0,
        "401: sign in to realm r\n",
        "",
    );
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
            && sections.contains("scroll events: 3;"),
        "{sections}"
    );
    let sections = md(&["--no-scroll"], "lazy_sections");
    assert!(
        !sections.contains("First section") && sections.contains("scroll events: 0;"),
        "{sections}"
    );
    // The native lazy image is there either way: it loads at once.
    assert!(
        sections.contains("![A native lazy image](data:image/png")
            && sections.contains("image loaded at"),
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

// ---- B8: layout and paint

/// Hit testing follows the painting order (`browser/tests/layout/hit.html`
/// in a 400x340 window, the page 40 px down): the topmost positioned box
/// wins, a positioned box hides the link under it, content clipped by
/// `overflow` is not hit outside the clip, and a link in an iframe's
/// document is hit there. The hovered link shows in the status line.
#[test]
fn window_hit_layers() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = format!("file://{}/browser/tests/layout/", root().display());
    let cases: &[(&str, &str)] = &[
        ("move:60,70", "high.html"),
        ("move:20,50", "low.html"),
        ("move:20,150", ""),
        ("move:20,175", "flow.html"),
        ("move:20,240", "clipped.html"),
        ("move:20,260", ""),
        ("move:340,60", "frame.html"),
    ];
    let mut failures = Vec::new();
    for (events, want) in cases {
        let out = run_window(&[
            "browser/tests/layout/hit.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&dir, "");
        let status = got
            .lines()
            .find(|l| l.starts_with("status "))
            .unwrap_or("")
            .to_string();
        let want_line = format!("status {want}");
        if !out.status.success() || !out.stderr.is_empty() || status != want_line {
            failures.push(format!(
                "--events {events:?}: {}\n{}--- expected\n{want_line}\n--- got\n{got}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

/// Fixed and sticky boxes follow the scroll offset
/// (`browser/tests/layout/scroll.html` in a 400x340 window: a 300 px
/// viewport 40 px down): the fixed bar stays at the top, the sticky link
/// stays 40 px below the top until the end of its section, the sticky
/// footer stays at the bottom, and a sticky box in a box that clips does
/// not move. Hit testing sees them where they are drawn.
/// A page wider than the window scrolls sideways (left and right
/// arrows); the mouse then points at the page under it.
#[test]
fn window_sideways_scroll() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = format!("file://{}/browser/tests/ui/", root().display());
    let cases: &[(&str, &str)] = &[
        ("move:250,80", ""),
        (
            "key:right key:right key:right key:right key:right key:right key:right key:right key:right move:250,80",
            "other.html",
        ),
        (
            "key:right key:right key:right key:right key:right key:right key:right key:right key:right key:left key:left key:left key:left key:left key:left key:left key:left key:left move:250,80",
            "",
        ),
        (
            "key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right key:right move:50,80",
            "other.html",
        ),
    ];
    let mut failures = Vec::new();
    for (events, want) in cases {
        let out = run_window(&[
            "browser/tests/ui/wide.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&dir, "");
        let status = got
            .lines()
            .find(|l| l.starts_with("status "))
            .unwrap_or("")
            .to_string();
        let want_line = format!("status {want}");
        if !out.status.success() || !out.stderr.is_empty() || status != want_line {
            failures.push(format!(
                "--events {events:?}: {}\n{}--- expected\n{want_line}\n--- got\n{got}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

#[test]
fn window_scroll_positions() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = format!("file://{}/browser/tests/layout/", root().display());
    let cases: &[(&str, &str)] = &[
        ("move:20,50", "fixed.html"),
        ("move:220,90", ""),
        ("move:320,320", "foot.html"),
        ("wheel:-5 move:20,50", "fixed.html"),
        ("wheel:-5 move:220,90", "sticky.html"),
        ("move:220,150", "sticky.html"),
        ("move:220,190", "under.html"),
        ("wheel:-17 move:220,90", ""),
        ("wheel:-17 move:220,50", "sticky.html"),
        ("wheel:-17 move:20,105", "clipped.html"),
        ("key:end move:20,50", "fixed.html"),
        ("key:end move:320,320", "foot.html"),
    ];
    let mut failures = Vec::new();
    for (events, want) in cases {
        let out = run_window(&[
            "browser/tests/layout/scroll.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&dir, "");
        let status = got
            .lines()
            .find(|l| l.starts_with("status "))
            .unwrap_or("")
            .to_string();
        let want_line = format!("status {want}");
        if !out.status.success() || !out.stderr.is_empty() || status != want_line {
            failures.push(format!(
                "--events {events:?}: {}\n{}--- expected\n{want_line}\n--- got\n{got}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
}

/// A frameset in the window (`browser/tests/ui/frames.html`, 400x340: a
/// 100 px menu frame, `long.html` in the rest): the wheel scrolls the
/// frame under the mouse (up to its document's end), the status line
/// shows a frame's link resolved in the frame, and clicking a link loads
/// it in its own frame, in the frame its target names, or (`_top`) in
/// the tab.
#[test]
fn window_framesets() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let ui = format!("file://{}/browser/tests/ui/", root().display());
    let page =
        "url frames.html\nscroll 0\nfocus page\naddress frames.html\nhistory 1 of 1\nmatches 0\n";
    let urls = "frame_urls frame_menu.html long.html\n";
    let cases: &[(&str, String)] = &[
        ("move:20,58", format!("{page}status other.html\nframes 0 0\n{urls}")),
        ("move:200,100 wheel:-2", format!("{page}status \nframes 0 80\n{urls}")),
        ("move:200,100 wheel:-100", format!("{page}status \nframes 0 960\n{urls}")),
        ("move:200,100 wheel:-2 move:20,200 wheel:-2 key:down", format!("{page}status \nframes 0 80\n{urls}")),
        ("click:20,58", format!("{page}status \nframes 0 0\nframe_urls other.html long.html\n")),
        ("click:20,95", format!("{page}status \nframes 0 0\nframe_urls frame_menu.html other.html\n")),
        ("click:20,131", "url other.html\nscroll 0\nfocus page\naddress other.html\nhistory 2 of 2\nmatches 0\nstatus \n".to_string()),
    ];
    let mut failures = Vec::new();
    for (events, want) in cases {
        let out = run_window(&[
            "browser/tests/ui/frames.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&ui, "");
        if !out.status.success() || !out.stderr.is_empty() || got != *want {
            failures.push(format!(
                "--events {events:?}: {}\n{}--- expected\n{want}--- got\n{got}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} cases failed:\n\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
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

/// P4.52: window.open, `target=_blank`, `window.opener` and postMessage
/// between tabs (engine/contexts.aye), on the test server's two origins
/// (`127.0.0.1` and `localhost`, `browser/tests/popup/`). The OAuth
/// popup round trip: a click opens the sign-in page of origin B in a
/// new tab, which sees its opener but not its document, and goes through
/// a redirect back to the opener's origin; the callback posts the code
/// to its opener (a message for the wrong origin is dropped) and closes
/// itself, so the opener's tab is shown with the message, its
/// `popup.closed` true and `event.source` the popup. `dump --js` loads
/// the popup in memory with the same result. `target=_blank` opens a tab
/// without an opener (with one for `rel=opener`); a named target is
/// reused by window.open and by a link. First the P10.20 fixture: a
/// click opens `login/popup.html`, which posts to its opener and closes,
/// and the opener shows the message.
#[test]
fn window_login_popup() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let opener = format!("{base}/login/opener.html");
    // The button is at y 10 to 40 of the page, under the 40 px toolbar.
    let got = login_window(&opener, "frame click:50,65 frame frame frame", None);
    assert!(
        got.contains(&format!("url {opener}\n")) && got.contains("message: alice"),
        "{got}"
    );
    let signed_in = "got code=c0de state=s1 seen=opener-true-SecurityError-signin from the popup closed true messages 1";
    let window = |page: &str, events: &str| {
        let url = format!("{base}/popup/{page}");
        let out = run_window(&[
            &url, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "30",
            "--events", events, "--state", "--text",
        ]);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{page} {events}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    let got = window(
        "opener.html",
        "frame click:50,60 frame frame frame frame frame",
    );
    assert!(
        got.starts_with(&format!("url {base}/popup/opener.html\n")),
        "{got}"
    );
    assert!(!got.contains("\ntabs "), "the popup did not close:\n{got}");
    assert!(got.contains(signed_in), "{got}");
    // Without a window: the popup loads in memory.
    let out = run(&[
        "dump",
        "--js",
        "--text",
        &format!("{base}/popup/opener.html#auto"),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains(signed_in),
        "{}",
        text(&out.stdout)
    );
    // target=_blank: a new tab, shown, without an opener; rel=opener
    // gives it one.
    for (y, search, opener) in [(60, "blank", "null"), (110, "opener", "set")] {
        let got = window("links.html", &format!("frame click:50,{y} frame frame"));
        assert!(
            got.starts_with(&format!("url {base}/popup/target.html?{search}\n")),
            "{got}"
        );
        assert!(got.contains("\ntabs 2, showing 2\n"), "{got}");
        assert!(
            got.contains(&format!("opener {opener} name  at ?{search}")),
            "{got}"
        );
    }
    // window.open(url, 'w1') twice, then a link with target=w1: one tab
    // (the tab strip moves the page down 28 px once there are two).
    let got = window(
        "links.html",
        "frame click:50,160 frame key:cmd+1 frame click:50,190 frame key:cmd+1 frame click:50,240 frame key:cmd+2 frame",
    );
    assert!(
        got.starts_with(&format!("url {base}/popup/target.html?link\n")),
        "{got}"
    );
    assert!(got.contains("\nhistory 3 of 3\n"), "{got}");
    assert!(got.contains("\ntabs 2, showing 2\n"), "{got}");
    assert!(got.contains("opener set name w1 at ?link"), "{got}");
}

/// Local telemetry (`browser/perf/telemetry.aye`): a window on a temporary
/// `DWELLER_PROFILE` showing `browser/tests/telemetry/page.html` (a JS
/// throw, an unknown CSS property and value, a missing API, a slow task,
/// a 404 image and a corrupt image) writes one record of each to the day
/// file, with origins only: no path, query, page text or form value.
/// `telemetry.py summary` lists them; `DWELLER_TELEMETRY=0` and a test
/// window without a profile write nothing; a crash (SIGSEGV) is recorded
/// with the window's last actions.
#[test]
fn window_telemetry() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = std::env::temp_dir().join(format!("dweller-telemetry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let profile = base.join("profile");
    let off = base.join("off");
    let home = base.join("home");
    for d in [&profile, &off, &home] {
        std::fs::create_dir_all(d).unwrap();
    }
    let page = format!("{}/telemetry/page.html?user=QUERYSECRET-2", srv.base);
    let window = |profile: Option<&Path>, telemetry: Option<&str>, frames: &str| {
        let mut c = Command::new(dweller());
        c.current_dir(root())
            .env("SDL_VIDEODRIVER", "dummy")
            .env("HOME", &home)
            .env_remove("DWELLER_PROFILE")
            .env_remove("DWELLER_TELEMETRY")
            .env_remove("DWELLER_TELEMETRY_DIR")
            .args([
                page.as_str(),
                "--font",
                "fixed",
                "--width",
                "400",
                "--height",
                "300",
                "--frames",
                frames,
                "--events",
                "frame wheel:-3 frame wheel:-3 frame wheel:-3 frame",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if let Some(p) = profile {
            c.env("DWELLER_PROFILE", p);
        }
        if let Some(t) = telemetry {
            c.env("DWELLER_TELEMETRY", t);
        }
        c
    };
    let records = |profile: &Path| -> String {
        let dir = profile.join("telemetry");
        let mut all = String::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut names: Vec<PathBuf> = entries.map(|e| e.unwrap().path()).collect();
            names.sort();
            for n in names {
                all.push_str(&std::fs::read_to_string(n).unwrap());
            }
        }
        all
    };

    let out = window(Some(&profile), None, "200").output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let day = chrono_free_today();
    let file = profile.join("telemetry").join(format!("{day}.jsonl"));
    let got = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let origin = srv.base.clone();
    for want in [
        "\"ev\":\"js_error\",".to_string(),
        "\"type\":\"Error\",\"message\":\"failed at ".to_string() + &origin + "\"",
        "\"feature\":\"css_property\",\"name\":\"zoomy-frobnicate\"".to_string(),
        "\"feature\":\"css_value\",\"name\":\"display: grid-lanes\"".to_string(),
        "\"feature\":\"js_global\",\"name\":\"XRSession\"".to_string(),
        "\"kind\":\"image\",\"status\":404,\"origin\":\"".to_string() + &origin + "\"",
        "\"ev\":\"image_decode\",".to_string(),
        "\"ev\":\"page\",".to_string(),
        "\"ev\":\"scroll\",".to_string(),
    ] {
        assert_eq!(got.matches(&want).count(), 1, "one {want} in:\n{got}");
    }
    // The slow task's loop iteration, by its script phase (a debug build
    // may have other slow ones).
    let slow_script = got
        .lines()
        .filter(|l| l.contains("\"ev\":\"slow_loop\""))
        .any(|l| {
            l.split("\"script\":")
                .nth(1)
                .and_then(|v| v.split(|c: char| !c.is_ascii_digit()).next())
                .and_then(|v| v.parse::<u64>().ok())
                .is_some_and(|ms| ms >= 200)
        });
    assert!(slow_script, "no slow_loop of the slow script in:\n{got}");
    for line in got.lines() {
        assert!(
            line.starts_with("{\"t\":\"") && line.ends_with('}'),
            "not a record: {line}"
        );
        assert!(
            line.contains(&format!("\"site\":\"{origin}\"")),
            "not of the page's site: {line}"
        );
    }
    // Origins only: no path, query, page text or form value.
    for secret in [
        "?",
        "/telemetry",
        "page.html",
        "missing.png",
        "broken.png",
        "QUERYSECRET",
        "PATHSECRET",
        "PAGETEXT",
        "FORMVALUE",
        "only the page has this text",
    ] {
        assert!(!got.contains(secret), "{secret} in:\n{got}");
    }

    let summary = Command::new("python3")
        .current_dir(root())
        .args([
            "browser/tools/telemetry.py",
            "summary",
            "--days",
            "1",
            "--profile",
        ])
        .arg(&profile)
        .output()
        .unwrap();
    let listed = text(&summary.stdout);
    assert!(summary.status.success(), "{}", text(&summary.stderr));
    for want in [
        "js_error      Error: failed at <origin>",
        "unsupported   css_property zoomy-frobnicate",
        "unsupported   css_value display: grid-lanes",
        "unsupported   js_global XRSession",
        "slow_loop     loop > 100 ms, mostly script",
        "load_failed   image 404",
        "image_decode  png ",
    ] {
        assert!(listed.contains(want), "{want} not in:\n{listed}");
    }

    // Off: DWELLER_TELEMETRY=0, and a test window without a profile.
    let out = window(Some(&off), Some("0"), "60").output().unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!off.join("telemetry").exists(), "{}", records(&off));
    let out = window(None, None, "60").output().unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!home.join(".dweller").exists());

    // A crash: the records so far and the crash, with the last actions.
    // Not on macOS until card P4.54 installs the crash handler there.
    if !cfg!(target_os = "macos") {
        let crashed = base.join("crashed");
        std::fs::create_dir_all(&crashed).unwrap();
        let mut child = window(Some(&crashed), None, "3000").spawn().unwrap();
        std::thread::sleep(Duration::from_millis(1500));
        let killed = Command::new("kill")
            .args(["-SEGV", &child.id().to_string()])
            .status()
            .unwrap();
        assert!(killed.success());
        let start = Instant::now();
        while child.try_wait().unwrap().is_none() {
            if start.elapsed() > Duration::from_secs(30) {
                let _ = child.kill();
                panic!("the window did not end on SIGSEGV");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let got = records(&crashed);
        let crash = got
            .lines()
            .find(|l| l.contains("\"ev\":\"crash\""))
            .unwrap_or_else(|| panic!("no crash record in:\n{got}"));
        assert!(
            crash.contains("\"signal\":\"SIGSEGV\"")
                && crash.contains("\"backtrace\":[\"0x")
                && crash.contains("\"actions\":[\"wheel\",\"wheel\",\"wheel\"]"),
            "{crash}"
        );
        assert!(got.contains("\"ev\":\"js_error\""), "{got}");
    }
    let _ = std::fs::remove_dir_all(&base);
}

/// Today's date in UTC as YYYY-MM-DD (the name of telemetry's day file).
fn chrono_free_today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        / 86400;
    // civil-from-days (Howard Hinnant), as std.time does.
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format!("{year:04}-{month:02}-{day:02}")
}
