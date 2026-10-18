(function (G, h, wire) {
  const MESSAGE = 0, ERROR = 1, CONSOLE = 2, FETCH = 3, FETCH_SAME_ORIGIN = 4, FRAME = 5;
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
      let source;
      if (href.startsWith('blob:')) {
        const blob = blobs.get(href);
        if (!blob) throw new DOMException("Failed to construct 'Worker': Script at '" + href + "' cannot be accessed from origin.", 'SecurityError');
        source = new TextDecoder().decode(blob._array());
      }
      this.onmessage = null;
      this.onmessageerror = null;
      this.onerror = null;
      hidden(this, '_id', h.start(href, module, source));
      workers.set(this._id, this);
    }
    postMessage(value, transfer) {
      if (!workers.has(this._id)) return;
      h.post(this._id, wire.send({ stage: b => h.stage(this._id, b) }, value, wire.transferOf(transfer)));
    }
    terminate() {
      if (!workers.delete(this._id)) return;
      h.terminate(this._id);
    }
  }
  hidden(G, 'Worker', Worker);
  hidden(G, '__workerDeliver', function (id, kind, json, n) {
    const w = workers.get(id);
    if (!w) return;
    if (kind === FETCH || kind === FETCH_SAME_ORIGIN) {
      if (json.startsWith('blob:')) {
        const blob = blobs.get(json);
        h.answer(id, blob ? blob._array().buffer : null);
      } else if (kind === FETCH_SAME_ORIGIN && new URL(json).origin !== location.origin) {
        h.answer(id, null);
      } else {
        h.fetch(id, json);
      }
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
