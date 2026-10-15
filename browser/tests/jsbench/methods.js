// Shared prototype methods and repeated own-property access.
var smokeScale = (typeof SMOKE === 'number') ? SMOKE : 1; // smoke test divisor; 1 = full size
class Point {
  constructor(x, y) { this.x = x; this.y = y; this.sum = 0; }
  move(dx, dy) { this.x += dx; this.y += dy; this.sum += this.x + this.y; }
  value() { return this.sum; }
}
function methods() {
  var points = [], sum = 0;
  for (var i = 0; i < 200; i++) points.push(new Point(i, -i));
  for (var r = 0; r < Math.floor(200000 / smokeScale); r++) {
    var p = points[r % 200];
    p.move(r % 7, r % 11);
    sum += p.value();
  }
  return sum;
}
console.log(methods());
