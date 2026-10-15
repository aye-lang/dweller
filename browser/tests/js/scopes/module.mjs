// Imports and exported locals are module cells; other top-level
// bindings are registers unless captured.
import { helper } from "./dep.mjs";
export let count = 0;
const step = 2;
let hidden = 5;
export function bump() {
  count += step;
  return helper(count);
}
let k = hidden + 1;
