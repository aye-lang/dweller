//! Opt-in page-load timing and immutable response archives.
use super::{dweller_command, has_sdl, root, scratch, server, text};

#[test]
fn live_frame_index_follows_mutations() {
    let page = "data:text/html,<p id=out></p><script>let r=[window.length];let a=document.createElement('iframe');a.setAttribute('name','a');document.body.appendChild(a);r.push(window.length);let b=document.createElement('iframe');b.setAttribute('name','b');document.body.appendChild(b);r.push(window.length);document.body.insertBefore(b,a);r.push(window.frames[0]===b.contentWindow,window.frames[1]===a.contentWindow);b.remove();r.push(window.length);a.remove();r.push(window.length);document.getElementById('out').textContent=r.join(',');</script>";
    let out = dweller_command()
        .current_dir(root())
        .args(["dump", "--text", "--js", page])
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(text(&out.stdout), "0,1,2,true,true,1,0\n");
}

#[test]
fn cached_sheets_follow_media_changes() {
    let page = "data:text/html,<style id=css>p{color:red}</style><p id=out></p><script>let p=document.getElementById('out');let css=document.getElementById('css');let r=[getComputedStyle(p).color];document.head.appendChild(document.createElement('style'));r.push(getComputedStyle(p).color);css.setAttribute('media','(max-width:1px)');r.push(getComputedStyle(p).color);p.textContent=r.join('|');</script>";
    let out = dweller_command()
        .current_dir(root())
        .args(["dump", "--text", "--js", page])
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert_eq!(
        text(&out.stdout),
        "rgb(255, 0, 0)|rgb(255, 0, 0)|rgb(0, 0, 0)\n"
    );
}

#[test]
fn script_layout_is_reused_until_a_change() {
    let page = "data:text/html,<div id=d>x</div><p id=out></p><script>let d=document.getElementById('d');let r=[d.offsetWidth,d.offsetHeight,d.getBoundingClientRect().width,d.clientWidth];d.style.width='10px';r.push(d.offsetWidth,d.getBoundingClientRect().width);document.getElementById('out').textContent=r.join(',');</script>";
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args(["dump", "--text", "--js", "--no-scroll", page])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout), "x\n\n784,20,784,784,10,10\n");
    let trace = text(&out.stderr);
    assert_eq!(
        trace.matches("DWELLER_TIMING\tscript_layout\t").count(),
        2,
        "{trace}"
    );
}

/// A script that writes and reads in turns (as frameworks do) forces a
/// layout per read, but the document is styled whole only once: each
/// later layout styles again what the change reaches. Changes to a tree
/// out of the document force none.
#[test]
fn interleaved_writes_and_reads_restyle_partially() {
    let page = "data:text/html,<style>.on{height:5px}</style><div id=l></div><p id=out></p><script>let l=document.getElementById('l');for(let i=0;i<20;i++)l.append(document.createElement('div'));let loose=document.createElement('div'),sum=0;for(const c of l.children){c.className='on';loose.append(document.createElement('i'));sum+=l.offsetHeight;}document.getElementById('out').textContent=sum;</script>";
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args(["dump", "--text", "--js", "--no-scroll", page])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert_eq!(text(&out.stdout), "1050\n");
    let trace = text(&out.stderr);
    assert_eq!(
        trace.matches("DWELLER_TIMING\tscript_layout\t").count(),
        20,
        "{trace}"
    );
    assert_eq!(
        trace.matches("DWELLER_TIMING\trestyle\t").count(),
        19,
        "{trace}"
    );
    assert!(
        trace.matches("DWELLER_TIMING\tcascade\t").count() <= 2,
        "{trace}"
    );
}

#[test]
fn timing_preserves_output() {
    let args = [
        "dump",
        "--md",
        "--js",
        "--no-scroll",
        "browser/tests/agent/noscript.html",
    ];
    let plain = dweller_command()
        .current_dir(root())
        .args(args)
        .output()
        .unwrap();
    let timed = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args(args)
        .output()
        .unwrap();
    assert!(
        plain.status.success() && plain.stderr.is_empty(),
        "{}",
        text(&plain.stderr)
    );
    assert!(timed.status.success(), "{}", text(&timed.stderr));
    assert_eq!(plain.stdout, timed.stdout);
    let trace = text(&timed.stderr);
    for phase in [
        "fetch",
        "decode",
        "html",
        "scripts",
        "css_parse",
        "cascade",
        "layout",
        "markdown",
    ] {
        assert!(
            trace.contains(&format!("DWELLER_TIMING\t{phase}\t")),
            "missing {phase}: {trace}"
        );
    }
    assert!(!trace.contains("leak"), "{trace}");
    assert_eq!(
        trace.matches("DWELLER_TIMING\tcascade\t").count(),
        1,
        "{trace}"
    );
}

