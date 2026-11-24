const log = console.log;
const { Object, Array, Symbol, Date, Map, Set, Error, RangeError, AggregateError, Promise } = globalThis;
const list = view => [...view].join(' ');
const keys = Object.keys;
let replacementCalls = 0;

function replace(holder, key) {
  holder[key] = function () {
    replacementCalls++;
    log('called', key);
    throw new Error('page replacement of ' + key + ' was called');
  };
}

const statics = {
  Array: ['from', 'isArray'],
  Object: ['assign', 'create', 'defineProperty', 'getOwnPropertyDescriptor', 'getOwnPropertyNames', 'getOwnPropertySymbols',
    'getPrototypeOf', 'is', 'isExtensible', 'keys', 'preventExtensions', 'setPrototypeOf'],
  Math: ['abs', 'floor', 'max', 'min', 'round', 'trunc'],
  BigInt: ['asIntN'],
  Date: ['UTC'],
  String: ['fromCharCode'],
  JSON: ['parse'],
};
for (const name of keys(statics)) for (const key of statics[name]) replace(globalThis[name], key);
const globals = ['Object', 'Array', 'String', 'Number', 'Boolean', 'Symbol', 'BigInt', 'Math', 'JSON', 'Date', 'RegExp', 'Map',
  'Set', 'WeakMap', 'WeakSet', 'Function', 'Error', 'EvalError', 'RangeError', 'ReferenceError', 'SyntaxError', 'TypeError',
  'URIError', 'isNaN', 'isFinite', 'parseInt'];
for (const name of globals) replace(globalThis, name);

function* gen() { yield 'g1'; yield 'g2'; }
const rejected = new Promise((res, rej) => rej('no'));
rejected.catch(() => {});

const steps = [
  () => Promise.all(new Set([1, new Promise(res => res(2))])).then(v => log('Promise.all', v)),
  () => Promise.allSettled(gen()).then(v => log('Promise.allSettled', v.map(r => r.status + ' ' + r.value).join(', '))),
  () => Promise.any([rejected, rejected]).catch(e => log('Promise.any', e instanceof AggregateError, e.errors)),
  () => log('AggregateError', new AggregateError(new Set(['a', 'b']), 'm').errors),
  () => log('Uint8Array.from', list(Uint8Array.from(gen(), (x, i) => i + 1))),
  () => log('new Uint8Array(iterable)', list(new Uint8Array(new Set([3, 4])))),
  () => log('new Int16Array(array-like)', list(new Int16Array({ length: 2, 0: -1, 1: 7 }))),
  () => log('new BigInt64Array', list(new BigInt64Array([1n, -1n]))),
  () => log('new Float64Array', list(new Float64Array([1.5, 2.25]).subarray(1))),
  () => { try { new Uint8Array(-1); } catch (e) { log('RangeError', e instanceof RangeError); } },
  () => log('Intl.ListFormat', new Intl.ListFormat('en', { type: 'conjunction' }).format(new Set(['a', 'b', 'c']))),
  () => log('Intl.Segmenter', [...new Intl.Segmenter('en', { granularity: 'word' }).segment('hi there')].map(s => s.segment + ':' + s.isWordLike).join(' ')),
  () => log('Intl.Segmenter grapheme', [...new Intl.Segmenter('en').segment('abc')].map(s => s.segment + s.index).join(' ')),
  () => {
    const buf = new ArrayBuffer(4);
    new Uint8Array(buf)[1] = 9;
    const copy = structuredClone({ buf, view: new Uint8Array(buf), list: [new Map([['k', new Set([1])]]), new Date(0)] }, { transfer: [buf] });
    log('structuredClone', buf.byteLength, copy.buf.byteLength, copy.view[1], copy.view.buffer === copy.buf, copy.list[0].get('k').has(1), copy.list[1].getTime());
  },
  () => log('Reflect.ownKeys', Reflect.ownKeys({ a: 1, [Symbol.iterator]: 2 }).length),
  () => log('Object.getOwnPropertyDescriptors', keys(Object.getOwnPropertyDescriptors({ a: 1, b: 2 }))),
  () => log('copyWithin', [1, 2, 3, 4, 5].copyWithin(0, 3)),
  () => log('Iterator take', [1, 2, 3].values().take(2).toArray()),
  () => log('setUTCFullYear', new Date(0).setUTCFullYear(2000)),
  () => log('unescape', unescape('%41%u0042')),
];
for (const step of steps) {
  try { step(); } catch (e) { log('threw', e.message); }
}

const savedResolve = Promise.resolve;
let resolveCalls = 0;
Promise.resolve = function (v) { resolveCalls++; return savedResolve.call(this, v); };
(async () => {
  await 1;
  await new Promise(res => res(2)).finally(() => 3);
  log('await and finally call Promise.resolve', resolveCalls);
  log('page replacements called', replacementCalls);
})();
