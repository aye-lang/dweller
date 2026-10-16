(function (G) {
  const errors = { Error, EvalError, RangeError, ReferenceError, SyntaxError, TypeError, URIError };
  function fail(v) {
    const msg = (typeof v === 'function' ? 'function' : String(v)) + ' could not be cloned.';
    throw typeof G.DOMException === 'function' ? new G.DOMException(msg, 'DataCloneError') : new TypeError(msg);
  }
  function encode(value, transfer) {
    const list = Array.from(transfer || []);
    for (const b of list) if (!(b instanceof ArrayBuffer) || b.detached) fail(b);
    const seen = new Map();
    const buffers = [];
    function number(v) {
      if (v !== v || v === Infinity || v === -Infinity || Object.is(v, -0)) return { t: 'n', v: String(Object.is(v, -0) ? '-0' : v) };
      return v;
    }
    function enc(v) {
      if (typeof v === 'symbol' || typeof v === 'function') fail(v);
      if (v === undefined) return { t: 'u' };
      if (typeof v === 'number') return number(v);
      if (typeof v === 'bigint') return { t: 'g', v: String(v) };
      if (v === null || typeof v !== 'object') return v;
      if (seen.has(v)) return { t: 'R', id: seen.get(v) };
      const id = seen.size;
      seen.set(v, id);
      const tag = Object.prototype.toString.call(v);
      if (Array.isArray(v)) return { t: 'a', id, n: v.length, p: Object.keys(v).map(k => [k, enc(v[k])]) };
      if (v instanceof Date) return { t: 'd', id, v: number(v.getTime()) };
      if (v instanceof RegExp) return { t: 'r', id, s: v.source, f: v.flags };
      if (v instanceof Map) return { t: 'm', id, e: Array.from(v, ([k, x]) => [enc(k), enc(x)]) };
      if (v instanceof Set) return { t: 's', id, e: Array.from(v, enc) };
      if (v instanceof Error) return { t: 'e', id, n: v.name, m: v.message };
      if (v instanceof ArrayBuffer) {
        buffers.push(v);
        return { t: 'B', id, i: buffers.length - 1 };
      }
      if (ArrayBuffer.isView(v)) {
        const c = v instanceof DataView ? 'DataView' : v.constructor.name;
        return { t: 'V', id, c, b: enc(v.buffer), o: v.byteOffset, n: v instanceof DataView ? v.byteLength : v.length };
      }
      if (tag === '[object Number]' || tag === '[object String]' || tag === '[object Boolean]') return { t: 'w', id, v: enc(v.valueOf()) };
      if (tag !== '[object Object]') fail(v);
      return { t: 'o', id, p: Object.keys(v).map(k => [k, enc(v[k])]) };
    }
    const json = JSON.stringify(enc(value));
    for (const b of list) if (!seen.has(b)) buffers.push(b);
    return { json, buffers, moved: list };
  }
  function decode(json, buffers) {
    const made = [];
    function dec(x) {
      if (x === null || typeof x !== 'object') return x;
      let out;
      switch (x.t) {
        case 'u': return undefined;
        case 'n': return Number(x.v);
        case 'g': return BigInt(x.v);
        case 'R': return made[x.id];
        case 'a': out = made[x.id] = new Array(x.n); for (const [k, v] of x.p) out[k] = dec(v); return out;
        case 'd': return made[x.id] = new Date(dec(x.v));
        case 'r': return made[x.id] = new RegExp(x.s, x.f);
        case 'm': out = made[x.id] = new Map(); for (const [k, v] of x.e) out.set(dec(k), dec(v)); return out;
        case 's': out = made[x.id] = new Set(); for (const v of x.e) out.add(dec(v)); return out;
        case 'e': return made[x.id] = new (errors[x.n] || Error)(x.m);
        case 'B': return made[x.id] = buffers[x.i];
        case 'V': return made[x.id] = new G[x.c](dec(x.b), x.o, x.n);
        case 'w': return made[x.id] = Object(dec(x.v));
        case 'o': out = made[x.id] = {}; for (const [k, v] of x.p) out[k] = dec(v); return out;
      }
      return undefined;
    }
    return dec(JSON.parse(json));
  }
  function send(host, value, transfer) {
    const w = encode(value, transfer);
    for (const b of w.buffers) host.stage(b);
    for (const b of w.moved) b.transfer();
    return w.json;
  }
  function receive(host, json, n) {
    const buffers = [];
    for (let i = 0; i < n; i++) {
      const b = new ArrayBuffer(host.size(i));
      host.fill(i, b);
      buffers.push(b);
    }
    return decode(json, buffers);
  }
  function transferOf(t) {
    if (t === undefined || t === null) return [];
    if (Array.isArray(t) || typeof t[Symbol.iterator] === 'function') return t;
    return t.transfer || [];
  }
  Object.defineProperty(G, '__wire', { value: { send, receive, transferOf }, configurable: true });
})(globalThis);
