use super::*;

fn downloads_run(name: &str, envs: &[(&str, &str)]) -> Option<(String, PathBuf, String)> {
    let srv = server()?;
    if !has_sdl() {
        return None;
    }
    let dir = profile(&format!("{name}-files"));
    let prof = profile(&format!("{name}-profile"));
    let mut all = vec![("DWELLER_DOWNLOADS", dir.to_str().unwrap())];
    all.extend_from_slice(envs);
    let page = login_window_with(&format!("{}/net/download.html", srv.base), "frame", Some(&prof), &all);
    let list = login_window_with("about:downloads", "frame", Some(&prof), &all);
    Some((page, dir, list))
}

fn downloads_check(page: &str, dir: &Path, list: &str) {
    assert!(page.contains("still here: Downloads /net/download.html history 1"), "{page}");
    assert!(page.contains("\ndownloads 7\n"), "{page}");
    let mut names: Vec<String> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    assert_eq!(
        names,
        ["_.._escape.txt", "attach", "download_file.txt", "renamed.txt", "report (1).pdf", "report.pdf", "shown.txt"]
    );
    let read = |n: &str| std::fs::read_to_string(dir.join(n)).unwrap();
    assert_eq!(read("renamed.txt"), "the file of the download test\n");
    assert_eq!(read("download_file.txt"), "the file of the download test\n");
    assert_eq!(read("report.pdf"), "attached report.pdf");
    assert_eq!(read("report (1).pdf"), "attached report.pdf");
    assert_eq!(read("shown.txt"), "attached shown.txt");
    assert_eq!(read("_.._escape.txt"), "attached ../../escape.txt");
    for n in &names {
        assert!(list.contains(n.as_str()), "{n} in {list}");
    }
    assert!(list.contains("19 bytes") && list.contains("/attach?name=report.pdf"), "{list}");
}

/// Q5.6: a response with `Content-Disposition: attachment` and a link
/// with a `download` attribute (same origin) are saved under
/// `DWELLER_DOWNLOADS` with the name the header, then the attribute, then
/// the URL gives (made safe, made unique), the page that started them
/// stays, and `about:downloads` lists them from the profile in a later
/// run.
#[test]
fn downloads_saved_and_listed() {
    let Some((page, dir, list)) = downloads_run("downloads", &[]) else {
        println!("skipped: no test server or no SDL2");
        return;
    };
    downloads_check(&page, &dir, &list);
}

/// Q5.6 with a renderer process per tab and per site: the browser process
/// saves the file and keeps the list; the renderer keeps its document.
#[test]
fn downloads_saved_and_listed_in_renderers() {
    for mode in ["tab", "site"] {
        let Some((page, dir, list)) = downloads_run(&format!("downloads-{mode}"), &[("DWELLER_PROCESS", mode)]) else {
            println!("skipped: no test server or no SDL2");
            return;
        };
        downloads_check(&page, &dir, &list);
    }
}

/// Q5.6: a URL the user opens that answers with an attachment is saved,
/// and the tab keeps what it showed (here nothing: no history entry).
#[test]
fn downloads_typed_url() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    for mode in ["single", "tab", "site"] {
        let dir = profile(&format!("downloads-typed-{mode}"));
        let envs = [("DWELLER_DOWNLOADS", dir.to_str().unwrap()), ("DWELLER_PROCESS", mode)];
        let state = login_window_with(&format!("{}/attach?name=typed.bin", srv.base), "frame", None, &envs);
        assert!(state.contains("\ndownloads 1\n"), "{mode}: {state}");
        assert!(!state.contains("attached typed.bin"), "{mode}: {state}");
        assert_eq!(std::fs::read_to_string(dir.join("typed.bin")).unwrap(), "attached typed.bin", "{mode}");
    }
}
