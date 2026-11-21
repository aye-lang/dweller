(function (G, h, wire) {
  const MESSAGE = 0, ERROR = 1, CONSOLE = 2, FETCH = 3, FETCH_SAME_ORIGIN = 4, FRAME = 5, FETCH_MODULE = 6, EVENT_SOURCE = 7, WEB_SOCKET = 8;
  const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
  const blobs = URL._blobStore;
  const workers = new Map();
  function fire(target, name, ev) {
    target.dispatchEvent(ev);
    return ev;
  }
  class Worker extends EventTarget {
    constructor(url, options) {
      super();
      if (arguments.length === 0) throw new TypeError("Failed to construct 'Worker': 1 argument required, but only 0 present.");
      const href = new URL(String(url), document.baseURI).href;
      const module = !!options && options.type === 'module';
      const credentials = module && options.credentials !== undefined ? String(options.credentials) : 'same-origin';
      if (!['omit', 'same-origin', 'include'].includes(credentials)) throw new TypeError("Failed to construct 'Worker': The provided value '" + credentials + "' is not a valid enum value of type RequestCredentials.");
      let source;
      if (href.startsWith('blob:')) {
        const blob = blobs.get(href);
        if (!blob) throw new DOMException("Failed to construct 'Worker': Script at '" + href + "' cannot be accessed from origin.", 'SecurityError');
        source = new TextDecoder().decode(blob._array());
      }
      this.onmessage = null;
      this.onmessageerror = null;
      this.onerror = null;
      hidden(this, '_credentials', credentials);
      if (h.refused(href)) {
        hidden(this, '_id', 0);
        setTimeout(() => fire(this, 'error', new Event('error')), 0);
        return;
      }
      hidden(this, '_id', h.start(href, module, source));
      workers.set(this._id, this);
    }
    postMessage(value, transfer) {
      if (!workers.has(this._id)) return;
      h.post(this._id, wire.send({ stage: b => h.stage(this._id, b) }, value, wire.transferOf(transfer)));
    }
    terminate() {
      if (!workers.delete(this._id)) return;
      G.__workerEventSource(this, '["end"]');
      G.__workerWebSocket(this, '["end"]');
      h.terminate(this._id);
    }
  }
  hidden(G, 'Worker', Worker);
  const sharedEntries = new Map();
  class SharedPort extends EventTarget {
    constructor(entry, id) {
      super();
      hidden(this, '_entry', entry);
      hidden(this, '_id', id);
      hidden(this, '_queue', []);
      hidden(this, '_started', false);
      hidden(this, '_onmessage', null);
    }
    get onmessage() { return this._onmessage; }
    set onmessage(f) {
      if (this._onmessage === null && typeof f === 'function') this.addEventListener('message', e => this._onmessage && this._onmessage.call(this, e));
      this._onmessage = typeof f === 'function' ? f : null;
      this.start();
    }
    postMessage(value, transfer) {
      const id = this._entry.id;
      if (!workers.has(id)) return;
      h.post(id, this._id + '\u0002' + wire.send({ stage: b => h.stage(id, b) }, value, wire.transferOf(transfer)));
    }
    start() {
      if (this._started) return;
      this._started = true;
      for (const ev of this._queue.splice(0)) super.dispatchEvent(ev);
    }
    close() { this._entry.ports.delete(this._id); }
    _receive(ev) {
      if (this._started) super.dispatchEvent(ev);
      else this._queue.push(ev);
    }
  }
  class SharedEntry {
    constructor(id, credentials) {
      this.id = id;
      this.ports = new Map();
      this.owners = [];
      this.next = 1;
      hidden(this, '_credentials', credentials);
      hidden(this, '_shared', true);
    }
    dispatchEvent(ev) {
      for (const o of this.owners) o.dispatchEvent(ev.type === 'error' ? new ErrorEvent('error', { message: ev.message, cancelable: true }) : new Event(ev.type));
      return true;
    }
    _deliver(json, n) {
      const at = json.indexOf('\u0002');
      const port = this.ports.get(json.slice(0, at));
      if (!port) return;
      let data;
      try {
        data = wire.receive({ size: i => h.size(this.id, i), fill: (i, b) => h.fill(this.id, i, b) }, json.slice(at + 1), n);
      } catch (e) {
        port._receive(new MessageEvent('messageerror', { data: null }));
        return;
      }
      port._receive(new MessageEvent('message', { data }));
    }
  }
  // spec: https://html.spec.whatwg.org/multipage/workers.html#shared-workers-and-the-sharedworker-interface
  class SharedWorker extends EventTarget {
    constructor(url, options) {
      super();
      if (arguments.length === 0) throw new TypeError("Failed to construct 'SharedWorker': 1 argument required, but only 0 present.");
      const href = new URL(String(url), document.baseURI).href;
      const opts = typeof options === 'string' ? { name: options } : (options || {});
      const name = opts.name === undefined ? '' : String(opts.name);
      const module = opts.type === 'module';
      const credentials = module && opts.credentials !== undefined ? String(opts.credentials) : 'same-origin';
      let source;
      if (href.startsWith('blob:')) {
        const blob = blobs.get(href);
        if (!blob) throw new DOMException("Failed to construct 'SharedWorker': Script at '" + href + "' cannot be accessed from origin.", 'SecurityError');
        source = new TextDecoder().decode(blob._array());
      }
      this.onerror = null;
      if (h.refused(href)) {
        hidden(this, 'port', new SharedPort(new SharedEntry(0, credentials), '0'));
        setTimeout(() => this.dispatchEvent(new Event('error')), 0);
        return;
      }
      const key = href + '\n' + name + '\n' + (module ? 'module' : 'classic');
      let entry = sharedEntries.get(key);
      if (!entry || !workers.has(entry.id)) {
        entry = new SharedEntry(h.start(href, module, source, name), credentials);
        sharedEntries.set(key, entry);
        workers.set(entry.id, entry);
      }
      const id = String(entry.next++);
      const port = new SharedPort(entry, id);
      entry.ports.set(id, port);
      entry.owners.push(this);
      hidden(this, 'port', port);
      h.post(entry.id, '\u0003' + id);
    }
  }
  hidden(G, 'SharedWorker', SharedWorker);
  hidden(G, '__workerDeliver', function (id, kind, json, n) {
    const w = workers.get(id);
    if (!w) return;
    if (kind === FETCH || kind === FETCH_SAME_ORIGIN || kind === FETCH_MODULE) {
      if (json.startsWith('blob:')) {
        const blob = blobs.get(json);
        h.answer(id, blob ? blob._array().buffer : null);
      } else if (kind === FETCH_SAME_ORIGIN && new URL(json).origin !== origin) {
        h.answer(id, null);
      } else {
        h.fetch(id, json, kind === FETCH_MODULE, w._credentials, kind !== FETCH);
      }
      return;
    }
    if (kind === EVENT_SOURCE) {
      G.__workerEventSource(w, json, line => h.post(id, line));
      return;
    }
    if (kind === WEB_SOCKET) {
      G.__workerWebSocket(w, json, line => h.post(id, line));
      return;
    }
    if (kind === FRAME) {
      const f = JSON.parse(json), pixels = new ArrayBuffer(h.size(id, 0));
      h.fill(id, 0, pixels);
      G.__placeholderFrame(f.id, f.w, f.h, pixels);
      return;
    }
    if (kind === CONSOLE) {
      const [level, text] = JSON.parse(json);
      console[level](text);
      return;
    }
    if (kind === ERROR) {
      const ev = fire(w, 'error', new ErrorEvent('error', { message: json, cancelable: true }));
      if (!ev.defaultPrevented) console.error(json);
      return;
    }
    if (w._shared) return w._deliver(json, n);
    let data;
    try {
      data = wire.receive({ size: i => h.size(id, i), fill: (i, b) => h.fill(id, i, b) }, json, n);
    } catch (e) {
      fire(w, 'messageerror', new MessageEvent('messageerror', { data: null }));
      return;
    }
    fire(w, 'message', new MessageEvent('message', { data }));
  });
})(globalThis, globalThis.__workerHost, globalThis.__wire);
