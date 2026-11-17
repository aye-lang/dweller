// spec: https://websockets.spec.whatwg.org/
// chromium: third_party/blink/renderer/modules/websockets/dom_websocket.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const CONNECTING = 0, OPEN = 1, CLOSING = 2, CLOSED = 3;
const OPENED = 0, TEXT = 1, BINARY = 2;
const TOKEN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
const dispatchTrusted = EventTarget.prototype.dispatchEvent;
const fail = (what, name) => new DOMException("Failed to " + what, name);

function socketUrl(url) {
  let u;
  try {
    u = new URL(String(url), document.baseURI);
  } catch (e) {
    throw fail("construct 'WebSocket': The URL '" + url + "' is invalid.", 'SyntaxError');
  }

  if (u.protocol === 'http:') u.protocol = 'ws:';
  if (u.protocol === 'https:') u.protocol = 'wss:';
  if (u.protocol !== 'ws:' && u.protocol !== 'wss:') throw fail("construct 'WebSocket': The URL's scheme must be either 'http', 'https', 'ws', or 'wss'. '" + u.protocol.slice(0, -1) + "' is not allowed.", 'SyntaxError');
  if (u.hash !== '' || u.href.endsWith('#')) throw fail("construct 'WebSocket': The URL contains a fragment identifier ('" + u.hash.slice(1) + "'). Fragment identifiers are not allowed in WebSocket URLs.", 'SyntaxError');

  return u.href;
}

function offered(protocols) {
  if (protocols === undefined) return [];
  if (protocols !== null && typeof protocols === 'object' && typeof protocols[Symbol.iterator] !== 'function') return offered(protocols.protocols);

  const list = typeof protocols === 'string' || protocols === null || typeof protocols[Symbol.iterator] !== 'function' ? [String(protocols)] : Array.from(protocols, String);

  for (const [i, p] of list.entries()) {
    if (!TOKEN.test(p)) throw fail("construct 'WebSocket': The subprotocol '" + p + "' is invalid.", 'SyntaxError');
    if (list.findIndex(q => q.toLowerCase() === p.toLowerCase()) !== i) throw fail("construct 'WebSocket': The subprotocol '" + p + "' is duplicated.", 'SyntaxError');
  }

  return list;
}

function wellFormed(s) {
  let out = '';
  for (let i = 0; i < s.length; i++) {
    const c = s.charCodeAt(i);
    const paired = c >= 0xd800 && c < 0xdc00 && s.charCodeAt(i + 1) >= 0xdc00 && s.charCodeAt(i + 1) < 0xe000;
    if (paired) out += s[i] + s[++i];
    else out += c >= 0xd800 && c < 0xe000 ? '\ufffd' : s[i];
  }

  return out;
}

function binaryString(data) {
  if (data instanceof Blob) return data._bytes;
  if (data instanceof ArrayBuffer || (typeof SharedArrayBuffer !== 'undefined' && data instanceof SharedArrayBuffer)) return ak.latin1Of(new Uint8Array(data));
  if (ArrayBuffer.isView(data)) return ak.latin1Of(new Uint8Array(data.buffer, data.byteOffset, data.byteLength));

  return null;
}

function received(binary, type) {
  const u = new Uint8Array(binary.length);
  ak.latin1Into(u, binary);

  return type === 'blob' ? new Blob([u]) : u.buffer;
}

class WebSocket extends EventTarget {
  constructor(url, protocols) {
    super();
    if (arguments.length < 1) throw new TypeError("Failed to construct 'WebSocket': 1 argument required, but only 0 present.");

    const href = socketUrl(url);
    const list = offered(protocols);

    hidden(this, '_url', href);
    hidden(this, '_state', CONNECTING);
    hidden(this, '_protocol', '');
    hidden(this, '_binaryType', 'blob');
    hidden(this, '_unsent', 0);
    hidden(this, '_handlers', {});
    hidden(this, '_id', ak.socketOpen(href, list.join(', '), (kind, data, clean, code) => this._note(kind, data, clean, code)));
    if (this._id === 0) ak.setTimer(() => this._note(CLOSED, '', false, 1006), 0, false, [], false);
  }

