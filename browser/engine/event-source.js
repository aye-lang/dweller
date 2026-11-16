// spec: https://html.spec.whatwg.org/multipage/server-sent-events.html
// chromium: third_party/blink/renderer/modules/eventsource/event_source.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const CONNECTING = 0, OPEN = 1, CLOSED = 2;
const DEFAULT_RETRY = 3000;
const REFUSED = /access denied|blocked|CORS/i;
const dispatchTrusted = EventTarget.prototype.dispatchEvent;

function utf8(r) {
  if (r.raw === undefined) return r.body;

  return new TextDecoder().decode(Uint8Array.from(r.raw, c => c.charCodeAt(0)));
}

function streamType(r) {
  return String(r.type || '').split(';')[0].trim().toLowerCase();
}

class EventSource extends EventTarget {
  constructor(url, init) {
    super();
    if (arguments.length < 1) throw new TypeError("Failed to construct 'EventSource': 1 argument required, but only 0 present.");

    let href;
    try {
      href = new URL(String(url), document.baseURI).href;
    } catch (e) {
      throw new DOMException("Failed to construct 'EventSource': Cannot open an EventSource to '" + url + "'. The URL is invalid.", 'SyntaxError');
    }

    hidden(this, '_url', href);
    hidden(this, '_credentials', !!(init !== undefined && init !== null && init.withCredentials));
    hidden(this, '_state', CONNECTING);
    hidden(this, '_task', 0);
    hidden(this, '_retry', DEFAULT_RETRY);
    hidden(this, '_lastEventId', '');
    hidden(this, '_onopen', null);
    this._connect();
  }

  get url() { return this._url; }
  get withCredentials() { return this._credentials; }
  get readyState() { return this._state; }
  get onopen() { return this._onopen; }
  set onopen(f) {
    if (!this._opening) this.addEventListener('open', e => typeof this._onopen === 'function' && this._onopen.call(this, e));
    hidden(this, '_opening', true);
    this._onopen = typeof f === 'function' ? f : null;
  }

  close() {
    if (this._task !== 0) ak.clearTimer(this._task);
    this._task = 0;
    this._state = CLOSED;
  }

  _connect() {
    let lines = 'accept: text/event-stream\ncache-control: no-cache';
    if (this._lastEventId !== '') lines += '\nlast-event-id: ' + this._lastEventId;

    const credentials = this._credentials ? 'include' : 'same-origin';
    this._task = ak.fetchStart(this._url, 'GET', lines, undefined, credentials, 'cors', x => {
      this._task = 0;
      if (this._state !== CLOSED) this._answer(typeof x === 'number' ? ak.fetchTake(x) : x);
    }, '');
  }

  _answer(r) {
    if (r.error !== undefined) {
      if (REFUSED.test(r.error)) this._fail();
      else this._reconnect();
      return;
    }

    if (r.status !== 200 || streamType(r) !== 'text/event-stream') {
      this._fail();
      return;
    }

    this._state = OPEN;
    this._fire(new Event('open'));
    this._read(utf8(r), new URL(r.url || this._url).origin);
    if (this._state !== CLOSED) this._reconnect();
  }

  _read(body, origin) {
    const [, id, , retry, dispatched, events] = ak.eventStream(body, this._lastEventId, false, true);
    if (retry >= 0) this._retry = retry;

    for (let i = 0; i < events.length && this._state !== CLOSED; i += 3) {
      this._lastEventId = events[i + 2];
      this._fire(new MessageEvent(events[i], { data: events[i + 1], origin, lastEventId: events[i + 2] }));
    }

    if (dispatched) this._lastEventId = id;
  }

  _reconnect() {
    this._state = CONNECTING;
    this._fire(new Event('error'));
    if (this._state === CLOSED) return;

    this._task = ak.setTimer(() => {
      this._task = 0;
      if (this._state === CONNECTING) this._connect();
    }, this._retry, false, [], false);
  }

  _fire(ev) {
    ev.isTrusted = true;
    dispatchTrusted.call(this, ev);
  }

  _fail() {
    this._state = CLOSED;
    this._fire(new Event('error'));
  }
}

for (const [name, value] of [['CONNECTING', CONNECTING], ['OPEN', OPEN], ['CLOSED', CLOSED]]) {
  Object.defineProperty(EventSource, name, { value, enumerable: true });
  Object.defineProperty(EventSource.prototype, name, { value, enumerable: true });
}

hidden(G, 'EventSource', EventSource);
})();
