// Calls and `new` from compiled code (P4.65): plain, method and
// constructor calls, natives and bound functions, missing and extra
// arguments, `arguments`, rest and default parameters, sloppy and
// strict `this`, globals and property access.
var g = 10;
let lex = 20;
function add(a, b) { return a + b; }
function missing(a, b, c) { return [a, b, c].length + "," + (c === undefined); }
function extra(a) { var s = a; return s; }
function sloppyThis() { return this === globalThis; }
function strictThis() { "use strict"; return this; }
function getG() { return g + lex; }
function Point(x, y) { this.x = x; this.y = y; }
Point.prototype.add = function (o) { return new Point(this.x + o.x, this.y + o.y); };
Point.prototype.len2 = function () { return this.x * this.x + this.y * this.y; };
function chain(n) { var p = new Point(0, 0); var d = new Point(1, 2); for (var i = 0; i < n; i++) p = p.add(d); return p.x * 31 + p.y; }
function fib(n) { if (n < 2) return n; return fib(n - 1) + fib(n - 2); }
function natives(x) { return Math.max(x, 3) + Math.floor(x / 2) + String(x).length; }
function bound(x) { var f = add.bind(null, 100); return f(x); }
function method(o) { return o.len2(); }
function strMethod(s) { return s.toUpperCase().charAt(1); }
function notFn(o) { try { o.nope(); } catch (e) { return e.constructor.name + ": " + e.message; } }
function callNotFn() { return notFn({}); }
function newNative(n) { var a = new Array(n); return a.length; }
function R() { this.v = 7; return { v: 8 }; }
function useR() { return new R().v; }
function setProp(o, v) { o.k = v; o.k = o.k + 1; return o.k; }
function recurse(n) { if (n == 0) return 0; return 1 + recurse(n - 1); }
function deep() { try { return recurse(100000); } catch (e) { return e.constructor.name; } }
function throwIn(o) { return o.x.y; }
function catchIn() { try { throwIn({}); } catch (e) { return e.constructor.name; } }
function args() { "use strict"; return arguments.length + ":" + Array.prototype.join.call(arguments, "-"); }
function argsMore(a, b) { "use strict"; return arguments.length + ":" + arguments[2] + ":" + (a + b); }
function rest(a, ...r) { return a + "|" + r.length + "|" + r.join(); }
function dflt(a, b = a * 2, c = b + 1) { return [a, b, c].join(); }
function dfltTdz(a = b, b = 1) { return a; }
function tdzCatch() { try { return dfltTdz(); } catch (e) { return e.constructor.name; } }
function both(a = 1, ...r) { return a + ":" + r.length + ":" + arguments.length; }
function viaArgs() { return args(1, 2, 3) + "/" + argsMore(1, 2, 3, 4) + "/" + args(); }
function nestedCalls(n) { return add(add(n, 1), add(n, 2)) + rest(n, n + 1, n + 2); }
var out = [];
out.push(add(1, 2), add("a", 1), missing(1), extra(5, 6, 7), sloppyThis(), strictThis(), getG());
out.push(chain(10), fib(15), natives(7), bound(5), method(new Point(3, 4)), strMethod("abc"));
out.push(viaArgs(), rest(1), rest(1, 2, 3), dflt(1), dflt(1, 5), dflt(1, undefined, 0), tdzCatch(), both(), both(undefined, 2, 3), nestedCalls(4));
out.push(callNotFn(), newNative(4), useR(), setProp({}, 3), recurse(50), deep(), catchIn());
console.log(out.join("\n"));