  get url() { return this._url; }
  get readyState() { return this._state; }
  get protocol() { return this._protocol; }
  get extensions() { return ''; }
  get bufferedAmount() { return this._unsent + (this._id === 0 ? 0 : ak.socketBuffered(this._id)); }
  get binaryType() { return this._binaryType; }
  set binaryType(v) {
    if (v === 'blob' || v === 'arraybuffer') this._binaryType = v;
  }

  send(data) {
    if (arguments.length < 1) throw new TypeError("Failed to execute 'send' on 'WebSocket': 1 argument required, but only 0 present.");
    if (this._state === CONNECTING) throw fail("execute 'send' on 'WebSocket': Still in CONNECTING state.", 'InvalidStateError');

    const binary = binaryString(data);
    const text = binary === null ? wellFormed(String(data)) : binary;
    const size = binary === null ? new TextEncoder().encode(text).length : binary.length;

    this._unsent += size;
    if (this._state !== OPEN) return;

    ak.socketSend(this._id, text, binary !== null);
    ak.setTimer(() => { this._unsent -= size; }, 0, false, [], false);
  }

  close(code, reason) {
    if (code !== undefined) {
      code = Math.min(Math.max(Math.trunc(Number(code)) || 0, 0), 65535);
      if (code !== 1000 && (code < 3000 || code > 4999)) throw fail("execute 'close' on 'WebSocket': The close code must be either 1000, or between 3000 and 4999. " + code + " is neither.", 'InvalidAccessError');
    }

    reason = reason === undefined ? '' : wellFormed(String(reason));
    if (new TextEncoder().encode(reason).length > 123) throw fail("execute 'close' on 'WebSocket': The close reason must not be greater than 123 UTF-8 bytes.", 'SyntaxError');
    if (this._state === CLOSING || this._state === CLOSED) return;

    this._state = CLOSING;
    if (this._id !== 0) ak.socketClose(this._id, code === undefined ? 1005 : code, reason);
  }

  _note(kind, data, clean, code) {
    if (kind === OPENED) {
      if (this._state !== CONNECTING) return;
      this._state = OPEN;
      this._protocol = data;
      this._fire(new Event('open'));
    } else if (kind === TEXT || kind === BINARY) {
      if (this._state !== OPEN) return;
      this._fire(new MessageEvent('message', { data: kind === TEXT ? data : received(data, this._binaryType), origin: new URL(this._url).origin }));
    } else {
      if (this._state === CLOSED) return;
      this._state = CLOSED;
      if (!clean) this._fire(new Event('error'));
      this._fire(new CloseEvent('close', { wasClean: clean, code, reason: data }));
    }
  }

  _fire(ev) {
    ev.isTrusted = true;
    dispatchTrusted.call(this, ev);
  }
}

for (const [name, value] of [['CONNECTING', CONNECTING], ['OPEN', OPEN], ['CLOSING', CLOSING], ['CLOSED', CLOSED]]) {
  Object.defineProperty(WebSocket, name, { value, enumerable: true });
  Object.defineProperty(WebSocket.prototype, name, { value, enumerable: true });
}

for (const type of ['open', 'message', 'error', 'close']) {
  Object.defineProperty(WebSocket.prototype, 'on' + type, {
    configurable: true,
    enumerable: true,
    get() { return this._handlers[type] || null; },
    set(f) {
      if (!(type in this._handlers)) this.addEventListener(type, e => typeof this._handlers[type] === 'function' && this._handlers[type].call(this, e));
      this._handlers[type] = typeof f === 'function' || (typeof f === 'object' && f !== null) ? f : null;
    },
  });
}

hidden(G, 'WebSocket', WebSocket);
})();
