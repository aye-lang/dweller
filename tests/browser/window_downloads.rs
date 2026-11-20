use super::*;

const SITE: (&str, &str) = ("DWELLER_PROCESS", "site");

fn downloads_window(url: &str, events: &str, files: &Path, site: bool) -> String {
    let mode = if site { SITE } else { IN_PROCESS };
    login_window_with(url, events, None, &[mode, ("DWELLER_DOWNLOADS", files.to_str().unwrap())])
}

fn line<'a>(out: &'a str, key: &str) -> &'a str {
    out.lines().find(|l| l.starts_with(key)).unwrap_or("")
}

/// Q5.6 (slice 6): after a download the toolbar has a downloads button
/// and its list opens: a click on the button closes and opens it, Escape
/// closes it, a row opens the file, the last row and Cmd+Shift+J open
/// `about:downloads`.
#[test]
fn window_downloads_button_and_list() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let page = format!("{}/ui/downloads.html", s.base);
    let got = "click:50,60 downloaded frame";
    for site in [false, true] {
        let start = "loaded frame";
        let wait = if site { "frame renderer:loaded frame" } else { "frame loaded frame" };
        let dir = profile(&format!("downloads_button_{site}"));
        let run = |more: &str| downloads_window(&page, &format!("{start} {got} {more}"), &dir, site);
        let open = run("");

        assert!(open.contains("\ndownloads 1\ndownloads-list\n"), "site {site}: {open}");
        assert_eq!(line(&open, "url "), format!("url {page}"), "site {site}");
        assert!(!run("click:378,20 frame").contains("downloads-list"), "site {site}");
        let twice = run("click:378,20 frame click:378,20 frame");

        assert!(twice.contains("downloads-list"), "site {site}: {twice}");
        assert!(!run("key:escape frame").contains("downloads-list"), "site {site}");
        assert!(!run("click:60,200 frame").contains("downloads-list"), "site {site}");

        let file = run(&format!("click:200,50 {wait}"));

        assert_eq!(line(&file, "url "), format!("url file://{}/notes%20(5).txt", dir.display()), "site {site}: {file}");
        assert!(file.contains("attached notes.txt") && !file.contains("downloads-list"), "site {site}: {file}");

        let all = run(&format!("click:200,76 {wait}"));

        assert_eq!(line(&all, "url "), "url about:downloads", "site {site}: {all}");
        assert!(all.contains("notes (6).txt") && all.contains("18 bytes"), "site {site}: {all}");

        let keyed = run(&format!("key:escape key:cmd+shift+j {wait}"));

        assert_eq!(line(&keyed, "url "), "url about:downloads", "site {site}: {keyed}");

        let none = downloads_window(&page, &format!("{start} click:378,20 frame frame"), &dir, site);

        assert!(!none.contains("downloads-list") && !none.contains("\ndownloads ") && none.contains("\nbookmarked\n"), "site {site}: {none}");

        let png = scratch().join(format!("downloads_list_{site}.png"));
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_DOWNLOADS", &dir)
            .env("DWELLER_PROCESS", if site { "site" } else { "single" })
            .args([&page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle", "--save"])
            .arg(&png)
            .args(["--events", &format!("{} {got}", if site { "renderer:loaded frame" } else { start })])
            .output()
            .unwrap();

        assert!(out.status.success() && out.stderr.is_empty(), "{}", text(&out.stderr));

        let shot = png_rgba(&std::fs::read(&png).unwrap());
        let at = |x: usize, y: usize| -> [u8; 3] { shot.2[(y * shot.0 + x) * 4..(y * shot.0 + x) * 4 + 3].try_into().unwrap() };
        let ink = |y: usize| (120..384).filter(|&x| at(x, y)[0] < 120).count();

        assert_eq!(at(378, 15), [40, 40, 40], "site {site}: the arrow of the button");
        assert_eq!(at(378, 27), [40, 40, 40], "site {site}: the tray of the button");
        assert_eq!(at(366, 8), [232, 232, 232], "site {site}: the button is pressed while the list shows");
        assert_eq!(at(359, 20), [200, 200, 200], "site {site}: the address bar ends before the button");
        assert_eq!(at(112, 50), [200, 200, 200], "site {site}: the list's left edge");
        assert_eq!(at(391, 50), [200, 200, 200], "site {site}: the list's right edge under the button's");
        assert_eq!(at(250, 40), [255, 255, 255], "site {site}: the list's paper");
        assert!((45..58).map(ink).sum::<usize>() > 40, "site {site}: no file name and size in the first row");
        assert!((71..84).map(ink).sum::<usize>() > 40, "site {site}: no Show all downloads row");
    }
}
