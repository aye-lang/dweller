(function (G, h, wire) {
  const MESSAGE = 0, ERROR = 1, CONSOLE = 2;
  const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
  const blobs = new Map();
  const workers = new Map();
  const makeUrl = URL.createObjectURL;
  hidden(URL, 'createObjectURL', function createObjectURL(blob) {
    if (!(blob instanceof Blob)) throw new TypeError("Failed to execute 'createObjectURL' on 'URL': Overload resolution failed.");
    const url = makeUrl(blob);
    blobs.set(url, blob);
    return url;
  });
  hidden(URL, 'revokeObjectURL', function revokeObjectURL(url) { blobs.delete(String(url)); });
  const pageFetch = G.fetch;
  hidden(G, 'fetch', function fetch(input, init) {
    const url = typeof input === 'string' || input instanceof URL ? String(input) : null;
    if (url === null || !url.startsWith('blob:')) return pageFetch.call(this, input, init);
    const blob = blobs.get(url.split('#')[0]);
    if (!blob) return Promise.reject(new TypeError('Failed to fetch'));
    return Promise.resolve(new Response(blob, { headers: { 'content-type': blob.type } }));
  });
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
