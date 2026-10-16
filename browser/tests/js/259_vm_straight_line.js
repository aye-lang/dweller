// Functions the bytecode VM compiles (P4.63: literals, registers,
// operators, assignments, return), with values that leave the numeric
// fast paths: strings, BigInts, objects with valueOf, null, undefined.
// The same output with DWELLER_JS_VM=1 and under DWELLER_JS_GC=stress.
function arith(a, b) { let c = a * b + 1; c += a; return c - b / 2; }
function cmp(x, y) { return [x < y, x > y, x <= y, x >= y, x == y, x != y, x === y, x !== y].join(); }
function str(s) { return s + "!" + typeof s + (-s) + (~s) + (+s) + !s; }
function compound(a) { let t = a; t **= 2; t %= 7; t <<= 3; t >>>= 1; t >>= 1; t |= 64; t &= 127; t ^= 5; t -= 1; t *= 3; t /= 2; return t; }
function bits(a, b) { return [a & b, a | b, a ^ b, a << b, a >> b, a >>> b, ~a].join(); }
function smi(a) { return [a + 1, a - 2, a * 3, a / 4, a % 5, a ** 2, a | 6, a ^ 7, a & 8, a << 1, a >> 1, a >>> 1].join(); }
function counts(x) { var o = x--; --x; ++x; var p = x++; return [o, p, x].join(); }
function sloppyThis() { return typeof this; }
function strictThis() { "use strict"; return this; }
function big(a, b) { return [a * b + 1n, a - b, -a, ~b, a ** 2n, a % b, a >> 1n].join(); }
function concat(a, b, c) { var s = a + b; s += c; s = s + s; return s + s.length; }
function missing(a, b, c) { return [a, b, c].join("|"); }
function voids(a) { var u = void a; return u === undefined && !a; }
function blocks(a) { var r = a; { let t = r * 2; r = t + 1; { const k = 3; r = r * k; } } return r; }
function decl(n) { let a; let b = n; const c = b + 1; var d; return [a, b, c, d].join(); }
function order(x, y) { return x * y - x < y; }
function vo(a, b) { return a + b; }
function seq(a) { var b = a; var r = b + (b = 10); return r + b; }
function neg0(a) { return 1 / -a; }
function typeofs(a) { return typeof a; }
function exprBody(a) { return a * a; }
const arrow = (a, b) => a * 10 + b;
var valueOf = { valueOf() { return 7; } };
var toStr = { toString() { return "ts"; } };
var log = "";
var ox = { valueOf() { log += "x"; return 1; } };
var oy = { valueOf() { log += "y"; return 2; } };
console.log(arith(3, 4), arith("3", 4), arith(valueOf, 2), arith(1));
console.log(cmp(1, 2), cmp("b", "a"), cmp(null, undefined), cmp(1, "1"), cmp(NaN, NaN), cmp(valueOf, 7));
console.log(str("5"), str(3), str(""), str(valueOf));
console.log(compound(10), compound(1.5), compound("4"));
console.log(bits(-5, 3), bits(2147483647, 1), bits("12", "2"), bits(1e10, 33));
console.log(smi(10), smi(-7.5), smi("3"));
console.log(counts(5), counts("7"), counts(null));
console.log(sloppyThis(), strictThis(), big(5n, 3n));
console.log(concat("a", "b", "c"), concat(1, 2, 3), concat(toStr, 1, 2));
console.log(missing(1), missing(1, 2, 3, 4), voids(0), voids(1));
console.log(blocks(2), decl(4), order(ox, oy), log, vo(valueOf, toStr), seq(1), neg0(0), neg0(-0));
console.log(typeofs(1), typeofs("s"), typeofs(1n), typeofs(Symbol()), typeofs(null), typeofs(console.log), typeofs({}), typeofs(undefined));
console.log(exprBody(9), arrow(4, 2));
for (const f of [() => arith(Symbol(), 1), () => str(Symbol()), () => smi(Symbol()), () => counts(Symbol()), () => big(1n, 2), () => compound(1n)]) {
  try { f(); console.log("no error"); } catch (e) { console.log(e.constructor.name); }
}
var acc = 0;
for (var i = 0; i < 2000; i++) acc = (acc + arith(i, 3) + exprBody(i % 13)) % 1000003;
console.log(acc);
