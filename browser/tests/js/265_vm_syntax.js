function mk(onClose) { return { [Symbol.iterator]() { var i = 0; return { next() { return {value: i++, done: false}; }, return() { onClose(); return {}; } }; } }; }
function f() {
  var out = [];
  var [a, , b = 5, ...rest] = [1, 2, undefined, 4, 6];
  out.push(a, b, rest.join("+"));
  let {x, y: {z = 9} = {}, ["w"]: w} = {x: 1, w: 3};
  out.push(x, z, w);
  [a, b] = [b, a];
  out.push(a, b);
  var o = {};
  ({p: o.q, r: o["s"]} = {p: 7, r: 8});
  out.push(o.q, o.s);
  function g({m, n = 2}, [k] = [3], ...more) { return m + n + k + more.length; }
  out.push(g({m: 1}), g({m: 1, n: 1}, [1], 0, 0));
  out.push(Math.max(...[1, 5, 3], 2), [0, ...new Set([1, 2]), ..."ab"].join());
  var s = 0;
  for (var v of [1, 2, 3]) s += v;
  for (const [k2, v2] of new Map([[1, 10], [2, 20]])) s += k2 * v2;
  var keys = [];
  for (var key in {a: 1, b: 2}) keys.push(key);
  for (let key in [5, 6]) keys.push(key);
  out.push(s, keys.join());
  var closed = 0;
  var it = mk(function () { closed++; });

  for (var q of it) { if (q > 2) break; }
  try { for (var q of it) { throw 1; } } catch (e) {}
  (function () { for (var q of it) { return; } })();

  out.push(closed);
  var fs = [];
  for (let i of [1, 2]) fs.push(() => i);
  out.push(fs[0]() + fs[1]());
  out.push(`a${1 + 1}b${"c"}`, ((s, ...v) => s.raw.join("|") + v.join())`x${1}\n${2}`);
  var n = null, obj = {a: {b() { return this.c; }, c: 4}};
  out.push(n?.a, n?.a.b.c, obj?.a?.b(), obj.a?.["c"], n?.(), obj.z?.(), 2 ** 10);
  out.push(/a+b/g.test("aab"));
  outer: for (var u of [1, 2]) { for (var t of [3, 4]) { continue outer; } }
  return out.join(",");
}
console.log(f());
function hot() {
  var t = 0;
  for (var i = 0; i < 2000; i++) {
    var [a, b] = [i, 1];
    var {c = 2} = {};
    t += a + b + c + Math.max(...[i, 0]) + `${i}`.length;
    for (var k in {x: 1}) t += k.length;
    t += ({v: {w: i}})?.v?.w ?? 0;
  }
  return t;
}
console.log(hot());
function lit() {
  var k = "dyn", s = Symbol("s");
  var base = {a: 1, get g() { return 2; }};
  var o = {...base, [k]: 3, [s]: 4, get [k + "G"]() { return 5; }, set x(v) { this._x = v; }, b: 6, ...null, ...{c: 7}};
  o.x = 8;
  return [Object.keys(o).join(), o.dyn, o[s], o.dynG, o._x, o.g, o.c, Object.getOwnPropertyDescriptor(o, "g").value].join();
}
console.log(lit());
function rest() {
  var k = "b";
  var {a, [k]: b, ...r} = {a: 1, b: 2, c: 3, d: 4};
  var x, y;
  ({x, ...y} = {x: 1, z: 2});
  return [a, b, Object.keys(r).join(), JSON.stringify(y)].join();
}
function C() { this.nt = new.target === C; var f = () => new.target; this.arrow = f() === C; }
function plain() { return new.target; }
console.log(rest(), new C().nt, new C().arrow, plain());
