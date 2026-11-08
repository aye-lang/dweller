function who() { return [typeof this, String(this === undefined ? "u" : this.n), arguments.length, Array.prototype.slice.call(arguments).join(",")]; }
function strictWho() { "use strict"; return [this, arguments.length]; }
var o = { n: "o" };
console.log(who.call(o, 1, 2, 3).join("|"));
console.log(who.call(o).join("|"));
console.log(who.call().join("|"));
console.log(strictWho.call().join("|"));
console.log(strictWho.call(null, 1).join("|"));
console.log(who.apply(o, [4, 5]).join("|"));
console.log(who.apply(o).join("|"));
console.log(who.apply(o, null).join("|"));
console.log(who.apply(o, { length: 2, 0: "a", 1: "b" }).join("|"));
console.log(who.apply(o, [, 7]).join("|"));
console.log((function () { return who.apply(o, arguments); })(8, 9).join("|"));
try { who.apply(o, 3); } catch (e) { console.log(e instanceof TypeError); }
function thrower() { throw new Error("from " + this.n); }
try { thrower.call(o); } catch (e) { console.log(e.message); }
function nt() { return new.target === undefined; }
console.log(nt.call(o), nt.apply(o, []));
class K {}
try { K.call(o); } catch (e) { console.log(e instanceof TypeError); }
function* g(a) { yield a + this.n; }
console.log(g.call(o, "x").next().value);
var bound = who.bind(o, "b");
console.log(bound.call(null, 1).join("|"));
console.log(Math.max.apply(null, [1, 9, 3]), String.prototype.slice.call("hello", 1, 3));
var depth = 0;
function deep() { depth++; deep.call(this); }
try { deep.call(o); } catch (e) { console.log(e instanceof RangeError, depth > 100); }
var sum = 0;
function add(x) { sum += x; }
for (var i = 0; i < 1000; i++) { add.call(null, i); add.apply(null, [i]); }
console.log(sum);
var call = Function.prototype.call;
console.log(call.call(who, o, 1).join("|"));
