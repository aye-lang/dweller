use super::*;

/// Three files to choose: text, every byte value four times, and a name
/// with a space. Gives their directory.
fn picked_files(name: &str) -> PathBuf {
    let dir = scratch().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.txt"), "alpha\nbeta\n").unwrap();
    std::fs::write(dir.join("b.bin"), (0..1024u32).map(|i| i as u8).collect::<Vec<u8>>()).unwrap();
    std::fs::write(dir.join("c d.png"), b"\x89PNG fake").unwrap();
    dir
}

/// A `DWELLER_PICKER` command: it notes how it was asked (`$1`, `$2`) in
/// `asked`, and answers a `multiple` request with the three files and a
/// single one with two (the input must take the first only).
fn picker(dir: &Path) -> String {
    let d = dir.display();
    format!("echo \"$1 $2\" >> '{d}/asked'; if [ \"$1\" = multiple ]; then printf '%s\\n' '{d}/a.txt' '{d}/b.bin' '{d}/c d.png'; else printf '%s\\n' '{d}/a.txt' '{d}/b.bin'; fi")
}

/// The window on `url` with `events` and the picker command; `~N` waits
/// until the page finished its N-th step (its title ends in `doneN`).
fn file_window(url: &str, events: &str, picker: &str, site: bool, png: Option<&Path>) -> String {
    let mut words = Vec::new();
    for w in events.split(' ') {
        match w.strip_prefix('~') {
            Some(n) if site => words.push(format!("renderer:input picker renderer:title:done{n} frame")),
            Some(_) => words.push("frame picker frame frame frame".to_string()),
            None => words.push(w.to_string()),
        }
    }
    let first = if site { "renderer:loaded frame" } else { "loaded frame" };
    let mut c = dweller_command();
    c.current_dir(root())
        .env("DWELLER_PICKER", picker)
        .env("DWELLER_PROCESS", if site { "site" } else { "single" })
        .args([url, "--font", "fixed", "--width", "400", "--height", "400", "--frames", "1500", "--until-idle", "--state", "--text"])
        .args(["--events", &format!("{first} {}", words.join(" "))]);
    if let Some(p) = png {
        c.arg("--save").arg(p);
    }
    let out = c.output().unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{events}: {}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

/// The lines the page logged, from `first` on.
fn logged(out: &str, first: &str) -> Vec<String> {
    out.lines().skip_while(|l| *l != first).take_while(|l| !l.starts_with("var ")).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

const ONE: [&str; 4] = [
    "input one",
    "change one",
    "one value=C:\\fakepath\\a.txt files=1 same=true list=true",
    "a.txt | text/plain | 11 | sum 950 | true | \"alpha\\nbeta\\n\" | true",
];

const MANY: [&str; 6] = [
    "input many",
    "change many",
    "many value=C:\\fakepath\\a.txt files=3 same=true list=true",
    "a.txt | text/plain | 11 | sum 950 | true | \"alpha\\nbeta\\n\" | true",
    "b.bin | application/octet-stream | 1024 | sum 130560 | true | \"\\u0000\\u0001\\u0002\\u0003\\u0004\\u0005\\u0006\\u0007\\b\\t\\n\\u000b\" | true",
    "c d.png | image/png | 9 | sum 805 | true | \"\u{fffd}PNG fake\" | true",
];

/// Q5.5: a click on a file input opens the picker with the input's
/// `multiple` and `accept`; the chosen files are the input's `files`
/// (`File`s with the name, the type of the extension and the bytes),
/// its `value` is the fake path, `input` and `change` fire; an input
/// without `multiple` takes one file; a picker that chooses nothing
/// fires `cancel` and keeps the files; `value = ''` empties them.
#[test]
fn window_file_pick() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let page = format!("{}/ui/file.html", s.base);
    for site in [false, true] {
        let dir = picked_files(&format!("file_pick_{site}"));
        let out = file_window(&page, "click:50,62 ~1 click:50,102 ~2 click:250,142 ~3", &picker(&dir), site, None);
        let mut want: Vec<&str> = ONE.to_vec();

        want.extend(MANY);
        want.push("one value= files=0 same=true list=true");
        assert_eq!(logged(&out, "input one"), want, "site {site}: {out}");
        assert_eq!(std::fs::read_to_string(dir.join("asked")).unwrap(), "single .txt,image/*\nmultiple \n", "site {site}");

        let none = file_window(&page, "click:50,62 ~1", "true", site, None);

        assert_eq!(logged(&none, "cancel one files=0"), ["cancel one files=0"], "site {site}: {none}");

        let gone = file_window(&page, "click:50,62 ~1", &format!("echo '{}/missing.txt'", dir.display()), site, None);

        assert_eq!(logged(&gone, "cancel one files=0"), ["cancel one files=0"], "site {site}: {gone}");
    }
}

/// Q5.5: the control shows the chosen file's name (or the count) where
/// it said "No file chosen".
#[test]
fn window_file_label() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let page = format!("{}/ui/file.html", s.base);
    let dir = picked_files("file_label");
    let mut shots = Vec::new();
    for (name, events, site) in [("none", "frame", false), ("one", "click:50,62 ~1", false), ("many", "click:50,102 ~1", false), ("one_site", "click:50,62 ~1", true), ("many_site", "click:50,102 ~1", true)] {
        let png = scratch().join(format!("file_label_{name}.png"));

        file_window(&page, events, &picker(&dir), site, Some(&png));
        shots.push(png_rgba(&std::fs::read(&png).unwrap()));
    }
    let control = |shot: &(usize, usize, Vec<u8>), top: usize| -> Vec<u8> {
        (top..top + 24).flat_map(|y| shot.2[(y * shot.0 + 10) * 4..(y * shot.0 + 190) * 4].to_vec()).collect()
    };

    assert!(control(&shots[0], 50) != control(&shots[1], 50), "the first control did not change");
    assert!(control(&shots[0], 90) == control(&shots[1], 90), "the second control changed");
    assert!(control(&shots[0], 90) != control(&shots[2], 90), "the second control did not change");
    assert!(control(&shots[1], 50) != control(&shots[2], 90), "a name and a count look the same");
    assert!(shots[1] == shots[3] && shots[2] == shots[4], "the renderer draws another label than one process");
}

