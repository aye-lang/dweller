// A closure in a parameter default: the parameters it reads go to the
// context and the body's vars get a scope of their own.
function f(a, b = () => a, { c } = {}) {
  var a2 = a;
  return b() + a2 + c;
}
// Plain defaults without a closure: no separate scope.
function g(x, y = x + 1) {
  return y;
}
