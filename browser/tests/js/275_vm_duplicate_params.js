function f(a, a) { return [a, arguments[0], arguments[1], arguments.length].join(); }
function g(a, b, a) { a = 9; return [a, b, arguments[0]].join(); }
function h(x, x) { return () => x; }
function* k(y, y) { yield y; }
console.log(f(1, 2), f(1), g(1, 2, 3), h(4, 5)(), k(6, 7).next().value, f.length);
