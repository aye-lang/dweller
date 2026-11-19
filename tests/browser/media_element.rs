use super::*;

/// Q5.9a slice 1: the media element (`browser/tests/media/element.html`,
/// a WAV file of half a second) must print `element.txt`: `canPlayType`,
/// the events of a load in the standard's order, `play()` to `ended` on
/// the media clock with `timeupdate`s and the `played` range, a seek,
/// `playbackRate`, `loop`, `volume`'s range, the autoplay policy (an
/// unmuted `play()` without a gesture is `NotAllowedError`), a resource
/// that is not media (`MEDIA_ERR_SRC_NOT_SUPPORTED`), `<source>` selection
/// with its `error` events, and `load()`'s `abort` and `emptied`. On the
/// virtual clock (`dump`) and in the window with `DWELLER_PROCESS=site`.
#[test]
fn media_element_wav() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/media/element.html", srv.base);
    let out = run(&["dump", "--text", "--js", &href]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/media/element.txt");
    let mut failures = Vec::new();
    check_golden(&golden, &text(&out.stdout), "media_element_wav", &mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let want = std::fs::read_to_string(golden).unwrap();
    let site = login_window_with(&href, "frame", None, &[("DWELLER_PROCESS", "site")]);
    for line in want.lines().filter(|l| l.contains(": ")) {
        assert!(site.contains(line), "{line} in {site}");
    }
}

/// Q5.9a slice 2: a media resource is loaded by `Range` requests
/// (`browser/tests/media/ranges.html` against the test server's
/// `/ranged`): `preload=metadata` asks one range and suspends,
/// `preload=auto` and playing ask the rest, a server without ranges
/// answers whole, and a seek past the buffered end asks from the seek's
/// byte, leaving two `buffered` ranges. On the virtual clock and in the
/// window with `DWELLER_PROCESS=site`.
#[test]
fn media_element_ranges() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/media/ranges.html", srv.base);
    let out = run(&["dump", "--text", "--js", &href]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/media/ranges.txt");
    let mut failures = Vec::new();
    check_golden(&golden, &text(&out.stdout), "media_element_ranges", &mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let Some(again) = server() else { return };
    let want = std::fs::read_to_string(golden).unwrap();
    let site = login_window_with(&format!("{}/media/ranges.html", again.base), "frame", None, &[("DWELLER_PROCESS", "site")]);
    for line in want.lines().filter(|l| l.contains(": ")) {
        assert!(site.contains(line), "{line} in {site}");
    }
}

/// Q5.9a slice 4a: an unmuted `<audio>` played from a click
/// (`browser/tests/media/sound.html`) sends its samples to the page's
/// audio output: one buffer of the file's half second, one channel, at
/// rate 1 and not silent, and `ended` comes on the real clock. In the
/// window in one process and with `DWELLER_PROCESS=site`.
#[test]
fn media_element_sound() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    for mode in ["single", "site"] {
        let Some(srv) = server() else {
            println!("skipped: cannot start python3 for the test server");
            return;
        };
        let href = format!("{}/media/sound.html", srv.base);
        let envs = [("DWELLER_PROCESS", mode), ("SDL_AUDIODRIVER", "dummy")];
        let got = login_window_with(&href, "painted click:100,90 frame", None, &envs);
        for line in [
            "activated: true",
            "playing: muted=false paused=false",
            "scheduled: 1 buffer(s), 0.50 s, channels 1, rate 1, loud true",
            "done",
        ] {
            assert!(got.contains(line), "{mode}: {line} in {got}");
        }
    }
}

/// Q5.9a slice 4b: a `<video>` whose frames come from the decoder
/// process (`browser/tests/media/video.html`, `clip.mkv` of
/// `make_clip.py`: uncompressed RGBA and PCM in Matroska) must print
/// `video.txt`: the natural size and the box, the first frame, the
/// frame of each seek (its colour and where its square is, read back
/// through `drawImage`), and playing to the end. On the virtual clock
/// and in the window with `DWELLER_PROCESS=site`.
#[test]
fn media_element_video() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/media/video.html", srv.base);
    let out = run(&["dump", "--text", "--js", &href]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/media/video.txt");
    let mut failures = Vec::new();
    check_golden(&golden, &text(&out.stdout), "media_element_video", &mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let want = std::fs::read_to_string(golden).unwrap();
    let site = login_window_with(&href, "frame", None, &[("DWELLER_PROCESS", "site")]);
    for line in want.lines().filter(|l| l.contains(": ")) {
        assert!(site.contains(line), "{line} in {site}");
    }
}

/// Q5.9a slice 4b: the frame of a `<video>` is painted in its box
/// (`browser/tests/media/painted.html`: three videos sought to the blue
/// frame, whose white square is 8 px wide at x 8): scaled to the box,
/// letterboxed by the UA sheet's `object-fit: contain` when the box has
/// another ratio, and stretched with `object-fit: fill`. In the window
/// in one process and with `DWELLER_PROCESS=site`, read from `--save`.
#[test]
fn media_element_video_painted() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    const TOP: usize = 40;
    const BLUE: [u8; 3] = [0, 0, 255];
    const WHITE: [u8; 3] = [255, 255, 255];
    for mode in ["single", "site"] {
        let Some(srv) = server() else {
            println!("skipped: cannot start python3 for the test server");
            return;
        };
        let png = scratch().join(format!("media_painted_{mode}.png"));
        let mut events = "frame";
        if mode == "site" {
            events = "renderer:loaded frame";
        }
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", mode)
            .args([&format!("{}/media/painted.html", srv.base), "--font", "fixed", "--width", "400", "--height", "300"])
            .args(["--frames", "2000", "--until-idle", "--text", "--save"])
            .arg(&png)
            .args(["--events", events])
            .output()
            .unwrap();
        let got = text(&out.stdout);
        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {}\n{}", out.status, text(&out.stderr));
        assert!(got.contains("shown: a blue, b blue, c blue"), "{mode}: {got}");
        let (width, _, rgba) = png_rgba(&std::fs::read(&png).unwrap());
        let at = |x: usize, y: usize| -> [u8; 3] {
            let i = ((TOP + y) * width + x) * 4;
            [rgba[i], rgba[i + 1], rgba[i + 2]]
        };
        for (what, x, y, want) in [
            ("a, scaled by 2: the frame", 4, 30, BLUE),
            ("a: the square", 20, 8, WHITE),
            ("a: right of the square", 40, 8, BLUE),
            ("b, letterboxed: the bar above", 68, 2, WHITE),
            ("b: the frame", 68, 38, BLUE),
            ("b: the square", 84, 16, WHITE),
            ("c, stretched: the frame", 132, 20, BLUE),
            ("c: the square", 148, 4, WHITE),
            ("c: below the square", 148, 12, BLUE),
            ("outside the videos", 200, 10, WHITE),
        ] {
            assert_eq!(at(x, y), want, "{mode}: {what} at {x},{y}");
        }
    }
}
