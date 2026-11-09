use super::*;

#[test]
fn http() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let basic = std::fs::read_to_string(root().join("browser/tests/basic.txt")).unwrap();
    // A page over HTTP reads as from a file, also after a redirect.
    expect(
        &["dump", "--text", &format!("{base}/basic.html")],
        0,
        &basic,
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/redirect")],
        0,
        &basic,
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/agent")],
        0,
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Dweller/0.1\n",
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/gzip")],
        0,
        &format!("{}\n", ["compressed"; 50].join(" ")),
        "",
    );
    // Style sheets linked from a page over HTTP are fetched relative to
    // the URL after redirects.
    let link_style =
        std::fs::read_to_string(root().join("browser/tests/style/link.style")).unwrap();
    expect(
        &["dump", "--style", &format!("{base}/redirect-style")],
        0,
        &link_style,
        "",
    );
    // The charset of the Content-Type header decodes the body.
    expect(
        &["dump", "--text", &format!("{base}/latin1")],
        0,
        "Café à la crème\n",
        "",
    );
    expect(
        &["dump", "--text", &format!("{base}/gbk")],
        0,
        "中文网页\n",
        "",
    );
    // An error status still has a page.
    let out = run(&["dump", "--text", &format!("{base}/missing.html")]);
    assert_eq!(out.status.code(), Some(0));
    assert!(text(&out.stdout).contains("404"), "{}", text(&out.stdout));
    expect(
        &["dump", "--text", &format!("{base}/redirect-loop")],
        1,
        "",
        &format!("error: cannot fetch {base}/redirect-loop: too many redirects\n"),
    );
    // Nothing listens on port 1.
    expect(
        &["dump", "--text", "http://127.0.0.1:1/"],
        1,
        "",
        "error: cannot fetch http://127.0.0.1:1/: cannot connect\n",
    );
}

// ---- B8: net

/// Request headers, cookies kept in a file, size limits, TLS errors and
/// `<meta charset>` over HTTP. (Cookies, the Referer and the cache within
/// one session: `browser/tests/agent/http/net_cookies.jsonl`.)
#[test]
fn net_http() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    // A URL the user typed: no Referer, no cookies yet.
    let headers = "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8\nAccept-Language: en-US,en;q=0.9\nReferer: \nCookie: \n";
    expect(
        &["dump", "--text", &format!("{base}/headers")],
        0,
        headers,
        "",
    );
    // DWELLER_COOKIE_FILE keeps the persistent cookies (p, Max-Age)
    // from one run to the next; session cookies (a, d) end with the run.
    let file = scratch().join("cookies.txt");
    let _ = std::fs::remove_file(&file);
    let with_file = |url: String| {
        dweller_command()
            .current_dir(root())
            .env("DWELLER_COOKIE_FILE", &file)
            .args(["dump", "--text", &url])
            .output()
            .unwrap()
    };
    let first = with_file(format!("{base}/net/start"));
    assert!(
        first.status.success() && first.stderr.is_empty(),
        "{}",
        text(&first.stderr)
    );
    let saved = std::fs::read_to_string(&file).unwrap();
    assert!(
        saved.starts_with("127.0.0.1\t1\t/\t0\t0\t")
            && saved.ends_with("\tp\t5\n")
            && saved.lines().count() == 1,
        "{saved}"
    );
    let second = with_file(format!("{base}/headers"));
    assert!(
        text(&second.stdout).contains("\nCookie: p=5\n"),
        "{}",
        text(&second.stdout)
    );
    // Too large: 33 MiB, with and without a Content-Length.
    for big in ["big", "big?stream"] {
        expect(
            &["dump", "--text", &format!("{base}/{big}")],
            1,
            "",
            &format!("error: cannot fetch {base}/{big}: the response is too large\n"),
        );
    }
    // TLS to a server that speaks plain HTTP.
    let https = base.replace("http://", "https://");
    expect(
        &["dump", "--text", &format!("{https}/")],
        1,
        "",
        &format!("error: cannot fetch {https}/: the server does not support TLS 1.2 or later\n"),
    );
    // No charset in the Content-Type: the <meta charset> decides.
    expect(
        &["dump", "--text", &format!("{base}/meta-charset")],
        0,
        "\u{201c}quoted\u{201d} caf\u{e9}\n",
        "",
    );
}

