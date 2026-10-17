// https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#reflecting-content-attributes-in-idl-attributes
(function (ak, table) {
const G = globalThis;
const maxInt = 2147483647;

const parseInteger = s => {
  const m = /^[ \t\n\f\r]*([-+]?)(\d+)/.exec(s);

  if (!m) return null;

  return m[1] === '-' ? -Number(m[2]) || 0 : Number(m[2]);
};

const parseDouble = s => {
  const m = /^[ \t\n\f\r]*([-+]?)(\d+(?:\.\d*)?|\.\d+)(?:[eE]([-+]?\d+))?/.exec(s);
  const v = m && Number(m[2] + (m[3] ? 'e' + m[3] : ''));

  if (!m || !Number.isFinite(v)) return null;

  return m[1] === '-' ? -v || 0 : v;
};

const indexSize = () => new DOMException('The index is not in the allowed range.', 'IndexSizeError');

const numberOf = (el, attr, parse, ok, fallback) => {
  const s = el.getAttribute(attr);
  const v = s === null ? null : parse(s);

  return v !== null && ok(v) ? v : fallback;
};

const enumOf = (el, attr, o) => {
  const s = el.getAttribute(attr);

  if (s === null) return o.isNullable ? null : (o.defaultVal ?? '');

  const lower = s.replace(/[A-Z]/g, c => c.toLowerCase());
  const k = o.keywords.find(w => w.toLowerCase() === lower);

  if (k === undefined) return o.invalidVal ?? o.defaultVal ?? '';

  return o.nonCanon && k in o.nonCanon ? o.nonCanon[k] : k;
};

const finite = v => {
  const n = Number(v);

  if (!Number.isFinite(n)) throw new TypeError('The value is not a finite number.');

  return n;
};

const kinds = {
  string: (a, o) => [function () { return this.getAttribute(a) ?? ''; },
    function (v) { this.setAttribute(a, v === null && o.treatNullAsEmptyString ? '' : String(v)); }],
  url: (a, o) => [function () {
    const s = this.getAttribute(a);

    if (s === null) return o.documentUrl ? ak.url() : '';

    return o.documentUrl && s === '' ? ak.url() : ak.resolve(s);
  }, function (v) { this.setAttribute(a, String(v)); }],
  enum: (a, o) => [function () { return enumOf(this, a, o); },
    function (v) { if (v === null && o.isNullable) this.removeAttribute(a); else this.setAttribute(a, String(v)); }],
  boolean: a => [function () { return this.hasAttribute(a); },
    function (v) { if (v) this.setAttribute(a, ''); else this.removeAttribute(a); }],
  long: (a, o) => [function () { return numberOf(this, a, parseInteger, v => v >= -maxInt - 1 && v <= maxInt, o.defaultVal ?? 0); },
    function (v) { this.setAttribute(a, String(Number(v) | 0)); }],
  'limited long': (a, o) => [function () { return numberOf(this, a, parseInteger, v => v >= 0 && v <= maxInt, o.defaultVal ?? -1); },
    function (v) {
      const n = Number(v) | 0;

      if (n < 0) throw indexSize();

      this.setAttribute(a, String(n));
    }],
  'unsigned long': (a, o) => [function () { return numberOf(this, a, parseInteger, v => v >= 0 && v <= maxInt, o.defaultVal ?? 0); },
    function (v) { const n = Number(v) >>> 0; this.setAttribute(a, String(n > maxInt ? o.defaultVal ?? 0 : n)); }],
  'limited unsigned long': (a, o) => [function () { return numberOf(this, a, parseInteger, v => v > 0 && v <= maxInt, o.defaultVal ?? 1); },
    function (v) {
      const n = Number(v) >>> 0;

      if (n === 0) throw indexSize();

      this.setAttribute(a, String(n > maxInt ? o.defaultVal ?? 1 : n));
    }],
  'limited unsigned long with fallback': (a, o) => [function () { return numberOf(this, a, parseInteger, v => v > 0 && v <= maxInt, o.defaultVal ?? 1); },
    function (v) { const n = Number(v) >>> 0; this.setAttribute(a, String(n === 0 || n > maxInt ? o.defaultVal ?? 1 : n)); }],
  'clamped unsigned long': (a, o) => [function () {
    const v = numberOf(this, a, parseInteger, v => v >= 0, null);

    return v === null ? o.defaultVal : Math.min(Math.max(v, o.min), o.max);
  }, function (v) { const n = Number(v) >>> 0; this.setAttribute(a, String(n > maxInt ? o.defaultVal : n)); }],
  double: (a, o) => [function () { return numberOf(this, a, parseDouble, () => true, o.defaultVal ?? 0); },
    function (v) { this.setAttribute(a, String(finite(v))); }],
  'limited double': (a, o) => [function () { return numberOf(this, a, parseDouble, v => v > 0, o.defaultVal ?? 0); },
    function (v) { const n = finite(v); if (n > 0) this.setAttribute(a, String(n)); }],
};

for (const name in table) {
  const proto = G[name] && G[name].prototype;

  if (!proto) continue;

  for (const [idl, attr, kind, opts] of table[name]) {
    const [get, set] = kinds[kind](attr, opts);

    Object.defineProperty(proto, idl, { get, set, enumerable: true, configurable: true });
  }
}

delete G.__reflection;
})(__ak, __reflection);
