// META: title=an .any.js test
test(() => {
    assert_equals([1, 2, 3].reduce((a, b) => a + b, 0), 6);
}, "reduce sums");
test(() => {
    assert_array_equals("a,b".split(","), ["a", "b"]);
}, "split");
