var g = 1;
let gl = 2;
const gc = 3;

function keyed(a, i) {
  a[i] = a[i - 1] * 2;
  a[i + 1] += 5;
  a[0]++;
  --a[1];
  return a[i] + a[i + 1];
}
var arr = [1, 2, 3, , 5];
console.log(keyed(arr, 2), arr.join(","), 3 in arr, arr.length);

function named(o) {
  o.x = 1;
  o.y += 2;
  o.z = o.x++ + ++o.y;
  o["w"] = o.z * 2;
  return o.x + o.y + o.z + o.w;
}
console.log(named({ y: 1 }));

function literals(a, b) {
  var o = { a: a, b: b, c: [a, b, [a + b]], "d e": 1, seven: 7 };
  var p = { m() { return this.a; }, get g() { return 42; }, set s(v) { this.a = v; }, __proto__: { inh: "yes" } };
  p.s = 9;
  var e = {};
  var q = [];
  var dup = { k: 1, k: 2 };
  return [o.a, o.b, o.c[2][0], o["d e"], o.seven, p.m(), p.g, p.a, p.inh, Object.keys(e).length, q.length, dup.k, JSON.stringify(o)].join("|");
}
console.log(literals(4, 5));

function ops(o) {
  var r = [];
  r.push("a" in o, "zz" in o, o instanceof Object, [] instanceof Array, o instanceof Array);
  r.push(delete o.a, "a" in o, delete o["b"], delete o.nope, typeof o, typeof o.c);
  r.push(typeof undeclaredGlobal, typeof g, typeof gl, typeof Math);
  return r.join(",");
}
console.log(ops({ a: 1, b: 2, c: "s" }));

function inNumber() { return 1 in 5; }
function instanceOfNumber(o) { return o instanceof 5; }
try { inNumber(); } catch (e) { console.log(e.constructor.name); }
try { instanceOfNumber({}); } catch (e) { console.log(e.constructor.name); }

function globals() {
  g = g + 1;
  g += 10;
  g++;
  ++g;
  gl = gl * 3;
  gl -= 1;
  newGlobal = 7;
  return [g, gl, gc, newGlobal].join(",");
}
console.log(globals(), globals());

function assignConst() { gc = 4; }
try { assignConst(); } catch (e) { console.log(e.constructor.name); }

function methodsByKey(o, k) {
  return o[k](3) + o["f"](1);
}
console.log(methodsByKey({ f(x) { return x * 10; } }, "f"));

function strings(s) {
  var n = 0;
  for (var i = 0; i < s.length; i++) if (s[i] === "a") n++;
  return n + s[100] + s["length"];
}
console.log(strings("banana"));

function setters() {
  var log = [];
  var o = { set v(x) { log.push(x); }, get v() { return 1; } };
  o.v = 5;
  o.v += 2;
  o["v"]++;
  return log.join(",");
}
console.log(setters());

function hot() {
  var pts = [];
  for (var i = 0; i < 1000; i++) pts.push({ x: i, y: i * 2 });
  var s = 0;
  for (var j = 0; j < pts.length; j++) s += pts[j].x + pts[j]["y"];
  return s;
}
console.log(hot());
