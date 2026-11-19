use super::*;

/// The window on `browser/tests/ui/ime.html` with `events` (`~` waits
/// for the renderer to handle the input before it); `--state`, `--text`
/// and, when asked, the PNG.
fn ime_window(events: &str, site: bool, png: Option<&Path>) -> String {
    let (first, wait) = if site { ("renderer:loaded frame", "renderer:input") } else { ("loaded frame", "frame") };
    let mut c = dweller_command();
    c.current_dir(root())
        .env("DWELLER_PROCESS", if site { "site" } else { "single" })
        .env("DWELLER_STEADY_CARET", "1")
        .args(["browser/tests/ui/ime.html", "--font", "fixed", "--width", "400", "--height", "400", "--frames", "1500", "--until-idle", "--state", "--text"])
        .args(["--events", &format!("{first} {} frame frame", events.replace('~', wait))]);
    if let Some(p) = png {
        c.arg("--save").arg(p);
    }
    let out = c.output().unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{events}: {}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

fn ime_events(out: &str) -> Vec<&str> {
    out.lines().filter(|l| ["compositionstart ", "compositionupdate ", "compositionend ", "beforeinput ", "input ", "blur "].iter().any(|p| l.starts_with(p))).collect()
}

fn state<'a>(out: &'a str, key: &str) -> &'a str {
    out.lines().find(|l| l.starts_with(key)).unwrap_or("")
}

/// Q5.4: text composed by an input method is in the field while it is
/// composed; a page sees `compositionstart`, then for each update
/// `beforeinput`, `compositionupdate` and `input` of
/// `insertCompositionText`; a commit ends with `compositionend` and the
/// committed text; an empty update cancels. The same in a renderer.
#[test]
fn window_ime_composition() {
    if !has_sdl() {
        return;
    }
    for site in [false, true] {
        let out = ime_window("click:100,62 ~ compose:に ~ compose:日本 ~ text:日本語 ~ compose:x ~ compose: ~", site, None);

        assert_eq!(
            ime_events(&out),
            [
                "compositionstart field data=\"\" value=\"ab\" true",
                "beforeinput field data=\"に\" value=\"ab\" insertCompositionText composing=true",
                "compositionupdate field data=\"に\" value=\"ab\" true",
                "input field data=\"に\" value=\"abに\" insertCompositionText composing=true",
                "beforeinput field data=\"日本\" value=\"abに\" insertCompositionText composing=true",
                "compositionupdate field data=\"日本\" value=\"abに\" true",
                "input field data=\"日本\" value=\"ab日本\" insertCompositionText composing=true",
                "beforeinput field data=\"日本語\" value=\"ab日本\" insertCompositionText composing=true",
                "compositionupdate field data=\"日本語\" value=\"ab日本\" true",
                "input field data=\"日本語\" value=\"ab日本語\" insertCompositionText composing=true",
                "compositionend field data=\"日本語\" value=\"ab日本語\" true",
                "compositionstart field data=\"\" value=\"ab日本語\" true",
                "beforeinput field data=\"x\" value=\"ab日本語\" insertCompositionText composing=true",
                "compositionupdate field data=\"x\" value=\"ab日本語\" true",
                "input field data=\"x\" value=\"ab日本語x\" insertCompositionText composing=true",
                "beforeinput field data=\"\" value=\"ab日本語x\" insertCompositionText composing=true",
                "compositionupdate field data=\"\" value=\"ab日本語x\" true",
                "input field data=\"\" value=\"ab日本語\" insertCompositionText composing=true",
                "compositionend field data=\"\" value=\"ab日本語\" true",
            ],
            "site {site}: {out}"
        );
        assert_eq!(state(&out, "composing "), "", "site {site}");

        let same = ime_window("click:100,62 ~ compose:´ ~ text:é ~ text:! ~", site, None);
        let events = ime_events(&same);

        assert_eq!(events[4], "beforeinput field data=\"é\" value=\"ab´\" insertCompositionText composing=true", "site {site}: {same}");
        assert_eq!(events[7], "compositionend field data=\"é\" value=\"abé\" true", "site {site}");
        assert_eq!(events[8..], ["beforeinput field data=\"!\" value=\"abé\" insertText composing=false", "input field data=\"!\" value=\"abé!\" insertText composing=false"], "site {site}");

        let left = ime_window("click:100,62 ~ compose:日本 ~ click:100,112 ~", site, None);
        let events = ime_events(&left);

        assert_eq!(events[4..], ["compositionend field data=\"日本\" value=\"ab日本\" true", "blur field"], "site {site}: {left}");
    }
}

