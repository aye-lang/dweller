const construct = Reflect.construct;
const isConstructor = function (f) {
  if (typeof f !== 'function') return false;
  try { construct(Object, [], f); return true; } catch (e) { return false; }
};
const nativeFrom = Array.from;
let calls = 0;
Array.from = function (items) {
  calls++;
  const O = Object(items);
  const len = O.length >>> 0;
  const r = isConstructor(this) ? new this(len) : Array(len);
  for (let i = 0; i < len; i++) r[i] = O[i];
  r.length = len;
  return r;
};
for (let i = 0; i < 20; i++) Array.from([1, 2, 3]);
console.log('core-js Array.from calls', calls);
Array.from = nativeFrom;

function Pair(a, b) { this.a = a; this.b = b; }
const show = p => JSON.stringify(p);
console.log(show(Reflect.construct(Pair, { length: 2, 0: 'x', 1: 'y' })));
console.log(show(Reflect.construct(Pair, [, 'y'])), show(Reflect.construct(Pair, { length: 1, 0: 'x', 1: 'y' })));
const iterated = ['x', 'y'];
iterated[Symbol.iterator] = function* () { yield 'it'; };
console.log(show(Reflect.construct(Pair, iterated)));
(function () { console.log(show(Reflect.construct(Pair, arguments))); })('p', 'q');
for (const bad of [undefined, null, 'ab', 3]) {
  try { Reflect.construct(Pair, bad); console.log('no throw', bad); } catch (e) { console.log(String(bad), e.constructor.name); }
}
let read = false;
const watched = { get length() { read = true; return 0; } };
try { Reflect.construct(Pair, watched, Math.max); } catch (e) { console.log('bad newTarget', e.constructor.name, 'args read', read); }
class Base { constructor() { this.nt = new.target.name; } }
class Derived extends Base {}
console.log(Reflect.construct(Base, [], Derived).nt, Reflect.construct(Base, []) instanceof Base);
