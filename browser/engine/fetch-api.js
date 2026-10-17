(function () {
const G = globalThis;
const HP = Headers.prototype;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const own = (o, k) => Object.prototype.hasOwnProperty.call(o, k);
const TOKEN = /^[!#$%&'*+\-.^_`|~0-9A-Za-z]+$/;
const trim = v => String(v).replace(/^[\t\n\r ]+|[\t\n\r ]+$/g, '');

// https://fetch.spec.whatwg.org/#forbidden-request-header
const FORBIDDEN = new Set(['accept-charset', 'accept-encoding', 'access-control-request-headers', 'access-control-request-method', 'connection', 'content-length', 'cookie', 'cookie2', 'date', 'dnt', 'expect', 'host', 'keep-alive', 'origin', 'referer', 'set-cookie', 'te', 'trailer', 'transfer-encoding', 'upgrade', 'via']);
function forbidden(name, value) {
  if (FORBIDDEN.has(name) || name.startsWith('proxy-') || name.startsWith('sec-')) return true;
  if (name !== 'x-http-method' && name !== 'x-http-method-override' && name !== 'x-method-override') return false;

  return value.split(',').some(m => ['CONNECT', 'TRACE', 'TRACK'].includes(trim(m).toUpperCase()));
}

// https://fetch.spec.whatwg.org/#cors-safelisted-request-header
function safelisted(name, value) {
  if (value.length > 128) return false;
  if (name === 'accept') return !/[\x00-\x08\x0a-\x1f"():<>?@\[\\\]{}\x7f]/.test(value);
  if (name === 'accept-language' || name === 'content-language') return /^[0-9A-Za-z *,\-.;=]*$/.test(value);
  if (name !== 'content-type' || /[\x00-\x08\x0a-\x1f"():<>?@\[\\\]{}\x7f]/.test(value)) return false;

  const essence = value.split(';')[0].trim().toLowerCase();

  return ['application/x-www-form-urlencoded', 'multipart/form-data', 'text/plain'].includes(essence);
}

const NO_CORS_NAMES = ['accept', 'accept-language', 'content-language', 'content-type'];
function allowed(guard, name, value) {
  if (guard === 'request') return !forbidden(name, value);
  if (guard === 'response') return name !== 'set-cookie' && name !== 'set-cookie2';
  if (guard === 'request-no-cors') return NO_CORS_NAMES.includes(name) && safelisted(name, value);

  return true;
}

Object.defineProperty(HP, '_guard', {
  set(g) {
    hidden(this, '_guard', g);
    this._list = this._list.filter(p => allowed(g, p[0], p[1]));
  },
  configurable: true,
});

// https://fetch.spec.whatwg.org/#concept-headers-append
function guarded(method, change) {
  const original = HP[method];
  hidden(HP, method, function (name, value) {
    const n = String(name).toLowerCase();
    const v = method === 'delete' ? '' : trim(value);
    if (!TOKEN.test(String(name)) || /[\0\r\n]/.test(v)) return original.apply(this, arguments);

    const guard = own(this, '_guard') ? this._guard : 'none';
    if (guard === 'immutable') throw new TypeError("Failed to execute '" + method + "' on 'Headers': Headers are immutable");
    if (guard === 'request-no-cors' && !change(this, n, v)) return;
    if (guard !== 'request-no-cors' && method !== 'delete' && !allowed(guard, n, v)) return;
    if (guard === 'request' && method === 'delete' && forbidden(n, '')) return;

    return original.apply(this, arguments);
  });
}
guarded('append', (h, n, v) => NO_CORS_NAMES.includes(n) && safelisted(n, h.has(n) ? h.get(n) + ', ' + v : v));
guarded('set', (h, n, v) => NO_CORS_NAMES.includes(n) && safelisted(n, v));
guarded('delete', (h, n) => NO_CORS_NAMES.includes(n));

hidden(HP, 'getSetCookie', function getSetCookie() {
  return this._list.filter(p => p[0] === 'set-cookie').map(p => p[1]);
});

const requestHeaders = Object.getOwnPropertyDescriptor(Request.prototype, 'headers').get;
Object.defineProperty(Request.prototype, 'headers', {
  get() {
    const h = requestHeaders.call(this);
    if (!own(h, '_guard')) h._guard = this.mode === 'no-cors' ? 'request-no-cors' : 'request';

    return h;
  },
  configurable: true,
});

// https://fetch.spec.whatwg.org/#concept-response-type
Object.defineProperty(Response.prototype, 'type', { get() { return own(this, '_type') ? this._type : 'default'; }, configurable: true, enumerable: true });
Object.defineProperty(Response.prototype, 'redirected', { get() { return own(this, '_redirected') && this._redirected; }, configurable: true, enumerable: true });

function headerLines(raw) {
  const h = new Headers();
  for (const line of raw.split('\n')) {
    const colon = line.indexOf(':');
    if (colon <= 0) continue;

    try {
      h.append(line.slice(0, colon), line.slice(colon + 1));
    } catch (e) {
    }
  }

  return h;
}

hidden(Response, 'error', function error() {
  const r = new Response(null, { status: 200 });
  r.status = 0;
  r.ok = false;
  hidden(r, '_type', 'error');
  r.headers._guard = 'immutable';

  return r;
});

// https://fetch.spec.whatwg.org/#dom-response-redirect
hidden(Response, 'redirect', function redirect(url, status) {
  let href;
  try {
    href = new URL(url, document.baseURI).href;
  } catch (e) {
    throw new TypeError("Failed to execute 'redirect' on 'Response': Invalid URL");
  }

  const s = status === undefined ? 302 : Number(status) & 0xffff;
  if (![301, 302, 303, 307, 308].includes(s)) throw new RangeError("Failed to execute 'redirect' on 'Response': Invalid status code");

  const r = new Response(null, { status: 200 });
  r.status = s;
  r.ok = false;
  r.headers.set('location', href);
  r.headers._guard = 'immutable';

  return r;
});

// https://fetch.spec.whatwg.org/#dom-response-json
hidden(Response, 'json', function json(data, init) {
  const body = JSON.stringify(data);
  if (body === undefined) throw new TypeError("Failed to execute 'json' on 'Response': The data is not JSON serializable");

  const options = init === undefined || init === null ? {} : init;
  const headers = new Headers(options.headers);
  if (!headers.has('content-type')) headers.set('content-type', 'application/json');

  return new Response(body, { status: options.status, statusText: options.statusText, headers });
});

// https://fetch.spec.whatwg.org/#cors-check
function corsOk(res, credentials) {
  const origin = res.headers.get('access-control-allow-origin');
  if (origin === null) return false;
  if (origin === '*' && credentials !== 'include') return true;
  if (origin !== location.origin) return false;

  return credentials !== 'include' || res.headers.get('access-control-allow-credentials') === 'true';
}

const CORS_SAFE = ['cache-control', 'content-language', 'content-length', 'content-type', 'expires', 'last-modified', 'pragma'];
function corsFiltered(res, credentials) {
  const exposed = (res.headers.get('access-control-expose-headers') || '').split(',').map(s => s.trim().toLowerCase());
  const all = exposed.includes('*') && credentials !== 'include';
  res.headers._list = res.headers._list.filter(p => CORS_SAFE.includes(p[0]) || exposed.includes(p[0]) || (all && p[0] !== 'set-cookie'));
}

function opaque(res, type, url) {
  hidden(res, '_type', type);
  res.status = 0;
  res.ok = false;
  res.statusText = '';
  res.url = url;
  res.headers._list = [];
  hidden(res, '_body', null);
  hidden(res, '_stream', null);
}

const withoutHash = u => u.split('#')[0];
const local = u => u === '' || /^(data|blob|about):/.test(u);
const originOf = u => local(u) ? location.origin : new URL(u).origin;

// https://fetch.spec.whatwg.org/#main-fetch
function filtered(req, res) {
  const url = withoutHash(res.url || req.url);
  const redirected = url !== withoutHash(req.url);
  if (res._fetched && res._fetched.headers) res.headers = headerLines(res._fetched.headers);
  hidden(res, '_redirected', redirected);
  if (redirected && req.redirect === 'error') throw new TypeError('Failed to fetch: redirect mode is error');

  if (redirected && req.redirect === 'manual') opaque(res, 'opaqueredirect', req.url);
  else if (originOf(url) === location.origin || local(req.url)) hidden(res, '_type', 'basic');
  else if (req.mode === 'same-origin') throw new TypeError('Failed to fetch: mode is same-origin');
  else if (req.mode === 'no-cors') opaque(res, 'opaque', '');
  else {
    if (!corsOk(res, req.credentials)) throw new TypeError('Failed to fetch: CORS check failed');

    corsFiltered(res, req.credentials);
    hidden(res, '_type', 'cors');
  }

  res.headers._guard = 'immutable';

  return res;
}

const fetchOriginal = G.fetch;
hidden(G, 'fetch', function fetch(input, init) {
  let req;
  try {
    req = new Request(input, init);
  } catch (e) {
    return Promise.reject(e);
  }

  if (req.mode === 'same-origin' && !local(req.url) && originOf(req.url) !== location.origin) return Promise.reject(new TypeError('Failed to fetch: mode is same-origin'));

  const p = fetchOriginal(req);

  return req.signal.aborted ? p : p.then(res => filtered(req, res));
});

// https://xhr.spec.whatwg.org/#the-getresponseheader()-method
function xhrHeaders(x) {
  if (x.readyState < 2 || !x._fetched) return null;

  const h = x._fetched.headers ? headerLines(x._fetched.headers) : new Headers(x._type ? [['content-type', x._type]] : []);
  h._guard = 'response';
  if (!local(x.responseURL) && originOf(x.responseURL) !== location.origin) corsFiltered({ headers: h }, x.withCredentials ? 'include' : 'same-origin');

  return h;
}
hidden(XMLHttpRequest.prototype, 'getResponseHeader', function getResponseHeader(name) {
  const h = xhrHeaders(this);

  return h === null ? null : h.get(String(name));
});
hidden(XMLHttpRequest.prototype, 'getAllResponseHeaders', function getAllResponseHeaders() {
  const h = xhrHeaders(this);
  if (h === null) return '';

  let out = '';
  for (const [k, v] of h) out += k + ': ' + v + '\r\n';

  return out;
});
})();
