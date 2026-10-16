/*---
description: an early SyntaxError, nothing runs before it
negative:
  phase: parse
  type: SyntaxError
---*/
print("must not print");
var 1x;
