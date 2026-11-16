use super::*;

const SITE: (&str, &str) = ("DWELLER_PROCESS", "site");

/// The window on `url` with `events` and a profile; each `~` waits for
/// the navigation the event before it started.
fn places_window(url: &str, events: &str, dir: &Path, search: &str, site: bool) -> String {
    let wait = if site { "frame renderer:loaded frame" } else { "frame loaded frame" };
    let events = format!("loaded frame {}", events.replace('~', wait));
    let envs: &[(&str, &str)] = if site { &[SITE, ("DWELLER_SEARCH", search)] } else { &[IN_PROCESS, ("DWELLER_SEARCH", search)] };
    login_window_with(url, &events, Some(dir), envs)
}

fn state_line<'a>(out: &'a str, key: &str) -> &'a str {
    out.lines().find(|l| l.starts_with(key)).unwrap_or("")
}

fn suggested(out: &str) -> Vec<&str> {
    out.lines().filter(|l| l.starts_with("suggest ")).collect()
}

/// Q5.13: what is typed in the address bar and is not a URL, a host or a
/// file goes to the search engine, form-encoded; a leading `?` forces a
/// search; `localhost` is a host.
#[test]
fn window_places_search() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let search = format!("{}/ui/search.html?q=%s", s.base);
    let page = format!("{}/ui/zoom.html", s.base);
    for site in [false, true] {
        let dir = profile(&format!("places_search_{site}"));
        let out = places_window(&page, "key:cmd+l text:two_words&more key:return", &dir, &search, site);
        assert_eq!(state_line(&out, "url "), format!("url {}/ui/search.html?q=two+words%26more", s.base), "site {site}: {out}");
        assert!(out.contains("search ?q=two+words%26more"), "site {site}: {out}");

        let forced = places_window(&page, "key:cmd+l text:?zoom.html key:return", &dir, &search, site);
        assert_eq!(state_line(&forced, "url "), format!("url {}/ui/search.html?q=zoom.html", s.base), "site {site}");

        let local = format!("localhost:{}/ui/zoom-next.html", s.base.rsplit(':').next().unwrap());
        let host = places_window(&page, &go_to(&local), &dir, &search, site);
        assert_eq!(state_line(&host, "url "), format!("url http://{local}"), "site {site}: {host}");
    }
}

/// Q5.13: the pages a person visited are kept in the profile and
/// suggested under the address bar as they type, the typed text first;
/// Down and Up choose a row, Enter goes to it, Escape closes the list and
/// leaves the bar, a click on a row goes; the next run suggests them too.
#[test]
fn window_places_suggest_history() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let search = format!("{}/ui/search.html?q=%s", s.base);
    let page = format!("{}/ui/zoom.html", s.base);
    let next = format!("{}/ui/zoom-next.html", s.base);
    for site in [false, true] {
        let dir = profile(&format!("places_suggest_{site}"));
        let visit = format!("{} key:cmd+l text:ZOOM frame", go_to(&next));
        let typed = places_window(&page, &visit, &dir, &search, site);

        assert_eq!(
            suggested(&typed),
            ["suggest > Search for ZOOM".to_string(), format!("suggest zoom next - {next}"), format!("suggest zoom - {page}")],
            "site {site}: {typed}"
        );
        assert_eq!(state_line(&typed, "address "), "address ZOOM", "site {site}");

        let chosen = places_window(&page, &format!("{visit} key:down key:down frame"), &dir, &search, site);

        assert_eq!(state_line(&chosen, "address "), format!("address {page}"), "site {site}");
        assert_eq!(suggested(&chosen)[2], format!("suggest > zoom - {page}"), "site {site}");

        let around = places_window(&page, &format!("{visit} key:up key:up key:up key:down frame"), &dir, &search, site);

        assert_eq!(state_line(&around, "address "), format!("address {next}"), "site {site}");

        let escaped = places_window(&page, &format!("{visit} key:down key:escape frame"), &dir, &search, site);

        assert_eq!(state_line(&escaped, "address "), format!("address {next}"), "site {site}");
        assert_eq!(state_line(&escaped, "focus "), "focus page", "site {site}");
        assert!(suggested(&escaped).is_empty(), "site {site}: {escaped}");

        let went = places_window(&next, "key:cmd+l text:zoom.h frame key:down key:return ~", &dir, &search, site);

        assert_eq!(state_line(&went, "url "), format!("url {page}"), "site {site}: {went}");
        assert!(suggested(&went).is_empty(), "site {site}");

        let narrowed = places_window(&next, "key:cmd+l text:zoom_next frame", &dir, &search, site);

        assert_eq!(suggested(&narrowed).len(), 2, "site {site}: {narrowed}");

        let clicked = places_window(&search.replace("%s", "x"), "key:cmd+l text:zoom-n frame click:200,76 ~", &dir, &search, site);

        assert_eq!(state_line(&clicked, "url "), format!("url {next}"), "site {site}: {clicked}");
        assert_eq!(state_line(&clicked, "focus "), "focus page", "site {site}");

        let history = std::fs::read_to_string(dir.join("history")).unwrap();
        let counts: Vec<String> = history.lines().skip(1).map(|l| {
            let f: Vec<&str> = l.splitn(4, ' ').collect();
            format!("{} {} {}", f[0], f[2], f[3])
        }).collect();

        assert!(history.starts_with("dweller-history 1\n"), "site {site}: {history}");
        assert!(counts.contains(&format!("5 {page} zoom")), "site {site}: {history}");
        assert!(counts.contains(&format!("7 {next} zoom next")), "site {site}: {history}");
    }
}

