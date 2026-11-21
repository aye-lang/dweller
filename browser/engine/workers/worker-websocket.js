// spec: https://websockets.spec.whatwg.org/
(function (G, h) {
  const WEB_SOCKET = 8, PREFIX = '\u0001ws';
  const CONNECTING = 0, OPEN = 1, CLOSING = 2, CLOSED = 3;
  const sockets = new Map();
  let nextSocket = 1;

  function syntaxError(message) {
    if (typeof G.DOMException === 'function') return new G.DOMException(message, 'SyntaxError');

    return Object.assign(new Error(message), { name: 'SyntaxError', code: 12 });
  }

  function socketUrl(url) {
    let href = h.resolve(String(url));
    if (href === null) throw syntaxError("Failed to construct 'WebSocket': The URL '" + url + "' is invalid.");
    if (href.startsWith('http:')) href = 'ws:' + href.slice(5);
    else if (href.startsWith('https:')) href = 'wss:' + href.slice(6);
    if (!href.startsWith('ws:') && !href.startsWith('wss:')) throw syntaxError("Failed to construct 'WebSocket': The URL's scheme must be either 'http', 'https', 'ws', or 'wss'.");
    if (href.includes('#')) throw syntaxError("Failed to construct 'WebSocket': The URL contains a fragment identifier.");

    return href;
  }

  class WebSocket {
    #id = nextSocket++;
    #url;
    #state = CONNECTING;
    #protocol = '';
    #listeners = new Map();

    constructor(url, protocols) {
      if (arguments.length < 1) throw new TypeError("Failed to construct 'WebSocket': 1 argument required, but only 0 present.");

      this.#url = socketUrl(url);
      const list = protocols === undefined ? [] : (typeof protocols === 'string' ? [protocols] : Array.from(protocols, String));
      this.binaryType = 'blob';
      this.onopen = this.onmessage = this.onerror = this.onclose = null;
      sockets.set(this.#id, this);
      h.post(WEB_SOCKET, JSON.stringify(['open', this.#id, this.#url, list]));
    }

    get url() { return this.#url; }
    get readyState() { return this.#state; }
    get protocol() { return this.#protocol; }
    get extensions() { return ''; }
    get bufferedAmount() { return 0; }

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

    send(data) {
      if (this.#state === CONNECTING) throw new G.DOMException("Failed to execute 'send' on 'WebSocket': Still in CONNECTING state.", 'InvalidStateError');
      if (this.#state !== OPEN) return;

      h.post(WEB_SOCKET, JSON.stringify(['send', this.#id, String(data)]));
    }

    close(code, reason) {
      if (this.#state === CLOSING || this.#state === CLOSED) return;

      this.#state = CLOSING;
      h.post(WEB_SOCKET, JSON.stringify(['close', this.#id, code === undefined ? null : code, reason === undefined ? '' : String(reason)]));
    }

    _relayed(state, protocol, ev) {
      if (this.#state === CLOSED) return;

      this.#state = state;
      this.#protocol = protocol;
      if (state === CLOSED) sockets.delete(this.#id);

      this.dispatchEvent(Object.assign(ev, { target: this, currentTarget: this, bubbles: false, cancelable: false }));
    }
  }

  for (const [name, value] of [['CONNECTING', CONNECTING], ['OPEN', OPEN], ['CLOSING', CLOSING], ['CLOSED', CLOSED]]) {
    Object.defineProperty(WebSocket, name, { value, enumerable: true });
    Object.defineProperty(WebSocket.prototype, name, { value, enumerable: true });
  }

  Object.defineProperty(G, 'WebSocket', { value: WebSocket, writable: true, configurable: true });
  Object.defineProperty(G, '__wWebSocket', { value: function (line) {
    if (!line.startsWith(PREFIX)) return false;

    const [id, state, protocol, ev] = JSON.parse(line.slice(PREFIX.length));
    const socket = sockets.get(id);
    if (socket) socket._relayed(state, protocol, ev);

    return true;
  }, configurable: true });
})(globalThis, globalThis.__worker);
