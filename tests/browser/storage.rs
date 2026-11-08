use super::*;

/// P4.51: each storage area holds 5 MB of keys and values; a setItem
/// past it throws a QuotaExceededError DOMException and changes nothing.
#[test]
fn storage_quota() {
    expect(
        &["dump", "--text", "--js", "browser/tests/storage/quota.html"],
        0,
        "6 MB: QuotaExceededError (DOMException) | 3 MB: ok | 3 MB more: QuotaExceededError (DOMException) | replace with 4 MB: ok | session 6 MB: QuotaExceededError (DOMException) | keys 1 a 4194304 | after removing, 5 MB: ok\n",
        "",
    );
}

/// P4.51: a tab sees the localStorage writes of the tabs it opened as
/// `storage` events; a tab opened with an opener starts with a copy of
/// its sessionStorage, one opened with `noopener` with none, and neither
/// child's sessionStorage reaches the opener.
#[test]
fn storage_between_tabs() {
    expect(
        &["dump", "--text", "--js", "browser/tests/storage/tabs.html"],
        0,
        "copy=from the opener (old null, local, from child.html?copy) | fresh=null (old null, local, from child.html?fresh) | own session child: null | own length 1\n",
        "",
    );
}
