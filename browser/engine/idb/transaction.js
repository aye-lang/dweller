// spec: https://w3c.github.io/IndexedDB/#transaction
(function (I) {
const G = globalThis;
const ak = I.ak;
const fresh = [];

I.waiting = new Map();

class IDBTransaction extends G.EventTarget {
  constructor(token, db, mode, scope, id, durability) {
    I.legal(token);
    super();
    I.hide(this, { _db: db, _mode: mode, _scope: scope, _id: id, _durability: durability, _started: true, _state: 'active', _queue: [], _scheduled: false, _error: null, _handles: new Map(), _openRequest: null, _before: null });
  }
  get objectStoreNames() { return I.names(this._mode === 'versionchange' ? Array.from(this._db._stores.keys()) : this._scope); }
  get mode() { return this._mode; }
  get durability() { return this._durability; }
  get db() { return this._db; }
  get error() { return this._error; }
  objectStore(name) {
    I.need(arguments, 1, "'objectStore' on 'IDBTransaction'");

    if (this._state === 'finished') throw I.error('InvalidStateError', "Failed to execute 'objectStore' on 'IDBTransaction': The transaction has finished.");

    const meta = this._db._stores.get(String(name));

    if (!meta || (this._mode !== 'versionchange' && !this._scope.includes(meta.name))) throw I.error('NotFoundError', "Failed to execute 'objectStore' on 'IDBTransaction': The specified object store was not found.");
    if (!this._handles.has(meta)) this._handles.set(meta, new G.IDBObjectStore(I.TOKEN, this, meta));

    return this._handles.get(meta);
  }
  abort() {
    if (this._state === 'committing' || this._state === 'finished') throw I.error('InvalidStateError', "Failed to execute 'abort' on 'IDBTransaction': The transaction has finished.");

    I.abortTransaction(this, null);
  }
  commit() {
    if (this._state !== 'active') throw I.error('InvalidStateError', "Failed to execute 'commit' on 'IDBTransaction': The transaction is not active.");

    this._state = 'committing';
    I.schedule(this);
  }
}
ak.defineHandlers(IDBTransaction.prototype, ['abort', 'complete', 'error']);
I.expose('IDBTransaction', IDBTransaction);

I.newTransaction = function (db, mode, scope, id, started, durability) {
  const tx = new IDBTransaction(I.TOKEN, db, mode, scope, id, durability);

  tx._started = started;
  db._txs.add(tx);

  if (!started) I.waiting.set(id, tx);

  if (mode !== 'versionchange') {
    fresh.push(tx);
    ak.idbFresh();
  }

  return tx;
};
// spec: https://w3c.github.io/IndexedDB/#cleanup-indexed-database-transactions
I.cleanup = function () {
  for (const tx of fresh.splice(0)) {
    if (tx._state === 'active') tx._state = 'inactive';

    I.schedule(tx);
  }
};
I.schedule = function (tx) {
  if (tx._scheduled) return;

  tx._scheduled = true;
  I.task(() => pump(tx));
};
I.request = function (source, tx, op, reused) {
  const req = reused || new G.IDBRequest(I.TOKEN);

  req._source = req._source || source;
  req._tx = tx;
  req._op = op;
  req._done = false;
  tx._queue.push(req);
  I.schedule(tx);

  return req;
};

function finish(tx) {
  tx._state = 'finished';
  tx._db._txs.delete(tx);
  I.waiting.delete(tx._id);

  if (tx._db._upgrade === tx) tx._db._upgrade = null;
}
// spec: https://w3c.github.io/IndexedDB/#commit-a-transaction
function commit(tx) {
  try {
    I.ask('commit', tx._id);
  } catch (e) {
    I.abortTransaction(tx, e);

    return;
  }

  finish(tx);
  I.fire([tx, tx._db], I.event('complete', false, false));

  if (tx._openRequest) I.upgradeEnded(tx, true);
}
// spec: https://w3c.github.io/IndexedDB/#asynchronously-execute-a-request
function pump(tx) {
  tx._scheduled = false;

  if (tx._state === 'finished' || !tx._started) return;

  const req = tx._queue.shift();

  if (!req) return commit(tx);

  let ev = null;

  try {
    req._result = req._op();
    req._error = null;
    ev = I.event('success', false, false);
  } catch (e) {
    req._result = undefined;
    req._error = e;
    ev = I.event('error', true, true);
  }

  req._done = true;
  I.fireAtRequest(req, ev);
  I.schedule(tx);
}
// spec: https://w3c.github.io/IndexedDB/#abort-a-transaction
I.abortTransaction = function (tx, error) {
  if (tx._state === 'finished') return;

  try {
    I.ask('abort', tx._id);
  } catch (e) {
    error = error || e;
  }

  const pending = tx._queue.splice(0);

  finish(tx);
  tx._error = error;

  if (tx._mode === 'versionchange') I.revertUpgrade(tx);

  I.task(() => {
    for (const req of pending) {
      req._done = true;
      req._result = undefined;
      req._error = I.error('AbortError', 'The transaction was aborted, so the request cannot be fulfilled.');
      I.fire([req, tx, tx._db], I.event('error', true, true));
    }

    I.fire([tx, tx._db], I.event('abort', true, false));

    if (tx._openRequest) I.upgradeEnded(tx, false);
  });
};
})(globalThis.__idb);
