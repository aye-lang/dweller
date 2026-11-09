use super::*;

/// Q3.8: `style-src`, `img-src`, `connect-src`, `frame-src` and
/// `base-uri` against the page of `browser/tests/csp/`. A style
/// element without the nonce and a `style` attribute do not apply (the
/// CSSOM still writes one), `document.styleSheets` leaves out the
/// blocked element, a cross-origin image, `fetch`, `sendBeacon`, frame
/// and `<base>` are refused, and each refusal fires one
/// `securitypolicyviolation` event with its effective directive.
#[test]
fn csp_directives_enforced() {
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/csp/directives.html?other={}", srv.base, other.base),
    ]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let want = "sheets 1 margins 3px 0px 0px\n\
        cssom 6px\n\
        base /csp/directives.html?other=OTHER\n\
        beacon false\n\
        violation base-uri OTHER/ enforce\n\
        violation style-src-elem inline enforce\n\
        violation connect-src OTHER/csp/cross.js enforce\n\
        violation connect-src OTHER/beacon enforce\n\
        violation img-src OTHER/csp/cross.js enforce\n\
        frame load no document\n\
        violation frame-src OTHER/csp/frame-none.html enforce\n\
        image error\n\
        fetch failed\n\
        same fetch ok\n\nx\n";
    assert_eq!(text(&out.stdout), want, "{}", text(&out.stderr));
}

/// Q3.8: a `Content-Security-Policy-Report-Only` header blocks nothing
/// (the style applies, the image, the script and the fetch load) and
/// fires `securitypolicyviolation` events with `disposition: "report"`,
/// the console lines marked `[Report Only]`.
#[test]
fn csp_directives_report_only() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/csp/report-only.html", srv.base)]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let want = "x\n\n\
        margin 3px padding 5px\n\
        violation connect-src /csp/same.js report style-src\n\
        violation img-src data report style-src\n\
        violation script-src-elem /csp/same.js report style-src\n\
        script load\n\
        image load\n\
        fetch ok\n";
    assert_eq!(text(&out.stdout), want, "{}", text(&out.stderr));
    let reported = text(&out.stderr).lines().filter(|l| l.starts_with("[error] [Report Only] Refused to ")).count();
    assert_eq!(reported, 6, "{}", text(&out.stderr));
}
