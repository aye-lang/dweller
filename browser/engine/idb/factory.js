// spec: https://w3c.github.io/IndexedDB/#factory-interface
(function (I) {
const G = globalThis;
const opens = new Map();
let serial = 0;
const versionEvent = (type, oldVersion, newVersion) => new G.IDBVersionChangeEvent(type, { oldVersion: Number(oldVersion), newVersion: newVersion === '0' || newVersion === null ? null : Number(newVersion) });

function openRequest(name) {
  const req = new G.IDBOpenDBRequest(I.TOKEN);

  I.hide(req, { _n: ++serial, _name: name });

  return req;
}

class IDBFactory {
  constructor(token) {
    I.legal(token);
  }
  open(name, version) {
    I.need(arguments, 1, "'open' on 'IDBFactory'");

    const req = openRequest(String(name));

    if (version !== undefined) version = I.enforceRange(version);
    if (version === 0) throw new TypeError("Failed to execute 'open' on 'IDBFactory': The version provided must not be 0.");

    I.ask('open', req._n, version || 0, JSON.stringify(req._name));
    opens.set(req._n, req);

    return req;
  }
  deleteDatabase(name) {
    I.need(arguments, 1, "'deleteDatabase' on 'IDBFactory'");

    const req = openRequest(String(name));

    I.ask('delete-database', req._n, JSON.stringify(req._name));
    opens.set(req._n, req);

    return req;
  }
  databases() {
    return new Promise(resolve => {
      const lines = I.pairs(I.ask('databases'));

      resolve(lines.filter((_, i) => i % 2 === 1).map((name, i) => ({ name: JSON.parse(name), version: Number(lines[i * 2]) })));
    });
  }
  cmp(first, second) {
    I.need(arguments, 2, "'cmp' on 'IDBFactory'");

    const a = I.toKey(first);
    const b = I.toKey(second);

    return a < b ? -1 : a > b ? 1 : 0;
  }
}
I.expose('IDBFactory', IDBFactory);

const factory = new IDBFactory(I.TOKEN);

Object.defineProperty(G, 'indexedDB', { get() { return factory; }, enumerable: true, configurable: true });

// spec: https://w3c.github.io/IndexedDB/#upgrade-a-database
function upgrade(req, f) {
  const db = I.openDatabase(req._name, [f[2], f[5]].concat(f.slice(6)));
  const tx = I.newTransaction(db, 'versionchange', [], Number(f[3]), true, 'default');

  db._upgrade = tx;
  tx._openRequest = req;
  I.keepUpgradeStart(tx, Number(f[4]));
  req._result = db;
  req._tx = tx;
  req._done = true;
  tx._state = 'inactive';
  I.fireAtRequest(req, versionEvent('upgradeneeded', f[4], f[5]));
  I.schedule(tx);
}
// spec: https://w3c.github.io/IndexedDB/#open-a-database-connection
I.upgradeEnded = function (tx, committed) {
  const req = tx._openRequest;
  const db = tx._db;

  req._tx = null;

  if (!committed) db.close();

  I.task(() => {
    opens.delete(req._n);

    if (committed && !db._closePending) return I.fire([req], I.event('success', false, false));

    req._result = undefined;
    req._error = I.error('AbortError', 'The connection was closed or the version change transaction was aborted.');
    I.fire([req], I.event('error', true, true));
  });
};

function versionChange(f) {
  const db = I.connections.get(Number(f[1]));

  if (!db || db._closePending) return;

  I.fire([db], versionEvent('versionchange', f[2], f[3]));

  if (!db._closePending) I.ask('ack', db._id);
}
function settled(req, f) {
  opens.delete(req._n);
  req._done = true;

  if (f[0] === 'success') {
    req._result = I.openDatabase(req._name, f.slice(2));

    return I.fire([req], I.event('success', false, false));
  }

  if (f[0] === 'deleted') return I.fire([req], versionEvent('success', f[2], null));

  req._error = I.error(f[2], 'The database could not be opened: ' + f[2] + '.');
  I.fire([req], I.event('error', true, true));
}
function onEvent(line) {
  const f = line.split('\n');

  if (f[0] === 'versionchange') return versionChange(f);

  if (f[0] === 'start') {
    const tx = I.waiting.get(Number(f[1]));

    if (!tx) return;

    I.waiting.delete(tx._id);
    tx._started = true;

    return I.schedule(tx);
  }

  const req = opens.get(Number(f[1]));

  if (!req) return;
  if (f[0] === 'upgradeneeded') return upgrade(req, f);
  if (f[0] === 'blocked') return I.fire([req], versionEvent('blocked', f[2], f[3]));

  settled(req, f);
}

I.ak.idbHandler(onEvent, I.cleanup);
delete G.__idb;
})(globalThis.__idb);
