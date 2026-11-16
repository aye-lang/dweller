use super::*;

const SITE: (&str, &str) = ("DWELLER_PROCESS", "site");
const TAB: (&str, &str) = ("DWELLER_PROCESS", "tab");

/// `events` for the process mode of `envs`: each `~` waits for the
/// renderer's frame at the new zoom level (nothing in one process).
fn zoom_events(events: &str, envs: &[(&str, &str)]) -> String {
    let wait = if in_renderer(envs) { "frame renderer:frame" } else { "frame" };
    format!("loaded {}", events.replace('~', wait))
}

fn zoom_page() -> String {
    root().join("browser/tests/ui/zoom.html").display().to_string()
}

/// The page's own line of `--text` (what its script saw).
fn seen(out: &str) -> &str {
    out.lines().find(|l| l.starts_with('w') || l.starts_with("next ")).unwrap_or("")
}

fn zoom_line(out: &str) -> &str {
    out.lines().find(|l| l.strip_prefix("zoom ").is_some_and(|n| n.parse::<u32>().is_ok())).unwrap_or("zoom 100")
}

/// Q5.1: Cmd `=` zooms the page in by Chromium's presets, Cmd `-` out and
/// Cmd `0` back: the page's viewport shrinks as its scale grows
/// (`innerWidth`, `devicePixelRatio`, a width media query), and a click
/// lands on the element drawn under it. The same in a renderer.
#[test]
fn window_zoom_keys() {
    if !has_sdl() {
        return;
    }
    for envs in [[IN_PROCESS], [SITE]] {
        let run = |events: &str| login_window_with(&zoom_page(), &zoom_events(events, &envs), None, &envs);
        let plain = run("click:115,115 frame");
        assert_eq!(seen(&plain), "w400 dpr1 widetrue down115,75 hit", "{envs:?}");
        assert_eq!(zoom_line(&plain), "zoom 100", "{envs:?}");

        let twice = run("key:cmd+= key:cmd+= key:cmd+= key:cmd+= key:cmd+= ~ click:230,170 frame");
        assert_eq!(seen(&twice), "w200 dpr2 widefalse down115,65 hit", "{envs:?}");
        assert_eq!(zoom_line(&twice), "zoom 200", "{envs:?}");

        let plus = run("key:cmd+shift+= ~ click:126,124 frame");
        assert_eq!(zoom_line(&plus), "zoom 110", "{envs:?}");
        assert!(seen(&plus).starts_with("w363.6") && seen(&plus).ends_with(" hit"), "{envs:?}: {plus}");

        let out = run("key:cmd+- key:cmd+- ~ click:100,100 frame");
        assert_eq!(seen(&out), "w500 dpr0.8 widetrue down125,75 hit", "{envs:?}");
        assert_eq!(zoom_line(&out), "zoom 80", "{envs:?}");

        let reset = run("key:cmd+= key:cmd+= ~ key:cmd+0 ~ click:115,115 frame");
        assert_eq!(seen(&reset), "w400 dpr1 widetrue down115,75 hit", "{envs:?}");
        assert_eq!(zoom_line(&reset), "zoom 100", "{envs:?}");

        let chip = run("key:cmd+= key:cmd+= key:cmd+= key:cmd+= key:cmd+= ~ click:345,20 ~ click:115,115 frame");
        assert_eq!(seen(&chip), "w400 dpr1 widetrue down115,75 hit", "{envs:?}");
        assert_eq!(zoom_line(&chip), "zoom 100", "{envs:?}");

        let ends = run("key:cmd+- key:cmd+- key:cmd+- key:cmd+- key:cmd+- key:cmd+- key:cmd+- key:cmd+- key:cmd+- ~ click:30,56 frame");
        assert_eq!(zoom_line(&ends), "zoom 25", "{envs:?}");
        assert_eq!(seen(&ends), "w1600 dpr0.25 widetrue down120,64 hit", "{envs:?}");
    }
}

fn zoom_shot(name: &str, events: &str, envs: &[(&str, &str)]) -> (usize, usize, Vec<u8>) {
    let png = scratch().join(format!("zoom_{name}.png"));
    let out = dweller_command()
        .current_dir(root())
        .envs(envs.iter().copied())
        .args([&zoom_page(), "--font", "fixed", "--width", "400", "--height", "300"])
        .args(["--frames", "400", "--until-idle", "--save"])
        .arg(&png)
        .args(["--events", &zoom_events(events, envs)])
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{name}: {}", text(&out.stderr));
    png_rgba(&std::fs::read(&png).unwrap())
}

