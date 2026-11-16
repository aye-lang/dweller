// spec: https://w3c.github.io/IndexedDB/#index-interface
(function (I) {
const what = name => "Failed to execute '" + name + "' on 'IDBIndex': ";

function readable(index, name) {
  if (index._meta.deleted || index._store._meta.deleted) throw I.error('InvalidStateError', what(name) + 'The index or its object store has been deleted.');
  if (index._store._tx._state !== 'active') throw I.error('TransactionInactiveError', what(name) + 'The transaction is not active.');

  return index._store._tx;
}
const from = index => ({ tx: index._store._tx, store: index._store._meta.id, index: index._meta.id });

class IDBIndex {
  constructor(token, store, meta) {
    I.legal(token);
    I.hide(this, { _store: store, _meta: meta, _path: undefined });
  }
  get name() { return this._meta.name; }
  set name(v) { renameIndex(this, String(v)); }
  get objectStore() { return this._store; }
  get keyPath() {
    if (this._path === undefined) this._path = Array.isArray(this._meta.keyPath) ? this._meta.keyPath.slice() : this._meta.keyPath;

    return this._path;
  }
  get multiEntry() { return this._meta.multiEntry; }
  get unique() { return this._meta.unique; }
  get(query) {
    I.need(arguments, 1, "'get' on 'IDBIndex'");
    readable(this, 'get');

    return I.readOne(this, from(this), I.rangeOf(query, false), I.picks.value);
  }
  getKey(query) {
    I.need(arguments, 1, "'getKey' on 'IDBIndex'");
    readable(this, 'getKey');

    return I.readOne(this, from(this), I.rangeOf(query, false), I.picks.primary);
  }
  getAll(query, count) {
    readable(this, 'getAll');

    return I.readAll(this, from(this), I.allArgs(query, count), I.picks.value);
  }
  getAllKeys(query, count) {
    readable(this, 'getAllKeys');

    return I.readAll(this, from(this), I.allArgs(query, count), I.picks.primary);
  }
  getAllRecords(options) {
    readable(this, 'getAllRecords');

    return I.readAll(this, from(this), I.allArgs(options === undefined ? {} : options), I.picks.record);
  }
  count(query) {
    readable(this, 'count');

    return I.count(this, from(this), I.rangeOf(query, true));
  }
  openCursor(query, direction) { return I.openCursor(this, from(this), readable, 'openCursor', query, direction, false); }
  openKeyCursor(query, direction) { return I.openCursor(this, from(this), readable, 'openKeyCursor', query, direction, true); }
}
I.expose('IDBIndex', IDBIndex);

// spec: https://w3c.github.io/IndexedDB/#dom-idbindex-name
function renameIndex(index, name) {
  const store = index._store;
  const tx = store._tx;

  if (tx._mode !== 'versionchange') throw I.error('InvalidStateError', "Failed to set the 'name' property on 'IDBIndex': The database is not running a version change transaction.");
  if (tx._state !== 'active') throw I.error('TransactionInactiveError', "Failed to set the 'name' property on 'IDBIndex': The transaction is not active.");
  if (index._meta.deleted || store._meta.deleted) throw I.error('InvalidStateError', "Failed to set the 'name' property on 'IDBIndex': The index or its object store has been deleted.");
  if (index._meta.name === name) return;
  if (store._meta.indexes.some(x => x.name === name)) throw I.error('ConstraintError', "Failed to set the 'name' property on 'IDBIndex': An index with the specified name already exists.");

  I.ask('rename-index', tx._id, store._meta.id, index._meta.id, I.nameText(name));
  index._meta.name = name;
}
// The keys of the records the store already holds.
function fill(tx, store) {
  const t = I.walk({ tx, store: store._meta.id, index: 0 }, { lo: '', hi: '', flags: 0 }, 0, 0);

  try {
    for (let i = 0; i < t.length; i += 3) I.ask('set-index-keys', tx._id, store._meta.id, t[i], I.indexKeys(store._meta, I.deserialize(t[i + 2])));
  } catch (e) {
    I.task(() => I.abortTransaction(tx, e));
  }
}
// spec: https://w3c.github.io/IndexedDB/#dom-idbobjectstore-createindex
I.createIndex = function (store, args) {
  I.need(args, 2, "'createIndex' on 'IDBObjectStore'");

  const tx = I.upgrading(store, 'createIndex');
  const name = String(args[0]);
  const keyPath = I.pathFrom(args[1]);
  const unique = !!(args[2] && args[2].unique);
  const multiEntry = !!(args[2] && args[2].multiEntry);
  const error = "Failed to execute 'createIndex' on 'IDBObjectStore': ";

  if (store._meta.indexes.some(x => x.name === name)) throw I.error('ConstraintError', error + 'An index with the specified name already exists.');
  if (keyPath === null || !I.validPath(keyPath)) throw I.error('SyntaxError', error + 'The keyPath argument is not a valid key path.');
  if (multiEntry && Array.isArray(keyPath)) throw I.error('InvalidAccessError', error + 'The keyPath argument was an array and the multiEntry option is true.');

  const id = Number(I.ask('create-index', tx._id, store._meta.id, unique ? 1 : 0, multiEntry ? 1 : 0, I.nameText(name), I.nameText(keyPath)));

  store._meta.indexes.push({ id, name, keyPath, unique, multiEntry, deleted: false });
  fill(tx, store);

  return store.index(name);
};
})(globalThis.__idb);
