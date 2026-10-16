/*---
description: an async test finishing through $DONE
flags: [async]
---*/
Promise.resolve(7).then(function (v) {
  assert.sameValue(v, 7);
}).then($DONE, $DONE);
