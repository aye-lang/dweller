// A direct eval may read any binding by name: every binding of the
// scopes around it goes to a context. An inner function without eval
// keeps its registers.
function f(a) {
  let x = 1;
  {
    let y = 2;
    eval("x + y");
  }
  function g(z) {
    return z;
  }
  return g;
}
