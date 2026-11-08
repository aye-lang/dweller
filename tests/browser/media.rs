use super::*;

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
    let out = dweller_command()
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
        let mut c = dweller_command();
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
    let mut window = dweller_command()
        .current_dir(root())
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
        let mut c = dweller_command();
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
        let mut c = dweller_command();
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

/// `loading=lazy` (`browser/tests/net/lazy_images.html`, 100 lazy
/// images below an eager one in a 300 px viewport): the window fetches
/// the eager image and the 15 lazy ones within 1250 px of the viewport at
/// load, and each other one once scrolling brings it near.
#[test]
fn window_lazy_images() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let scroll = format!("loaded{}", " key:pagedown frame frame".repeat(40));
    for (events, want) in [("loaded", 16), (scroll.as_str(), 101)] {
        let Some(srv) = server() else {
            println!("skipped: cannot start python3 for the test server");
            return;
        };
        let page = format!("{}/net/lazy_images.html", srv.base);
        let out = run_window(&[
            &page, "--width", "400", "--height", "300", "--frames", "200", "--until-idle",
            "--events", events,
        ]);
        assert!(
            out.status.success(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        let log = text(&run(&["dump", "--text", &format!("{}/log", srv.base)]).stdout);
        let images: Vec<&str> = log.lines().filter(|l| l.contains(".png")).collect();
        let once: std::collections::BTreeSet<&&str> = images.iter().collect();
        assert_eq!((images.len(), once.len()), (want, want), "{events}\n{log}");
    }
}

/// A horizontal scroller of 30 lazy 200px images in a 400px window:
/// the 9 within 1250px of its visible part load, and 9 more once a
/// script scrolls it 2000px to the right.
#[test]
fn window_lazy_images_in_scroller() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let frames = " frame".repeat(20);
    let scroll = format!("loaded{frames} key:pagedown{frames}");
    for (events, want) in [("loaded", 9), (scroll.as_str(), 18)] {
        let Some(srv) = server() else {
            println!("skipped: cannot start python3 for the test server");
            return;
        };
        let page = format!("{}/net/lazy_scroller.html", srv.base);
        let out = run_window(&[
            &page, "--width", "400", "--height", "300", "--frames", "200", "--until-idle",
            "--events", events,
        ]);
        assert!(
            out.status.success(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        let log = text(&run(&["dump", "--text", &format!("{}/log", srv.base)]).stdout);
        let images: Vec<&str> = log.lines().filter(|l| l.contains(".png")).collect();
        let once: std::collections::BTreeSet<&&str> = images.iter().collect();
        assert_eq!((images.len(), once.len()), (want, want), "{events}\n{log}");
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
    let mut window = dweller_command()
        .current_dir(root())
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
