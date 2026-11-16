// spec: https://w3c.github.io/IndexedDB/#object-store-interface
(function (I) {
const MAX_GENERATED = 9007199254740992n;
const what = name => "Failed to execute '" + name + "' on 'IDBObjectStore': ";

I.readable = function (store, name) {
  if (store._meta.deleted) throw I.error('InvalidStateError', what(name) + 'The object store has been deleted.');
  if (store._tx._state !== 'active') throw I.error('TransactionInactiveError', what(name) + 'The transaction is not active.');

  return store._tx;
};
I.writable = function (store, name) {
  const tx = I.readable(store, name);

  if (tx._mode === 'readonly') throw I.error('ReadOnlyError', what(name) + 'The transaction is read-only.');

  return tx;
};
I.countOf = function (count) {
  if (count === undefined) return 0;

  const n = I.enforceRange(count);

  if (n > 4294967295) throw new TypeError('Value is outside the range of unsigned long.');

  return n;
};
I.pairs = function (reply) {
  return reply === '' ? [] : reply.split('\n');
};

function cloneFor(tx, value, path) {
  tx._state = 'inactive';

  try {
    const text = I.serialize(value);

    return { text, copy: path === null ? null : I.deserialize(text) };
  } finally {
    tx._state = 'active';
  }
}
function generated(tx, meta, w) {
  const next = BigInt(I.ask('generator', tx._id, meta.id));

  if (next > MAX_GENERATED) throw I.error('ConstraintError', 'The key generator has reached its maximum value.');

  I.ask('set-generator', tx._id, meta.id, String(next + 1n));

  if (meta.keyPath !== null) {
    I.inject(w.copy, meta.keyPath, Number(next));
    w.text = I.serialize(w.copy);
  }

  return I.encodeKey(Number(next));
}
function raiseGenerator(tx, meta, key) {
  const n = I.decodeKey(key);

  if (n >= 1) I.ask('set-generator', tx._id, meta.id, String(BigInt(Math.min(Math.floor(n), 9007199254740992)) + 1n));
}
// spec: https://w3c.github.io/IndexedDB/#add-or-put
function write(store, name, args, overwrite) {
  I.need(args, 1, "'" + name + "' on 'IDBObjectStore'");

  const tx = I.writable(store, name);
  const meta = store._meta;
  const path = meta.keyPath;
  let key = args[1];

  if (path !== null && key !== undefined) throw I.error('DataError', what(name) + 'The object store uses in-line keys and the key parameter was provided.');
  if (path === null && !meta.autoIncrement && key === undefined) throw I.error('DataError', what(name) + 'The object store uses out-of-line keys and has no key generator and the key parameter was not provided.');
  if (key !== undefined) key = I.toKey(key);

  const w = cloneFor(tx, args[0], path);

  if (path !== null) {
    key = I.extractKey(w.copy, path);

    if (key === null) throw I.error('DataError', what(name) + 'Evaluating the object store\'s key path yielded a value that is not a valid key.');
    if (key === undefined && !meta.autoIncrement) throw I.error('DataError', what(name) + 'Evaluating the object store\'s key path did not yield a value.');
    if (key === undefined && !I.canInject(w.copy, path)) throw I.error('DataError', what(name) + 'A generated key could not be inserted into the value.');
  }

  return I.request(store, tx, () => {
    if (key === undefined) key = generated(tx, meta, w);
    else if (meta.autoIncrement && key[0] === 'b') raiseGenerator(tx, meta, key);

    I.ask('put', tx._id, meta.id, overwrite ? 1 : 0, key, I.indexKeys ? I.indexKeys(meta, w, key) : '', w.text);

    return I.decodeKey(key);
  });
}
function read(store, name, r, limit, map) {
  const tx = store._tx;
  const id = store._meta.id;

  return I.request(store, tx, () => map(I.pairs(I.ask(limit === 1 ? 'get' : 'get-all', tx._id, id, r.lo, r.hi, r.flags, limit))));
}
const values = pairs => pairs.filter((_, i) => i % 2 === 1).map(I.deserialize);
const keys = pairs => pairs.filter((_, i) => i % 2 === 0).map(I.decodeKey);

class IDBObjectStore {
  constructor(token, tx, meta) {
    I.legal(token);
    I.hide(this, { _tx: tx, _meta: meta, _indexes: new Map(), _path: undefined });
  }
  get name() { return this._meta.name; }
  set name(v) { I.renameStore(this, String(v)); }
  get keyPath() {
    if (this._path === undefined) this._path = Array.isArray(this._meta.keyPath) ? this._meta.keyPath.slice() : this._meta.keyPath;

    return this._path;
  }
  get indexNames() { return I.names(this._meta.indexes.map(x => x.name)); }
  get transaction() { return this._tx; }
  get autoIncrement() { return this._meta.autoIncrement; }
  put(value, key) { return write(this, 'put', arguments, true); }
  add(value, key) { return write(this, 'add', arguments, false); }
  delete(query) {
    I.need(arguments, 1, "'delete' on 'IDBObjectStore'");

    const tx = I.writable(this, 'delete');
    const r = I.rangeOf(query, false);
    const id = this._meta.id;

    return I.request(this, tx, () => void I.ask('delete', tx._id, id, r.lo, r.hi, r.flags));
  }
  clear() {
    const tx = I.writable(this, 'clear');
    const id = this._meta.id;

    return I.request(this, tx, () => void I.ask('delete', tx._id, id, '', '', 0));
  }
  get(query) {
    I.need(arguments, 1, "'get' on 'IDBObjectStore'");
    I.readable(this, 'get');

    return read(this, 'get', I.rangeOf(query, false), 1, pairs => values(pairs)[0]);
  }
  getKey(query) {
    I.need(arguments, 1, "'getKey' on 'IDBObjectStore'");
    I.readable(this, 'getKey');

    return read(this, 'getKey', I.rangeOf(query, false), 1, pairs => keys(pairs)[0]);
  }
  getAll(query, count) {
    I.readable(this, 'getAll');

    return read(this, 'getAll', I.rangeOf(query, true), I.countOf(count), values);
  }
  getAllKeys(query, count) {
    I.readable(this, 'getAllKeys');

    return read(this, 'getAllKeys', I.rangeOf(query, true), I.countOf(count), keys);
  }
  count(query) {
    const tx = I.readable(this, 'count');
    const r = I.rangeOf(query, true);
    const id = this._meta.id;

    return I.request(this, tx, () => Number(I.ask('count', tx._id, id, r.lo, r.hi, r.flags)));
  }
}
I.expose('IDBObjectStore', IDBObjectStore);
})(globalThis.__idb);
