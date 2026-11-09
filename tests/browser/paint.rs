use super::*;

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
        let out = dweller_command()
            .current_dir(root())
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
                "600",
                "--until-idle",
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

/// Q4.6a: texts next to floats lay out again from the formatting root
/// that holds the floats (`relayoutFrom`), up to the next root when that
/// one changes size: no full layout after the first frame, the same
/// boxes as a full layout (`DWELLER_LAYOUT_CHECK`).
#[test]
fn window_relayout_floats() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_LAYOUT_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/relayout_floats.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "600",
            "--until-idle",
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
    assert_eq!(
        count("layout_full") + count("layout"),
        0,
        "full layouts after the first frame:\n{err}"
    );
    assert!(
        count("layout_dirty") >= 30,
        "{} dirty relayouts:\n{err}",
        count("layout_dirty")
    );
}

/// Attribute changes are laid out again incrementally (Q4.6b): one that
/// a selector names, presentational ones (`width` of an image,
/// `hidden`) and ones layout reads (`colspan`, a canvas's `width`, `ol
/// start`) relay out their boxes, with the same boxes as a full layout
/// (`DWELLER_LAYOUT_CHECK`); ones that no selector names (`data-x`,
/// `aria-label`) lay out nothing.
#[test]
fn window_relayout_attr() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let layouts = |page: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_LAYOUT_CHECK", "1")
            .env("DWELLER_TIMING", "1")
            .args([
                page,
                "--font",
                "fixed",
                "--width",
                "400",
                "--height",
                "300",
                "--frames",
                "600",
                "--until-idle",
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
        (count("layout_full") + count("layout"), count("layout_dirty"), err.clone())
    };
    let (full, dirty, err) = layouts("browser/tests/ui/relayout_attr.html");
    assert_eq!(full, 0, "full layouts after the first frame:\n{err}");
    assert!(dirty >= 30, "{dirty} dirty relayouts:\n{err}");
    let (full, dirty, err) = layouts("browser/tests/ui/relayout_attr_unmatched.html");
    assert_eq!(full + dirty, 0, "layouts after the first frame:\n{err}");
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
    let out = dweller_command()
        .current_dir(root())
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

/// The DWELLER_TIMING records of each frame of a window's `err`, as
/// (name, label) pairs: those written up to and including each `ui_draw`.
fn frame_records(err: &str) -> Vec<Vec<(String, String)>> {
    let mut frames = vec![Vec::new()];
    for line in err.lines() {
        let Some(record) = line.strip_prefix("info: DWELLER_TIMING\t") else {
            continue;
        };
        let fields: Vec<&str> = record.split('\t').collect();
        let name = fields[0].to_string();
        let label = fields.get(2).unwrap_or(&"").to_string();
        let end = name == "ui_draw";
        frames.last_mut().unwrap().push((name, label));
        if end {
            frames.push(Vec::new());
        }
    }
    frames.pop();
    frames
}

// The window and page rasters of a frame's records ("PIXELS RECTS").
fn rasters(frame: &[(String, String)]) -> Vec<(&str, &str)> {
    frame
        .iter()
        .filter(|(n, _)| n == "ui_window_px" || n == "raster_px")
        .map(|(n, l)| (n.as_str(), l.as_str()))
        .collect()
}

fn damage_window_run(page: &str, png: &std::path::Path, events: &str) -> String {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            page,
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "200",
            "--until-idle",
            "--save",
            png.to_str().unwrap(),
            "--events",
            events,
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("DAMAGE CHECK"), "{err}");
    err
}

/// Q4.15: a still page with a focused field (an overlay over the page)
/// rasters nothing once loaded: 20 more frames draw no page pixel and
/// compose no window pixel.
#[test]
fn window_idle_no_raster() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let png = std::env::temp_dir().join(format!("dweller-q415-idle-{}.png", std::process::id()));
    let events = format!("painted{}", " frame".repeat(20));
    let err = damage_window_run("browser/tests/ui/damage_window_final.html", &png, &events);
    let _ = std::fs::remove_file(&png);
    let frames = frame_records(&err);
    assert!(frames.len() >= 20, "{} frames:\n{err}", frames.len());
    let idle: Vec<_> = frames[frames.len() - 15..].iter().map(|f| rasters(f)).collect();
    assert!(idle.iter().all(|r| r.is_empty()), "rasters while idle: {idle:?}");
}

