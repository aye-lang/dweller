// spec: https://html.spec.whatwg.org/multipage/server-sent-events.html
(function (G, h) {
  const EVENT_SOURCE = 7, PREFIX = '\u0001es';
  const CONNECTING = 0, OPEN = 1, CLOSED = 2;
  const sources = new Map();
  let nextSource = 1;

  function invalid(url) {
    const message = "Failed to construct 'EventSource': Cannot open an EventSource to '" + url + "'. The URL is invalid.";
    if (typeof G.DOMException === 'function') return new G.DOMException(message, 'SyntaxError');

    return Object.assign(new Error(message), { name: 'SyntaxError', code: 12 });
  }

  class EventSource {
    #id = nextSource++;
    #url;
    #credentials;
    #state = CONNECTING;
    #listeners = new Map();

    constructor(url, init) {
      if (arguments.length < 1) throw new TypeError("Failed to construct 'EventSource': 1 argument required, but only 0 present.");

      const href = h.resolve(String(url));
      if (href === null) throw invalid(url);

      this.#url = href;
      this.#credentials = !!(init !== undefined && init !== null && init.withCredentials);
      this.onopen = null;
      this.onmessage = null;
      this.onerror = null;
      sources.set(this.#id, this);
      h.post(EVENT_SOURCE, JSON.stringify(['open', this.#id, href, this.#credentials]));
    }

    get url() { return this.#url; }
    get withCredentials() { return this.#credentials; }
    get readyState() { return this.#state; }

    addEventListener(type, f) {
      if (!this.#listeners.has(type)) this.#listeners.set(type, []);
      if (f && !this.#listeners.get(type).includes(f)) this.#listeners.get(type).push(f);
    }

    removeEventListener(type, f) {
      const l = this.#listeners.get(type) || [];
      if (l.includes(f)) l.splice(l.indexOf(f), 1);
    }

    dispatchEvent(ev) {
      const on = this['on' + ev.type];
      if (typeof on === 'function') on.call(this, ev);

      for (const f of (this.#listeners.get(ev.type) || []).slice()) typeof f === 'function' ? f.call(this, ev) : f.handleEvent(ev);

      return true;
    }

    close() {
      if (this.#state === CLOSED) return;

      this.#state = CLOSED;
      sources.delete(this.#id);
      h.post(EVENT_SOURCE, JSON.stringify(['close', this.#id]));
    }

    _relayed(state, ev) {
      if (this.#state === CLOSED) return;

      this.#state = state;
      if (state === CLOSED) sources.delete(this.#id);

      this.dispatchEvent(Object.assign(ev, { target: this, currentTarget: this, bubbles: false, cancelable: false }));
    }
  }

  for (const [name, value] of [['CONNECTING', CONNECTING], ['OPEN', OPEN], ['CLOSED', CLOSED]]) {
    Object.defineProperty(EventSource, name, { value, enumerable: true });
    Object.defineProperty(EventSource.prototype, name, { value, enumerable: true });
  }

  Object.defineProperty(G, 'EventSource', { value: EventSource, writable: true, configurable: true });
  Object.defineProperty(G, '__wEventSource', { value: function (line) {
    if (!line.startsWith(PREFIX)) return false;

    const [id, state, ev] = JSON.parse(line.slice(PREFIX.length));
    const source = sources.get(id);
    if (source) source._relayed(state, ev);

    return true;
  }, configurable: true });
})(globalThis, globalThis.__worker);
