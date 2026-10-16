var out = [];
function thrower(x) { if (x > 2) throw new RangeError('big ' + x); return x; }
function a(x) { try { return thrower(x); } catch (e) { return e.message; } }
out.push(a(1), a(5));
function b() { var log = []; try { log.push(1); throw 'x'; } catch (e) { log.push(e); } finally { log.push('f'); } return log.join(); }
out.push(b());
function c() { try { return 'try'; } finally { out.push('fin-c'); } }
out.push(c());
function d() { try { return 'try'; } finally { return 'fin'; } }
out.push(d());
function e1() { for (var i = 0; i < 5; i++) { try { if (i == 1) continue; if (i == 3) break; out.push('i' + i); } finally { out.push('f' + i); } } return i; }
out.push(e1());
function f() { outer: for (var i = 0; i < 3; i++) { for (var j = 0; j < 3; j++) { try { try { if (j == 1) continue outer; } finally { out.push('in' + i + j); } } finally { out.push('out' + i + j); } } } return 'f'; }
out.push(f());
function g() { try { try { throw 1; } finally { out.push('g1'); } } catch (e) { return 'caught' + e; } }
out.push(g());
function h() { try { null.x; } catch (e) { return e instanceof TypeError; } }
out.push(h());
function k() { let fs = []; try { let v = 7; fs.push(() => v); throw fs; } catch (err) { let w = 3; fs.push(() => w + err.length); } return fs[0]() + fs[1](); }
out.push(k());
function deep(n) { if (n == 0) throw new Error('bottom'); return deep(n - 1) + 1; }
function m() { try { return deep(50); } catch (e) { return e.message; } }
out.push(m());
function rec(n) { return rec(n + 1); }
function ov() { try { rec(0); } catch (e) { return e instanceof RangeError; } }
out.push(ov());
function nested() { var s = ''; for (var i = 0; i < 3; i++) { try { try { throw i; } catch (e) { s += 'c' + e; if (e == 1) throw 'again'; } } catch (e2) { s += 'C' + e2; } finally { s += 'F'; } } return s; }
out.push(nested());
function ret2() { var x = 0; try { x = 1; return x; } finally { x = 2; out.push('x' + x); } }
out.push(ret2());
function noparam() { try { throw 1 } catch { return 'np'; } }
out.push(noparam());
function sw(v) { switch (v) { case 1: try { break; } finally { out.push('swf'); } case 2: return 'two'; } return 'after'; }
out.push(sw(1), sw(2));
function fin3() { try { throw new Error('lost'); } finally { return 'swallowed'; } }
out.push(fin3());
function cb() { try { [1,2].forEach(function (q) { if (q == 2) throw 'cb' + q; }); } catch (e) { return e; } }
out.push(cb());
var top1; try { throw 'top'; } catch (e) { top1 = e; } finally { out.push('topfin'); }
out.push(top1);
for (var z = 0; z < 100000; z++) { try { if (z % 1000 == 0) throw z; } catch (e) { top1 = e; } }
out.push(top1);
console.log(out.join('|'));
function inClosureCtx() { var r = []; for (let i = 0; i < 3; i++) { try { r.push(() => i); if (i == 1) throw i; } catch (e) { r.push(() => e * 10); } } return r.map(f => f()).join(); }
console.log(inClosureCtx());
function viaNative() { try { JSON.parse('{bad'); } catch (e) { return e.name; } }
console.log(viaNative());
function throwFromFinally() { try { try { throw 'a'; } finally { throw 'b'; } } catch (e) { return e; } }
console.log(throwFromFinally());
function retInCatchFin() { try { throw 1; } catch (e) { return 'c' + e; } finally { console.log('rcf'); } }
console.log(retInCatchFin());
