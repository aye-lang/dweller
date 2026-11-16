// spec: https://w3c.github.io/IndexedDB/#database-interface
(function (I) {
const G = globalThis;
const DURABILITY = ['default', 'strict', 'relaxed'];
const what = name => "Failed to execute '" + name + "' on 'IDBDatabase': ";

I.connections = new Map();

function upgradeOf(db, name) {
  const tx = db._upgrade;

  if (!tx) throw I.error('InvalidStateError', what(name) + 'The database is not running a version change transaction.');
  if (tx._state !== 'active') throw I.error('TransactionInactiveError', what(name) + 'The transaction is not active.');

  return tx;
}
function scopeOf(storeNames) {
  if (typeof storeNames === 'string' || storeNames === null || typeof storeNames !== 'object' || typeof storeNames[Symbol.iterator] !== 'function') return [String(storeNames)];

  return Array.from(new Set(Array.from(storeNames, String))).sort();
}
function durabilityOf(options) {
  const d = options === undefined || options === null || options.durability === undefined ? 'default' : String(options.durability);

  if (!DURABILITY.includes(d)) throw new TypeError(what('transaction') + "The provided value '" + d + "' is not a valid enum value of type IDBTransactionDurability.");

  return d;
}

class IDBDatabase extends G.EventTarget {
  constructor(token, id, name, version, stores) {
    I.legal(token);
    super();
    I.hide(this, { _id: id, _name: name, _version: version, _stores: stores, _closePending: false, _upgrade: null, _txs: new Set() });
  }
  get name() { return this._name; }
  get version() { return this._version; }
  get objectStoreNames() { return I.names(Array.from(this._stores.keys())); }
  createObjectStore(name, options) {
    I.need(arguments, 1, "'createObjectStore' on 'IDBDatabase'");

    const tx = upgradeOf(this, 'createObjectStore');
    const keyPath = I.pathFrom(options === undefined || options === null ? null : options.keyPath);
    const autoIncrement = !!(options && options.autoIncrement);

    name = String(name);

    if (keyPath !== null && !I.validPath(keyPath)) throw I.error('SyntaxError', what('createObjectStore') + 'The keyPath option is not a valid key path.');
    if (this._stores.has(name)) throw I.error('ConstraintError', what('createObjectStore') + 'An object store with the specified name already exists.');
    if (autoIncrement && (keyPath === '' || Array.isArray(keyPath))) throw I.error('InvalidAccessError', what('createObjectStore') + 'The autoIncrement option was set but the keyPath option was empty or an array.');

    const id = Number(I.ask('create-store', tx._id, autoIncrement ? 1 : 0, I.nameText(name), I.nameText(keyPath)));
    const meta = { id, name, keyPath, autoIncrement, indexes: [], deleted: false };

    this._stores.set(name, meta);

    return tx.objectStore(name);
  }
  deleteObjectStore(name) {
    I.need(arguments, 1, "'deleteObjectStore' on 'IDBDatabase'");

    const tx = upgradeOf(this, 'deleteObjectStore');
    const meta = this._stores.get(String(name));

    if (!meta) throw I.error('NotFoundError', what('deleteObjectStore') + 'The specified object store was not found.');

    I.ask('delete-store', tx._id, meta.id);
    meta.deleted = true;
    this._stores.delete(meta.name);
    tx._handles.delete(meta);
  }
  transaction(storeNames, mode, options) {
    I.need(arguments, 1, "'transaction' on 'IDBDatabase'");

    if (this._upgrade) throw I.error('InvalidStateError', what('transaction') + 'A version change transaction is running.');
    if (this._closePending) throw I.error('InvalidStateError', what('transaction') + 'The database connection is closing.');

    const scope = scopeOf(storeNames);

    for (const n of scope) if (!this._stores.has(n)) throw I.error('NotFoundError', what('transaction') + 'One of the specified object stores was not found.');

    if (scope.length === 0) throw I.error('InvalidAccessError', what('transaction') + 'The storeNames parameter was empty.');

    mode = mode === undefined ? 'readonly' : String(mode);

    if (mode !== 'readonly' && mode !== 'readwrite') throw new TypeError(what('transaction') + "The provided value '" + mode + "' is not a valid enum value of type IDBTransactionMode.");

    const durability = durabilityOf(options);
    const reply = I.ask('begin', this._id, mode === 'readwrite' ? 1 : 0, ...scope.map(n => this._stores.get(n).id)).split('\n');

    return I.newTransaction(this, mode, scope, Number(reply[0]), reply[1] === '1', durability);
  }
  close() {
    if (this._closePending) return;

    this._closePending = true;
    I.connections.delete(this._id);

    try {
      I.ask('close', this._id);
    } catch (e) {}
  }
}
I.ak.defineHandlers(IDBDatabase.prototype, ['abort', 'close', 'error', 'versionchange']);
I.expose('IDBDatabase', IDBDatabase);

I.openDatabase = function (name, lines) {
  const stores = new Map();

  const byId = new Map();

  for (let i = 2; i < lines.length; i += lines[i] === 'store' ? 5 : 7) {
    if (lines[i] === 'index') {
      byId.get(lines[i + 1]).indexes.push({ id: Number(lines[i + 2]), name: JSON.parse(lines[i + 5]), keyPath: JSON.parse(lines[i + 6]), unique: lines[i + 3] === '1', multiEntry: lines[i + 4] === '1', deleted: false });

      continue;
    }

    const meta = { id: Number(lines[i + 1]), name: JSON.parse(lines[i + 3]), keyPath: JSON.parse(lines[i + 4]), autoIncrement: lines[i + 2] === '1', indexes: [], deleted: false };

    stores.set(meta.name, meta);
    byId.set(lines[i + 1], meta);
  }

  const db = new IDBDatabase(I.TOKEN, Number(lines[0]), name, Number(lines[1]), stores);

  I.connections.set(db._id, db);

  return db;
};
I.keepUpgradeStart = function (tx, oldVersion) {
  tx._before = { version: oldVersion, stores: Array.from(tx._db._stores.values()).map(meta => ({ meta, name: meta.name, indexes: meta.indexes.map(x => ({ x, name: x.name })) })) };
};
// spec: https://w3c.github.io/IndexedDB/#abort-an-upgrade-transaction
I.revertUpgrade = function (tx) {
  const db = tx._db;

  for (const meta of db._stores.values()) {
    meta.deleted = true;

    for (const x of meta.indexes) x.deleted = true;
  }

  db._stores = new Map();
  db._version = tx._before.version;

  for (const s of tx._before.stores) {
    for (const x of s.meta.indexes) x.deleted = true;

    s.meta.deleted = false;
    s.meta.name = s.name;
    s.meta.indexes = s.indexes.map(i => Object.assign(i.x, { name: i.name, deleted: false }));
    db._stores.set(s.name, s.meta);
  }
};
})(globalThis.__idb);