#[test]
fn tls_errors() {
    use std::io::{BufRead, Write};
    let tls = root().join("browser/tests/tls");
    let Ok(mut child) = Command::new("python3")
        .arg("-I")
        .arg(root().join("browser/tests/tls_server.py"))
        .arg(&tls)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let _srv = Server {
        child,
        base: String::new(),
    };
    let ports: Vec<u16> = line
        .split_whitespace()
        .map(|p| p.parse().unwrap())
        .collect();
    assert_eq!(ports.len(), 6, "{line}");
    let with_ca = |args: &[&str]| {
        dweller_command()
            .current_dir(root())
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_CA", tls.join("ca.pem"))
            .args(args)
            .output()
            .unwrap()
    };
    let at = |port: u16| format!("https://127.0.0.1:{port}/");
    let valid = at(ports[0]);
    let untrusted = "the certificate is self-signed or from an unknown issuer";

    let out = with_ca(&["dump", "--text", &valid]);
    assert_eq!(text(&out.stdout), "secure page\n", "{}", text(&out.stderr));
    expect(
        &["dump", "--text", &valid],
        1,
        "",
        &format!("error: cannot fetch {valid}: {untrusted}\n"),
    );
    let out = dweller_command()
        .current_dir(root())
        .env_remove("DWELLER_TEST")
        .env("DWELLER_TEST_CA", tls.join("ca.pem"))
        .args(["dump", "--text", &valid])
        .output()
        .unwrap();
    assert_eq!(
        text(&out.stderr),
        format!("error: cannot fetch {valid}: {untrusted}\n"),
        "the test CA is trusted only with DWELLER_TEST=1"
    );
    let out = with_ca(&[
        "dump",
        "--text",
        "--js",
        &format!("{valid}fetch.html?{valid}"),
    ]);
    assert_eq!(
        text(&out.stdout),
        "fetch: status 200\nxhr: load status 200\n",
        "{}",
        text(&out.stderr)
    );

    let mut cases = vec![
        (ports[1], "the certificate has expired"),
        (ports[2], untrusted),
        (ports[3], "the certificate is for another host"),
    ];
    if ports[4] == 0 {
        println!("skipped TLS 1.1: this platform's OpenSSL does not offer it");
    } else {
        cases.push((ports[4], "the server does not support TLS 1.2 or later"));
    }
    for (port, why) in cases {
        let bad = at(port);
        let out = with_ca(&["dump", "--text", &bad]);
        assert_eq!(out.status.code(), Some(1), "{bad}");
        assert_eq!(
            text(&out.stderr),
            format!("error: cannot fetch {bad}: {why}\n")
        );

        let out = with_ca(&[
            "dump",
            "--text",
            "--js",
            &format!("{valid}fetch.html?{bad}"),
        ]);
        assert_eq!(
            text(&out.stdout),
            "fetch: TypeError\nxhr: error status 0\n",
            "{bad}: {}",
            text(&out.stderr)
        );

        if has_sdl() {
            let out = with_ca(&[&bad, "--frames", "1", "--text"]);
            let shown = text(&out.stdout);
            assert!(
                shown.contains("Your connection is not private")
                    && shown.contains("127.0.0.1")
                    && shown.contains(why),
                "{bad}: {shown}\n{}",
                text(&out.stderr)
            );
        }

        let mut agent = dweller_command()
            .current_dir(root())
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_CA", tls.join("ca.pem"))
            .arg("agent")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        agent
            .stdin
            .take()
            .unwrap()
            .write_all(format!("{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"{bad}\"}}\n").as_bytes())
            .unwrap();
        let reply = text(&agent.wait_with_output().unwrap().stdout);
        assert!(
            reply.contains("\"ok\":false") && reply.contains(&format!("cannot fetch {bad}: {why}")),
            "{reply}"
        );
    }
}

