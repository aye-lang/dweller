(function (G) {
  G.__makeWorkerXhr = function (h, loaded, bytes) {
    const UNSENT = 0, OPENED = 1, DONE = 4;
    class XMLHttpRequest {
      constructor() {
        this.readyState = UNSENT; this.status = 0; this.statusText = ''; this.responseType = '';
        this.response = null; this.responseText = ''; this.responseURL = ''; this._on = new Map();
        this.onload = this.onerror = this.onloadend = this.onreadystatechange = null;
      }
      open(method, url, async) {
        this._method = String(method).toUpperCase();
        this._url = new URL(String(url), h.href).href;
        this.readyState = OPENED;
      }
      setRequestHeader() {}
      getResponseHeader() { return null; }
      getAllResponseHeaders() { return ''; }
      overrideMimeType() {}
      abort() { this._aborted = true; }
      addEventListener(type, f) {
        if (!this._on.has(type)) this._on.set(type, []);
        this._on.get(type).push(f);
      }
      removeEventListener(type, f) {
        const l = this._on.get(type) || [];
        if (l.includes(f)) l.splice(l.indexOf(f), 1);
      }
      _fire(type) {
        const ev = { type, target: this, currentTarget: this };
        const on = this['on' + type];
        if (typeof on === 'function') on.call(this, ev);
        for (const f of (this._on.get(type) || []).slice()) typeof f === 'function' ? f.call(this, ev) : f.handleEvent(ev);
      }
      send() {
        if (this.readyState !== OPENED) throw new DOMException('The object is in an invalid state.', 'InvalidStateError');
        G.setTimeout(() => {
          if (this._aborted) return;
          const raw = this._method === 'GET' || this._method === 'HEAD' ? loaded(this._url) : null;
          this.readyState = DONE;
          if (raw === null) {
            this._fire('readystatechange');
            this._fire('error');
            this._fire('loadend');
            return;
          }
          const text = new TextDecoder().decode(bytes(raw));
          this.status = 200; this.statusText = 'OK'; this.responseURL = this._url;
          if (this.responseType === 'json') {
            try { this.response = JSON.parse(text); } catch (e) { this.response = null; }
          } else if (this.responseType === 'arraybuffer') {
            this.response = bytes(raw).buffer;
          } else {
            this.responseText = text; this.response = text;
          }
          this._fire('readystatechange');
          this._fire('load');
          this._fire('loadend');
        }, 0);
      }
    }
    Object.assign(XMLHttpRequest, { UNSENT, OPENED, HEADERS_RECEIVED: 2, LOADING: 3, DONE });
    return XMLHttpRequest;
  };
})(globalThis);
