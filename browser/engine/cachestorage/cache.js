// spec: https://w3c.github.io/ServiceWorker/#cache-objects
(function () {
const G = globalThis;
const TOKEN = {};
const DB = 'dweller:cache-storage';
const hide = (o, fields) => { for (const k of Object.keys(fields)) Object.defineProperty(o, k, { value: fields[k], writable: true, configurable: true }); };
const done = req => new Promise((resolve, reject) => {
  req.onsuccess = () => resolve(req.result);
  req.onerror = e => { e.preventDefault(); reject(req.error); };
});
let opening = null;

function database() {
  if (opening) return opening;

  opening = new Promise((resolve, reject) => {
    const req = G.indexedDB.open(DB, 1);

    req.onupgradeneeded = () => {
      req.result.createObjectStore('caches', { autoIncrement: true }).createIndex('name', 'name', { unique: true });
      req.result.createObjectStore('entries', { autoIncrement: true }).createIndex('cache', 'cache');
    };
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });

  return opening;
}
async function stores(mode) {
  const tx = (await database()).transaction(['caches', 'entries'], mode);

  return { caches: tx.objectStore('caches'), entries: tx.objectStore('entries'), ended: new Promise((resolve, reject) => { tx.oncomplete = () => resolve(); tx.onabort = () => reject(tx.error); }) };
}
function plain(href, ignoreSearch) {
  const at = new G.URL(href);

  at.hash = '';

  if (ignoreSearch) at.search = '';

  return at.href;
}
function queryOf(request) {
  return request instanceof G.Request ? request : new G.Request(request);
}
// spec: https://w3c.github.io/ServiceWorker/#request-matches-cached-item-algorithm
function matches(entry, query, options) {
  if (plain(entry.url, options.ignoreSearch) !== plain(query.url, options.ignoreSearch)) return false;
  if (options.ignoreVary) return true;

  return entry.vary.every(([name, value]) => (query.headers.get(name) || '') === value);
}
function responseOf(entry) {
  const r = entry.response;
  const res = new G.Response(r.body === null ? null : r.body.slice(0), { status: r.status === 0 ? 200 : r.status, statusText: r.statusText, headers: r.headers });

  for (const k of ['url', 'type', 'redirected', 'status', 'ok']) Object.defineProperty(res, k, { value: r[k], configurable: true });

  return res;
}
function requestOf(entry) {
  return new G.Request(entry.url, { headers: entry.requestHeaders });
}
async function found(id, request, options, keys) {
  const s = await stores('readonly');
  const all = await done(keys ? s.entries.index('cache').getAllRecords({ query: id }) : s.entries.index('cache').getAll(id));
  const entry = r => keys ? r.value : r;

  if (request === undefined) return all;

  const query = queryOf(request);

  if (query.method !== 'GET' && !options.ignoreMethod) return [];

  return all.filter(r => matches(entry(r), query, options));
}
// spec: https://w3c.github.io/ServiceWorker/#cache-put
async function entryOf(id, request, response) {
  const query = queryOf(request);
  const vary = (response.headers.get('vary') || '').split(',').map(n => n.trim().toLowerCase()).filter(n => n !== '');

  if (!/^https?:$/.test(new G.URL(query.url).protocol) || query.method !== 'GET') throw new TypeError("Failed to execute 'put' on 'Cache': The request must be a GET of an http or https URL.");
  if (!(response instanceof G.Response)) throw new TypeError("Failed to execute 'put' on 'Cache': parameter 2 is not of type 'Response'.");
  if (response.status === 206) throw new TypeError("Failed to execute 'put' on 'Cache': Partial response (status code 206) is unsupported.");
  if (vary.includes('*')) throw new TypeError("Failed to execute 'put' on 'Cache': Vary header contains *.");
  if (response.bodyUsed) throw new TypeError("Failed to execute 'put' on 'Cache': The response body is already used.");

  const empty = response.body === null;
  const body = await response.arrayBuffer();

  return { cache: id, url: query.url, requestHeaders: Array.from(query.headers), vary: vary.map(n => [n, query.headers.get(n) || '']), response: { status: response.status, statusText: response.statusText, headers: Array.from(response.headers), body: empty ? null : body, url: response.url, type: response.type, redirected: response.redirected, ok: response.ok } };
}
async function store(entries) {
  const s = await stores('readwrite');

  for (const e of entries) {
    const query = new G.Request(e.url, { headers: e.requestHeaders });
    const old = await done(s.entries.index('cache').getAllRecords({ query: e.cache }));

    for (const r of old) if (matches(r.value, query, {})) s.entries.delete(r.primaryKey);

    s.entries.add(e);
  }

  await s.ended;
}
async function fetched(request) {
  const query = queryOf(request);

  if (!/^https?:$/.test(new G.URL(query.url).protocol) || query.method !== 'GET') throw new TypeError("Failed to execute 'add' on 'Cache': The request must be a GET of an http or https URL.");

  const response = await G.fetch(query);

  if (!response.ok) throw new TypeError("Failed to execute 'add' on 'Cache': Request failed.");

  return [query, response];
}

class Cache {
  constructor(token, id) {
    if (token !== TOKEN) throw new TypeError('Illegal constructor');

    hide(this, { _id: id });
  }
  match(request, options) {
    return this.matchAll(request, options).then(list => list[0]);
  }
  async matchAll(request, options) {
    return (await found(this._id, request, options || {}, false)).map(responseOf);
  }
  async add(request) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'add' on 'Cache': 1 argument required, but only 0 present.");

    return this.addAll([request]);
  }
  async addAll(requests) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'addAll' on 'Cache': 1 argument required, but only 0 present.");

    const pairs = await Promise.all(Array.from(requests, fetched));
    const entries = await Promise.all(pairs.map(([q, r]) => entryOf(this._id, q, r)));

    for (let i = 0; i < entries.length; i++) for (let j = 0; j < i; j++) if (matches(entries[j], new G.Request(entries[i].url, { headers: entries[i].requestHeaders }), {})) throw new G.DOMException("Failed to execute 'addAll' on 'Cache': duplicate requests.", 'InvalidStateError');

    await store(entries);
  }
  async put(request, response) {
    if (arguments.length < 2) throw new TypeError("Failed to execute 'put' on 'Cache': 2 arguments required, but only " + arguments.length + ' present.');

    await store([await entryOf(this._id, request, response)]);
  }
  async delete(request, options) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'delete' on 'Cache': 1 argument required, but only 0 present.");

    const hits = await found(this._id, request, options || {}, true);
    const s = await stores('readwrite');

    for (const r of hits) s.entries.delete(r.primaryKey);

    await s.ended;

    return hits.length > 0;
  }
  async keys(request, options) {
    return (await found(this._id, request, options || {}, false)).map(requestOf);
  }
}
function expose(name, C) {
  Object.defineProperty(C.prototype, Symbol.toStringTag, { value: name, configurable: true });
  Object.defineProperty(G, name, { value: C, writable: true, configurable: true });
}
expose('Cache', Cache);

