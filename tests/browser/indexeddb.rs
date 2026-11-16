use super::*;

const OPEN: &str = "state pending | upgrade 0->2 done versionchange | stores notes,titles | added 1 | upgrade complete | open books v2 | get First | all First,Second | count 2 | note key 1 | error ConstraintError | keys 1 | complete | databases books:2 | lower VersionError";
const STORE: &str = "order -Infinity -0.5 3 date5 \"\" \"a\" \"b\" bytes2 bytes2.1 [] [1,[0]] | range -0.5 3 date5 \"\" | count 7 cmp 101 | made 1 2 10.7 11 k 12 | path 1 7 {\"name\":\"gen\",\"a\":{\"b\":1}} a.b | inject DataError | inline DataError | pair [1,\"a\"] [1,\"b\"] 2 | value 86400000 true 1.2.255 x+gi 12345678901234567890 3hole NaN true true {\"list\":[true,null]} | clone DataCloneError | after delete 8 get undefined key \"a\" | cleared 0 all 0 | complete";
const TRANSACTION: &str = "first complete TransactionInactiveError InvalidStateError | inside undone | aborted abort null InvalidStateError | readonly ReadOnlyError NotFoundError NotFoundError InvalidAccessError TypeError readonly a,b | rolled back kept/0 | promise none | timeout TransactionInactiveError complete | unhandled abort ConstraintError | order other,one,two,read two | commit TransactionInactiveError InvalidStateError complete | final kept!,two,committed";
const VERSIONS: &str = "versionchange 1->2 | closed | upgrader: blocked 1->2, upgrade 1->2, read from the opener | reopened v2 added,notes from the upgrader";

fn dumped(page: &str, want: &str, stderr: &str) {
    expect(&["dump", "--text", "--js", &format!("browser/tests/indexeddb/{page}")], 0, &format!("{want}\n"), stderr);
}

/// Q5.11a: `indexedDB.open` runs the upgrade (a `versionchange`
/// transaction that creates object stores and writes), then `success`;
/// requests of a transaction answer in order, a failed `add` is an
/// `error` event, `databases()` lists the database and a lower version
/// is a VersionError.
#[test]
fn indexeddb_open() {
    dumped("open.html", OPEN, "");
}

/// Q5.11a: keys order as number < date < string < binary < array, key
/// ranges select by that order, key generators and key paths make and
/// place keys, and values come back as structured clones.
#[test]
fn indexeddb_store() {
    dumped("store.html", STORE, "");
}

/// Q5.11a: a transaction commits when its requests are done, `abort()`
/// and an unhandled request error roll its writes back, it stays active
/// through a promise reaction of a request's handler and not through a
/// timer, `readwrite` transactions on one store run in the order made,
/// and an exception in a handler aborts with AbortError.
#[test]
fn indexeddb_transaction() {
    dumped("transaction.html", TRANSACTION, "");
    dumped("throws.html", "throw abort AbortError | left 0", "[error] Uncaught Error: in a handler\n");
}

/// Q5.11a: a second tab's open with a higher version fires
/// `versionchange` at the first tab's connection and `blocked` at the
/// request until that connection closes; the upgrade then sees the
/// first tab's records.
#[test]
fn indexeddb_versions() {
    dumped("versions.html", VERSIONS, "");
}

/// Q5.11a: with a profile a database is saved under
/// `<profile>/indexeddb/` and the next run opens it without an upgrade.
#[test]
fn indexeddb_profile() {
    let dir = profile("indexeddb_profile");
    let visit = |want: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROFILE", &dir)
            .args(["dump", "--text", "--js", "browser/tests/indexeddb/profile.html"])
            .output()
            .unwrap();
        assert_eq!((text(&out.stdout).as_str(), text(&out.stderr).as_str()), (want, ""));
    };
    visit("created from 0 | visits 1 | first visit 7,8 | saved\n");
    visit("visits 1,2 | first visit 7,8 | saved\n");
    assert!(dir.join("indexeddb").join("file_3a_2f_2f").read_dir().unwrap().count() == 1);
}

/// Q5.11a: with `DWELLER_PROCESS=tab` and `site` the databases are the
/// browser's: a renderer's pages work as in one process, two renderers
/// see each other's version changes, a profile keeps the data, and a
/// renderer that names a document it was not given is killed.
#[test]
fn window_renderer_indexeddb() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    for mode in ["tab", "site"] {
        let envs = [("DWELLER_PROCESS", mode)];
        let shown = |page: &str, events: &str, dir: Option<&Path>| login_window_with(&format!("{base}/indexeddb/{page}"), events, dir, &envs);
        for (page, want) in [("open.html", OPEN), ("store.html", STORE), ("transaction.html", TRANSACTION)] {
            let got = shown(page, "renderer:title:done", None);
            assert!(got.contains(want), "{mode} {page}: want {want}, got:\n{got}");
        }
        let got = shown("versions.html", "renderer:title:upgraded key:cmd+1 renderer:title:done", None);
        assert!(got.contains(VERSIONS), "{mode} versions: {got}");
        let dir = profile(&format!("window_renderer_indexeddb_{mode}"));
        let first = shown("profile.html", "renderer:title:done", Some(&dir));
        let second = shown("profile.html", "renderer:title:done", Some(&dir));
        assert!(first.contains("created from 0 | visits 1 |") && second.contains("visits 1,2 | first visit 7,8 | saved"), "{mode} profile: {first}\n{second}");
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .env("DWELLER_TEST", "1")
        .env("DWELLER_TEST_EVIL", "idb")
        .args([&format!("{base}/indexeddb/open.html"), "--font", "fixed", "--frames", "300", "--state", "--events", "renderer:gone"])
        .output()
        .unwrap();
    let state = text(&out.stdout);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or_else(|| panic!("no renderer line in\n{state}"));
    let r: Vec<&str> = line.splitn(4, ' ').collect();
    assert_eq!(&r[2..], ["gone", "bad idb"], "{state}");
    assert_eq!(text(&out.stderr), format!("error: bad message: bad idb {}\n", r[1]));
}