fn pixel(shot: &(usize, usize, Vec<u8>), x: usize, y: usize) -> [u8; 3] {
    let at = (y * shot.0 + x) * 4;
    [shot.2[at], shot.2[at + 1], shot.2[at + 2]]
}

/// Q5.1: at 200% the page's pixels are twice as large and the chrome's
/// are not; the address bar shows the level; one process, a tab's
/// renderer and a site's renderer draw the same window.
#[test]
fn window_zoom_pixels() {
    if !has_sdl() {
        return;
    }
    const BLUE: [u8; 3] = [0, 0x66, 0xcc];
    const WHITE: [u8; 3] = [255, 255, 255];
    let zoom_in = "key:cmd+= key:cmd+= key:cmd+= key:cmd+= key:cmd+= ~ frame";
    let plain = zoom_shot("plain", "frame", &[IN_PROCESS]);
    let zoomed = zoom_shot("single", zoom_in, &[IN_PROCESS]);

    assert_eq!((plain.0, plain.1), (400, 300));
    assert_eq!(pixel(&plain, 130, 115), BLUE);
    assert_eq!(pixel(&plain, 250, 180), WHITE);
    assert_eq!(pixel(&zoomed, 250, 180), BLUE);
    assert_eq!(pixel(&zoomed, 201, 161), BLUE);
    assert_eq!(pixel(&zoomed, 199, 161), WHITE);
    assert_eq!(pixel(&zoomed, 319, 219), BLUE);
    assert_eq!(pixel(&zoomed, 321, 219), WHITE);
    assert_eq!(pixel(&zoomed, 130, 115), WHITE);

    for y in 0..40 {
        for x in 0..300 {
            assert_eq!(pixel(&plain, x, y), pixel(&zoomed, x, y), "the toolbar at {x},{y}");
        }
    }

    let chip = (320..368).flat_map(|x| (8..32).map(move |y| (x, y))).filter(|&(x, y)| pixel(&plain, x, y) != pixel(&zoomed, x, y)).count();

    assert!(chip > 40, "the zoom chip changed only {chip} pixels");

    for (name, envs) in [("tab", [TAB]), ("site", [SITE])] {
        let shot = zoom_shot(name, zoom_in, &envs);
        assert!(shot == zoomed, "{name}: the renderer's window differs from the one-process window (zoom_{name}.png, zoom_single.png in {})", scratch().display());
    }
}

/// Q5.1: the level belongs to the host and is kept in the profile:
/// another page of the host opens at it, in this run and in the next,
/// and a page of another host (here a file) opens at 100%.
#[test]
fn window_zoom_per_host_remembered() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    for (name, envs) in [("single", [IN_PROCESS]), ("site", [SITE])] {
        let dir = profile(&format!("zoom_profile_{name}"));
        let page = format!("{}/ui/zoom.html", s.base);
        let next = format!("{}/ui/zoom-next.html", s.base);
        let first = login_window_with(&page, &format!("loaded key:cmd+= key:cmd+= frame {} frame", go_to(&next)), Some(&dir), &envs);

        assert_eq!(seen(&first), "next w320", "{name}: {first}");
        assert_eq!(zoom_line(&first), "zoom 125", "{name}");
        assert_eq!(std::fs::read_to_string(dir.join("zoom")).unwrap(), "dweller-zoom 1\n125 127.0.0.1\n", "{name}");

        let again = login_window_with(&next, "loaded frame", Some(&dir), &envs);

        assert_eq!(seen(&again), "next w320", "{name}: {again}");
        assert_eq!(zoom_line(&again), "zoom 125", "{name}");

        let other = login_window_with(&zoom_page(), &format!("loaded frame {} frame", go_to(&next)), Some(&dir), &envs);

        assert_eq!(seen(&other), "next w320", "{name}: {other}");

        let file = login_window_with(&zoom_page(), "loaded frame", Some(&dir), &envs);

        assert!(seen(&file).contains(" dpr1 "), "{name}: {file}");
        assert_eq!(zoom_line(&file), "zoom 100", "{name}");

        let back = login_window_with(&next, "loaded key:cmd+0 frame", Some(&dir), &envs);

        assert_eq!(zoom_line(&back), "zoom 100", "{name}");
        assert_eq!(std::fs::read_to_string(dir.join("zoom")).unwrap(), "dweller-zoom 1\n", "{name}");
    }
}
