// spec: https://w3c.github.io/IndexedDB/#key-construct
(function () {
const G = globalThis;
const I = {};
Object.defineProperty(G, '__idb', { value: I, configurable: true });
const ak = I.ak = G.__canvasHost;
const DOMException = G.DOMException;
const HEX = '0123456789abcdef';
const own = (o, k) => Object.prototype.hasOwnProperty.call(o, k) || (Array.isArray(o) && Reflect.ownKeys(o).includes(String(k)));
const tagOf = v => Object.prototype.toString.call(v);

I.TOKEN = {};
I.error = (name, message) => new DOMException(message, name);
I.task = fn => ak.setTimer(fn, 0, false, [], false);
I.ask = function (...lines) {
  const reply = ak.idb(lines.join('\n'));

  if (reply[0] === '!') throw I.error(reply.slice(1), 'The operation failed: ' + reply.slice(1) + '.');

  return reply.slice(3);
};

function hexOf(bytes) {
  let out = '';

  for (let i = 0; i < bytes.length; i++) out += HEX[bytes[i] >> 4] + HEX[bytes[i] & 15];

  return out;
}
function unhex(s, from, to) {
  const out = new Uint8Array((to - from) >> 1);

  for (let i = 0; i < out.length; i++) out[i] = parseInt(s.substr(from + i * 2, 2), 16);

  return out;
}
I.ascii = s => s.replace(/[\u0080-\uffff]/g, c => '\\u' + c.charCodeAt(0).toString(16).padStart(4, '0'));
I.nameText = name => I.ascii(JSON.stringify(name));
I.hexOf = hexOf;
I.unhex = unhex;

function numberKey(tag, v) {
  const bytes = new Uint8Array(8);
  const view = new DataView(bytes.buffer);

  view.setFloat64(0, v === 0 ? 0 : v);

  if (bytes[0] & 0x80) for (let i = 0; i < 8; i++) bytes[i] ^= 0xff;
  else bytes[0] |= 0x80;

  return tag + hexOf(bytes);
}
function stringKey(s) {
  let out = 'd';

  for (let i = 0; i < s.length; i++) out += s.charCodeAt(i).toString(16).padStart(4, '0');

  return out + '!';
}
function bytesOf(v) {
  if (ArrayBuffer.isView(v)) return v.buffer.detached ? null : new Uint8Array(v.buffer, v.byteOffset, v.byteLength);

  return v.detached ? null : new Uint8Array(v);
}
function arrayKey(v, seen) {
  if (seen.includes(v)) return null;

  seen.push(v);

  let out = 'f';
  const length = v.length;

  for (let i = 0; i < length; i++) {
    if (!own(v, i)) return null;

    const item = encode(v[i], seen);

    if (item === null) return null;

    out += item;
  }

  return out + '!';
}
// spec: https://w3c.github.io/IndexedDB/#convert-value-to-key
function encode(v, seen) {
  if (typeof v === 'number') return v !== v ? null : numberKey('b', v);
  if (typeof v === 'string') return stringKey(v);
  if (v === null || typeof v !== 'object') return null;

  if (tagOf(v) === '[object Date]') {
    const ms = Date.prototype.getTime.call(v);

    return ms !== ms ? null : numberKey('c', ms);
  }

  if (v instanceof ArrayBuffer || ArrayBuffer.isView(v)) {
    const bytes = bytesOf(v);

    return bytes === null ? null : 'e' + hexOf(bytes) + '!';
  }

  return Array.isArray(v) ? arrayKey(v, seen) : null;
}
function decodeNumber(s, at) {
  const bytes = unhex(s, at.i, at.i + 16);

  at.i += 16;

  if (bytes[0] & 0x80) bytes[0] &= 0x7f;
  else for (let i = 0; i < 8; i++) bytes[i] ^= 0xff;

  return new DataView(bytes.buffer).getFloat64(0);
}
function decodeAt(s, at) {
  const tag = s[at.i++];

  if (tag === 'b') return decodeNumber(s, at);
  if (tag === 'c') return new Date(decodeNumber(s, at));

  const end = tag === 'f' ? -1 : s.indexOf('!', at.i);

  if (tag === 'd') {
    const units = [];

    for (let i = at.i; i < end; i += 4) units.push(parseInt(s.substr(i, 4), 16));

    at.i = end + 1;

    return units.length < 4096 ? String.fromCharCode.apply(null, units) : units.map(u => String.fromCharCode(u)).join('');
  }

  if (tag === 'e') {
    const bytes = unhex(s, at.i, end);

    at.i = end + 1;

    return bytes.buffer;
  }

  const items = [];

  while (s[at.i] !== '!') items.push(decodeAt(s, at));

  at.i++;

  return items;
}

I.encodeKey = v => encode(v, []);
I.decodeKey = s => decodeAt(s, { i: 0 });
I.toKey = function (v) {
  const key = encode(v, []);

  if (key === null) throw I.error('DataError', 'The parameter is not a valid key.');

  return key;
};

// spec: https://w3c.github.io/IndexedDB/#valid-key-path
const IDENTIFIER = /^[\p{L}$_][\p{L}\p{N}\p{Mn}$_‌‍]*$/u;
const validPart = p => p === '' || p.split('.').every(id => IDENTIFIER.test(id));

I.pathFrom = function (v) {
  if (v === undefined || v === null) return null;
  if (typeof v === 'object' && typeof v[Symbol.iterator] === 'function') return Array.from(v, String);

  return String(v);
};
I.validPath = p => typeof p === 'string' ? validPart(p) : p.length > 0 && p.every(validPart);

const NOT_FOUND = {};

// spec: https://w3c.github.io/IndexedDB/#evaluate-a-key-path-on-a-value
function step(value, id) {
  if (typeof value === 'string' && id === 'length') return value.length;
  if (value === null || typeof value !== 'object') return NOT_FOUND;
  if (Array.isArray(value) && id === 'length') return value.length;
  if (value instanceof G.Blob && (id === 'size' || id === 'type')) return value[id];
  if (value instanceof G.File && (id === 'name' || id === 'lastModified')) return value[id];
  if (value instanceof G.File && id === 'lastModifiedDate') return new Date(value.lastModified);
  if (!own(value, id) || value[id] === undefined) return NOT_FOUND;

  return value[id];
}
function evaluate(value, path) {
  if (path === '') return value;

  for (const id of path.split('.')) {
    value = step(value, id);

    if (value === NOT_FOUND) return NOT_FOUND;
  }

  return value;
}
// A key, undefined when the path finds nothing, null when what it finds is not a key.
I.extractKey = function (value, path) {
  if (typeof path === 'string') {
    const found = evaluate(value, path);

    return found === NOT_FOUND ? undefined : encode(found, []);
  }

  const items = [];

  for (const p of path) {
    const found = evaluate(value, p);

    if (found === NOT_FOUND) return undefined;

    items.push(found);
  }

  return encode(items, []);
};
// spec: https://w3c.github.io/IndexedDB/#check-that-a-key-could-be-injected-into-a-value
I.canInject = function (value, path) {
  const ids = path.split('.');

  ids.pop();

  for (const id of ids) {
    if (value === null || typeof value !== 'object') return false;
    if (!own(value, id)) return true;

    value = value[id];
  }

  return value !== null && typeof value === 'object';
};
I.inject = function (value, path, key) {
  const ids = path.split('.');
  const last = ids.pop();

  for (const id of ids) {
    if (!own(value, id)) Object.defineProperty(value, id, { value: {}, writable: true, enumerable: true, configurable: true });

    value = value[id];
  }

  Object.defineProperty(value, last, { value: key, writable: true, enumerable: true, configurable: true });
};
})();
