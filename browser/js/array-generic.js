(function () {
'use strict';
// v8: src/builtins/array-foreach.tq, array-map.tq, array-filter.tq, array-reduce.tq (spec order on generic objects)
const A = Array.prototype;
const native = {};
const apply = Reflect.apply;
const isArray = Array.isArray;
const MAX = 9007199254740991;
function toObject(o) {
  if (o === null || o === undefined) throw new TypeError('Array.prototype method called on null or undefined');
  return Object(o);
}
function toInteger(v) {
  const n = +v;
  return n !== n ? 0 : Math.trunc(n);
}
function lengthOf(o) {
  const n = toInteger(o.length);
  return n <= 0 ? 0 : n > MAX ? MAX : n;
}
function callable(f) {
  if (typeof f !== 'function') throw new TypeError(String(typeof f === 'symbol' ? 'Symbol()' : f) + ' is not a function');
  return f;
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
// spec: https://tc39.es/ecma262/#sec-arrayspeciescreate
function speciesCreate(o, n, C) {
  if (!isArray(o)) return new Array(n);
  if (C !== null && (typeof C === 'object' || typeof C === 'function')) {
    C = C[Symbol.species];
    if (C === null) C = undefined;
  }
  if (C === undefined) return new Array(n);
  if (!isConstructor(C)) throw new TypeError('object.constructor[Symbol.species] is not a constructor');
  return new C(n);
}
function put(o, k, v) {
  Object.defineProperty(o, k, { value: v, writable: true, enumerable: true, configurable: true });
}
function search(o, len, x, from, step, strict) {
  for (let k = from; step > 0 ? k < len : k >= 0; k += step) {
    if (!strict) {
      const v = o[k];
      if (v === x || (x !== x && v !== v)) return k;
    } else if (k in o && o[k] === x) return k;
  }
  return -1;
}
function startIndex(v, len, def) {
  if (v === undefined) return def;
  const n = toInteger(v);
  return n < 0 ? Math.max(len + n, 0) : Math.min(n, len);
}
const generic = {
  forEach(f, t) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    for (let k = 0; k < len; k++) if (k in o) apply(f, t, [o[k], k, o]);
  },
  some(f, t) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    for (let k = 0; k < len; k++) if (k in o && apply(f, t, [o[k], k, o])) return true;
    return false;
  },
  every(f, t) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    for (let k = 0; k < len; k++) if (k in o && !apply(f, t, [o[k], k, o])) return false;
    return true;
  },
  map(f, t) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    const out = speciesCreate(o, len, isArray(o) ? o.constructor : undefined);
    for (let k = 0; k < len; k++) if (k in o) put(out, k, apply(f, t, [o[k], k, o]));
    return out;
  },
  filter(f, t) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    const out = speciesCreate(o, 0, isArray(o) ? o.constructor : undefined);
    let to = 0;
    for (let k = 0; k < len; k++) {
      if (!(k in o)) continue;
      const v = o[k];
      if (apply(f, t, [v, k, o])) put(out, to++, v);
    }
    return out;
  },
  reduce(f, init) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    let k = 0;
    let acc;
    if (arguments.length > 1) acc = init;
    else {
      while (k < len && !(k in o)) k++;
      if (k >= len) throw new TypeError('Reduce of empty array with no initial value');
      acc = o[k++];
    }
    for (; k < len; k++) if (k in o) acc = f(acc, o[k], k, o);
    return acc;
  },
  reduceRight(f, init) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    let k = len - 1;
    let acc;
    if (arguments.length > 1) acc = init;
    else {
      while (k >= 0 && !(k in o)) k--;
      if (k < 0) throw new TypeError('Reduce of empty array with no initial value');
      acc = o[k--];
    }
    for (; k >= 0; k--) if (k in o) acc = f(acc, o[k], k, o);
    return acc;
  },
  indexOf(x, from) {
    const o = toObject(this);
    const len = lengthOf(o);
    if (len === 0) return -1;
    return search(o, len, x, startIndex(from, len, 0), 1, true);
  },
  lastIndexOf(x, from) {
    const o = toObject(this);
    const len = lengthOf(o);
    if (len === 0) return -1;
    const n = arguments.length > 1 ? toInteger(from) : len - 1;
    return search(o, len, x, n < 0 ? len + n : Math.min(n, len - 1), -1, true);
  },
  includes(x, from) {
    const o = toObject(this);
    const len = lengthOf(o);
    if (len === 0) return false;
    return search(o, len, x, startIndex(from, len, 0), 1, false) >= 0;
  },
};
function findIn(fromEnd, wantIndex) {
  return function (f, t) {
    const o = toObject(this);
    const len = lengthOf(o);
    callable(f);
    for (let i = 0; i < len; i++) {
      const k = fromEnd ? len - 1 - i : i;
      const v = o[k];
      if (apply(f, t, [v, k, o])) return wantIndex ? k : v;
    }
    return wantIndex ? -1 : undefined;
  };
}
const finders = { find: findIn(false, false), findIndex: findIn(false, true), findLast: findIn(true, false), findLastIndex: findIn(true, true) };
const speciesUsers = { map: true, filter: true };
function wrap(k, g) {
  const n = native[k] = A[k];
  const fast = speciesUsers[k]
    ? function () { return isArray(this) && this.constructor === Array; }
    : function () { return isArray(this); };
  const w = {
    [k]() {
      return apply(fast.call(this) ? n : g, this, arguments);
    },
  }[k];
  Object.defineProperty(w, 'length', { value: n.length, configurable: true });
  Object.defineProperty(A, k, { value: w, writable: true, configurable: true });
}
for (const k of Object.keys(generic)) wrap(k, generic[k]);
for (const k of Object.keys(finders)) wrap(k, finders[k]);
})();
