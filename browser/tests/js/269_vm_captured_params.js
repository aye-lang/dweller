function f(a, b = 1) { return () => a + b }
console.log(f(1)(), f(1, 5)())
function h(x, ...a) { return () => [x, a.length] }
console.log(h(1, 2, 3)())
function m(a = 2, {y} = {y: 3}) { a++; return () => a * y }
console.log(m()(), m(4, {y: 1})())
function n(a, b = () => a) { a = 9; return b() }
console.log(n(1))
function q(a, b = a + 1) { return [() => a, () => b] }
console.log(q(1)[1](), q(1, 7)[0]())
async function r(a, b = 2) { await null; return () => a + b }
r(1).then(g => console.log("async", g()))
function* s(a, b = 3) { yield () => a + b }
console.log(s(1).next().value())
