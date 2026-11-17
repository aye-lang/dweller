// spec: https://w3c.github.io/ServiceWorker/#document-context
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const DOMException = G.DOMException;
const TOKEN = {};
const STATES = ['', 'installing', 'installed', 'activating', 'activated', 'redundant'];
const workers = new Map();
const registrations = new Map();
const readyWaiters = [];
let controller = null;

const hide = (o, fields) => { for (const k of Object.keys(fields)) Object.defineProperty(o, k, { value: fields[k], writable: true, configurable: true }); };
const legal = token => { if (token !== TOKEN) throw new TypeError('Illegal constructor'); };
const task = fn => ak.setTimer(fn, 0, false, [], false);
const fire = (target, type) => task(() => target.dispatchEvent(new G.Event(type)));
function expose(name, C) {
  Object.defineProperty(C.prototype, Symbol.toStringTag, { value: name, configurable: true });
  Object.defineProperty(G, name, { value: C, writable: true, configurable: true });
}
let known = ak.swWorker() !== null;
function ask(...lines) {
  if (!known) {
    known = true;
    ak.idb('sw\ncontrol\n' + ak.swController() + '\n' + G.location.href);
  }

  const reply = ak.idb('sw\n' + lines.join('\n'));

  if (reply[0] === '!') throw new DOMException('The operation is insecure or failed: ' + reply.slice(1) + '.', reply.slice(1));

  return reply === 'ok' ? [] : reply.slice(3).split('\n');
}
ak.swAsk = ask;

class ServiceWorker extends G.EventTarget {
  constructor(token, id, scriptURL) {
    legal(token);
    super();
    hide(this, { _id: id, _scriptURL: scriptURL, _state: 'parsed' });
  }
  get scriptURL() { return this._scriptURL; }
  get state() { return this._state; }
  postMessage(message, transfer) {
    if (arguments.length === 0) throw new TypeError("Failed to execute 'postMessage' on 'ServiceWorker': 1 argument required, but only 0 present.");
    if (this._state === 'redundant') throw new DOMException('The service worker is redundant.', 'InvalidStateError');

    ak.swPost(this._id, ak.serialize(message));
  }
}
ak.defineHandlers(ServiceWorker.prototype, ['statechange', 'error']);
expose('ServiceWorker', ServiceWorker);

function setState(worker, state) {
  if (STATES.indexOf(state) <= STATES.indexOf(worker._state)) return;

  worker._state = state;
  worker.dispatchEvent(new G.Event('statechange'));
}
function workerOf(id, state, scriptURL) {
  if (id === 0) return null;

  if (!workers.has(id)) {
    workers.set(id, new ServiceWorker(TOKEN, id, scriptURL));
    workers.get(id)._state = STATES[state];
  }

  setState(workers.get(id), STATES[state]);

  return workers.get(id);
}

class ServiceWorkerRegistration extends G.EventTarget {
  constructor(token, id, scope) {
    legal(token);
    super();
    hide(this, { _id: id, _scope: scope, _scriptURL: '', _installing: null, _waiting: null, _active: null, _gone: false });
  }
  get installing() { return this._installing; }
  get waiting() { return this._waiting; }
  get active() { return this._active; }
  get scope() { return this._scope; }
  get updateViaCache() { return 'imports'; }
  update() {
    if (!this._installing && !this._waiting && !this._active) return Promise.reject(new DOMException('The registration has no worker to update.', 'InvalidStateError'));

    return install(this._scope, this._scriptURL).then(() => undefined);
  }
  unregister() {
    return new Promise(resolve => resolve(ask('unregister', this._id)[0] === '1'));
  }
}
ak.defineHandlers(ServiceWorkerRegistration.prototype, ['updatefound']);
expose('ServiceWorkerRegistration', ServiceWorkerRegistration);

function settleReady() {
  let mine = null;

  for (const reg of registrations.values()) if (reg._active && G.location.href.startsWith(reg._scope) && (!mine || reg._scope.length > mine._scope.length)) mine = reg;

  if (mine) for (const resolve of readyWaiters.splice(0)) resolve(mine);
}
// A registration and its three workers, as the browser reports them: nine lines.
function applied(f, at) {
  const id = Number(f[at]);

  if (!registrations.has(id)) registrations.set(id, new ServiceWorkerRegistration(TOKEN, id, f[at + 1]));

  const reg = registrations.get(id);
  const before = [reg._installing, reg._waiting, reg._active];

  reg._scriptURL = f[at + 2];
  reg._installing = workerOf(Number(f[at + 3]), Number(f[at + 4]), f[at + 2]);
  reg._waiting = workerOf(Number(f[at + 5]), Number(f[at + 6]), f[at + 2]);
  reg._active = workerOf(Number(f[at + 7]), Number(f[at + 8]), f[at + 2]);

  for (const old of before) if (old && old !== reg._installing && old !== reg._waiting && old !== reg._active) setState(old, 'redundant');

  if (reg._installing && reg._installing !== before[0]) fire(reg, 'updatefound');
  if (!reg._installing && !reg._waiting && !reg._active) registrations.delete(id);

  settleReady();

  return reg;
}
function current(href) {
  const f = ask('registration', href);

  return f.length < 9 ? undefined : applied(f, 0);
}
ak.swRegistration = current;

