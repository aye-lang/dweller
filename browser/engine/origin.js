// spec: https://html.spec.whatwg.org/multipage/browsers.html#the-origin-interface
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const TUPLE_SCHEMES = ['http:', 'https:', 'ws:', 'wss:', 'ftp:'];
const BLOB_INNER = ['http:', 'https:'];
const NEW = Symbol('origin');

function notAnOrigin(what) {
  return new TypeError("Failed to execute 'from' on 'Origin': " + what);
}

class Origin {
  #scheme = '';
  #host = '';
  #port = '';
  #opaque = true;

  constructor(token) {
    if (token !== NEW) return;
  }

  static #tuple(scheme, host, port) {
    const o = new Origin(NEW);
    o.#opaque = false;
    o.#scheme = scheme;
    o.#host = host;
    o.#port = port;
    return o;
  }

  static #ofUrl(u) {
    if (u.protocol === 'blob:') {
      let inner = null;
      try { inner = new URL(u.pathname); } catch (e) { inner = null; }
      return inner && BLOB_INNER.includes(inner.protocol) ? Origin.#ofUrl(inner) : new Origin(NEW);
    }
    if (!TUPLE_SCHEMES.includes(u.protocol)) return new Origin(NEW);

    return Origin.#tuple(u.protocol.slice(0, -1), u.hostname, u.port);
  }

  static #ofString(s) {
    let u;
    try { u = new URL(s); } catch (e) { throw notAnOrigin("'" + s + "' is not a valid URL."); }

    return Origin.#ofUrl(u);
  }

  static #ofSerialized(s) {
    return s === 'null' ? new Origin(NEW) : Origin.#ofString(s);
  }

  static from(value) {
    if (typeof value === 'string') return Origin.#ofString(value);
    if (value instanceof Origin) return value.#opaque ? value : Origin.#tuple(value.#scheme, value.#host, value.#port);
    if (typeof URL === 'function' && value instanceof URL) return Origin.#ofUrl(value);
    if ((value instanceof HTMLAnchorElement || value instanceof HTMLAreaElement) && value.hasAttribute('href')) return Origin.#ofString(value.href);
    if (typeof SVGAElement === 'function' && value instanceof SVGAElement && (value.hasAttribute('href') || value.hasAttributeNS('http://www.w3.org/1999/xlink', 'href'))) return Origin.#ofString(new URL(value.href.baseVal, document.baseURI).href);
    if (typeof MessageEvent === 'function' && value instanceof MessageEvent) return Origin.#ofSerialized(value.origin);
    if (value !== null && typeof value === 'object' && (value === G || isLocation(value) || isWindowProxy(value))) {
      let serialized;
      try { serialized = isLocation(value) ? value.origin : value.location.origin; } catch (e) { throw notAnOrigin('the window is not same-origin.'); }
      return Origin.#ofSerialized(serialized);
    }

    throw notAnOrigin('the value cannot be converted to an origin.');
  }

  get opaque() { return this.#opaque; }

  isSameOrigin(other) {
    if (!(other instanceof Origin)) throw new TypeError("Failed to execute 'isSameOrigin' on 'Origin': parameter 1 is not of type 'Origin'.");
    if (this.#opaque || other.#opaque) return this === other;

    return this.#scheme === other.#scheme && this.#host === other.#host && this.#port === other.#port;
  }

  isSameSite(other) {
    if (!(other instanceof Origin)) throw new TypeError("Failed to execute 'isSameSite' on 'Origin': parameter 1 is not of type 'Origin'.");
    if (this.#opaque || other.#opaque) return this === other;

    return this.#scheme === other.#scheme && ak.siteOfHost(this.#host) === ak.siteOfHost(other.#host);
  }
}

function isLocation(v) {
  return typeof Location === 'function' ? v instanceof Location : v === G.location;
}

function isWindowProxy(v) {
  try { return v.window === v && v.self === v; } catch (e) { return false; }
}

Object.defineProperty(G, 'Origin', { value: Origin, writable: true, configurable: true });
})();
