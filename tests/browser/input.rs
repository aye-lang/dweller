use super::*;

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

/// `pointer-events: none` (`browser/tests/ui/pointer_events.html`): the
/// click and the hover go to the link beneath an overlay; a child with
/// `auto` takes them back; `none` inherits to the children.
#[test]
fn window_pointer_events() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let ui = format!("file://{}/browser/tests/ui/", root().display());
    let cases: &[(&str, &str, &str)] = &[
        ("click:50,90", "url other.html\n", "status \n"),
        ("click:250,90", "url result.html\n", "status \n"),
        ("move:50,90", "url pointer_events.html\n", "status other.html\n"),
        ("move:250,90", "url pointer_events.html\n", "status result.html\n"),
        ("move:50,202", "url pointer_events.html\n", "status long.html\n"),
        ("click:50,202", "url long.html\n", "status \n"),
    ];
    let mut failures = Vec::new();
    for (events, url, status) in cases {
        let out = run_window(&[
            "browser/tests/ui/pointer_events.html",
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
        let ok = got.lines().any(|l| format!("{l}\n") == *url)
            && got.lines().any(|l| format!("{l}\n") == *status);
        if !out.status.success() || !out.stderr.is_empty() || !ok {
            failures.push(format!("--events {events:?}: want {url}{status}got\n{got}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
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
            "key:tab key:tab key:tab key:space keyup:space key:tab key:down key:down key:tab text:a key:return text:b",
            state("form.html", "element textarea name=note\nvalue a\nb\n"),
        ),
        (
            "key:tab key:tab key:tab key:space keyup:space key:tab key:down key:down key:tab text:a key:return text:b click:170,60",
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
        let out = dweller_command()
            .current_dir(root())
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

/// The window loop's budget for timers and network tasks (`Ui.loop`,
/// TASK_BUDGET_MS): a task that began before it ran out still ends.
const TASK_BUDGET_MS: u64 = 50;

/// Input comes before the page's tasks (`browser/tests/net/timer_storm.html`):
/// for 1.5 s the page queues hundreds of 3 ms tasks (setTimeout(0),
/// chains of them, setInterval(1), 20 slow responses) and scripted input
/// types into its field. Each iteration's tasks (its `timers` records)
/// stop once the budget is spent (at most one task over it), so the
/// window takes input at least once per budget; the input is handled
/// while the storm still runs, and the storm's tasks all run afterwards.
#[test]
fn window_input_priority() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/timer_storm.html", srv.base);
    let mut window = dweller_command()
        .current_dir(root())
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
            "--until-idle",
            "--events",
            "frame frame frame frame frame frame frame frame text:abc frame",
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
    for (i, it) in loops.iter().enumerate() {
        let tasks = it.ms_in(&["timers"]);
        let longest = it
            .phases
            .iter()
            .filter(|p| p.name == "timers")
            .map(|p| p.ms)
            .max()
            .unwrap_or(0);
        assert!(
            tasks <= TASK_BUDGET_MS + longest + 5,
            "iteration {i} of {} ran tasks for {tasks} ms (longest {longest} ms): {it:#?}",
            loops.len()
        );
    }
    let got = text(&out.stdout);
    let want = [
        "input during the storm: true",
        "storm done: 20 responses, tasks after the input: true",
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

/// A pointer moving over 50 cards (P10.51e): `:hover` and `:active`
/// restyle only the elements entering or leaving the chains, with the
/// boxes kept; a page without `:hover` rules restyles nothing.
#[test]
fn window_focus_local() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut events = String::from("painted");
    for _ in 0..20 {
        events += " key:tab frame";
    }
    for i in 10..20 {
        let y = 36 + i * 26;
        events += &format!(" click:140,{y} frame release:140,{y} frame");
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_LAYOUT_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/focus_local.html",
            "--font",
            "fixed",
            "--width",
            "800",
            "--height",
            "700",
            "--frames",
            "120",
            "--events",
            &events,
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
    let restyled: Vec<usize> = recs
        .iter()
        .filter(|r| r.0 == "restyled_elements")
        .map(|r| r.1.parse().unwrap())
        .collect();
    let full = recs.iter().filter(|r| r.0 == "layout_full").count();
    assert!(full == 0, "{full} full layouts:\n{err}");
    assert!(restyled.len() >= 20, "restyled {restyled:?}");
    assert!(
        restyled.iter().filter(|&&n| n <= 18).count() >= 20,
        "restyled {restyled:?}"
    );
}

/// Checkbox clicks and typing in a required field (P10.51i): the form
/// states restyle only the controls and the elements whose selectors see
/// them, with no full restyle or layout after the load.
#[test]
fn window_form_states() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut events = String::from("painted");
    for i in 0..10 {
        let y = 55 + i * 30;
        events += &format!(" click:5,{y} frame release:5,{y} frame");
    }
    events += " click:60,383 frame release:60,383 frame text:ab frame key:backspace frame";
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_LAYOUT_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/form_states.html",
            "--font",
            "fixed",
            "--width",
            "800",
            "--height",
            "700",
            "--frames",
            "120",
            "--events",
            &events,
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
    let restyled: Vec<usize> = recs
        .iter()
        .filter(|r| r.0 == "restyled_elements")
        .map(|r| r.1.parse().unwrap())
        .collect();
    let layouts = recs.iter().filter(|r| r.0 == "layout").count();
    assert!(layouts == 1, "{layouts} full layouts:\n{err}");
    assert!(restyled.len() >= 12, "restyled {restyled:?}");
    assert!(restyled.iter().all(|&n| n <= 12), "restyled {restyled:?}");
}

#[test]
fn window_hover_local() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut events = String::from("painted");
    for row in 0..5 {
        for k in 0..10 {
            events += &format!(" move:{},{} frame", 15 + k * 80, 22 + row * 46);
        }
    }
    events += " click:15,22 frame release:15,22 frame";
    for page in ["hover_local", "hover_none"] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_LAYOUT_CHECK", "1")
            .env("DWELLER_TIMING", "1")
            .args([
                &format!("browser/tests/ui/{page}.html"),
                "--font",
                "fixed",
                "--width",
                "800",
                "--height",
                "600",
                "--frames",
                "150",
                "--events",
                &events,
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
        let restyled: Vec<usize> = recs
            .iter()
            .filter(|r| r.0 == "restyled_elements")
            .map(|r| r.1.parse().unwrap())
            .collect();
        let full = recs.iter().filter(|r| r.0 == "layout_full").count();
        assert!(full <= 1, "{page}: {full} full layouts:\n{err}");
        if page == "hover_none" {
            assert!(restyled.is_empty(), "{page}: restyled {restyled:?}");
            continue;
        }
        assert!(restyled.len() >= 30, "{page}: restyled {restyled:?}");
        assert!(
            restyled.iter().skip(1).all(|&n| n <= 8),
            "{page}: restyled {restyled:?}"
        );
    }
}
