function locals(a) {
  var x = 1;
  eval("x = x + a");
  return x + eval("a * 10");
}
console.log("locals", locals(2));

function closures() {
  var n = 0;
  var inc = function () { return eval("++n"); };
  inc();
  inc();
  return eval("n") + inc();
}
console.log("closures", closures());

function leak() {
  var before = typeof z;
  eval("var y = 5; var z = 7");
  return [before, y, z];
}
console.log("leak", leak().join());

function overwrite() {
  var w = 1;
  eval("var w = 2");
  return w;
}
console.log("overwrite", overwrite());

function strictEval() {
  "use strict";
  var s = 1;
  eval("var hidden = 3; s = s + hidden");
  return [s, typeof hidden];
}
console.log("strict", strictEval().join());

function strictInside() {
  eval("'use strict'; var gone = 4");
  return typeof gone;
}
console.log("strict-inside", strictInside());

function lexical() {
  var t = 1;
  eval("let t = 2; const u = 3");
  return [t, typeof u];
}
console.log("lexical", lexical().join());

var arrow = (p) => eval("p + 1");
console.log("arrow", arrow(41));

class Box {
  constructor(v) { this.v = v; }
  get() { return eval("this.v"); }
}
console.log("class", new Box(9).get());

function* gen(k) {
  yield eval("k");
  yield eval("k * 2");
}
console.log("generator", [...gen(3)].join());

async function asy(q) {
  var r = eval("q + 1");
  await null;
  return eval("r * 2");
}
asy(4).then((v) => console.log("async", v));

function shadowed() {
  var eval = function (s) { return "mine:" + s; };
  return eval("1 + 1");
}
console.log("shadowed", shadowed());

globalThis.g = "global";
function indirect() {
  var g = "local";
  return [(0, eval)("g"), eval("g")];
}
console.log("indirect", indirect().join());

function nested(m) {
  return eval("eval('m + 1') + 1");
}
console.log("nested", nested(5));

function args(a, b) {
  return eval("arguments.length + ':' + arguments[1]");
}
console.log("arguments", args(1, 2, 3));

function thisv() {
  return eval("this.tag");
}
console.log("this", thisv.call({ tag: "T" }));

function completion() {
  return [eval("1; 2;"), eval("if (true) { 'yes' } else { 'no' }"), eval("var q = 1"), eval(42)];
}
console.log("completion", completion().join());

function inner() {
  var c = 10;
  var f = eval("(function () { return c + 1; })");
  c = 20;
  return f();
}
console.log("inner", inner());

function syntax() {
  try {
    eval("var = ;");
  } catch (e) {
    return e.name;
  }
}
console.log("syntax", syntax());

function blockEval() {
  var out = [];
  for (let i = 0; i < 3; i++) {
    out.push(eval("i * i"));
  }
  {
    let blk = "b";
    out.push(eval("blk"));
  }
  return out.join();
}
console.log("block", blockEval());

function catchEval() {
  try {
    throw 7;
  } catch (err) {
    return eval("err + 1");
  }
}
console.log("catch", catchEval());

console.log("top", eval("var topLevel = 8; topLevel + 1"));

function fnLeak() {
  var first = eval("h(); function h() { return 'h'; }");
  return first + h();
}
console.log("function-leak", fnLeak());

function deadZone() {
  try {
    eval("typeof tz; let tz;");
  } catch (e) {
    return e.name;
  }
}
console.log("dead-zone", deadZone());
