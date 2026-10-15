// A tight integer loop over function locals (BYTECODE.md section 1).
// SMOKE (a global number, set by the smoke test) divides the size.
function loop(n) {
  var s = 0;
  for (var i = 0; i < n; i++) s += i & 7;
  return s;
}
var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
console.log(loop(Math.floor(10000000 / scale)));
