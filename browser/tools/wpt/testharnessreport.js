// Dweller's testharnessreport.js (P10.1): WPT pages load it right after
// testharness.js. When the harness completes, it writes one console
// line, "DWELLER_WPT " and then
//
//     {"status":"OK","message":null,"subtests":[{"name":"...","status":"PASS","message":null}, ...]}
//
// which `dweller wpt URL` prints on stdout (browser/main.aye). The harness
// does not time itself (explicit_timeout): `dweller wpt` calls the
// harness's timeout() when its own --timeout passes in the page's time.
(function () {
    var HARNESS = ["OK", "ERROR", "TIMEOUT", "PRECONDITION_FAILED"];
    var SUBTEST = ["PASS", "FAIL", "TIMEOUT", "NOTRUN", "PRECONDITION_FAILED"];

    function text(v) {
        return v === null || v === undefined ? null : String(v);
    }

    setup({ explicit_timeout: true, output: false });

    add_completion_callback(function (tests, harness_status) {
        var subtests = [];
        for (var i = 0; i < tests.length; i++) {
            subtests.push({
                name: String(tests[i].name),
                status: SUBTEST[tests[i].status] || "FAIL",
                message: text(tests[i].message)
            });
        }
        var result = {
            status: HARNESS[harness_status.status] || "ERROR",
            message: text(harness_status.message),
            subtests: subtests
        };
        console.log("DWELLER_WPT " + JSON.stringify(result));
    });
})();