#[test]
fn timing_preserves_paint() {
    let plain = scratch().join("perf-plain.png");
    let timed = scratch().join("perf-timed.png");
    let args = [
        "screenshot",
        "browser/tests/paint/text.html",
        "--font",
        "fixed",
        "-o",
    ];
    let first = dweller_command()
        .current_dir(root())
        .args(args)
        .arg(&plain)
        .output()
        .unwrap();
    let second = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args(args)
        .arg(&timed)
        .output()
        .unwrap();
    assert!(
        first.status.success() && first.stderr.is_empty(),
        "{}",
        text(&first.stderr)
    );
    assert!(second.status.success(), "{}", text(&second.stderr));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(std::fs::read(plain).unwrap(), std::fs::read(timed).unwrap());
    let trace = text(&second.stderr);
    for phase in ["paint_build", "paint"] {
        assert!(
            trace.contains(&format!("DWELLER_TIMING\t{phase}\t")),
            "{trace}"
        );
    }
    assert!(!trace.contains("leak"), "{trace}");
}

#[test]
fn response_archive_replays_without_server() {
    let Some(srv) = server() else { return };
    let dir = scratch().join("perf-archive");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // External CSS (including a redirect), JS, and an image. The server
    // serves /log/* with appropriate content types and counts requests.
    let page = "data:text/html,<link rel=stylesheet href='BASE/log/style.css'><script src='BASE/log/script.js'></script><p id=x>Archived</p><img src='BASE/log/image.png'><script>document.getElementById('x').textContent += fromScript</script>".replace("BASE", &srv.base);
    let args = ["dump", "--md", "--js", &page];
    let recorded = dweller_command()
        .current_dir(root())
        .env("DWELLER_RECORD", &dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        recorded.status.success() && recorded.stderr.is_empty(),
        "{}",
        text(&recorded.stderr)
    );
    // The stylesheet, script and image; not the data: page, which holds
    // its own response.
    assert!(std::fs::read_dir(&dir).unwrap().count() >= 3);
    let failure_url = "http://127.0.0.1:1/unavailable";
    let recorded_failure = dweller_command()
        .current_dir(root())
        .env("DWELLER_RECORD", &dir)
        .args(["dump", "--md", "--js", failure_url])
        .output()
        .unwrap();
    assert!(!recorded_failure.status.success());
    let replayed_failure = dweller_command()
        .current_dir(root())
        .env("DWELLER_REPLAY", &dir)
        .args(["dump", "--md", "--js", failure_url])
        .output()
        .unwrap();
    assert_eq!(recorded_failure.stdout, replayed_failure.stdout);
    assert_eq!(recorded_failure.stderr, replayed_failure.stderr);
    let redirect = format!("{}/redirect", srv.base);
    let recorded_redirect = dweller_command()
        .current_dir(root())
        .env("DWELLER_RECORD", &dir)
        .args(["dump", "--md", "--js", &redirect])
        .output()
        .unwrap();
    assert!(
        recorded_redirect.status.success() && recorded_redirect.stderr.is_empty(),
        "{}",
        text(&recorded_redirect.stderr)
    );
    // Replay must not reach the network, even with HTTP prefetch enabled.
    drop(srv);
    let replayed = dweller_command()
        .current_dir(root())
        .env("DWELLER_REPLAY", &dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        replayed.status.success() && replayed.stderr.is_empty(),
        "{}",
        text(&replayed.stderr)
    );
    assert_eq!(recorded.stdout, replayed.stdout);
    let replayed_redirect = dweller_command()
        .current_dir(root())
        .env("DWELLER_REPLAY", &dir)
        .args(["dump", "--md", "--js", &redirect])
        .output()
        .unwrap();
    assert!(
        replayed_redirect.status.success() && replayed_redirect.stderr.is_empty(),
        "{}",
        text(&replayed_redirect.stderr)
    );
    assert_eq!(recorded_redirect.stdout, replayed_redirect.stdout);
    let missing = dweller_command()
        .current_dir(root())
        .env("DWELLER_REPLAY", &dir)
        .args(["dump", "--md", "--js", "http://127.0.0.1:1/missing"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(
        text(&missing.stderr).contains("archive miss:"),
        "{}",
        text(&missing.stderr)
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A replayed POST whose body has no record (a beacon that an engine
/// change added or moved) gets the latest recorded POST response for its
/// URL and reports `archive fallback:`, not a miss (P1.12).
#[test]
fn archive_replays_unrecorded_post_bodies() {
    let Some(srv) = server() else { return };
    let dir = scratch().join("perf-archive-post");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let base = srv.base.clone();
    let page = |bodies: &[&str]| {
        let posts: String = bodies.iter().enumerate().map(|(i, b)| format!(
            "fetch('BASE/fetch-echo',{{method:'POST',body:'{b}'}}).then(r=>r.json()).then(j=>{{document.getElementById('p{i}').textContent='{b} got '+j.body}});"
        )).collect();
        let ps: String = (0..bodies.len())
            .map(|i| format!("<p id=p{i}></p>"))
            .collect();
        format!("data:text/html,{ps}<script>{posts}</script>").replace("BASE", &base)
    };
    let dump = |env: &str, page: &str| {
        dweller_command()
            .current_dir(root())
            .env(env, &dir)
            .args(["dump", "--md", "--js", page])
            .output()
            .unwrap()
    };
    // "one" is recorded before "two", so "two" is the latest.
    for body in ["one", "two"] {
        let recorded = dump("DWELLER_RECORD", &page(&[body]));
        assert!(
            recorded.status.success() && recorded.stderr.is_empty(),
            "{}",
            text(&recorded.stderr)
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    drop(srv);
    let replayed = dump("DWELLER_REPLAY", &page(&["one", "three", "four"]));
    let (out, err) = (text(&replayed.stdout), text(&replayed.stderr));
    assert!(
        replayed.status.success() && !err.contains("archive miss:"),
        "{err}"
    );
    assert_eq!(err.matches("archive fallback:").count(), 2, "{err}");
    assert!(
        out.contains("one got one")
            && out.contains("three got two")
            && out.contains("four got two"),
        "{out}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Q2.13b: a page next to a large linked sheet inserts 20 `style`
/// elements, each in a task of its own that reads a computed style, and
/// gives one of two `style` elements with the same text a rule
/// (`insertRule`). The large sheet is parsed once (one `css_parse`
/// record of its URL; it was one a rebuild), and the styles are those of
/// every sheet, the twin untouched, in one process and with
/// `DWELLER_PROCESS=site`.
#[test]
fn style_sheets_parsed_once_per_source() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = scratch().join("sheets-parsed-once");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let big: String = (0..3000)
        .map(|i| format!(".r{i} {{ color: rgb(1, 2, 3); margin: 1px 2px }}\n"))
        .collect::<String>()
        + "#big { color: rgb(0, 128, 0) }\n";
    std::fs::write(dir.join("big.css"), big).unwrap();
    let page = "<!doctype html><link rel=stylesheet href=big.css>\
        <style>#twin { color: rgb(255, 0, 0) }</style><style>#twin { color: rgb(255, 0, 0) }</style>\
        <p id=big>big</p><p id=twin>twin</p><p id=out>wait</p><script>\
        let n = 0, got = [];\
        function step() {\
          const s = document.createElement('style'); s.textContent = '.s' + n + ' { margin-left: ' + n + 'px }'; document.head.appendChild(s);\
          const p = document.createElement('p'); p.className = 's' + n; document.body.appendChild(p);\
          got.push(getComputedStyle(p).marginLeft);\
          if (n == 10) document.querySelectorAll('style')[0].sheet.insertRule('#twin { color: rgb(0, 0, 255) }', 1);\
          n++;\
          if (n < 20) { setTimeout(step, 16); return; }\
          const css = (id) => getComputedStyle(document.getElementById(id)).color;\
          document.getElementById('out').textContent = got.join(',') + ' ' + css('big') + ' ' + css('twin') + ' ' + document.querySelectorAll('style')[1].sheet.cssRules.length;\
          document.title = 'done';\
        }\
        setTimeout(step, 0);</script>";
    std::fs::write(dir.join("page.html"), page).unwrap();
    let want = (0..20).map(|i| format!("{i}px")).collect::<Vec<_>>().join(",")
        + " rgb(0, 128, 0) rgb(255, 0, 0) 1";
    for mode in ["single", "site"] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", mode)
            .env("DWELLER_TIMING", "1")
            .arg(dir.join("page.html"))
            .args(["--font", "fixed", "--width", "400", "--height", "300"])
            .args(["--frames", "400", "--until-idle", "--text", "--events", "renderer:title:done frame"])
            .output()
            .unwrap();
        let trace = text(&out.stderr);
        assert!(out.status.success(), "{mode}: {}\n{trace}", out.status);
        let got = text(&out.stdout);
        assert!(got.contains(&want), "{mode}: {got}");
        let parses = trace
            .lines()
            .filter(|l| l.contains("\tcss_parse\t") && l.ends_with("/sheets-parsed-once/big.css"))
            .count();
        assert_eq!(parses, 1, "{mode}: the large sheet was parsed {parses} times");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
