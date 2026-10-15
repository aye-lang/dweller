// Arrows have no `this` or `arguments` of their own: using them captures
// the enclosing function's.
function Outer() {
  this.n = 1;
  const plain = (k) => k * 2;
  const reads = () => this.n + arguments.length;
  const nested = () => () => this;
  return [plain, reads, nested];
}
const top = (x) => x + 1;
