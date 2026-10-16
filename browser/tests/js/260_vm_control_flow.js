// Control flow the bytecode VM compiles (P4.64): if, loops, break and
// continue with labels, switch (tests and SwitchOnSmi), ?:, &&, ||, ??,
// comma, void. The same output with DWELLER_JS_VM=1 and under GC stress.
function sign(x) { if (x < 0) return -1; else if (x > 0) return 1; return 0; }
function sumTo(n) { var s = 0; var i = 0; while (i < n) { s += i; i++; } return s; }
function fact(n) { var r = 1; do { r *= n; n--; } while (n > 1); return r; }
function forLoop(n) { var s = ""; for (let i = 0; i < n; i++) { s += i; } return s; }
function forEmpty(n) { var i = 0; for (;;) { if (i >= n) break; i += 2; } return i; }
function brk(n) { var s = 0; for (var i = 0; i < 100; i++) { if (i == n) break; s += i; } return s; }
function cont(n) { var s = 0; for (var i = 0; i < n; i++) { if (i % 2) continue; s += i; } return s; }
function contDo(n) { var s = 0, i = 0; do { i++; if (i == 3) continue; s += i; } while (i < n); return s; }
function contWhile(n) { var s = 0, i = 0; while (i < n) { i++; if (i == 2) continue; s += i; } return s; }
function labels(n) {
  var c = 0;
  outer: for (var i = 0; i < n; i++) {
    for (var j = 0; j < n; j++) {
      if (j == 2) continue outer;
      if (i == 3) break outer;
      c += 10 * i + j;
    }
  }
  return c;
}
function twoLabels(n) { var c = 0; a: for (var i = 0; i < n; i++) { for (;;) { if (i == 1) continue a; if (i == 4) break a; break; } c += i; } return c; }
function labeledBlock(x) { var r = 1; blk: { if (x) break blk; r = 2; } return r; }
function labeledWhile(n) { var i = 0; w: while (true) { i++; do { if (i > n) break w; continue w; } while (false); } return i; }
function sw(x) { var r = ""; switch (x) { case 1: r += "one"; case 2: r += "two"; break; case "3": r += "str"; break; default: r += "dflt"; case 9: r += "nine"; } return r; }
function swNoDefault(x) { switch (x) { case 0: return "zero"; case 5: return "five"; } return "none"; }
function dense(x) { var r; switch (x) { case 0: r = "a"; break; case 1: r = "b"; break; case 2: r = "c"; break; case 4: r = "e"; break; case 5: r = "f"; break; default: r = "d"; } return r; }
function denseMidDefault(x) { var r = 0; switch (x) { case 1: r += 1; case 2: r += 2; break; default: r += 100; break; case 4: r += 4; case 6: r += 6; } return r; }
function denseNeg(x) { switch (x) { case -2: return "m2"; case -1: return "m1"; case 0: return "z"; case 1: return "p1"; } return "?"; }
function dupCase(x) { switch (x) { case 1: return "first"; case 2: return "x"; case 3: return "y"; case 1: return "second"; } return "none"; }
function swInLoop(n) { var s = ""; for (var i = 0; i < n; i++) { switch (i) { case 1: continue; case 3: break; default: s += "d"; } s += i; } return s; }
function cond(x) { return x > 5 ? x < 10 ? "mid" : "big" : "small"; }
function logic(a, b) { return [a && b, a || b, a ?? b, !a && b, a || b && 2].join(); }
function nullish(a) { return [a ?? "d", (a ?? 0) + 1].join(); }
function comma(a) { var x = (a++, a++, a); return x + "," + a; }
function voidIt(a) { return void (a = 5); }
function nested(n) { var c = 0; for (var i = 0; i < n; i++) { for (var j = i; j < n; j++) { if ((i + j) & 1) { c += j; } else { c -= 1; } } } return c; }
function lets(n) { var s = 0; for (let i = 0; i < n; i++) { let t = i * 2; const u = t + 1; s += u; } return s; }
function longLoop(n) { var s = 0; for (var i = 0; i < n; i++) { s += i; s -= 1; s += 2; s ^= 3; s += 4; s -= 5; s += 6; s ^= 7; s += 8; s -= 9; s += 10; s ^= 11; s += 12; s -= 13; s += 14; s ^= 15; s += 16; s -= 17; s += 18; s ^= 19; s += 20; s -= 21; s += 22; s ^= 23; s += 24; s -= 25; s += 26; s ^= 27; s += 28; s -= 29; s += 30; s ^= 31; s += 32; s -= 33; s += 34; s ^= 35; s += 36; s -= 37; s += 38; s ^= 39; s += 40; s -= 41; s += 42; s ^= 43; s += 44; s -= 45; s += 46; s ^= 47; s += 48; s -= 49; s += 50; s ^= 51; s += 52; s -= 53; s += 54; s ^= 55; s += 56; s -= 57; s += 58; s ^= 59; s += 60; s -= 61; s += 62; s ^= 63; s += 64; s -= 65; s += 66; s ^= 67; s += 68; s -= 69; s += 70; s ^= 71; s += 72; s -= 73; s += 74; s ^= 75; s += 76; s -= 77; s += 78; s ^= 79; if (s > 1e9) { break; } } return s; }
function strLoop(s) { var n = 0; for (var i = "a"; i != "aaaa"; i += "a") n++; return n + s; }
var out = [];
for (var k = -1; k <= 1; k++) out.push(sign(k));
out.push(sumTo(10), fact(5), forLoop(5), forEmpty(7), brk(4), cont(9), contDo(5), contWhile(4));
out.push(labels(5), twoLabels(6), labeledBlock(0), labeledBlock(1), labeledWhile(3));
for (var v of [1, 2, "3", 3, 9, 7]) out.push(sw(v));
out.push(swNoDefault(0), swNoDefault(5), swNoDefault(6));
for (var v of [0, 1, 2, 3, 4, 5, 6, 1.5, "1", -0, NaN, undefined]) out.push(dense(v));
for (var v of [1, 2, 4, 6, 3, 0]) out.push(denseMidDefault(v));
for (var v of [-2, -1, 0, 1, 2]) out.push(denseNeg(v));
out.push(dupCase(1), dupCase(3), dupCase(9), swInLoop(5));
out.push(cond(1), cond(7), cond(12));
out.push(logic(0, 3), logic(2, 3), logic(null, 3), logic("", 0), nullish(undefined), nullish(null), nullish(0), nullish(false));
out.push(comma(1), voidIt(1), nested(6), lets(5), longLoop(3), longLoop(3000000), strLoop("x"));
console.log(out.join("\n"));
