(function () {
'use strict';
// spec: https://tc39.es/proposal-explicit-resource-management/
const { Object, String, Symbol, Error, ReferenceError, TypeError, WeakMap, Reflect, Promise, Iterator } = globalThis;
const ObjectCreate = Object.create;
const ObjectGetPrototypeOf = Object.getPrototypeOf;
const ObjectSetPrototypeOf = Object.setPrototypeOf;
const PromiseResolve = Promise.resolve.bind(Promise);
const PromiseReject = Promise.reject.bind(Promise);
const apply = Reflect.apply;
const construct = Reflect.construct;
const defineProperty = Object.defineProperty;
const dispose = Symbol('Symbol.dispose');
const asyncDispose = Symbol('Symbol.asyncDispose');
const slots = new WeakMap();
const asyncSlots = new WeakMap();

function hidden(o, k, v) {
  defineProperty(o, k, { value: v, writable: true, enumerable: false, configurable: true });
}
function callable(f, what) {
  if (typeof f !== 'function') throw new TypeError(what + ' is not a function');
  return f;
}
function isObject(o) {
  return o !== null && (typeof o === 'object' || typeof o === 'function');
}
function pending(map, o, name) {
  const s = map.get(o);
  if (s === undefined) throw new TypeError(name + ' method called on incompatible receiver');
  if (s.disposed) throw new ReferenceError(name + ' has already been disposed');
  return s;
}
function method(value, key) {
  const m = value[key];
  if (m === undefined || m === null) return undefined;
  return callable(m, 'The dispose method');
}

function SuppressedError(error, suppressed, message) {
  const o = construct(Error, [], new.target === undefined ? SuppressedError : new.target);
  if (message !== undefined) hidden(o, 'message', String(message));
  hidden(o, 'error', error);
  hidden(o, 'suppressed', suppressed);
  return o;
}
ObjectSetPrototypeOf(SuppressedError, Error);
defineProperty(SuppressedError, 'prototype', { value: ObjectCreate(Error.prototype), writable: false, enumerable: false, configurable: false });
hidden(SuppressedError.prototype, 'constructor', SuppressedError);
hidden(SuppressedError.prototype, 'message', '');
hidden(SuppressedError.prototype, 'name', 'SuppressedError');

function suppress(state, e) {
  state.error = state.failed ? new SuppressedError(e, state.error) : e;
  state.failed = true;
}
function disposeResources(list) {
  const state = { failed: false, error: undefined };
  for (let i = list.length - 1; i >= 0; i--) {
    try {
      apply(list[i].method, list[i].value, []);
    } catch (e) {
      suppress(state, e);
    }
  }
  if (state.failed) throw state.error;
}
async function disposeResourcesAsync(list) {
  const state = { failed: false, error: undefined };
  for (let i = list.length - 1; i >= 0; i--) {
    try {
      await (list[i].method === undefined ? undefined : apply(list[i].method, list[i].value, []));
    } catch (e) {
      suppress(state, e);
    }
  }
  if (state.failed) throw state.error;
}

class DisposableStack {
  constructor() {
    slots.set(this, { disposed: false, list: [] });
  }
  get disposed() {
    const s = slots.get(this);
    if (s === undefined) throw new TypeError('DisposableStack.prototype.disposed getter called on incompatible receiver');
    return s.disposed;
  }
  use(value) {
    const s = pending(slots, this, 'DisposableStack');
    if (value === null || value === undefined) return value;
    if (!isObject(value)) throw new TypeError('The resource is not an object');
    const m = method(value, dispose);
    if (m === undefined) throw new TypeError('The resource has no [Symbol.dispose] method');
    s.list.push({ value, method: m });
    return value;
  }
  adopt(value, onDispose) {
    const s = pending(slots, this, 'DisposableStack');
    callable(onDispose, 'onDispose');
    s.list.push({ value: undefined, method() { return onDispose(value); } });
    return value;
  }
  defer(onDispose) {
    const s = pending(slots, this, 'DisposableStack');
    s.list.push({ value: undefined, method: callable(onDispose, 'onDispose') });
  }
  move() {
    const s = pending(slots, this, 'DisposableStack');
    const moved = new DisposableStack();
    slots.get(moved).list = s.list;
    s.list = [];
    s.disposed = true;
    return moved;
  }
  dispose() {
    const s = slots.get(this);
    if (s === undefined) throw new TypeError('DisposableStack.prototype.dispose called on incompatible receiver');
    if (s.disposed) return undefined;
    s.disposed = true;
    const list = s.list;
    s.list = [];
    disposeResources(list);
    return undefined;
  }
}
hidden(DisposableStack.prototype, dispose, DisposableStack.prototype.dispose);
defineProperty(DisposableStack.prototype, Symbol.toStringTag, { value: 'DisposableStack', configurable: true });

function asyncMethod(value) {
  const m = method(value, asyncDispose);
  if (m !== undefined) return m;
  const sync = method(value, dispose);
  if (sync === undefined) throw new TypeError('The resource has no [Symbol.asyncDispose] or [Symbol.dispose] method');
  return function () { apply(sync, this, []); };
}

class AsyncDisposableStack {
  constructor() {
    asyncSlots.set(this, { disposed: false, list: [] });
  }
  get disposed() {
    const s = asyncSlots.get(this);
    if (s === undefined) throw new TypeError('AsyncDisposableStack.prototype.disposed getter called on incompatible receiver');
    return s.disposed;
  }
  use(value) {
    const s = pending(asyncSlots, this, 'AsyncDisposableStack');
    if (value === null || value === undefined) {
      s.list.push({ value: undefined, method: undefined });
      return value;
    }
    if (!isObject(value)) throw new TypeError('The resource is not an object');
    s.list.push({ value, method: asyncMethod(value) });
    return value;
  }
  adopt(value, onDisposeAsync) {
    const s = pending(asyncSlots, this, 'AsyncDisposableStack');
    callable(onDisposeAsync, 'onDisposeAsync');
    s.list.push({ value: undefined, method() { return onDisposeAsync(value); } });
    return value;
  }
  defer(onDisposeAsync) {
    const s = pending(asyncSlots, this, 'AsyncDisposableStack');
    s.list.push({ value: undefined, method: callable(onDisposeAsync, 'onDisposeAsync') });
  }
  move() {
    const s = pending(asyncSlots, this, 'AsyncDisposableStack');
    const moved = new AsyncDisposableStack();
    asyncSlots.get(moved).list = s.list;
    s.list = [];
    s.disposed = true;
    return moved;
  }
  disposeAsync() {
    const s = asyncSlots.get(this);
    if (s === undefined) return PromiseReject(new TypeError('AsyncDisposableStack.prototype.disposeAsync called on incompatible receiver'));
    if (s.disposed) return PromiseResolve(undefined);
    s.disposed = true;
    const list = s.list;
    s.list = [];
    return disposeResourcesAsync(list);
  }
}
hidden(AsyncDisposableStack.prototype, asyncDispose, AsyncDisposableStack.prototype.disposeAsync);
defineProperty(AsyncDisposableStack.prototype, Symbol.toStringTag, { value: 'AsyncDisposableStack', configurable: true });

const iteratorDispose = {
  [dispose]() {
    const r = this.return;
    if (r !== undefined) apply(r, this, []);
  },
};
const asyncIteratorDispose = {
  [asyncDispose]() {
    const iterator = this;
    return new Promise(function (resolve, reject) {
      try {
        const r = iterator.return;
        if (r === undefined) return resolve(undefined);
        PromiseResolve(apply(r, iterator, [undefined])).then(function () { resolve(undefined); }, reject);
      } catch (e) {
        reject(e);
      }
    });
  },
};
const asyncIteratorPrototype = ObjectGetPrototypeOf(ObjectGetPrototypeOf(async function* () {}.prototype));

defineProperty(Symbol, 'dispose', { value: dispose });
defineProperty(Symbol, 'asyncDispose', { value: asyncDispose });
hidden(Iterator.prototype, dispose, iteratorDispose[dispose]);
hidden(asyncIteratorPrototype, asyncDispose, asyncIteratorDispose[asyncDispose]);
hidden(globalThis, 'SuppressedError', SuppressedError);
hidden(globalThis, 'DisposableStack', DisposableStack);
hidden(globalThis, 'AsyncDisposableStack', AsyncDisposableStack);
})();