/// Q3.4b: the window's certificate bar (browser chrome, not the page)
/// offers "Advanced", then "Proceed to HOST (unsafe)". Proceeding past
/// the self-signed server loads its page, whose script fetches the same
/// host, and a reload does not ask again; another process with another
/// profile asks again. A page script that fetches the bad host gets no
/// bar. An HSTS host with a bad certificate has no bar: clicks where
/// the button would be change nothing. Q3.4c: Tab and Enter reach the
/// bar; the exception is for the accepted key (the switching server
/// changes its certificate after `/switch`); an HSTS header over a
/// bypassed connection is not stored.
#[test]
fn cert_bypass() {
    use std::io::BufRead;
    if !has_sdl() {
        println!("skipped: no SDL");
        return;
    }
    let tls = root().join("browser/tests/tls");
    let Ok(mut child) = Command::new("python3")
        .arg("-I")
        .arg(root().join("browser/tests/tls_server.py"))
        .arg(&tls)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let _srv = Server {
        child,
        base: String::new(),
    };
    let ports: Vec<u16> = line
        .split_whitespace()
        .map(|p| p.parse().unwrap())
        .collect();
    let bad = format!("https://127.0.0.1:{}/", ports[2]);
    let valid = format!("https://127.0.0.1:{}/", ports[0]);
    let window = |profile: &Path, page: &str, events: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_CA", tls.join("ca.pem"))
            .env("DWELLER_PROFILE", profile)
            .args([page, "--width", "800", "--height", "600", "--frames", "300", "--until-idle", "--state", "--text", "--events", events])
            .output()
            .unwrap();
        (text(&out.stdout), text(&out.stderr))
    };
    let first = scratch().join("cert-bypass-1");
    let second = scratch().join("cert-bypass-2");
    for dir in [&first, &second] {
        let _ = std::fs::remove_dir_all(dir);
        std::fs::create_dir_all(dir).unwrap();
    }
    let button = "click:780,56";
    let page = format!("{bad}fetch.html?{bad}");

    let (shown, err) = window(&first, &page, "frame");
    assert!(
        shown.contains("Your connection is not private") && shown.contains("cert Advanced\n"),
        "{shown}\n{err}"
    );
    let (shown, err) = window(&first, &page, &format!("frame {button} frame"));
    assert!(shown.contains("cert Proceed to 127.0.0.1 (unsafe)\n"), "{shown}\n{err}");

    let (shown, err) = window(&first, &page, &format!("frame {button} {button} loaded key:cmd+r loaded"));
    assert!(
        shown.contains("fetch: status 200\nxhr: load status 200\n") && !shown.contains("cert ") && !shown.contains("not private"),
        "{shown}\n{err}"
    );

    let (shown, err) = window(&first, &page, "frame key:tab key:return frame");
    assert!(shown.contains("cert Proceed to 127.0.0.1 (unsafe)\n"), "Tab and Enter reach the bar: {shown}\n{err}");
    let (shown, err) = window(&first, &page, "frame key:tab key:return key:return loaded");
    assert!(shown.contains("fetch: status 200\n") && !shown.contains("cert "), "{shown}\n{err}");

    let switching = format!("https://127.0.0.1:{}/", ports[5]);
    let switched = format!("frame {button} {button} loaded key:cmd+l text:{switching}switch key:return loaded key:cmd+r loaded");
    let (shown, err) = window(&first, &switching, &switched);
    assert!(
        shown.contains("Your connection is not private") && shown.contains("cert Advanced\n"),
        "an exception is for the key that was accepted: {shown}\n{err}"
    );
    let (shown, err) = window(&first, &switching, &format!("{switched} {button} {button} loaded"));
    assert!(
        shown.contains("secure page") && !shown.contains("cert "),
        "proceeding past the other key too: {shown}\n{err}"
    );

    for (port, why) in [(ports[1], "has expired"), (ports[3], "is for another host")] {
        let other = format!("https://127.0.0.1:{port}/");
        let events = format!("frame {button} {button} loaded key:cmd+l text:{other} key:return loaded");
        let (shown, err) = window(&first, &page, &events);
        assert!(
            shown.contains("Your connection is not private") && shown.contains(why) && shown.contains("cert Advanced\n"),
            "an exception is for one kind of error: {shown}\n{err}"
        );
    }

    let (shown, err) = window(&second, &page, "frame");
    assert!(shown.contains("cert Advanced\n"), "a new profile asks again: {shown}\n{err}");

    let (shown, err) = window(&second, &format!("{valid}fetch.html?{bad}"), &format!("frame {button} {button} frame"));
    assert!(
        shown.contains("fetch: TypeError\nxhr: error status 0\n") && !shown.contains("cert "),
        "a page's fetch gets no bypass: {shown}\n{err}"
    );

    let Some(site) = HstsSite::start() else {
        println!("skipped the HSTS case: cannot start python3 for the test server");
        return;
    };
    let hsts = scratch().join("cert-bypass-hsts");
    let _ = std::fs::remove_dir_all(&hsts);
    std::fs::create_dir_all(&hsts).unwrap();
    let site_url = format!("https://site.test:{}/", site.tls);
    let out = site.run(Some(&hsts), &["dump", "--text", &format!("{site_url}hsts")]);
    assert_eq!(text(&out.stdout), "https page\n", "{}", text(&out.stderr));
    let untrusted_site = |profile: &Path, page: &str, events: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_CA", tls.join("ca.pem"))
            .env("DWELLER_RESOLVE", format!("site.test:{}:127.0.0.1", site.tls))
            .env("DWELLER_PROFILE", profile)
            .args([page, "--width", "800", "--height", "600", "--frames", "300", "--until-idle", "--state", "--text", "--events", events])
            .output()
            .unwrap();
        text(&out.stdout)
    };
    let proceed = format!("frame {button} {button} loaded");
    let shown = untrusted_site(&hsts, &site_url, &proceed);
    assert!(
        shown.contains("Your connection is not private") && !shown.contains("cert "),
        "no bypass on an HSTS host: {shown}"
    );
    let fresh = scratch().join("cert-bypass-fresh");
    let _ = std::fs::remove_dir_all(&fresh);
    std::fs::create_dir_all(&fresh).unwrap();
    let shown = untrusted_site(&fresh, &format!("{site_url}hsts"), &proceed);
    assert!(
        shown.contains("https page") && !shown.contains("not private"),
        "the same clicks proceed on the host before its HSTS header: {shown}"
    );
    let saved = std::fs::read_to_string(fresh.join("hsts")).unwrap_or_default();
    assert!(!saved.contains("site.test"), "no HSTS from a bypassed connection: {saved}");
    let insecure = format!("http://site.test:{}/", site.tls);
    let shown = untrusted_site(&fresh, &format!("{site_url}hsts"), &format!("{proceed} key:cmd+l text:{insecure} key:return loaded"));
    assert!(
        shown.contains("Cannot load this page") && !shown.contains("not private"),
        "http to the host is not upgraded after a bypassed HSTS header: {shown}"
    );
}

/// `browser/tests/hsts_server.py`: one TLS and one plain port on
/// 127.0.0.1 with `site.test` (and `sub.site.test`) mapped to both
/// through `DWELLER_RESOLVE`; runs `dweller args...` in `profile` when
/// given.
struct HstsSite {
    _srv: Server,
    tls: u16,
    plain: u16,
}

impl HstsSite {
    fn start() -> Option<HstsSite> {
        use std::io::BufRead;
        let mut child = Command::new("python3")
            .arg("-I")
            .arg(root().join("browser/tests/hsts_server.py"))
            .arg(root().join("browser/tests/hsts"))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .ok()?;
        let mut line = String::new();
        std::io::BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        let ports: Vec<u16> = line
            .split_whitespace()
            .map(|p| p.parse().unwrap())
            .collect();
        Some(HstsSite {
            _srv: Server {
                child,
                base: String::new(),
            },
            tls: ports[0],
            plain: ports[1],
        })
    }

    fn run(&self, profile: Option<&Path>, args: &[&str]) -> Output {
        let (t, p) = (self.tls, self.plain);
        let mut c = dweller_command();
        c.current_dir(root())
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_CA", root().join("browser/tests/hsts/ca.pem"))
            .env(
                "DWELLER_RESOLVE",
                format!("site.test:{t}:127.0.0.1,site.test:{p}:127.0.0.1,sub.site.test:{t}:127.0.0.1"),
            )
            .args(args);
        if let Some(dir) = profile {
            c.env("DWELLER_PROFILE", dir);
        }
        c.output().unwrap()
    }
}

