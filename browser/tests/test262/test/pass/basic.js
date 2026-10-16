/*---
description: a passing test with an include
includes: [compareArray.js]
---*/
assert.compareArray([1, 2].concat([3]), [1, 2, 3]);
assert.sameValue(typeof $262.global, "object");
assert.sameValue($262.evalScript("1 + 2"), 3);
$262.gc();
var ab = new ArrayBuffer(4);
$262.detachArrayBuffer(ab);
assert.sameValue(ab.byteLength, 0);
