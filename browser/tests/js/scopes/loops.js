// A `for (let ...)` whose variable a closure captures gets a context per
// iteration; the other loop's variables stay registers.
function f(n) {
  const fns = [];
  for (let i = 0; i < n; i++) {
    fns.push(() => i);
  }
  let sum = 0;
  for (let j = 0; j < n; j++) {
    sum += j;
  }
  for (const k of fns) {
    let v = k();
    fns.push(() => v);
  }
  return sum;
}
