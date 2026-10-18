var gv = 1;
let gl = 2;
const gc = 3;
function gf() { return 4; }

function direct(s) { try { return eval(s); } catch (e) { return e.name; } }
function indirect(s) { try { return (0, eval)(s); } catch (e) { return e.name; } }
function viaGlobal(s) { try { return globalThis.eval(s); } catch (e) { return e.name; } }
function top(s) { return [direct(s), indirect(s), viaGlobal(s), eval(s)].join(); }

console.log("var", top("gv"));
console.log("let", top("gl"));
console.log("const", top("gc"));
console.log("function", top("gf()"));
console.log("typeof", top("typeof gl"), top("typeof missing"));
console.log("missing", direct("missing"), indirect("missing"));
console.log("assign let", indirect("gl = 20, gl"), direct("gl = 21, gl"), gl);
console.log("assign const", direct("gc = 30"), indirect("gc = 31"), gc);
console.log("new var", indirect("var nv = 5; nv"), nv, typeof globalThis.nv);
console.log("new let", indirect("let nl = 6; nl"), typeof nl);

var Utils = (function () {
  "use strict";
  var failIf = function (gl, s) {
    var ex;
    try { eval(s); } catch (e) { ex = e; }
    return ex ? ex.name : "ok";
  };
  return { failIf: failIf };
}());
var calls = [];
var ctx = { NONE: 0, K: 1, pixelStorei: function (k, v) { calls.push(k + ":" + v); } };
console.log("parameter", Utils.failIf(ctx, "gl.pixelStorei(gl.K, gl.NONE);"), calls.join());
console.log("parameter shadows", Utils.failIf(ctx, "if (gl !== ctx) throw new TypeError()"));

(0, eval)("let hidden = 7; const hc = 8; class HK {}");
console.log("eval lexicals stay inside", direct("hidden"), indirect("typeof hidden"), typeof hc, typeof HK);
console.log("eval function", indirect("function ef() { return gl + 1; } ef()"), ef(), typeof globalThis.ef);
console.log("strict var", indirect("'use strict'; var sv = 9; sv"), typeof sv);
console.log("this", (function () { return (0, eval)("this") === globalThis; }).call({}), direct("this") === undefined || direct("this") === globalThis);
console.log("nested", indirect("(0, eval)('let q = 1; q + gv')"), indirect("eval('gl + gc')"));
console.log("completion", indirect("1; if (true) { 2; }"), indirect(""), indirect("var cv = 3"), indirect(42));
