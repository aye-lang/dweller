function iterable(trace, values, extra) {
  let i = 0;
  const iterator = Object.defineProperties({
    next() {
      trace.push('next');
      return i < values.length ? { value: values[i++], done: false } : { value: undefined, done: true };
    },
  }, Object.getOwnPropertyDescriptors(extra));
  return { [Symbol.iterator]() { return iterator; }, iterator };
}

function run(name, values, mapfn, extra) {
  const trace = [];
  const source = iterable(trace, values, extra(trace));
  let result;
  try {
    result = 'returned ' + JSON.stringify(Array.from(source, (v, k) => mapfn(trace, v, k)));
  } catch (e) {
    result = 'threw ' + e;
  }
  console.log(name + ': ' + result + ' | ' + trace.join(' '));
}

const closer = trace => ({
  return(...args) {
    trace.push('return this=' + (this === undefined ? 'undefined' : 'iterator') + ' args=' + args.length);
    return {};
  },
});
const mapThrowsAt1 = (trace, v, k) => {
  trace.push('map ' + v + ',' + k);
  if (k === 1) throw 'mapfn error';
  return v * 2;
};

run('mapfn throws', [1, 2, 3], mapThrowsAt1, closer);
run('mapfn returns', [1, 2], (trace, v, k) => { trace.push('map ' + v + ',' + k); return v + k; }, closer);
run('next throws', [1], (trace, v) => v, trace => ({
  next() { trace.push('next'); throw 'next error'; },
  return() { trace.push('return'); return {}; },
}));
run('return throws', [1, 2], mapThrowsAt1, trace => ({
  return() { trace.push('return'); throw 'return error'; },
}));
run('return getter', [1, 2], mapThrowsAt1, trace => ({
  get return() { trace.push('get return'); return undefined; },
}));
run('no return', [1, 2], mapThrowsAt1, () => ({}));

function* gen() {
  try {
    yield 1;
    yield 2;
    console.log('generator: not reached');
  } finally {
    console.log('generator: finally');
  }
}
try { Array.from(gen(), v => { throw 'gen ' + v; }); } catch (e) { console.log('generator threw', e); }
console.log('generator mapped', Array.from(gen(), (v, k) => v * 10 + k));
