use super::*;

/// The window on `page` with `events` (after the load; `~` waits for the
/// print to end) and `DWELLER_PRINT_TO`; its `--state` and `--text`.
fn print_window(page: &str, events: &str, mode: &str, to: Option<&Path>, home: Option<&Path>) -> String {
    let in_renderer = mode != "single";
    let first = if in_renderer { "renderer:loaded frame frame frame" } else { "loaded frame frame frame" };
    let wait = if in_renderer { "renderer:input frame renderer:printed frame frame" } else { "frame frame" };
    let mut c = dweller_command();
    c.current_dir(root())
        .env("DWELLER_PROCESS", mode)
        .args([page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "1500", "--until-idle", "--state", "--text"])
        .args(["--events", &format!("{first} {}", events.replace('~', wait))]);
    if let Some(p) = to {
        c.env("DWELLER_PRINT_TO", p);
    }
    if let Some(h) = home {
        c.env("HOME", h);
    }
    let out = c.output().unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{mode} {events}: {}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

/// Q5.15: Cmd+P prints the shown page to a PDF, the same bytes as
/// `dweller print` gives, in one process, in a tab's renderer and in a
/// site's; the status line names the file; afterwards the page has its
/// screen styles and its window's width again.
#[test]
fn window_print_shortcut() {
    if !has_sdl() {
        return;
    }
    let cli = scratch().join("window_print_cli.pdf");
    let run = dweller_command().current_dir(root()).args(["print", "browser/tests/print/doc.html", "--font", "fixed", "-o"]).arg(&cli).output().unwrap();

    assert!(run.status.success(), "{}", text(&run.stderr));

    let want = std::fs::read(&cli).unwrap();

    for mode in ["single", "tab", "site"] {
        let to = scratch().join(format!("window_print_{mode}.pdf"));
        let _ = std::fs::remove_file(&to);
        let out = print_window("browser/tests/print/doc.html", "key:cmd+p ~", mode, Some(&to), None);

        assert!(out.contains(&format!("status Printed 5 pages to {}\n", to.display())), "{mode}: {out}");
        assert!(std::fs::read(&to).unwrap() == want, "{mode}: {} differs from {}", to.display(), cli.display());
        assert!(out.contains("before seen print=false screen=true\n"), "{mode}: {out}");
        assert!(out.contains("after seen print=false width=400\n"), "{mode}: {out}");
    }
}

/// Q5.15 (follow-up): Cmd+P on a page with an `@page` rule writes the
/// PDF `dweller print` gives (the rule's paper, not the window's
/// default), in one process and in a site's renderer.
#[test]
fn window_print_page_rule() {
    if !has_sdl() {
        return;
    }
    let cli = scratch().join("window_print_rule_cli.pdf");
    let run = dweller_command().current_dir(root()).args(["print", "browser/tests/print/page-rule.html", "--font", "fixed", "-o"]).arg(&cli).output().unwrap();

    assert!(run.status.success(), "{}", text(&run.stderr));

    let want = std::fs::read(&cli).unwrap();

    assert!(String::from_utf8_lossy(&want).contains("/MediaBox [0 0 432 288]"));

    for mode in ["single", "site"] {
        let to = scratch().join(format!("window_print_rule_{mode}.pdf"));
        let _ = std::fs::remove_file(&to);
        let out = print_window("browser/tests/print/page-rule.html", "key:cmd+p ~", mode, Some(&to), None);

        assert!(out.contains(&format!("status Printed 1 pages to {}\n", to.display())), "{mode}: {out}");
        assert!(std::fs::read(&to).unwrap() == want, "{mode}: {} differs from {}", to.display(), cli.display());
    }
}

/// Q5.15 (follow-up): Cmd+P with the system's fonts writes the vector
/// text `dweller print` writes, in one process and in a site's renderer
/// (sandboxed: the outlines come from the faces it opened before).
#[test]
fn window_print_vector_text() {
    if !has_sdl() {
        return;
    }
    let cli = scratch().join("window_print_vector_cli.pdf");
    let run = dweller_command().current_dir(root()).args(["print", "browser/tests/print/vector.html", "--font", "system", "-o"]).arg(&cli).output().unwrap();

    if text(&run.stderr).contains("no system font found") {
        println!("skipped: no system font");
        return;
    }
    assert!(run.status.success(), "{}", text(&run.stderr));

    let want = std::fs::read(&cli).unwrap();

    assert!(String::from_utf8_lossy(&want).contains("/Subtype /Type3"));

    for mode in ["single", "site"] {
        let to = scratch().join(format!("window_print_vector_{mode}.pdf"));
        let _ = std::fs::remove_file(&to);
        let events = if mode == "single" { "loaded frame frame frame key:cmd+p frame frame" } else { "renderer:loaded frame frame frame key:cmd+p renderer:input frame renderer:printed frame frame" };
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", mode)
            .env("DWELLER_PRINT_TO", &to)
            .args(["browser/tests/print/vector.html", "--font", "system", "--width", "400", "--height", "300", "--frames", "1500", "--until-idle", "--state", "--events", events])
            .output()
            .unwrap();

        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {}\n{}", out.status, text(&out.stderr));
        assert!(text(&out.stdout).contains(&format!("status Printed 1 pages to {}\n", to.display())), "{mode}: {}", text(&out.stdout));
        assert!(std::fs::read(&to).unwrap() == want, "{mode}: {} differs from {}", to.display(), cli.display());
    }
}

/// Q5.15: `window.print()` prints too (`onbeforeprint`, `onafterprint`);
/// without `DWELLER_PRINT_TO` the PDF is named after the title in the
/// downloads directory, and a second print does not replace the first.
#[test]
fn window_print_from_script() {
    if !has_sdl() {
        return;
    }
    for mode in ["single", "site"] {
        let home = scratch().join(format!("print_home_{mode}"));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(home.join("Downloads")).unwrap();

        let out = print_window("browser/tests/print/ask.html", "click:50,65 ~ click:50,65 ~", mode, None, Some(&home));
        let first = home.join("Downloads/Asked_ to_print.pdf");
        let second = home.join("Downloads/Asked_ to_print (2).pdf");

        assert!(out.contains(&format!("status Printed 1 pages to {}\n", second.display())), "{mode}: {out}");
        assert!(out.contains("printed once\n"), "{mode}: {out}");

        let pdf = std::fs::read(&first).unwrap();
        let text_layer = inflate(&pdf[pdf.windows(7).position(|w| w == b"stream\n").unwrap() + 9..]);

        assert!(String::from_utf8_lossy(&text_layer).contains("(asked by the page) Tj"), "{mode}");
        assert!(second.exists(), "{mode}");
    }
}
