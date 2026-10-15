// Sloppy, simple parameters: `arguments` aliases them, so they live in
// the context. Strict code and non-simple parameters do not alias.
function sloppy(a, b) {
  return arguments.length + a;
}
function strict(a, b) {
  "use strict";
  return arguments[0] + b;
}
function rest(a, ...more) {
  return arguments.length + more.length;
}
function unused(a, b) {
  return a + b;
}
function viaArrow(a) {
  return () => arguments[0];
}
