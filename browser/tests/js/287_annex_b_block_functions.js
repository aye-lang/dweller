var log = [];
(function () {
  log.push(typeof f);
  f = 123;
  log.push(f);
  {
    log.push(typeof f);
    function f() { return 1; }
    f = 5;
    log.push(f);
  }
  log.push(typeof f);
})();
(function () {
  { let g = 1; { function g() {} } }
  log.push(typeof g);
  try { throw 1; } catch (h) { { function h() {} } }
  log.push(typeof h);
  try { throw {}; } catch ({ k }) { { function k() {} } }
  log.push(typeof k);
  for (let m of [1]) { { function m() {} } }
  log.push(typeof m);
})();
(function (p) {
  { function p() {} }
  log.push(typeof p);
})(7);
(function () {
  if (true) function a() { return "a"; }
  if (false) function b() {} else function c() { return "c"; }
  log.push(a(), typeof b, c());
  switch (1) { case 1: function s() { return "s"; } }
  log.push(s());
  { function n() { return 1; } { function n() { return 2; } log.push(n()); } log.push(n()); }
  { function* gen() {} async function asy() {} }
  log.push(typeof gen, typeof asy);
})();
log.push(typeof top1);
{ function top1() {} }
log.push(typeof top1);
(function () { "use strict"; { function st() {} } log.push(typeof st); })();
console.log(log.join(" "));
for (const src of ["while (0) function w() {}", "'use strict'; if (1) function w() {}", "if (1) function* w() {}", "for (;;) class C {}", "l: function ok() {}"]) {
  try { new Function(src); console.log("ok"); } catch (e) { console.log(e.name); }
}