/// Q4.15: with a focused field over the page, a rAF tick that changes
/// one box's color rasters that box alone (its 40x30 rect with the
/// one-pixel margin of the damage) in the page and in the window, and
/// the window equals the page drawn with that color from the start.
#[test]
fn window_damage_rect_only() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = std::env::temp_dir();
    let id = std::process::id();
    let changed = dir.join(format!("dweller-q415-changed-{id}.png"));
    let whole = dir.join(format!("dweller-q415-whole-{id}.png"));
    let err = damage_window_run("browser/tests/ui/damage_window.html", &changed, "painted");
    damage_window_run("browser/tests/ui/damage_window_final.html", &whole, "painted");
    let (a, b) = (std::fs::read(&changed).unwrap(), std::fs::read(&whole).unwrap());
    let _ = std::fs::remove_file(&changed);
    let _ = std::fs::remove_file(&whole);
    assert!(a == b, "the window differs from a whole drawing");
    let frames = frame_records(&err);
    let painted = frames
        .iter()
        .rposition(|f| f.iter().any(|(n, _)| n == "ui_window_px"))
        .unwrap_or_else(|| panic!("no window raster:\n{err}"));
    let last = rasters(&frames[painted]);
    assert_eq!(last, vec![("raster_px", "1344 1"), ("ui_window_px", "1344 1")], "{err}");
}

/// Q4.19: a rAF tick that writes the same text into the top document
/// builds the display list of that document alone; the still iframe keeps
/// its list, the iframe whose own script changes its text is built again
/// in the frames it changes, and the last frame equals the final page
/// drawn from the start.
#[test]
fn window_frame_lists_reused() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = std::env::temp_dir();
    let id = std::process::id();
    let changed = dir.join(format!("dweller-q419-changed-{id}.png"));
    let whole = dir.join(format!("dweller-q419-whole-{id}.png"));
    let err = damage_window_run("browser/tests/ui/frame_lists.html", &changed, "painted");
    damage_window_run("browser/tests/ui/frame_lists_final.html", &whole, "painted");
    let (a, b) = (std::fs::read(&changed).unwrap(), std::fs::read(&whole).unwrap());
    let _ = std::fs::remove_file(&changed);
    let _ = std::fs::remove_file(&whole);
    assert!(a == b, "the window differs from the final page drawn whole");
    let builds: Vec<usize> = frame_records(&err)
        .iter()
        .map(|f| f.iter().filter(|(n, _)| n == "paint_build").count())
        .collect();
    let top_only = builds.iter().filter(|&&n| n == 1).count();
    let with_b = builds.iter().filter(|&&n| n == 2).count();
    let all = builds.iter().filter(|&&n| n >= 3).count();
    assert!(top_only >= 20, "{top_only} frames built the top list alone: {builds:?}");
    assert!(with_b >= 5, "{with_b} frames built the changing iframe: {builds:?}");
    assert!(all <= 2, "{all} frames built every list: {builds:?}");
}

/// Q4.22: a rAF tick that writes the text an element already has, with
/// no MutationObserver on the page, changes nothing: only the load and
/// the tick that writes other text build the display list.
#[test]
fn window_same_text_write_builds_nothing() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let png = std::env::temp_dir().join(format!("dweller-q422-{}.png", std::process::id()));
    let err = damage_window_run("browser/tests/ui/same_text.html", &png, "painted");
    let _ = std::fs::remove_file(&png);
    let builds: Vec<usize> = frame_records(&err)
        .iter()
        .map(|f| f.iter().filter(|(n, _)| n == "paint_build").count())
        .collect();
    let built = builds.iter().filter(|&&n| n > 0).count();
    assert!(built >= 2, "{built} frames built a list: {builds:?}");
    assert!(built <= 4, "{built} frames built a list: {builds:?}");
}

