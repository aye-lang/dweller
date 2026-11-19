use super::*;

/// Q3.15c: a media element that a script makes and inserts loads its
/// source without a script touching a media property, once.
#[test]
fn media_scripted_element_loads() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/media/scripted.html", srv.base)]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    assert_eq!(got.matches("loadeddata without a touch\n").count(), 1, "{got}");
    assert_eq!(got.matches("error at the source of a missing file\n").count(), 1, "{got}");
}