/// Q3.5: `Strict-Transport-Security` over https upgrades later http
/// requests to the host (and its subdomains with includeSubDomains) to
/// https on the same port, kept in the profile; max-age=0 clears it;
/// the header over http and on an IP host is ignored. The http URL of
/// the TLS port fails unless upgraded.
#[test]
fn hsts() {
    let Some(site) = HstsSite::start() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let profile = scratch().join("hsts-profile");
    let _ = std::fs::remove_dir_all(&profile);
    std::fs::create_dir_all(&profile).unwrap();
    let at = |scheme: &str, host: &str, path: &str| format!("{scheme}://{host}:{}{path}", site.tls);
    let dump = |url: &str| {
        let out = site.run(Some(&profile), &["dump", "--text", url]);
        (out.status.code(), text(&out.stdout))
    };
    let upgraded = (Some(0), "https page\n".to_string());
    let insecure = at("http", "site.test", "/");
    let sub = at("http", "sub.site.test", "/");

    assert_ne!(dump(&insecure), upgraded);
    let plain_header = format!("http://site.test:{}/hsts", site.plain);
    assert_eq!(dump(&plain_header).0, Some(0));
    assert_ne!(dump(&insecure), upgraded, "the header over http is ignored");
    assert_eq!(dump(&at("https", "127.0.0.1", "/hsts")), upgraded);
    assert_ne!(dump(&at("http", "127.0.0.1", "/")), upgraded, "IP hosts are ignored");

    assert_eq!(dump(&at("https", "site.test", "/hsts")), upgraded);
    assert_eq!(dump(&insecure), upgraded);
    assert_eq!(dump(&sub), upgraded);
    let saved = std::fs::read_to_string(profile.join("hsts")).unwrap();
    assert!(saved.starts_with("site.test ") && saved.ends_with(" *\n"), "{saved}");

    assert_eq!(dump(&at("https", "site.test", "/hsts-clear")), upgraded);
    assert_ne!(dump(&insecure), upgraded);
    assert_ne!(dump(&sub), upgraded);
}

/// Q3.5: on an https page an http script, iframe and fetch are blocked
/// and an http image is upgraded (the image URL is the TLS port, so it
/// loads only when upgraded); on an http page all of them load as
/// asked. `upgrade-insecure-requests` (CSP header or `<meta>`) upgrades
/// the page's script and fetch; Q3.5b: also its same-host navigation
/// (`location` to the http URL of the TLS port shows the https page),
/// and its frames inherit it (the frame's own http script is upgraded).
#[test]
fn mixed_content() {
    let Some(site) = HstsSite::start() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let (t, p) = (site.tls, site.plain);
    for page in ["uir-header.html", "uir-meta.html"] {
        let out = site.run(None, &["dump", "--text", "--js", &format!("https://site.test:{t}/{page}")]);
        assert_eq!(
            text(&out.stdout),
            "script: https\nfetch: https data\ndone\n",
            "{page}: {}",
            text(&out.stderr)
        );
    }
    if !has_sdl() {
        println!("skipped the iframe and image cases: no SDL");
        return;
    }
    let cases = [
        (
            format!("https://site.test:{t}/mixed.html?{t},{p}"),
            "script: error\nfetch: TypeError\nimg: load\ndone\n",
        ),
        (
            format!("http://site.test:{p}/mixed.html?{t},{p}"),
            "script: http\niframe: http:\nfetch: http data\nimg: error\ndone\n",
        ),
    ];
    for (page, want) in cases {
        let out = site.run(None, &[&page, "--frames", "200", "--until-idle", "--text"]);
        let shown = text(&out.stdout);
        assert!(shown.starts_with(&format!("Mixed content\n{want}")), "{page}: {shown}\n{}", text(&out.stderr));
    }
    let uir = [
        ("uir-nav.html", "https page"),
        ("uir-frame.html", "UIR frame\nframe: https: script: https\n"),
    ];
    for (page, want) in uir {
        let at = format!("https://site.test:{t}/{page}");
        let out = site.run(None, &[&at, "--frames", "200", "--until-idle", "--text"]);
        let shown = text(&out.stdout);
        assert!(shown.contains(want), "{page}: {shown}\n{}", text(&out.stderr));
    }
}

