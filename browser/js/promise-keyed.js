(function () {
'use strict';
// spec: https://tc39.es/proposal-await-dictionary/
const apply = Reflect.apply;
function capability(C) {
  let resolve;
  let reject;
  const promise = new C((res, rej) => {
    if (resolve !== undefined || reject !== undefined) throw new TypeError('Promise executor has already been invoked');
    resolve = res;
    reject = rej;
  });
  if (typeof resolve !== 'function' || typeof reject !== 'function') throw new TypeError('Promise resolve or reject function is not callable');
  return { promise, resolve, reject };
}
function keyedResult(keys, values) {
  const o = Object.create(null);
  for (let i = 0; i < keys.length; i++) Object.defineProperty(o, keys[i], { value: values[i], writable: true, enumerable: true, configurable: true });
  return o;
}
function element(settle, keys, values, index, state, cap, once) {
  return x => {
    if (once.called) return;
    once.called = true;
    values[index] = settle(x);
    if (--state.remaining === 0) cap.resolve(keyedResult(keys, values));
  };
}
function keyed(C, promises, fulfilled, rejected) {
  const cap = capability(C);
  try {
    const resolve = C.resolve;
    if (typeof resolve !== 'function') throw new TypeError('Promise resolve is not a function');
    if (promises === null || (typeof promises !== 'object' && typeof promises !== 'function')) throw new TypeError('Promise keyed combinator called on a non-object');
    const keys = [];
    const values = [];
    const state = { remaining: 1 };
    for (const key of Reflect.ownKeys(promises)) {
      const desc = Reflect.getOwnPropertyDescriptor(promises, key);
      if (desc === undefined || !desc.enumerable) continue;
      const value = promises[key];
      const index = keys.length;
      keys.push(key);
      values.push(undefined);
      const next = apply(resolve, C, [value]);
      state.remaining++;
      const once = { called: false };
      const onFulfilled = element(fulfilled, keys, values, index, state, cap, once);
      const onRejected = rejected === undefined ? cap.reject : element(rejected, keys, values, index, state, cap, once);
      next.then(onFulfilled, onRejected);
    }
    if (--state.remaining === 0) cap.resolve(keyedResult(keys, values));
  } catch (e) {
    cap.reject(e);
  }
  return cap.promise;
}
const methods = {
  allKeyed(promises) {
    return keyed(this, promises, value => value, undefined);
  },
  allSettledKeyed(promises) {
    return keyed(this, promises, value => ({ status: 'fulfilled', value }), reason => ({ status: 'rejected', reason }));
  },
};
for (const k of Object.keys(methods)) Object.defineProperty(Promise, k, { value: methods[k], writable: true, configurable: true });
})();
