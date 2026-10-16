/*---
description: a module
flags: [module]
---*/
import { v } from "./dep_FIXTURE.js";
assert.sameValue(v, 5);
assert.sameValue(this, undefined);
