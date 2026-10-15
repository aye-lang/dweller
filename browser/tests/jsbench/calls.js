// Method call, `new` and two property writes per iteration.
// SMOKE (a global number, set by the smoke test) divides the size.
function Point(x, y) {
  this.x = x;
  this.y = y;
}
Point.prototype.add = function(o) {
  return new Point(this.x + o.x, this.y + o.y);
};
var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
var n = Math.floor(2000000 / scale);
var p = new Point(0, 0);
var d = new Point(1, 2);
for (var i = 0; i < n; i++) p = p.add(d);
console.log(p.x * 31 + p.y);
