// spec: https://w3c.github.io/IndexedDB/#request-api
(function (I) {
const G = globalThis;
const ak = I.ak;
const Event = G.Event;
const EventTarget = G.EventTarget;

I.hide = function (o, fields) {
  for (const k of Object.keys(fields)) Object.defineProperty(o, k, { value: fields[k], writable: true, configurable: true });
};
I.legal = function (token) {
  if (token !== I.TOKEN) throw new TypeError('Illegal constructor');
};
I.expose = function (name, C) {
  Object.defineProperty(C.prototype, Symbol.toStringTag, { value: name, configurable: true });
  Object.defineProperty(G, name, { value: C, writable: true, configurable: true });
};
I.need = function (args, n, what) {
  if (args.length < n) throw new TypeError("Failed to execute " + what + ": " + n + " argument" + (n > 1 ? 's' : '') + " required, but only " + args.length + " present.");
};

class DOMStringList {
  constructor(token, items) {
    I.legal(token);
    I.hide(this, { _items: items });
    items.forEach((s, i) => Object.defineProperty(this, i, { value: s, enumerable: true, configurable: true }));
  }
  get length() { return this._items.length; }
  item(i) {
    I.need(arguments, 1, "'item' on 'DOMStringList'");
    i = i >>> 0;

    return i < this._items.length ? this._items[i] : null;
  }
  contains(s) {
    I.need(arguments, 1, "'contains' on 'DOMStringList'");

    return this._items.includes(String(s));
  }
  [Symbol.iterator]() { return this._items[Symbol.iterator](); }
}
I.expose('DOMStringList', DOMStringList);
I.names = list => new DOMStringList(I.TOKEN, list.slice().sort());

class IDBRequest extends EventTarget {
  constructor(token) {
    I.legal(token);
    super();
    I.hide(this, { _done: false, _result: undefined, _error: null, _source: null, _tx: null, _op: null });
  }
  get result() {
    if (!this._done) throw I.error('InvalidStateError', "Failed to read the 'result' property from 'IDBRequest': The request has not finished.");

    return this._result;
  }
  get error() {
    if (!this._done) throw I.error('InvalidStateError', "Failed to read the 'error' property from 'IDBRequest': The request has not finished.");

    return this._error;
  }
  get source() { return this._source; }
  get transaction() { return this._tx; }
  get readyState() { return this._done ? 'done' : 'pending'; }
}
ak.defineHandlers(IDBRequest.prototype, ['success', 'error']);
I.expose('IDBRequest', IDBRequest);

class IDBOpenDBRequest extends IDBRequest {}
ak.defineHandlers(IDBOpenDBRequest.prototype, ['blocked', 'upgradeneeded']);
I.expose('IDBOpenDBRequest', IDBOpenDBRequest);

class IDBVersionChangeEvent extends Event {
  constructor(type, init) {
    I.need(arguments, 1, "to construct 'IDBVersionChangeEvent'");
    super(type, init);
    init = init || {};
    I.hide(this, { _old: versionOf(init.oldVersion, 0), _new: versionOf(init.newVersion, null) });
  }
  get oldVersion() { return this._old; }
  get newVersion() { return this._new; }
}
function versionOf(v, missing) {
  if (v === undefined || (v === null && missing === null)) return missing;

  return I.enforceRange(v);
}
I.expose('IDBVersionChangeEvent', IDBVersionChangeEvent);

I.enforceRange = function (v) {
  const n = Math.trunc(Number(v));

  if (n !== n || n < 0 || n > 9007199254740991) throw new TypeError('Value is outside the range of unsigned long long.');

  return n;
};
I.event = (type, bubbles, cancelable) => new Event(type, { bubbles, cancelable });
I.fire = function (path, ev) {
  ev.isTrusted = true;
  I.hide(ev, { _task: true, _threw: false });
  ak.dispatchAlong(path, ev);
  ev._task = false;

  return ev;
};
// spec: https://w3c.github.io/IndexedDB/#fire-a-success-event
I.fireAtRequest = function (req, ev) {
  const tx = req._tx;

  if (!tx) return I.fire([req], ev);
  if (tx._state === 'inactive') tx._state = 'active';

  I.fire([req, tx, tx._db], ev);

  if (tx._state !== 'active') return ev;

  tx._state = 'inactive';

  if (ev._threw) I.abortTransaction(tx, I.error('AbortError', 'An event handler threw: the transaction was aborted.'));
  else if (ev.type === 'error' && !ev.defaultPrevented) I.abortTransaction(tx, req._error);

  return ev;
};
})(globalThis.__idb);