/// Q5.13: Cmd+D and the mark at the end of the address bar bookmark the
/// page and take the bookmark away; bookmarks are kept in the profile,
/// carry a mark among the suggestions, and are what the selected
/// (or empty) address bar lists with Down.
#[test]
fn window_places_bookmarks() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let search = format!("{}/ui/search.html?q=%s", s.base);
    let page = format!("{}/ui/zoom.html", s.base);
    let next = format!("{}/ui/zoom-next.html", s.base);
    for site in [false, true] {
        let dir = profile(&format!("places_bookmarks_{site}"));
        let marked = places_window(&page, &format!("key:cmd+d frame {}", go_to(&next)), &dir, &search, site);

        assert!(!marked.contains("\nbookmarked\n"), "site {site}: {marked}");
        assert_eq!(std::fs::read_to_string(dir.join("bookmarks")).unwrap(), format!("dweller-bookmarks 1\n{page} zoom\n"), "site {site}");

        let listed = places_window(&next, "key:cmd+l key:down frame", &dir, &search, site);

        assert_eq!(suggested(&listed), [format!("suggest > * zoom - {page}")], "site {site}: {listed}");
        assert_eq!(state_line(&listed, "address "), format!("address {page}"), "site {site}");

        let first = places_window(&next, "key:cmd+l text:zoom frame", &dir, &search, site);

        assert!(suggested(&first).contains(&format!("suggest * zoom - {page}").as_str()), "site {site}: {first}");

        let back = places_window(&next, "key:cmd+l key:down key:return ~", &dir, &search, site);

        assert_eq!(state_line(&back, "url "), format!("url {page}"), "site {site}: {back}");
        assert!(back.contains("\nbookmarked\n"), "site {site}: {back}");

        let png = scratch().join(format!("places_star_{site}.png"));
        let shot = |events: &str| {
            let out = dweller_command()
                .current_dir(root())
                .env("DWELLER_PROFILE", &dir)
                .env("DWELLER_PROCESS", if site { "site" } else { "single" })
                .args([&page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle", "--state", "--save"])
                .arg(&png)
                .args(["--events", events])
                .output()
                .unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{}", text(&out.stderr));
            (text(&out.stdout), png_rgba(&std::fs::read(&png).unwrap()))
        };
        let wait = if site { "renderer:loaded frame" } else { "loaded frame" };
        let (state, filled) = shot(&format!("{wait} frame"));

        assert!(state.contains("\nbookmarked\n"), "site {site}: {state}");

        let (state, hollow) = shot(&format!("{wait} click:380,20 frame"));

        assert!(!state.contains("\nbookmarked\n"), "site {site}: {state}");
        assert_eq!(std::fs::read_to_string(dir.join("bookmarks")).unwrap(), "dweller-bookmarks 1\n", "site {site}");

        let dark = |p: &(usize, usize, Vec<u8>)| (370..392).flat_map(|x| (8..32).map(move |y| (y * p.0 + x) * 4)).filter(|&at| p.2[at] < 100).count();

        assert!(dark(&filled) > dark(&hollow) + 20 && dark(&hollow) > 10, "site {site}: {} and {} dark pixels", dark(&filled), dark(&hollow));

        let (_, list) = shot(&format!("{wait} key:cmd+l text:zoom frame"));
        let paper = (110..390).filter(|&x| list.2[(70 * list.0 + x) * 4..][..3] == [255, 255, 255]).count();

        assert!(paper > 100 && dark(&list) > 10, "site {site}: the list is not drawn under the address bar");
    }
}
