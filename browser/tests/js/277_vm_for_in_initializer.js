const out = [];
let n = 0;
for (var a = 1 in { p: 1 }) out.push(a);
out.push(a);
for (var b = (n++, 5) in {}) ;
out.push(b, n);
for (var f = function () {} in null) ;
out.push(f.name);
function g() {
  for (var y = 7 in undefined) ;
  return y;
}
out.push(g());
var order = [];
for (var z = order.push('init') in (order.push('obj'), { q: 1 })) order.push(z);
out.push(order.join());
console.log(out.join(' '));
