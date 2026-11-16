// spec: https://w3c.github.io/ServiceWorker/#execution-context
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const boot = ak.swWorker();

if (boot === null) return;

const [version, scope, scriptURL, installs] = boot.split('\n');
const ask = ak.swAsk;
const hide = (o, fields) => { for (const k of Object.keys(fields)) Object.defineProperty(o, k, { value: fields[k], writable: true, configurable: true }); };
function expose(name, C) {
  Object.defineProperty(C.prototype, Symbol.toStringTag, { value: name, configurable: true });
  Object.defineProperty(G, name, { value: C, writable: true, configurable: true });
}

class ExtendableEvent extends G.Event {
  constructor(type, init) {
    super(type, init);
    hide(this, { _waits: [] });
  }
  waitUntil(promise) {
    if (!this._dispatching && this._waits.length === 0) throw new G.DOMException('The event is not being dispatched.', 'InvalidStateError');

    this._waits.push(Promise.resolve(promise));
  }
}
expose('ExtendableEvent', ExtendableEvent);

// Resolves true when every promise the listeners gave settled without a rejection.
async function extended(ev) {
  let ok = true;

  for (let i = 0; i < ev._waits.length; i++) await ev._waits[i].catch(() => { ok = false; });

  return ok;
}

class FetchEvent extends ExtendableEvent {
  constructor(type, init) {
    if (!init || !(init.request instanceof G.Request)) throw new TypeError("Failed to construct 'FetchEvent': member request is required.");

    super(type, init);
    hide(this, { _request: init.request, _clientId: init.clientId || '', _resultingClientId: init.resultingClientId || '', _responded: false, _response: null });
  }
  get request() { return this._request; }
  get clientId() { return this._clientId; }
  get resultingClientId() { return this._resultingClientId; }
  get replacesClientId() { return ''; }
  get handled() { return Promise.resolve(); }
  get preloadResponse() { return Promise.resolve(undefined); }
  respondWith(response) {
    if (!this._dispatching || this._responded) throw new G.DOMException('respondWith() was already called or the event is not being dispatched.', 'InvalidStateError');

    this._responded = true;
    this._response = Promise.resolve(response);
    this._waits.push(this._response);
    this.stopImmediatePropagation();
  }
}
expose('FetchEvent', FetchEvent);

class ExtendableMessageEvent extends ExtendableEvent {
  constructor(type, init) {
    super(type, init);
    init = init || {};
    hide(this, { _data: init.data === undefined ? null : init.data, _origin: init.origin || '', _source: init.source || null, _ports: Object.freeze(init.ports || []) });
  }
  get data() { return this._data; }
  get origin() { return this._origin; }
  get lastEventId() { return ''; }
  get source() { return this._source; }
  get ports() { return this._ports; }
}
expose('ExtendableMessageEvent', ExtendableMessageEvent);

class Client {
  constructor(id, url) {
    hide(this, { _id: id, _url: url });
  }
  get id() { return String(this._id); }
  get url() { return this._url; }
  get type() { return 'window'; }
  get frameType() { return 'top-level'; }
  postMessage(message) {
    ask('post', this._id, ak.serialize(message));
  }
}
class WindowClient extends Client {
  get focused() { return false; }
  get visibilityState() { return 'visible'; }
  focus() { return Promise.resolve(this); }
  navigate() { return Promise.reject(new TypeError('The client cannot be navigated.')); }
}
expose('Client', Client);
expose('WindowClient', WindowClient);

function clientList(all) {
  const f = ask('clients', version, all ? 1 : 0);
  const out = [];

  for (let i = 0; i + 2 < f.length; i += 3) out.push(new WindowClient(f[i], f[i + 2]));

  return out;
}
class Clients {
  matchAll(options) {
    return new Promise(resolve => resolve(clientList(!!(options && options.includeUncontrolled))));
  }
  get(id) {
    return new Promise(resolve => resolve(clientList(true).find(c => c.id === String(id))));
  }
  claim() {
    return new Promise(resolve => resolve(void ask('claim', version)));
  }
  openWindow() { return Promise.reject(new TypeError('This worker may not open a window.')); }
}
expose('Clients', Clients);