/// A trackpad scrolls by fractions of a device pixel (P4.36b): the window
/// draws the page at its scroll offset snapped to device pixels, so such a
/// frame moves the last frame's pixels and draws the band, the fixed
/// header and the sticky heading (or nothing), never the whole page, and
/// `DWELLER_DAMAGE_CHECK` finds each frame equal to a whole drawing. The
/// fixed and sticky boxes are layers moved at draw time (P4.36a): no
/// scroll frame builds the display list again.
#[test]
fn window_fractional_scroll() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut events = vec!["wheel:-0.03 frame"; 30];
    events.extend(vec!["wheel:-0.37 frame"; 20]);
    events.extend(vec!["wheel:0.11 frame"; 10]);
    let out = dweller_command()
        .current_dir(root())
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
    let builds = err.matches("DWELLER_TIMING\tpaint_build\t").count();
    assert!(builds <= 2, "{builds} display lists built:\n{err}");
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
        let out = dweller_command()
            .current_dir(root())
            .env(env, "1")
            .env_remove(without)
            .args([
                &url, "--font", "fixed", "--width", "800", "--height", "600", "--frames", "600",
                "--until-idle", "--text",
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
    let out = dweller_command()
        .current_dir(root())
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
            "600",
            "--until-idle",
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

/// A CSS-only animation (P10.51b): `browser/tests/ui/css_spinner.html`
/// moves and fades a box twice with no script. The window starts it from
/// the styles, composites its frames without layout, damages only the
/// box's rect, and once it ended runs no more animation frames. The
/// animation runs on the wall clock (2 x 600 ms), so 15 composited frames
/// fit at up to 80 ms a frame, and 300 frames leave 50 after its end at
/// 10 ms a frame; `--until-idle` would end the run before those 50.
#[test]
fn window_css_animation() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/css_spinner.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "300",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("CHECK"), "{err}");
    let recs: Vec<(&str, &str)> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\t").nth(1))
        .map(|r| {
            let f: Vec<&str> = r.split('\t').collect();
            (f[0], *f.get(2).unwrap_or(&""))
        })
        .collect();
    let first = recs
        .iter()
        .position(|r| r.0 == "ui_composite")
        .expect("no composited frame");
    let after = &recs[first..];
    let composites = after.iter().filter(|r| r.0 == "ui_composite").count();
    assert!(composites >= 15, "{composites} composited frames:\n{err}");
    let damage: Vec<u64> = after
        .iter()
        .filter(|r| r.0 == "ui_damage_px")
        .map(|r| r.1.split(' ').next().unwrap().parse().unwrap())
        .collect();
    assert!(
        damage.iter().all(|&px| px <= 70 * 50),
        "damage beyond the box: {damage:?}"
    );
    let last_raf = recs.iter().rposition(|r| r.0 == "ui_raf").unwrap();
    let idle = recs[last_raf..].iter().filter(|r| r.0 == "ui_draw").count();
    assert!(idle >= 50, "animation frames until the end ({idle} idle frames):\n{err}");
    assert!(
        recs[last_raf..]
            .iter()
            .filter(|r| r.0 == "ui_damage_px")
            .skip(1)
            .all(|r| r.1.starts_with("0 ")),
        "damage after the animation ended:\n{err}"
    );
}

/// A CSS transition on `:hover` (P10.51d): `browser/tests/ui/hover_transition.html`
/// moves the pointer over a box with `transition: background-color .6s,
/// transform .6s`, with no script. Its frames damage only the box's rect,
/// and once it ended no more animation frames run. As in
/// `window_css_animation` the transition runs on the wall clock: 10
/// frames fit in it at up to 60 ms a frame, and 300 frames leave 50 after
/// it at 10 ms a frame.
#[test]
fn window_css_transition() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/hover_transition.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "300",
            "--events",
            "painted move:120,160",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("CHECK"), "{err}");
    let recs: Vec<(&str, &str)> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\t").nth(1))
        .map(|r| {
            let f: Vec<&str> = r.split('\t').collect();
            (f[0], *f.get(2).unwrap_or(&""))
        })
        .collect();
    let first = recs
        .iter()
        .position(|r| r.0 == "ui_raf")
        .expect("no transition frame");
    let after = &recs[first..];
    let damage: Vec<u64> = after
        .iter()
        .filter(|r| r.0 == "ui_damage_px")
        .map(|r| r.1.split(' ').next().unwrap().parse().unwrap())
        .collect();
    let moving = damage.iter().filter(|&&px| px > 0).count();
    assert!(moving >= 10, "{moving} frames with damage: {damage:?}");
    assert!(
        damage.iter().all(|&px| px <= 70 * 50),
        "damage beyond the box: {damage:?}"
    );
    assert_eq!(
        after.iter().filter(|r| r.0 == "layout_full").count(),
        0,
        "a full layout during the transition:\n{err}"
    );
    let last_raf = recs.iter().rposition(|r| r.0 == "ui_raf").unwrap();
    let idle = recs[last_raf..].iter().filter(|r| r.0 == "ui_draw").count();
    assert!(
        idle >= 50,
        "transition frames until the end ({idle} idle frames):\n{err}"
    );
    assert!(
        recs[last_raf..]
            .iter()
            .filter(|r| r.0 == "ui_damage_px")
            .skip(1)
            .all(|r| r.1.starts_with("0 ")),
        "damage after the transition ended:\n{err}"
    );
}