/// P10.21: SameSite (Lax by default), the third-party policy and
/// fetch/XHR `credentials`, on the test server's two sites (`127.0.0.1`
/// and `localhost` of one port; `/samesite/` in `server.py`), cookies
/// kept between runs in a profile. `localhost` sets strict, lax, dflt
/// (no SameSite) and none (`SameSite=None` without Secure: rejected).
/// A same-site request gets all three; a cross-site top-level GET
/// navigation (a redirect from the other site, a form) lax and dflt; a
/// cross-site POST navigation and every third-party request (an
/// iframe, a script, fetch with any credentials mode) none. fetch with
/// `omit` sends none and does not store its Set-Cookie. (The public
/// suffix list: the unit tests "net: public suffixes..." and "net:
/// SameSite, the public suffix list...".)
#[test]
fn net_samesite() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let site = srv.base.clone();
    let other = site.replace("127.0.0.1", "localhost");
    let dir = profile("samesite");
    let dump = |args: &[&str]| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROFILE", &dir)
            .arg("dump")
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{args:?}: {}",
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    assert_eq!(dump(&["--text", &format!("{other}/samesite/set")]), "set\n");
    let saved = std::fs::read_to_string(dir.join("cookies")).unwrap();
    assert_eq!(
        saved,
        "localhost\t1\t/\t0\t0\t0\tstrict\tstrict\t1\n\
         localhost\t1\t/\t0\t0\t0\tlax\tlax\t1\n\
         localhost\t1\t/\t0\t0\t0\tlax\tdflt\t1\n"
    );
    let all = "strict=1; lax=1; dflt=1";
    // A URL the user typed: same-site.
    assert_eq!(
        dump(&["--text", &format!("{other}/samesite/show")]),
        format!("GET cookies: {all}\n")
    );
    // Typed on 127.0.0.1, redirected to localhost: cross-site GET.
    assert_eq!(
        dump(&[
            "--text",
            &format!("{site}/samesite/go?to={other}/samesite/show")
        ]),
        "GET cookies: lax=1; dflt=1\n"
    );
    // Embedded in a page of the other site: nothing (third-party); in
    // a page of its own site: all, but for fetch's omit. The frames'
    // requests are in the server's log.
    assert_eq!(
        dump(&[
            "--text",
            "--js",
            &format!("{site}/samesite/embed?host=localhost")
        ]),
        "script cookies:\n\nomit:\n\nsame:\n\ninclude:\n\nxhr:\n\nxhrcred:\n\nafter:\n"
    );
    assert_eq!(
        dump(&[
            "--text",
            "--js",
            &format!("{other}/samesite/embed?host=localhost")
        ]),
        format!(
            "script cookies: {all}\n\nomit:\n\nsame: {all}\n\ninclude: {all}\n\n\
             xhr: {all}\n\nxhrcred: {all}\n\nafter: {all}\n"
        )
    );
    let log = dump(&["--text", &format!("{site}/log")]);
    let frames: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("samesite-frame"))
        .collect();
    assert!(
        frames.len() == 2
            && frames.iter().filter(|l| l.contains("| cookie:")).count() == 1
            && frames.iter().any(|l| l.contains(&format!(
                "referer: {other}/samesite/embed?host=localhost | cookie: {all}"
            ))),
        "{log}"
    );
    // fetch with omit stored nothing.
    assert_eq!(
        dump(&["--text", &format!("{other}/samesite/show")]),
        format!("GET cookies: {all}\n")
    );
    if !has_sdl() {
        println!("skipped the navigations: no SDL2");
        return;
    }
    // A form submitted by the page's script: by GET from the other site
    // Lax goes, by POST nothing; from its own site all.
    let all_get = format!("GET cookies: {all}");
    let all_post = format!("POST cookies: {all}");
    for (method, from, want) in [
        ("get", &site, "GET cookies: lax=1; dflt=1"),
        ("post", &site, "POST cookies:"),
        ("get", &other, all_get.as_str()),
        ("post", &other, all_post.as_str()),
    ] {
        let got = login_window(
            &format!("{from}/samesite/nav?method={method}&to={other}/samesite/show"),
            "frame frame",
            Some(&dir),
        );
        assert!(
            got.contains(&format!("url {other}/samesite/show")) && got.trim_end().ends_with(want),
            "{method} from {from}: want {want:?}, got:\n{got}"
        );
    }
}

/// `fetch` with each method, kind of body and request headers
/// (`browser/tests/net/fetch_methods.html`, through the server's
/// `/fetch-echo`) must print `fetch_methods.txt`: what the server
/// received, as Node 22's fetch sends it (Content-Type defaults, the
/// multipart body, redirects that keep or drop the method and body),
/// except that forbidden headers (Sec-*, Proxy-*, Content-Length) are
/// not sent.
#[test]
fn fetch_methods() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/fetch_methods.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/fetch_methods.txt"),
        &text(&out.stdout),
        "fetch_methods",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// XMLHttpRequest with each method, kind of body and request header
/// (`browser/tests/net/xhr_methods.html`, through `/fetch-echo`) must
/// print `xhr_methods.txt`: the bodies and Content-Type defaults of
/// `fetch_methods`, plus a Document body; GET and HEAD send no body;
/// setRequestHeader combines repeated names and drops forbidden ones;
/// redirects as fetch's; InvalidStateError, SyntaxError and SecurityError
/// where the XHR standard throws them.
#[test]
fn xhr_methods() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/xhr_methods.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/xhr_methods.txt"),
        &text(&out.stdout),
        "xhr_methods",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// CORS is enforced in net (Q3.2a): from `127.0.0.1` to `localhost` of
/// the same port, XHR and fetch read a body only when the response's
/// Access-Control-Allow-Origin allows the page (`*` not for a
/// credentialed request); else XHR ends with status 0 and `error`, fetch
/// rejects. A request whose preflight fails is not sent.
/// (`browser/tests/net/cors.html` must print `cors.txt`.)
#[test]
fn cors_in_net() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/cors.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/cors.txt"),
        &text(&out.stdout),
        "cors_in_net",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Q3.11: a cross-origin redirect drops `Authorization` (a script's,
/// fetch and sync XHR) and the URL's credentials; a same-origin one
/// keeps them.
#[test]
fn redirect_credentials() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/redirect-credentials.html", srv.base),
    ]);
    assert_eq!(
        text(&out.stdout).trim_end(),
        "fetch same-origin: authorization true\n\
         fetch cross-origin: authorization false\n\
         sync xhr cross-origin: authorization false",
        "{}",
        text(&out.stderr)
    );
    let userinfo = srv.base.replace("http://", "http://u:p@");
    let other = srv.base.replace("127.0.0.1", "localhost");
    let direct = run(&["dump", "--text", &format!("{userinfo}/fetch-echo")]);
    assert!(text(&direct.stdout).contains("\"authorization\""));
    let moved = run(&[
        "dump",
        "--text",
        &format!("{userinfo}/fetch-redirect-to?{other}/fetch-echo"),
    ]);
    assert!(text(&moved.stdout).contains("/fetch-echo"), "{}", text(&moved.stdout));
    assert!(!text(&moved.stdout).contains("\"authorization\""), "{}", text(&moved.stdout));
}