// spec: https://w3c.github.io/ServiceWorker/#cachestorage-interface
class CacheStorage {
  constructor(token) {
    if (token !== TOKEN) throw new TypeError('Illegal constructor');
  }
  async open(name) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'open' on 'CacheStorage': 1 argument required, but only 0 present.");

    const s = await stores('readwrite');
    const id = await done(s.caches.index('name').getKey(String(name)));

    return new Cache(TOKEN, id === undefined ? await done(s.caches.add({ name: String(name) })) : id);
  }
  async has(name) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'has' on 'CacheStorage': 1 argument required, but only 0 present.");

    return (await done((await stores('readonly')).caches.index('name').getKey(String(name)))) !== undefined;
  }
  async delete(name) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'delete' on 'CacheStorage': 1 argument required, but only 0 present.");

    const s = await stores('readwrite');
    const id = await done(s.caches.index('name').getKey(String(name)));

    if (id === undefined) return false;

    s.caches.delete(id);

    for (const key of await done(s.entries.index('cache').getAllKeys(id))) s.entries.delete(key);

    await s.ended;

    return true;
  }
  async keys() {
    return (await done((await stores('readonly')).caches.getAll())).map(c => c.name);
  }
  async match(request, options) {
    options = options || {};

    const s = await stores('readonly');
    const ids = options.cacheName === undefined ? await done(s.caches.getAllKeys()) : [await done(s.caches.index('name').getKey(String(options.cacheName)))];

    for (const id of ids) {
      const hit = id === undefined ? undefined : await new Cache(TOKEN, id).match(request, options);

      if (hit) return hit;
    }

    return undefined;
  }
}
expose('CacheStorage', CacheStorage);

const storage = new CacheStorage(TOKEN);

Object.defineProperty(G, 'caches', { get() { return storage; }, enumerable: true, configurable: true });
})();
