# WPT fixtures (P10.1)

`resources/testharness.js` is web-platform-tests' harness at
e58225abb92a4f3b2efa0f5ab6bb1dd99f3dfa73, under its 3-clause BSD
license (`resources/LICENSE.md`). The pages load it and Dweller's
`browser/tools/wpt/testharnessreport.js`; `tests/browser.rs`
(`wpt_runner`) runs each with `dweller wpt` and checks its JSON line.

- `pass.html`: every subtest passes (a test, an async_test, a promise_test)
- `fail.html`: one of two subtests fails
- `error.html`: a script throws outside any test (harness ERROR)
- `timeout.html`: an async_test never finishes (TIMEOUT)
- `slow.html`: finishes after 15 s of the page's time (TIMEOUT by default,
  OK with `--timeout 60`)
- `sum.any.js` with `sum.any.html`, its wrapper written by hand