/// Q3.11: window.name survives a same-origin navigation and is cleared
/// on a cross-origin one (no opener); a page cannot navigate the top
/// level to `data:` (it stays, with a console error), a `data:` iframe
/// loads, and a `data:` URL the person types loads.
#[test]
fn window_name_and_top_data() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let other = base.replace("127.0.0.1", "localhost");
    let window = |url: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_CONSOLE", "1")
            .args([
                url, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "200",
                "--until-idle", "--state", "--text",
            ])
            .output()
            .unwrap();
        assert!(out.status.success(), "{url}: {}\n{}", out.status, text(&out.stderr));
        format!("{}{}", text(&out.stdout), text(&out.stderr))
    };
    let same = window(&format!("{base}/net/name-set.html?/net/name-show.html"));
    assert!(same.contains("name \"secret\""), "{same}");
    let cross = window(&format!("{base}/net/name-set.html?{other}/net/name-show.html"));
    assert!(cross.contains("name \"\""), "{cross}");
    let stayed = window(&format!("{base}/net/data-nav.html"));
    assert!(
        stayed.contains(&format!("url {base}/net/data-nav.html\n"))
            && stayed.contains("history 1 of 1\n")
            && stayed.contains("stayed, frame data loaded")
            && stayed.contains("[error] Not allowed to navigate top frame to data URL: data:text/html,"),
        "{stayed}"
    );
    let typed = window("data:text/html,<p>typed data</p>");
    assert!(typed.contains("typed data"), "{typed}");
}

/// Q3.11: a public page (`DWELLER_PUBLIC_ADDRESS` makes 127.0.0.1 public)
/// cannot fetch or XHR a loopback address (`[::1]`); a loopback page can.
#[test]
fn private_network_access() {
    let (Some(page), Some(local)) = (server(), server_on("::1")) else {
        println!("skipped: cannot start python3 for the test servers");
        return;
    };
    let href = format!("{}/net/pna.html?{}", page.base, local.base);
    let public = dweller_command()
        .current_dir(root())
        .env("DWELLER_PUBLIC_ADDRESS", "127.0.0.1")
        .args(["dump", "--text", "--js", &href])
        .output()
        .unwrap();
    assert_eq!(
        text(&public.stdout).trim_end(),
        "fetch: TypeError\nsync xhr: NetworkError",
        "{}",
        text(&public.stderr)
    );
    let loopback = run(&["dump", "--text", "--js", &href]);
    assert_eq!(
        text(&loopback.stdout).trim_end(),
        "fetch: 200\nsync xhr: 200",
        "{}",
        text(&loopback.stderr)
    );
}

/// CORS preflight in net (Q3.2b): a cross-origin PUT with `x-test` is
/// preceded by one OPTIONS, the same request within Max-Age by none; a
/// denied preflight leaves the PUT unsent; a redirect to another URL is
/// preflighted again, one to a URL with credentials fails. Both the
/// background loader (fetch) and the synchronous path (sync XHR).
/// (`browser/tests/net/cors-preflight.html` must print `cors-preflight.txt`.)
#[test]
fn cors_preflight() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/cors-preflight.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/cors-preflight.txt"),
        &text(&out.stdout),
        "cors_preflight",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// `crossorigin` on script, img and link (Q3.3): a CORS-mode request
/// that loads when the other origin sends ACAO and is a load error when
/// it does not; a CORS image is readable by canvas, a no-cors one taints
/// it. ORB in net: a no-cors script, image or fetch of a cross-origin
/// JSON, HTML or nosniff text/plain response is blocked, a real script
/// or image is not. (`browser/tests/net/crossorigin.html` must print
/// `crossorigin.txt`.)
#[test]
fn crossorigin_orb() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/crossorigin.html", srv.base),
    ]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/crossorigin.txt"),
        &text(&out.stdout),
        "crossorigin_orb",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Module scripts are fetched in CORS mode (Q3.3b): a cross-origin module,
/// or a module it imports, without ACAO is a load error (an error event,
/// a rejected import()); with ACAO it runs, as a same-origin graph does.
/// Credentials are same-origin by default and sent cross-origin only for
/// `crossorigin=use-credentials`, which also needs ACAC. The other origin
/// is a second server on 127.0.0.1: same-site, so its cookies are sent.
/// (`browser/tests/net/module-cors.html` must print `module-cors.txt`.)
#[test]
fn module_cors() {
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/module-cors.html?other={}", srv.base, other.base),
    ]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/module-cors.txt"),
        &text(&out.stdout),
        "module_cors",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Scripts of the page's HTML fire `load` once they ran and `error` when
/// the fetch or its CORS check failed (Q3.3c); `import()` takes the
/// credentials of the script that calls it, a module worker those of its
/// `credentials` option. A cross-origin module with ACAO is fetched by the
/// page's prefetch (also when a script removes it before it runs) and runs
/// from the cache, without a second request.
/// (`browser/tests/net/script-events.html` must print `script-events.txt`.)
#[test]
fn script_events() {
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/with-other/net/script-events.html?other={}", srv.base, other.base),
    ]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/script-events.txt"),
        &text(&out.stdout),
        "script_events",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    let log = text(&run(&["dump", "--text", &format!("{}/log", other.base)]).stdout);
    let origin = format!("origin={}", srv.base);
    for query in ["kept", "gone"] {
        let hits: Vec<&str> = log.lines().filter(|l| l.contains(&format!("ran.js?{query} "))).collect();
        assert_eq!(hits.len(), 1, "{query}: {log}");
        assert!(hits[0].ends_with(&format!("{origin} cookie=")), "{query}: {log}");
    }
}

/// The HTTP cache is keyed by the top-level site, the frame site and the
/// credentials (Q3.13). A page on `127.0.0.1` embeds a script of
/// `localhost`, then a frame of `localhost` embeds it (another frame
/// site: a second request), then a frame of `127.0.0.1` (the page's
/// partition: from the cache). A credentialed load of a script, then an
/// uncredentialed one (`crossorigin=anonymous`): a second request.
#[test]
fn cache_partition() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let other = srv.base.replace("127.0.0.1", "localhost");
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/with-other/net/cache-partition.html?other={other}", srv.base),
    ]);
    assert_eq!(
        text(&out.stdout).trim_end(),
        "credentialed: load\nuncredentialed: load\ndone",
        "{}",
        text(&out.stderr)
    );
    let log = text(&run(&["dump", "--text", &format!("{}/log", srv.base)]).stdout);
    for query in ["embed", "cred"] {
        let hits = log.lines().filter(|l| l.contains(&format!("r.js?{query} "))).count();
        assert_eq!(hits, 2, "{query}: {log}");
    }
}

