(function () {
'use strict';
// spec: https://tc39.es/ecma262/#sec-array.prototype.concat
const MAX = 9007199254740991;
const isArray = Array.isArray;
const define = Object.defineProperty;
function isObject(o) {
  return o !== null && (typeof o === 'object' || typeof o === 'function');
}
function isConstructor(C) {
  if (typeof C !== 'function') return false;
  try {
    Reflect.construct(String, [], C);
    return true;
  } catch (e) {
    return false;
  }
}
function speciesCreate(o) {
  if (!isArray(o)) return [];
  let C = o.constructor;
  if (isObject(C)) {
    C = C[Symbol.species];
    if (C === null) C = undefined;
  }
  if (C === undefined) return [];
  if (!isConstructor(C)) throw new TypeError('object.constructor[Symbol.species] is not a constructor');
  return new C(0);
}
function spreadable(o) {
  if (!isObject(o)) return false;
  const s = o[Symbol.isConcatSpreadable];
  return s !== undefined ? !!s : isArray(o);
}
function lengthOf(o) {
  const n = +o.length;
  if (n !== n || n <= 0) return 0;
  return n > MAX ? MAX : Math.floor(n);
}
function put(a, n, v) {
  define(a, String(n), { value: v, writable: true, enumerable: true, configurable: true });
}
return function concat(...items) {
  if (this === null || this === undefined) throw new TypeError('Array.prototype.concat called on null or undefined');
  const o = Object(this);
  const a = speciesCreate(o);
  let n = 0;
  for (let i = -1; i < items.length; i++) {
    const e = i < 0 ? o : items[i];
    if (!spreadable(e)) {
      if (n >= MAX) throw new TypeError('Array.prototype.concat: result is too long');
      put(a, n++, e);
      continue;
    }
    const len = lengthOf(e);
    if (n + len > MAX) throw new TypeError('Array.prototype.concat: result is too long');
    for (let k = 0; k < len; k++, n++) {
      const p = String(k);
      if (p in e) put(a, n, e[p]);
    }
  }
  a.length = n;
  return a;
};
})();
