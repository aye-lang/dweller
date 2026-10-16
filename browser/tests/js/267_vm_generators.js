var print = (...a) => console.log(a.join(" "));
function* g(a, b = 2) { var x = yield a; try { yield x + b; yield* [7, 8]; } finally { print("fin"); } return 9; }
let it = g(1);
print(JSON.stringify(it.next()), JSON.stringify(it.next(5)), JSON.stringify(it.next()), JSON.stringify(it.next()), JSON.stringify(it.next()), JSON.stringify(it.next()));
let i2 = g(1); i2.next(); i2.next(); print(JSON.stringify(i2.return(42)), JSON.stringify(i2.next()));
function* h() { try { yield 1; } catch (e) { print("caught " + e); yield 2; } }
let i3 = h(); i3.next(); print(JSON.stringify(i3.throw("boom")), JSON.stringify(i3.next()));
try { h().throw("x"); } catch (e) { print("thrown " + e); }
function* inner() { try { yield 'a'; yield 'b'; } finally { print('inner fin'); } return 'r'; }
function* outer() { let v = yield* inner(); print('got ' + v); yield 'c'; }
print([...outer()].join());
let o = outer(); o.next(); print(JSON.stringify(o.return(5)));
function* ctx() { let fs = []; for (let i = 0; i < 3; i++) { fs.push(() => i); yield i; } return fs.map(f => f()).join(); }
let c = ctx(); c.next(); c.next(); c.next(); print(JSON.stringify(c.next()));
function* fin() { try { yield 1; } finally { yield 'f'; } }
let fz = fin(); fz.next(); print(JSON.stringify(fz.return(3)), JSON.stringify(fz.next()));
print(Object.keys(g(1)).length, g(1) instanceof g, Object.prototype.toString.call(g(1)));
function* rec() { const me = yield; me.next(); } let r = rec(); r.next(); try { r.next(r) } catch (e) { print(e.message) }
var obj = { *m() { yield this.v } , v: 3}; print(obj.m().next().value);
function* args() { yield arguments.length; yield arguments[1]; } let ag = args(1,2,3); print(ag.next().value, ag.next().value);
function* deep(n) { if (n > 0) { yield* deep(n - 1); } yield n; }
let total = 0; for (const v of deep(50)) { total += v; } print(total);
function* hot() { let i = 0; while (true) { yield i++; } }
let hs = 0; const hg = hot(); for (let k = 0; k < 2000; k++) { hs += hg.next().value; } print(hs);
const many = []; for (let k = 0; k < 3000; k++) { const x = hot(); x.next(); many.push(x); } print(many.length, many[2999].next().value);
function* thrower() { yield 1; throw new Error('inside'); }
try { [...thrower()]; } catch (e) { print('err', e.message); }
function* noThrow() { yield* { [Symbol.iterator]() { return { next() { return { value: 1, done: false }; }, return() { print('closed'); return {}; } }; } }; }
const nt = noThrow(); nt.next(); try { nt.throw(new Error('t')); } catch (e) { print('caught'); }
function* destr([a, b], { c } = { c: 3 }) { yield a + b + c; }
print(destr([1, 2]).next().value);
function* early() { yield 1; } const ea = early(); print(JSON.stringify(ea.return(7)), JSON.stringify(ea.next()));
function* lbl() { outer: for (const x of [1, 2]) { for (const y of [3, 4]) { if (y === 4) continue outer; yield x * y; } } }
print([...lbl()].join());
