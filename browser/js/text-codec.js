(function (data) {
'use strict';
const G = globalThis;
const { ENCODING_LABELS, SINGLE_BYTE_INDEXES } = data;
const natives = G.__codec;
delete G.__codec;
const { utf8Length, utf8Encode, utf8Decode, legacyDecode } = natives;
const LEGACY = new Set(['GBK', 'gb18030', 'Big5', 'EUC-JP', 'ISO-2022-JP', 'Shift_JIS', 'EUC-KR']);
const CHUNK = 8192;

function hidden(o, k, v) {
  Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
}

// spec: https://encoding.spec.whatwg.org/#concept-encoding-get
function encodingFor(label) {
  const key = String(label).replace(/^[\t\n\f\r ]+|[\t\n\f\r ]+$/g, '').toLowerCase();
  return Object.prototype.hasOwnProperty.call(ENCODING_LABELS, key) ? ENCODING_LABELS[key] : null;
}

const singleByteTables = new Map();

function singleByteTable(name) {
  let table = singleByteTables.get(name);
  if (table === undefined) {
    const hex = SINGLE_BYTE_INDEXES[name];
    table = new Uint16Array(128);
    for (let i = 0; i < 128; i++) table[i] = parseInt(hex.substr(i * 4, 4), 16);
    singleByteTables.set(name, table);
  }
  return table;
}

function bytesOf(input) {
  if (input === undefined) return new Uint8Array(0);
  if (input instanceof ArrayBuffer || (typeof SharedArrayBuffer === 'function' && input instanceof SharedArrayBuffer)) return new Uint8Array(input);
  if (ArrayBuffer.isView(input)) return new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
  throw new TypeError("Failed to execute 'decode' on 'TextDecoder': The provided value is not of type '(ArrayBuffer or ArrayBufferView)'.");
}

function concat(a, b) {
  if (a.length === 0) return b;
  const out = new Uint8Array(a.length + b.length);
  out.set(a);
  out.set(b, a.length);
  return out;
}

function fromUnits(units) {
  let s = '';
  for (let i = 0; i < units.length; i += CHUNK) s += String.fromCharCode.apply(null, units.slice(i, i + CHUNK));
  return s;
}

function fail(name) {
  throw new TypeError("Failed to execute 'decode' on 'TextDecoder': The encoded data was not valid for encoding " + name);
}

function utf8Tail(b) {
  for (let back = 1; back <= 3 && back <= b.length; back++) {
    const lead = b[b.length - back];
    if ((lead & 0xC0) === 0x80) continue;
    const need = lead >= 0xF0 && lead <= 0xF4 ? 4 : lead >= 0xE0 && lead <= 0xEF ? 3 : lead >= 0xC2 && lead <= 0xDF ? 2 : 0;
    if (need <= back) return 0;
    if (back === 1) return 1;
    const second = back > 1 ? b[b.length - back + 1] : 0x80;
    const low = lead === 0xE0 ? 0xA0 : lead === 0xF0 ? 0x90 : 0x80;
    const high = lead === 0xED ? 0x9F : lead === 0xF4 ? 0x8F : 0xBF;
    return second >= low && second <= high ? back : 0;
  }
  return 0;
}

function decodeUtf16(b, bigEndian, fatal, stream, state) {
  const bytes = b;
  let usable = bytes.length - (bytes.length % 2);
  const unit = i => bigEndian ? (bytes[i] << 8) | bytes[i + 1] : (bytes[i + 1] << 8) | bytes[i];
  if (stream && usable >= 2) {
    const last = unit(usable - 2);
    if (last >= 0xD800 && last <= 0xDBFF) usable -= 2;
  }
  const units = [];
  let tailHigh = false;
  for (let i = 0; i < usable; i += 2) {
    const u = unit(i);
    if (u >= 0xD800 && u <= 0xDBFF && i + 2 < usable && unit(i + 2) >= 0xDC00 && unit(i + 2) <= 0xDFFF) {
      units.push(u, unit(i + 2));
      i += 2;
    } else if (u >= 0xD800 && u <= 0xDFFF) {
      if (fatal) fail('utf-16');
      tailHigh = u <= 0xDBFF && i + 2 >= usable;
      units.push(0xFFFD);
    } else {
      units.push(u);
    }
  }
  state.pending = bytes.subarray(usable);
  if (!stream && state.pending.length > 0) {
    if (fatal) fail('utf-16');
    if (!tailHigh) units.push(0xFFFD);
  }
  return fromUnits(units);
}

function decodeSingleByte(name, b, fatal) {
  const table = singleByteTable(name);
  const units = new Array(b.length);
  for (let i = 0; i < b.length; i++) {
    const v = b[i] < 0x80 ? b[i] : table[b[i] - 0x80];
    if (v === 0 && b[i] >= 0x80) {
      if (fatal) fail(name);
      units[i] = 0xFFFD;
    } else {
      units[i] = v;
    }
  }
  return fromUnits(units);
}

function decodeUserDefined(b) {
  const units = new Array(b.length);
  for (let i = 0; i < b.length; i++) units[i] = b[i] < 0x80 ? b[i] : 0xF780 + b[i] - 0x80;
  return fromUnits(units);
}

function decodeBody(name, bytes, fatal, stream, state) {
  if (name === 'UTF-8') {
    const hold = stream ? utf8Tail(bytes) : 0;
    state.pending = bytes.subarray(bytes.length - hold);
    const s = utf8Decode(bytes.subarray(0, bytes.length - hold), fatal);
    if (s === null) fail('utf-8');
    return s;
  }
  if (name === 'UTF-16LE' || name === 'UTF-16BE') return decodeUtf16(bytes.subarray(0), name === 'UTF-16BE', fatal, stream, state);
  if (name === 'x-user-defined') return decodeUserDefined(bytes);
  if (LEGACY.has(name)) {
    const hold = stream && bytes.length > 0 && bytes[bytes.length - 1] >= 0x81 ? 1 : 0;
    state.pending = bytes.subarray(bytes.length - hold);
    const s = legacyDecode(name, bytes.subarray(0, bytes.length - hold));
    if (fatal && s.includes('�')) fail(name);
    return s;
  }
  return decodeSingleByte(name, bytes, fatal);
}

const BOM_ENCODINGS = new Set(['UTF-8', 'UTF-16LE', 'UTF-16BE']);

class TextDecoder {
  #name; #fatal; #ignoreBOM; #state = { pending: new Uint8Array(0) }; #bomSeen = false;
  constructor(label = 'utf-8', options = {}) {
    const name = encodingFor(label);
    if (name === null || name === 'replacement') throw new RangeError("Failed to construct 'TextDecoder': The encoding label provided ('" + label + "') is invalid.");
    if (options !== undefined && options !== null && typeof options !== 'object' && typeof options !== 'function') throw new TypeError("Failed to construct 'TextDecoder': The provided value is not of type 'TextDecoderOptions'.");
    this.#name = name;
    this.#fatal = !!(options && options.fatal);
    this.#ignoreBOM = !!(options && options.ignoreBOM);
  }
  get encoding() { return this.#name.toLowerCase(); }
  get fatal() { return this.#fatal; }
  get ignoreBOM() { return this.#ignoreBOM; }
  decode(input, options = {}) {
    if (options !== undefined && options !== null && typeof options !== 'object' && typeof options !== 'function') throw new TypeError("Failed to execute 'decode' on 'TextDecoder': The provided value is not of type 'TextDecodeOptions'.");
    const stream = !!(options && options.stream);
    const chunk = bytesOf(input);
    let s;
    try {
      s = decodeBody(this.#name, concat(this.#state.pending, chunk), this.#fatal, stream, this.#state);
    } catch (e) {
      this.#state.pending = new Uint8Array(0);
      this.#bomSeen = false;
      throw e;
    }
    if (BOM_ENCODINGS.has(this.#name) && !this.#bomSeen && s.length > 0) {
      this.#bomSeen = true;
      if (!this.#ignoreBOM && s.charCodeAt(0) === 0xFEFF) s = s.slice(1);
    }
    if (!stream) {
      this.#state.pending = new Uint8Array(0);
      this.#bomSeen = false;
    }
    return s;
  }
}

class TextEncoder {
  get encoding() { return 'utf-8'; }
  encode(input = '') {
    const s = String(input);
    const u = new Uint8Array(utf8Length(s));
    utf8Encode(u, s);
    return u;
  }
  encodeInto(source, destination) {
    const s = String(source);
    if (!(destination instanceof Uint8Array)) throw new TypeError("Failed to execute 'encodeInto' on 'TextEncoder': parameter 2 is not of type 'Uint8Array'.");
    let read = 0;
    let written = 0;
    while (read < s.length) {
      const cp = s.codePointAt(read);
      const units = cp > 0xFFFF ? 2 : 1;
      const code = cp >= 0xD800 && cp <= 0xDFFF ? 0xFFFD : cp;
      const size = code < 0x80 ? 1 : code < 0x800 ? 2 : code < 0x10000 ? 3 : 4;
      if (written + size > destination.length) break;
      const piece = this.encode(String.fromCodePoint(code));
      destination.set(piece, written);
      written += size;
      read += units;
    }
    return { read, written };
  }
}

for (const C of [TextDecoder, TextEncoder]) {
  Object.defineProperty(C.prototype, Symbol.toStringTag, { value: C.name, configurable: true });
  for (const k of Object.getOwnPropertyNames(C.prototype)) {
    if (k !== 'constructor') Object.defineProperty(C.prototype, k, { enumerable: true });
  }
}
hidden(G, 'TextEncoder', TextEncoder);
hidden(G, 'TextDecoder', TextDecoder);
})