function ServiceWorkerGlobalScope() {
  throw new TypeError('Illegal constructor');
}
ServiceWorkerGlobalScope.prototype = Object.create(Object.getPrototypeOf(G), { constructor: { value: ServiceWorkerGlobalScope, writable: true, configurable: true }, [Symbol.toStringTag]: { value: 'ServiceWorkerGlobalScope', configurable: true } });
Object.setPrototypeOf(G, ServiceWorkerGlobalScope.prototype);
Object.defineProperty(G, 'ServiceWorkerGlobalScope', { value: ServiceWorkerGlobalScope, writable: true, configurable: true });

const clients = new Clients();

Object.defineProperty(G, 'clients', { get() { return clients; }, configurable: true });
Object.defineProperty(G, 'registration', { get() { return ak.swRegistration(scope); }, configurable: true });
Object.defineProperty(G, 'serviceWorker', { get() { return G.registration.installing || G.registration.waiting || G.registration.active; }, configurable: true });
G.skipWaiting = () => new Promise(resolve => resolve(void ask('skip-waiting', version)));
// spec: https://html.spec.whatwg.org/multipage/workers.html#dom-workerglobalscope-importscripts
G.importScripts = function (...urls) {
  for (const u of urls) {
    const xhr = new G.XMLHttpRequest();

    xhr.open('GET', new G.URL(String(u), scriptURL).href, false);
    xhr.send();

    if (xhr.status < 200 || xhr.status > 299) throw new G.DOMException("The script at '" + u + "' failed to load.", 'NetworkError');

    (0, eval)(xhr.responseText);
  }
};
ak.defineHandlers(G, ['install', 'activate', 'fetch', 'message', 'messageerror']);

// spec: https://w3c.github.io/ServiceWorker/#handle-fetch
function answer(id, ev) {
  if (!ev._responded) return ak.swDone(id, 1, 0, '', '');

  ev._response.then(async res => {
    if (!(res instanceof G.Response) || res.type === 'error' || res.bodyUsed) return ak.swDone(id, 2, 0, '', '');

    const bytes = new Uint8Array(await res.arrayBuffer());
    let headers = '';
    let body = '';

    for (const [k, v] of res.headers) headers += k + ': ' + v + '\n';
    for (let i = 0; i < bytes.length; i += 4096) body += String.fromCharCode.apply(null, bytes.subarray(i, i + 4096));

    ak.swDone(id, 3, res.status, headers, body);
  }).catch(() => ak.swDone(id, 2, 0, '', ''));
}
function requestOf(url, method, headers, body, destination, mode) {
  const init = { method, headers: new G.Headers(), mode: mode === 'navigate' ? 'same-origin' : mode };

  for (const line of headers.split('\n')) if (line.includes(':')) init.headers.append(line.slice(0, line.indexOf(':')), line.slice(line.indexOf(':') + 1).trim());

  if (method !== 'GET' && method !== 'HEAD') init.body = Uint8Array.from(body, c => c.charCodeAt(0));

  const request = new G.Request(url, init);

  Object.defineProperty(request, 'destination', { value: destination, configurable: true });
  Object.defineProperty(request, 'mode', { value: mode, configurable: true });

  return request;
}
ak.swHandler(function (kind, a, b, c, d, e, f, g, h) {
  if (kind === 'message') return void G.dispatchEvent(new ExtendableMessageEvent('message', { data: ak.deserialize(a), origin: G.location.origin, source: new WindowClient(String(b), '') }));

  const ev = new FetchEvent('fetch', { request: requestOf(b, c, d, e, f, g), clientId: h ? String(h) : '', cancelable: true });

  G.dispatchEvent(ev);
  answer(a, ev);
});

// spec: https://w3c.github.io/ServiceWorker/#installation-algorithm
ak.swActivate = async function () {
  const ev = new ExtendableEvent('activate');

  G.dispatchEvent(ev);
  await extended(ev);
  ask('activated', version);
};
if (installs === '1') {
  ak.setTimer(async () => {
    const ev = new ExtendableEvent('install');

    ask('adopt', version);

    if (!ak.swBroken()) G.dispatchEvent(ev);

    ask('installed', version, !ak.swBroken() && await extended(ev) ? 1 : 0);
  }, 0, false, [], false);
}
})();
