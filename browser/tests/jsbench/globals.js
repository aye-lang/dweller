// The same loop at the top level: `var` names are global object properties.
// SMOKE (a global number, set by the smoke test) divides the size.
var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
var n = Math.floor(3000000 / scale);
var s = 0;
for (var i = 0; i < n; i++) s += i & 7;
console.log(s);
