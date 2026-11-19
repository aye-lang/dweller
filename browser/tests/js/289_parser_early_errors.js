const bad = [
  "var [...a, b] = [];", "([...a, b]) => 1;", "[...a, b] = [];", "var {...a, b} = {};", "({...a, b} = {});", "function f(...a, b) {}",
  "async function f() { var await; }", "async () => { await: ; }", "function* g() { var yield; }", "function* g() { void yield; }",
  "'use strict'; var yield;", "'use strict'; var let;", "'use strict'; var static;", "'use strict'; delete x;",
  "function f(a = 1) { 'use strict'; }", "function f([a]) { 'use strict'; }", "(...a) => { 'use strict'; }",
  "class C { #x; m() { delete this.#x; } }", "class C { #x; m() { delete (this.#x); } }", "class C { x = arguments; }", "class C { x = () => arguments; }",
  "class C extends Object { x = super(); }",
];
const good = [
  "var [a, ...b] = [];", "var {a, ...b} = {};", "function f(a, ...b) {}", "var await; var yield; var static;", "delete x;",
  "function f(a) { 'use strict'; }", "class C { #x; m() { delete this.x; } }", "class C { x = function () { return arguments; }; }",
  "async function f() { await 1; }", "function* g() { yield; yield 1; }",
];
for (const src of bad) { try { new Function(src); console.log("accepted: " + src); } catch (e) { if (!(e instanceof SyntaxError)) console.log(e.name + ": " + src); } }
for (const src of good) { try { new Function(src); } catch (e) { console.log("refused: " + src); } }
class P { #f = 1; #m() { return 2; } static #s = 3; get #g() { return 4; } static keys(o) { return [Object.getOwnPropertyNames(o).length, Reflect.ownKeys(P.prototype).join(), Object.getOwnPropertyNames(P).sort().join(), #f in o, o.#m.name, JSON.stringify(o), o.hasOwnProperty("#f"), o.#f + o.#m() + P.#s + o.#g]; } }
const p = new P(); p["#f"] = "string key";
console.log(P.keys(p).join(" "));
console.log("done");
