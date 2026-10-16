/*---
description: only strict; a plain call has an undefined this
flags: [onlyStrict]
---*/
assert.sameValue((function () { return this; })(), undefined);
