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
I.upgrading = function (store, name) {
  if (store._meta.deleted) throw I.error('InvalidStateError', what(name) + 'The object store has been deleted.');
  if (store._tx._mode !== 'versionchange') throw I.error('InvalidStateError', what(name) + 'The database is not running a version change transaction.');
  if (store._tx._state !== 'active') throw I.error('TransactionInactiveError', what(name) + 'The transaction is not active.');

  return store._tx;
};
I.cloneFor = function (tx, value, meta) {
  tx._state = 'inactive';

  try {
    const text = I.serialize(value);

    return { text, copy: meta.keyPath === null && meta.indexes.length === 0 ? null : I.deserialize(text) };
  } finally {
    tx._state = 'active';
  }
};
// spec: https://w3c.github.io/IndexedDB/#store-a-record-into-an-object-store
function keysFor(x, value) {
  if (!x.multiEntry) {
    const key = I.extractKey(value, x.keyPath);

    return typeof key === 'string' ? [key] : [];
  }

  const found = I.evaluate(value, x.keyPath);

  if (!Array.isArray(found)) return found === I.NOT_FOUND || I.encodeKey(found) === null ? [] : [I.encodeKey(found)];

  return Array.from(new Set(found.map(I.encodeKey).filter(k => k !== null)));
}
I.indexKeys = function (meta, copy) {
  const out = [];

  for (const x of meta.indexes) for (const key of keysFor(x, copy)) out.push(x.id + ' ' + key);

  return out.join(' ');
};
I.putRecord = function (tx, meta, key, w, overwrite) {
  I.ask('put', tx._id, meta.id, overwrite ? 1 : 0, key, meta.indexes.length ? I.indexKeys(meta, w.copy) : '', w.text);

  return I.decodeKey(key);
};
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

  const w = I.cloneFor(tx, args[0], meta);

  if (path !== null) {
    key = I.extractKey(w.copy, path);

    if (key === null) throw I.error('DataError', what(name) + 'Evaluating the object store\'s key path yielded a value that is not a valid key.');
    if (key === undefined && !meta.autoIncrement) throw I.error('DataError', what(name) + 'Evaluating the object store\'s key path did not yield a value.');
    if (key === undefined && !I.canInject(w.copy, path)) throw I.error('DataError', what(name) + 'A generated key could not be inserted into the value.');
  }

  return I.request(store, tx, () => {
    if (key === undefined) key = generated(tx, meta, w);
    else if (meta.autoIncrement && key[0] === 'b') raiseGenerator(tx, meta, key);

    return I.putRecord(tx, meta, key, w, overwrite);
  });
}
const from = store => ({ tx: store._tx, store: store._meta.id, index: 0 });

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

    return I.readOne(this, from(this), I.rangeOf(query, false), I.picks.value);
  }
  getKey(query) {
    I.need(arguments, 1, "'getKey' on 'IDBObjectStore'");
    I.readable(this, 'getKey');

    return I.readOne(this, from(this), I.rangeOf(query, false), I.picks.primary);
  }
  getAll(query, count) {
    I.readable(this, 'getAll');

    return I.readAll(this, from(this), I.allArgs(query, count), I.picks.value);
  }
  getAllKeys(query, count) {
    I.readable(this, 'getAllKeys');

    return I.readAll(this, from(this), I.allArgs(query, count), I.picks.primary);
  }
  getAllRecords(options) {
    I.readable(this, 'getAllRecords');

    return I.readAll(this, from(this), I.allArgs(options === undefined ? {} : options), I.picks.record);
  }
  count(query) {
    I.readable(this, 'count');

    return I.count(this, from(this), I.rangeOf(query, true));
  }
  openCursor(query, direction) { return I.openCursor(this, from(this), I.readable, 'openCursor', query, direction, false); }
  openKeyCursor(query, direction) { return I.openCursor(this, from(this), I.readable, 'openKeyCursor', query, direction, true); }
  index(name) {
    I.need(arguments, 1, "'index' on 'IDBObjectStore'");

    if (this._meta.deleted) throw I.error('InvalidStateError', what('index') + 'The object store has been deleted.');
    if (this._tx._state === 'finished') throw I.error('InvalidStateError', what('index') + 'The transaction has finished.');

    const x = this._meta.indexes.find(x => x.name === String(name));

    if (!x) throw I.error('NotFoundError', what('index') + 'The specified index was not found.');
    if (!this._indexes.has(x)) this._indexes.set(x, new globalThis.IDBIndex(I.TOKEN, this, x));

    return this._indexes.get(x);
  }
  createIndex(name, keyPath, options) { return I.createIndex(this, arguments); }
  deleteIndex(name) {
    I.need(arguments, 1, "'deleteIndex' on 'IDBObjectStore'");

    const tx = I.upgrading(this, 'deleteIndex');
    const x = this._meta.indexes.find(x => x.name === String(name));

    if (!x) throw I.error('NotFoundError', what('deleteIndex') + 'The specified index was not found.');

    I.ask('delete-index', tx._id, this._meta.id, x.id);
    x.deleted = true;
    this._meta.indexes.splice(this._meta.indexes.indexOf(x), 1);
    this._indexes.delete(x);
  }
}
I.expose('IDBObjectStore', IDBObjectStore);

// spec: https://w3c.github.io/IndexedDB/#dom-idbobjectstore-name
I.renameStore = function (store, name) {
  const tx = I.upgrading(store, 'name');
  const meta = store._meta;
  const stores = tx._db._stores;

  if (meta.name === name) return;
  if (stores.has(name)) throw I.error('ConstraintError', "Failed to set the 'name' property on 'IDBObjectStore': An object store with the specified name already exists.");

  I.ask('rename-store', tx._id, meta.id, I.nameText(name));
  stores.delete(meta.name);
  meta.name = name;
  stores.set(name, meta);
};
})(globalThis.__idb);