/// Q3.14: Fetch Metadata, `Origin` on a POST navigation and referrer
/// policies, over https (`server.py` with `hsts/site.pem`; `site.test`
/// and `127.0.0.1` are two sites of one port). `/meta/` records the
/// headers in `/meta-log`. A plain http host that is not potentially
/// trustworthy gets no `Sec-Fetch-*`. Q3.14b: `referrerpolicy` and
/// `rel=noreferrer` on links, forms and frames, fetch's `referrerPolicy`,
/// a redirect's `Referrer-Policy`, no `Origin` after a POST's 303,
/// `referrerpolicy` on `img` and `link`. Q3.14c: `font` and `worker`
/// destinations; a script's `form.submit()` without user activation
/// sends no `Sec-Fetch-User`.
#[test]
fn fetch_metadata() {
    use std::io::BufRead;
    let tests = root().join("browser/tests");
    let Ok(mut child) = Command::new("python3")
        .arg("-I")
        .arg(tests.join("server.py"))
        .arg(&tests)
        .arg("127.0.0.1")
        .arg(tests.join("hsts/site.pem"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let _srv = Server { child, base: String::new() };
    let port: u16 = line.trim().parse().unwrap();
    let site = format!("https://site.test:{port}");
    let ip = format!("https://127.0.0.1:{port}");
    let resolve = format!("site.test:{port}:127.0.0.1");
    let ca = tests.join("hsts/ca.pem");
    let ca = ca.to_str().unwrap();
    let tls = [("DWELLER_TEST", "1"), ("DWELLER_TEST_CA", ca), ("DWELLER_RESOLVE", resolve.as_str())];
    let dump = |args: &[&str]| {
        let out = dweller_command().current_dir(root()).envs(tls).arg("dump").args(args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", text(&out.stderr));
        text(&out.stdout)
    };
    let meta = |path: &str, want: &str| {
        let log = dump(&["--text", &format!("{ip}/meta-log")]);
        assert!(log.lines().any(|l| l.starts_with(path) && l.ends_with(want)), "{path}: want {want:?} in\n{log}");
    };
    let page = format!("{site}/with-other/net/fetch-metadata.html?other={ip}");
    dump(&["--text", "--js", &page]);
    let shot = scratch().join("fetch_metadata_font.png");
    let fonts = dweller_command().current_dir(root()).envs(tls).args(["screenshot", &page, "-o", shot.to_str().unwrap(), "--font", "system", "--width", "400", "--height", "300"]).output().unwrap();
    dump(&["--text", &format!("{site}/meta/typed.html")]);
    dump(&["--text", "--js", &format!("{site}/net/fetch-metadata-meta.html")]);
    dump(&["--text", "--js", &format!("{site}/policy/no-referrer/net/fetch-metadata-plain.html")]);
    let policy = format!("{site}/with-other/net/fetch-metadata-policy.html?other={ip}");
    dump(&["--text", "--js", &policy]);
    let none = "sec-fetch-user= | origin=";
    meta("GET /meta/same.png", &format!("sec-fetch-site=same-origin | sec-fetch-mode=no-cors | sec-fetch-dest=image | {none} | referer={page}"));
    meta("GET /meta/cross.js", &format!("sec-fetch-site=cross-site | sec-fetch-mode=no-cors | sec-fetch-dest=script | {none} | referer={site}/"));
    meta("GET /meta/frame.html", &format!("sec-fetch-site=cross-site | sec-fetch-mode=navigate | sec-fetch-dest=iframe | {none} | referer={site}/"));
    meta("GET /meta/noref.js", &format!("sec-fetch-dest=script | {none} | referer="));
    meta("GET /meta/typed.html", "sec-fetch-site=none | sec-fetch-mode=navigate | sec-fetch-dest=document | sec-fetch-user=?1 | origin= | referer=");
    meta("GET /meta/meta-origin.png", &format!("sec-fetch-site=same-origin | sec-fetch-mode=no-cors | sec-fetch-dest=image | {none} | referer={site}/"));
    meta("GET /meta/header-none.png", &format!("sec-fetch-dest=image | {none} | referer="));
    meta("GET /meta/frame-noref.html", &format!("sec-fetch-dest=iframe | {none} | referer="));
    meta("GET /meta/redirected.js", &format!("sec-fetch-dest=script | {none} | referer="));
    meta("GET /meta/fetch-noref", "referer=");
    meta("GET /meta/fetch-303", &format!("origin= | referer={policy}"));
    meta("GET /meta/img-noref.png", &format!("sec-fetch-dest=image | {none} | referer="));
    meta("GET /meta/img-unsafe.png", &format!("sec-fetch-dest=image | {none} | referer={policy}"));
    meta("GET /meta/sheet-noref.css", &format!("sec-fetch-dest=style | {none} | referer="));
    if !text(&fonts.stderr).contains("no system font found") {
        meta("GET /meta/font.ttf", &format!("sec-fetch-site=same-origin | sec-fetch-mode=no-cors | sec-fetch-dest=font | {none} | referer={page}"));
    }
    meta("GET /meta/worker.js", &format!("sec-fetch-site=same-origin | sec-fetch-mode=no-cors | sec-fetch-dest=worker | {none} | referer={page}"));

    if let Some(plain) = server() {
        let plain_port = plain.base.rsplit(':').next().unwrap().to_string();
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_RESOLVE", format!("plain.test:{plain_port}:127.0.0.1"))
            .args(["dump", "--text", &format!("http://plain.test:{plain_port}/meta/plain.html")])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", text(&out.stderr));
        let log = text(&run(&["dump", "--text", &format!("{}/meta-log", plain.base)]).stdout);
        assert_eq!(log, "GET /meta/plain.html | sec-fetch-site= | sec-fetch-mode= | sec-fetch-dest= | sec-fetch-user= | origin= | referer=\n");
    }

    if !has_sdl() {
        println!("skipped the navigations: no SDL2");
        return;
    }
    let scripted = format!("{site}/net/scripted_submit.html?{ip}/meta/scripted.html");
    let out = dweller_command()
        .current_dir(root())
        .envs(tls)
        .args([scripted.as_str(), "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle", "--state", "--text", "--events", "loaded loaded"])
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "scripted submit: {}", text(&out.stderr));
    meta("GET /meta/scripted.html", &format!("sec-fetch-site=cross-site | sec-fetch-mode=navigate | sec-fetch-dest=document | {none} | referer={site}/"));
    for tab in [false, true] {
        let name = if tab { "tab" } else { "window" };
        let target = format!("{ip}/meta/nav-{name}.html");
        let samesite = format!("{site}/net/samesite_links.html?{target}");
        let referrer = format!("{site}/net/referrer_links.html?{ip}/meta/policy-{name}-");
        let (wait, then) = if tab { ("renderer:loaded ", " renderer:input renderer:loaded") } else { ("loaded ", " loaded") };
        let clicks = [(&samesite, "click:80,65"), (&samesite, "click:60,115"), (&referrer, "click:80,65"), (&referrer, "click:80,115"), (&referrer, "click:60,165"), (&referrer, "click:60,215")];
        for (links, click) in clicks {
            let mut c = dweller_command();
            c.current_dir(root()).envs(tls);
            if tab {
                c.env("DWELLER_PROCESS", "tab");
            }
            let events = format!("{wait}{click}{then}");
            let out = c
                .args([links, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle", "--state", "--text", "--events", &events])
                .output()
                .unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{name} {click}: {}", text(&out.stderr));
        }
        let path = format!("/meta/nav-{name}.html");
        meta(&format!("GET {path}"), &format!("sec-fetch-site=cross-site | sec-fetch-mode=navigate | sec-fetch-dest=document | sec-fetch-user=?1 | origin= | referer={site}/"));
        meta(&format!("POST {path}"), &format!("sec-fetch-site=cross-site | sec-fetch-mode=navigate | sec-fetch-dest=document | sec-fetch-user=?1 | origin={site} | referer={site}/"));
        let nav = "sec-fetch-site=cross-site | sec-fetch-mode=navigate | sec-fetch-dest=document | sec-fetch-user=?1";
        meta(&format!("GET /meta/policy-{name}-unsafe.html"), &format!("{nav} | origin= | referer={referrer}"));
        meta(&format!("GET /meta/policy-{name}-noref.html"), &format!("{nav} | origin= | referer="));
        meta(&format!("POST /meta/policy-{name}-nopost.html"), &format!("{nav} | origin=null | referer="));
        meta(&format!("GET /meta/policy-{name}-see.html"), &format!("{nav} | origin= | referer={site}/"));
    }
}

/// An asynchronous XMLHttpRequest fires its events in the XHR standard's
/// order (`browser/tests/net/xhr_events.html` must print
/// `xhr_events.txt`): loadstart at send, then in a networking task
/// readystatechange 2 and 3, progress, readystatechange 4, load and
/// loadend; a network error ends with readystatechange 4, error and
/// loadend; abort() in flight fires readystatechange 4, abort and
/// loadend and nothing after; open() again drops the request in flight.
/// A synchronous one is done when send returns; fetch() settles in a
/// task (after the microtasks), and aborting its signal rejects it.
#[test]
fn xhr_events() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/xhr_events.html", srv.base),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/xhr_events.txt"),
        &text(&out.stdout),
        "xhr_events",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// P10.22: an agent never gets the sign-in prompt: its navigation to a
/// 401 with a Basic challenge shows the 401 page and the reply says
/// `auth_required` with the realm; it signs in with URL userinfo, and
/// the credentials are cached for the process. `dump` shows the 401
/// page.
#[test]
fn net_basic_auth() {
    use std::io::Write;
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    let userinfo = basic.replace("http://", "http://alice:wonderland@");
    let mut child = dweller_command()
        .current_dir(root())
        .arg("agent")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!(
        "{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"{basic}\"}}\n\
         {{\"id\":2,\"cmd\":\"navigate\",\"url\":\"{userinfo}\"}}\n\
         {{\"id\":3,\"cmd\":\"navigate\",\"url\":\"{basic}\"}}\n"
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(lines.len(), 3, "{got}");
    assert!(
        lines[0].contains(&format!("\"result\":\"loaded {basic} (status 401)\""))
            && lines[0].contains("\"auth_required\":{\"scheme\":\"Basic\",\"realm\":\"r\"}"),
        "{got}"
    );
    for line in &lines[1..] {
        assert!(
            line.contains("Basic: signed in as alice") && !line.contains("auth_required"),
            "{got}"
        );
    }
    expect(
        &["dump", "--text", &basic],
        0,
        "401: sign in to realm r\n",
        "",
    );
}

/// With `nosniff`, a script must have a JavaScript MIME type and a sheet
/// `text/css`, else it is a load error (script error event, sheet not
/// applied); without it any type loads. (`browser/tests/net/nosniff.html`
/// must print `nosniff.txt`.)
#[test]
fn nosniff_script_style() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/net/nosniff.html", srv.base),
    ]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/nosniff.txt"),
        &text(&out.stdout),
        "nosniff_script_style",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
