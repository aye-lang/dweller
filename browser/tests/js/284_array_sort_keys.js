console.log([1, '1', 10, '10', true, 'true', null, 'null', 2, 'a'].sort().map((v) => typeof v + ':' + v).join());
console.log([NaN, -0, 0, Infinity, -Infinity, 1e21, 1e-7, 0.1, -5].sort().join());
console.log([3n, 20n, 100n, 'x', undefined, 1].sort().join());
const xs = [];
for (let i = 0; i < 300; i++) {
  xs.push(((i * 7919) % 1000) / 8);
}
console.log(xs.sort().slice(0, 12).join(), xs.length);
const o = { toString() { return 'b'; } };
console.log([o, 'c', 'a', undefined, 'b'].sort().map(String).join());
const sym = Symbol('s');
console.log([sym].sort().length);
try {
  [sym, 'a'].sort();
} catch (e) {
  console.log(e instanceof TypeError);
}
console.log([, 'b', undefined, , 'a', 2].sort(), 3 in [, 'b', , 'a'].sort());