/// Q5.4: the candidate window's place (`SDL_SetTextInputRect`) is the
/// caret of the focused field, in window points: it moves with the
/// composed text, with the page's zoom, into a `textarea`'s second line;
/// the composed text is underlined; one process and a renderer draw the
/// same window.
#[test]
fn window_ime_caret_rect() {
    if !has_sdl() {
        return;
    }
    let mut shots = Vec::new();
    for site in [false, true] {
        let name = if site { "site" } else { "single" };
        let composing = scratch().join(format!("ime_composing_{name}.png"));
        let committed = scratch().join(format!("ime_committed_{name}.png"));
        let focused = ime_window("click:100,62 ~ compose:x ~ compose: ~", site, None);
        let one = ime_window("click:100,62 ~ compose:に ~", site, None);
        let two = ime_window("click:100,62 ~ compose:日本 ~", site, Some(&composing));
        let done = ime_window("click:100,62 ~ compose:日本 ~ text:日本 ~", site, Some(&committed));

        assert_eq!(state(&focused, "ime-rect "), "ime-rect 27 53 1 16", "site {site}: {focused}");
        assert_eq!(state(&one, "ime-rect "), "ime-rect 43 53 1 16", "site {site}");
        assert_eq!(state(&two, "ime-rect "), "ime-rect 59 53 1 16", "site {site}");
        assert_eq!(state(&two, "composing "), "composing 日本", "site {site}");
        assert_eq!(state(&done, "ime-rect "), "ime-rect 59 53 1 16", "site {site}");

        let a = png_rgba(&std::fs::read(&composing).unwrap());
        let b = png_rgba(&std::fs::read(&committed).unwrap());
        let differ: Vec<(usize, usize)> = (0..a.0 * 80).filter(|&i| a.2[i * 4..i * 4 + 4] != b.2[i * 4..i * 4 + 4]).map(|i| (i % a.0, i / a.0)).collect();

        assert!(!differ.is_empty() && differ.len() <= 32, "site {site}: {} pixels differ", differ.len());
        assert!(differ.iter().all(|&(x, y)| (27..59).contains(&x) && (66..72).contains(&y)), "site {site}: {differ:?}");
        assert!(differ.iter().all(|&(_, y)| y == differ[0].1), "site {site}: the underline is more than one row");

        let area = ime_window("click:100,102 ~ compose:あ ~ text:あ ~ key:return ~ compose:い ~", site, None);

        assert_eq!(state(&area, "ime-rect "), "ime-rect 27 113 1 16", "site {site}: {area}");

        let zoomed_wait = if site { "frame renderer:frame" } else { "frame" };
        let zoomed = ime_window(&format!("key:cmd+= key:cmd+= key:cmd+= key:cmd+= key:cmd+= {zoomed_wait} click:200,84 ~ compose:に ~"), site, None);

        assert_eq!(state(&zoomed, "ime-rect "), "ime-rect 86 66 1 32", "site {site}: {zoomed}");
        shots.push(std::fs::read(&composing).unwrap());
    }

    assert!(shots[0] == shots[1], "the renderer draws another composition than one process");
}

/// Q5.4: the address bar composes too: the text shows in the bar (and
/// the suggestions follow it), a commit replaces it, and the candidate
/// window sits at the bar's caret.
#[test]
fn window_ime_address_bar() {
    if !has_sdl() {
        return;
    }
    for site in [false, true] {
        let png = scratch().join(format!("ime_address_{site}.png"));
        let one = ime_window("key:cmd+l compose:に frame", site, None);
        let two = ime_window("key:cmd+l compose:にほ frame", site, Some(&png));
        let done = ime_window("key:cmd+l compose:にほ frame text:日本 frame text:! frame", site, None);
        let gone = ime_window("key:cmd+l text:a compose:にほ frame compose: frame", site, None);
        let rect = |out: &str| -> Vec<i32> { state(out, "ime-rect ").split(' ').skip(1).map(|n| n.parse().unwrap()).collect() };

        assert_eq!(state(&one, "address "), "address に", "site {site}: {one}");
        assert_eq!(state(&two, "address "), "address にほ", "site {site}");
        assert_eq!(state(&two, "composing "), "composing にほ", "site {site}");
        assert_eq!(state(&two, "suggest "), "suggest > Search for にほ", "site {site}");
        assert_eq!(state(&done, "address "), "address 日本!", "site {site}: {done}");
        assert_eq!(state(&done, "composing "), "", "site {site}");
        assert_eq!(state(&gone, "address "), "address a", "site {site}: {gone}");
        assert_eq!(rect(&one)[1..], [6, 1, 28], "site {site}: {one}");
        assert_eq!(rect(&two)[0] - rect(&one)[0], 14, "site {site}");

        let shot = png_rgba(&std::fs::read(&png).unwrap());
        let x0 = rect(&one)[0] as usize - 14;
        let dark = |x: usize| shot.2[(27 * shot.0 + x) * 4] < 100;

        assert!((x0..x0 + 28).all(dark) && !dark(x0 - 1) && !dark(x0 + 30), "site {site}: the underline of the composed text");
    }
}
