(function (G, h, wire) {
  const MESSAGE = 0, ERROR = 1, CONSOLE = 2;
  const def = (name, value) => Object.defineProperty(G, name, { value, writable: true, configurable: true });
  const listeners = new Map();
  const timers = new Map();
  let nextTimer = 1;
  const relayedSource = G.__wEventSource;
  const relayedSocket = G.__wWebSocket;
  const relayed = line => relayedSource(line) || relayedSocket(line);
  delete G.__wEventSource;
  delete G.__wWebSocket;
  function describe(e) {
    return e instanceof Error ? e.name + ': ' + e.message : String(e);
  }
  function report(e) {
    h.post(ERROR, 'Uncaught ' + describe(e));
  }
  function dispatch(ev) {
    const on = G['on' + ev.type];
    try {
      if (typeof on === 'function') on.call(G, ev);
    } catch (e) {
      report(e);
    }
    for (const f of (listeners.get(ev.type) || []).slice()) {
      try {
        typeof f === 'function' ? f.call(G, ev) : f.handleEvent(ev);
      } catch (e) {
        report(e);
      }
    }
  }
  function bytes(s) {
    const u = new Uint8Array(s.length);
    h.latin1Into(u, s);
    return u;
  }
  function loadScript(url) {
    try {
      return h.load(url);
    } catch (e) {
      const msg = "Failed to execute 'importScripts' on 'WorkerGlobalScope': The script at '" + url + "' failed to load.";
      throw typeof G.DOMException === 'function' ? new G.DOMException(msg, 'NetworkError') : Object.assign(new Error(msg), { name: 'NetworkError' });
    }
  }
  class WorkerLocation {
    constructor(href) { this.href = href; }
    get hash() { return this.href.includes('#') ? this.href.slice(this.href.indexOf('#')) : ''; }
    get origin() { return new URL(this.href).origin; }
    get protocol() { return new URL(this.href).protocol; }
    get host() { return new URL(this.href).host; }
    get hostname() { return new URL(this.href).hostname; }
    get port() { return new URL(this.href).port; }
    get pathname() { return new URL(this.href).pathname; }
    get search() { return new URL(this.href).search; }
    toString() { return this.href; }
  }
  if (typeof G.URL !== 'function') def('URL', G.__makeWorkerUrl(h));
  delete G.__makeWorkerUrl;
  function WorkerGlobalScope() { throw new TypeError('Illegal constructor'); }
  function DedicatedWorkerGlobalScope() { throw new TypeError('Illegal constructor'); }
  WorkerGlobalScope.prototype = Object.create(Object.getPrototypeOf(G), { constructor: { value: WorkerGlobalScope, writable: true, configurable: true } });
  DedicatedWorkerGlobalScope.prototype = Object.create(WorkerGlobalScope.prototype, { constructor: { value: DedicatedWorkerGlobalScope, writable: true, configurable: true }, [Symbol.toStringTag]: { value: 'DedicatedWorkerGlobalScope', configurable: true } });
  Object.setPrototypeOf(G, DedicatedWorkerGlobalScope.prototype);
  def('WorkerGlobalScope', WorkerGlobalScope);
  def('DedicatedWorkerGlobalScope', DedicatedWorkerGlobalScope);
  def('self', G);
  def('location', new WorkerLocation(h.href));
  def('origin', new URL(h.href).origin);
  def('onmessage', null);
  def('onmessageerror', null);
  def('onerror', null);
  def('postMessage', function postMessage(value, transfer) {
    h.post(MESSAGE, wire.send(h, value, wire.transferOf(transfer)));
  });
  def('addEventListener', function (type, f) {
    if (!listeners.has(type)) listeners.set(type, []);
    if (f && !listeners.get(type).includes(f)) listeners.get(type).push(f);
  });
  def('removeEventListener', function (type, f) {
    const l = listeners.get(type) || [];
    if (l.includes(f)) l.splice(l.indexOf(f), 1);
  });
  def('dispatchEvent', function (ev) { dispatch(ev); return true; });
  def('close', function close() { h.close(); });
  def('importScripts', function importScripts(...urls) {
    for (const u of urls) (0, eval)(new TextDecoder().decode(bytes(loadScript(String(u)))));
  });
  def('setTimeout', function setTimeout(f, ms, ...args) {
    const id = nextTimer++;
    timers.set(id, { f, args, due: h.now() + Math.max(0, Number(ms) || 0), every: -1 });
    return id;
  });
  def('setInterval', function setInterval(f, ms, ...args) {
    const id = G.setTimeout(f, ms, ...args);
    timers.get(id).every = Math.max(1, Number(ms) || 0);
    return id;
  });
  def('clearTimeout', function clearTimeout(id) { timers.delete(id); });
  def('clearInterval', function clearInterval(id) { timers.delete(id); });
  def('performance', { now: () => h.now(), timeOrigin: Date.now() - h.now() });
  function loaded(url) {
    try {
      return h.load(url);
    } catch (e) {
      return null;
    }
  }
  def('fetch', function fetch(input) {
    return new Promise((resolve, reject) => {
      const url = new URL(String(input && input.url !== undefined ? input.url : input), h.href).href;
      const raw = loaded(url);
      if (raw === null) {
        reject(new TypeError('Failed to fetch'));
        return;
      }
      const text = () => Promise.resolve(new TextDecoder().decode(bytes(raw)));
      resolve({
        ok: true, status: 200, statusText: 'OK', url, type: 'basic', redirected: false,
        headers: { get: () => null, has: () => false },
        text, json: () => text().then(JSON.parse),
        arrayBuffer: () => Promise.resolve(bytes(raw).buffer),
      });
    });
  });
  def('XMLHttpRequest', G.__makeWorkerXhr(h, loaded, bytes));
  delete G.__makeWorkerXhr;
  for (const level of ['log', 'info', 'debug', 'warn', 'error']) {
    console[level] = function (...a) { h.post(CONSOLE, JSON.stringify([level, a.map(String).join(' ')])); };
  }
  def('__wDeliver', function (json, n) {
    if (relayed(json)) return;
    let data;
    try {
      data = wire.receive(h, json, n);
    } catch (e) {
      dispatch({ type: 'messageerror', data: null, target: G });
      return;
    }
    dispatch({ type: 'message', data, target: G, currentTarget: G, origin: '', ports: [], source: null });
  });
  def('__wTick', function () {
    let now = h.now();
    let due;
    while ((due = [...timers].filter(([, t]) => t.due <= now).sort((a, b) => a[1].due - b[1].due || a[0] - b[0])[0])) {
      const [id, t] = due;
      if (t.every >= 0) t.due = now + t.every;
      else timers.delete(id);
      try {
        typeof t.f === 'function' ? t.f(...t.args) : (0, eval)(String(t.f));
      } catch (e) {
        report(e);
      }
      now = h.now();
    }
    let next = -1;
    for (const t of timers.values()) if (next < 0 || t.due - now < next) next = Math.max(0, t.due - now);
    return next;
  });
  delete G.__worker;
})(globalThis, globalThis.__worker, globalThis.__wire);