#[test]
fn window_transition_kept() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut events = String::from("painted");
    for _ in 0..250 {
        events += " move:120,160 frame move:300,250 frame";
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/hover_transition.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "1100",
            "--events",
            &events,
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    let kept: Vec<usize> = err
        .lines()
        .filter_map(|l| l.split("DWELLER_TIMING\ttransitions_kept\t").nth(1))
        .map(|r| r.split('\t').nth(1).unwrap().split(' ').next().unwrap().parse().unwrap())
        .collect();
    assert!(kept.len() >= 100, "{} transition frames", kept.len());
    assert!(kept.iter().all(|&n| n <= 32), "kept grows: {kept:?}");
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
    let out = dweller_command()
        .current_dir(root())
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
            "600",
            "--until-idle",
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

/// Wheel, trackpad and keys scroll the scroll container under the
/// pointer (P10.58, ui/wheel-scroll-element.aye): it scrolls and the page
/// does not; at its edge the next wheel scrolls the page, unless
/// `overscroll-behavior: contain`; x deltas scroll it sideways; a
/// trackpad gesture stays latched to it past its edge, and one that began
/// on the page stays on the page; keys after a click in it scroll it;
/// `scroll` and `scrollend` fire at it.
#[test]
fn window_scroll_containers() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = root().join("browser/tests/ui/scroller.html");
    let words = |word: &str, n: usize| vec![word; n].join(" ");
    let notch = "move:100,100 wheel:-1 frame";
    let cases = [
        ("", "move:100,100 wheel:-1".to_string(), 0, "top=40 left=0 y=0 scrolled=true ends=1"),
        ("", words(notch, 6), 40, "top=200 left=0 y=40 scrolled=true"),
        ("?contain", words(notch, 6), 0, "top=200 left=0 y=0 scrolled=true"),
        ("", "move:100,100 wheel:3,0 frame".to_string(), 0, "top=0 left=120 y=0"),
        ("", "move:100,300 wheel:-1".to_string(), 40, "top=0 left=0 y=40 scrolled=false"),
        ("", words("move:100,100 wheel:-0.5", 50), 0, "top=200 left=0 y=0 scrolled=true ends=1"),
        ("", format!("{} {}", words("move:100,300 wheel:-0.5", 3), words("move:100,100 wheel:-0.5", 3)), 30, "top=0 left=0 y=30"),
        ("", "click:100,100 key:down key:down key:pagedown key:end key:right".to_string(), 0, "top=200 left=40 y=0"),
    ];
    let mut failures = Vec::new();
    for (query, events, scroll, line) in &cases {
        let target = format!("file://{}{query}", page.display());
        let out = run_window(&[
            &target, "--font", "fixed", "--width", "400", "--height", "340", "--frames", "120", "--state", "--text",
            "--events", events,
        ]);
        let got = text(&out.stdout);
        let want = format!("\nscroll {scroll}\n");
        if !out.status.success() || !out.stderr.is_empty() || !got.contains(&want) || !got.contains(line) {
            failures.push(format!("{query} {events}: expected {want:?} and {line:?}:\n{got}{}", text(&out.stderr)));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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