/// Q5.5: a form with chosen files submits them as multipart/form-data
/// parts with their names, types and bytes (the server's SHA-256 of each
/// part), an input without files as one empty part; `fetch` of a
/// `FormData` of the form sends the same parts.
#[test]
fn window_file_upload() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let page = format!("{}/ui/file.html", s.base);
    let parts = [
        "part note | - | - | 5 | 2cf24dba5fb0a30e",
        "part one | a.txt | text/plain | 11 | e49c81e2d2f84e25",
        "part many | a.txt | text/plain | 11 | e49c81e2d2f84e25",
        "part many | b.bin | application/octet-stream | 1024 | 785b0751fc2c53dc",
        "part many | c d.png | image/png | 9 | 075f8e3f22fcb592",
    ];
    for site in [false, true] {
        let dir = picked_files(&format!("file_upload_{site}"));
        let fetched = file_window(&page, "click:50,62 ~1 click:50,102 ~2 click:150,142 ~3", &picker(&dir), site, None);
        let mut want = vec!["fetched", "<!doctype html><title>done9</title><pre>upload multipart/form-data"];

        want.extend(parts);
        want.push("part extra | made.txt | text/plain | 5 | fade95397337ba0a</pre>");
        assert_eq!(logged(&fetched, "fetched"), want, "site {site}: {fetched}");

        let posted = file_window(&page, "click:50,102 ~1 click:50,142 ~9", &picker(&dir), site, None);
        let mut want = vec![parts[0], "part one |  | application/octet-stream | 0 | e3b0c44298fc1c14"];

        want.extend(&parts[2..]);
        assert!(posted.contains(&format!("url {}/upload\n", s.base)), "site {site}: {posted}");
        assert!(posted.contains("upload multipart/form-data\n"), "site {site}: {posted}");
        assert_eq!(logged(&posted, parts[0]), want, "site {site}: {posted}");
    }
}

/// Q5.5: a file input in a cross-site frame (its own renderer) gets the
/// picker and the files too: the frame read 1044 bytes of three files.
#[test]
fn window_file_in_cross_site_frame() {
    if !has_sdl() {
        return;
    }
    let Some(s) = server() else {
        return;
    };
    let dir = picked_files("file_frame");
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PICKER", picker(&dir))
        .env("DWELLER_PROCESS", "site")
        .arg(format!("{}/ui/file-frame.html", s.base))
        .args(["--font", "fixed", "--width", "400", "--height", "400", "--frames", "1500", "--until-idle", "--state"])
        .args(["--events", "renderer:loaded renderer:child frame click:50,102 renderer:input renderer:child-title:done1 frame"])
        .output()
        .unwrap();
    let state = text(&out.stdout);
    let child = state.lines().find(|l| l.starts_with("renderer-child ")).unwrap_or("");

    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    assert!(child.ends_with(" title 1044 done1"), "{state}");
}