// spec: https://w3c.github.io/ServiceWorker/#update-algorithm
async function install(scope, scriptURL) {
  const res = await G.fetch(scriptURL, { headers: { 'Service-Worker': 'script' }, credentials: 'same-origin', cache: 'no-cache' });

  if (!res.ok) throw new TypeError("Failed to register a ServiceWorker for scope ('" + scope + "') with script ('" + scriptURL + "'): A bad HTTP response code (" + res.status + ') was received when fetching the script.');
  if (!/^(text|application)\/(x-)?(java|ecma)script/.test(res.headers.get('content-type') || '')) throw new DOMException('The script has an unsupported MIME type.', 'SecurityError');

  const allowed = res.headers.get('service-worker-allowed');
  const widest = allowed === null ? new G.URL('./', scriptURL).href : new G.URL(allowed, scriptURL).href;

  if (!scope.startsWith(widest)) throw new DOMException("The path of the provided scope ('" + scope + "') is not under the max scope allowed ('" + widest + "').", 'SecurityError');

  const f = ask('register', scope, scriptURL, await res.text());
  const reg = applied(f, 0);

  if (f[9] === '1' && !ak.swStart(reg._installing._id, true)) throw new TypeError('The service worker could not be started.');

  return reg;
}
function checked(what, href) {
  const at = new G.URL(href);

  if (at.protocol !== 'http:' && at.protocol !== 'https:') throw new TypeError('The URL protocol of the ' + what + " ('" + href + "') is not supported.");
  if (/%2f|%5c/i.test(at.pathname)) throw new TypeError('The ' + what + " ('" + href + "') must not contain an escaped slash or backslash.");
  if (at.origin !== G.location.origin) throw new DOMException('The origin of the ' + what + " ('" + href + "') does not match the current origin.", 'SecurityError');

  return at.href;
}

class ServiceWorkerContainer extends G.EventTarget {
  constructor(token) {
    legal(token);
    super();
  }
  get controller() { return controller; }
  get ready() {
    return new Promise(resolve => {
      readyWaiters.push(resolve);
      current(G.location.href);
      settleReady();
    });
  }
  register(scriptURL, options) {
    return new Promise(resolve => {
      if (arguments.length === 0) throw new TypeError("Failed to execute 'register' on 'ServiceWorkerContainer': 1 argument required, but only 0 present.");

      const script = checked('script', new G.URL(String(scriptURL), G.document.baseURI).href);
      const scope = options && options.scope !== undefined ? new G.URL(String(options.scope), G.document.baseURI).href : new G.URL('./', script).href;

      resolve(install(checked('scope', scope).split('#')[0], script.split('#')[0]));
    });
  }
  getRegistration(clientURL) {
    return new Promise(resolve => resolve(current(checked('document', new G.URL(clientURL === undefined ? '' : String(clientURL), G.document.baseURI).href))));
  }
  getRegistrations() {
    return new Promise(resolve => {
      const f = ask('registrations');
      const out = [];

      for (let i = 0; i + 8 < f.length; i += 9) out.push(applied(f, i));

      resolve(out);
    });
  }
  startMessages() {}
}
ak.defineHandlers(ServiceWorkerContainer.prototype, ['controllerchange', 'message', 'messageerror']);
expose('ServiceWorkerContainer', ServiceWorkerContainer);

const container = new ServiceWorkerContainer(TOKEN);

Object.defineProperty(G.Navigator.prototype, 'serviceWorker', { get() { return container; }, enumerable: true, configurable: true });

ak.swEvent = function (f) {
  if (f[0] === 'registration') return void applied(f, 1);
  if (f[0] === 'activate') return void (ak.swActivate && ak.swActivate());

  if (f[0] === 'controller') {
    controller = applied(f, 1)._active;
    ak.swControl(controller._id);

    return fire(container, 'controllerchange');
  }

  if (f[0] === 'message') container.dispatchEvent(new G.MessageEvent('message', { data: ak.deserialize(f.slice(1).join('\n')), origin: G.location.origin, source: controller }));
};

if (ak.swController() !== 0) {
  const mine = current(G.location.href);

  controller = mine && mine._active && mine._active._id === ak.swController() ? mine._active : null;
}
})();
