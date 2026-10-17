(function () {
'use strict';
// v8: src/builtins/regexp-match.tq, regexp-replace.tq, regexp-search.tq, regexp-split.tq, regexp-match-all.tq
const slot = globalThis.__regexpSlot;
delete globalThis.__regexpSlot;
const P = RegExp.prototype;
const builtinExec = P.exec;
const apply = Reflect.apply;
const MAX = 9007199254740991;
function isObject(o) {
  return o !== null && (typeof o === 'object' || typeof o === 'function');
}
function thisObject(o, name) {
  if (!isObject(o)) throw new TypeError('RegExp.prototype.' + name + ' called on a non-object');
  return o;
}
function toStr(v) {
  return `${v}`;
}
function toLength(v) {
  const n = Math.trunc(+v);
  return n !== n || n <= 0 ? 0 : n > MAX ? MAX : n;
}
function set(o, k, v) {
  if (!Reflect.set(o, k, v)) throw new TypeError('Cannot assign to read only property ' + k);
}
function advance(s, i, unicode) {
  if (!unicode || i + 1 >= s.length) return i + 1;
  return i + (s.codePointAt(i) > 0xffff ? 2 : 1);
}
function isConstructor(C) {
  if (typeof C !== 'function') return false;
  try {
    Reflect.construct(String, [], C);
    return true;
  } catch (e) {
    return false;
  }
}
// spec: https://tc39.es/ecma262/#sec-speciesconstructor
function speciesConstructor(o, D) {
  const C = o.constructor;
  if (C === undefined) return D;
  if (!isObject(C)) throw new TypeError('object.constructor is not an object');
  const S = C[Symbol.species];
  if (S === undefined || S === null) return D;
  if (isConstructor(S)) return S;
  throw new TypeError('object.constructor[Symbol.species] is not a constructor');
}
// spec: https://tc39.es/ecma262/#sec-regexpexec
function exec(R, S) {
  const f = R.exec;
  if (typeof f === 'function') {
    const result = apply(f, R, [S]);
    if (result !== null && !isObject(result)) throw new TypeError('RegExp exec method returned something other than an Object or null');
    return result;
  }
  if (slot(R) === undefined) throw new TypeError('RegExp method called on incompatible receiver');
  return apply(builtinExec, R, [S]);
}
function define(name, value) {
  Object.defineProperty(P, name, { value, writable: true, configurable: true });
}
function getter(name, o) {
  Object.defineProperty(P, name, { get: Object.getOwnPropertyDescriptor(o, name).get, configurable: true });
}
const flagNames = [['hasIndices', 'd'], ['global', 'g'], ['ignoreCase', 'i'], ['multiline', 'm'], ['dotAll', 's'], ['unicode', 'u'], ['unicodeSets', 'v'], ['sticky', 'y']];
for (const [name, c] of flagNames) {
  getter(name, { get [name]() {
    thisObject(this, name);
    const flags = slot(this);
    if (flags === undefined) {
      if (this === P) return undefined;
      throw new TypeError('RegExp.prototype.' + name + ' getter called on non-RegExp object');
    }
    return flags.includes(c);
  } });
}
getter('flags', { get flags() {
  thisObject(this, 'flags');
  let r = '';
  for (const [name, c] of flagNames) if (this[name]) r += c;
  return r;
} });
getter('source', { get source() {
  thisObject(this, 'source');
  const src = slot(this, true);
  if (src === undefined) {
    if (this === P) return '(?:)';
    throw new TypeError('RegExp.prototype.source getter called on non-RegExp object');
  }
  if (src === '') return '(?:)';
  let r = '';
  let inClass = false;
  for (let i = 0; i < src.length; i++) {
    const c = src[i];
    if (c === '\\' && i + 1 < src.length) {
      r += c + src[++i];
      continue;
    }
    if (c === '[') inClass = true;
    else if (c === ']') inClass = false;
    r += c === '/' && !inClass ? '\\/' : c === '\n' ? '\\n' : c === '\r' ? '\\r' : c === ' ' ? '\\u2028' : c === ' ' ? '\\u2029' : c;
  }
  return r;
} });
define('toString', { toString() {
  thisObject(this, 'toString');
  return '/' + toStr(this.source) + '/' + toStr(this.flags);
} }.toString);
define(Symbol.match, { [Symbol.match](string) {
  const rx = thisObject(this, '@@match');
  const S = toStr(string);
  const flags = toStr(rx.flags);
  if (!flags.includes('g')) return exec(rx, S);
  const fullUnicode = flags.includes('u') || flags.includes('v');
  set(rx, 'lastIndex', 0);
  const A = [];
  for (;;) {
    const result = exec(rx, S);
    if (result === null) return A.length === 0 ? null : A;
    const m = toStr(result[0]);
    A.push(m);
    if (m === '') set(rx, 'lastIndex', advance(S, toLength(rx.lastIndex), fullUnicode));
  }
} }[Symbol.match]);
define(Symbol.search, { [Symbol.search](string) {
  const rx = thisObject(this, '@@search');
  const S = toStr(string);
  const previous = rx.lastIndex;
  if (!Object.is(previous, 0)) set(rx, 'lastIndex', 0);
  const result = exec(rx, S);
  const current = rx.lastIndex;
  if (!Object.is(current, previous)) set(rx, 'lastIndex', previous);
  return result === null ? -1 : result.index;
} }[Symbol.search]);
// spec: https://tc39.es/ecma262/#sec-getsubstitution
function substitution(matched, str, position, captures, named, template) {
  const m = captures.length;
  const tail = position + matched.length;
  let r = '';
  let i = 0;
  while (i < template.length) {
    const c = template[i];
    const d = template[i + 1];
    if (c !== '$' || d === undefined) {
      r += c;
      i++;
    } else if (d === '$') {
      r += '$';
      i += 2;
    } else if (d === '`') {
      r += str.slice(0, position);
      i += 2;
    } else if (d === '&') {
      r += matched;
      i += 2;
    } else if (d === "'") {
      if (tail < str.length) r += str.slice(tail);
      i += 2;
    } else if (d >= '0' && d <= '9') {
      const d2 = template[i + 2];
      let count = d2 !== undefined && d2 >= '0' && d2 <= '9' ? 2 : 1;
      let index = count === 2 ? +(d + d2) : +d;
      if (index > m && count === 2) {
        count = 1;
        index = +d;
      }
      if (index >= 1 && index <= m) {
        const cap = captures[index - 1];
        if (cap !== undefined) r += cap;
      } else {
        r += template.slice(i, i + 1 + count);
      }
      i += 1 + count;
    } else if (d === '<') {
      const gt = template.indexOf('>', i);
      if (gt === -1 || named === undefined) {
        r += '$<';
        i += 2;
      } else {
        const cap = named[template.slice(i + 2, gt)];
        if (cap !== undefined) r += toStr(cap);
        i = gt + 1;
      }
    } else {
      r += '$';
      i++;
    }
  }
  return r;
}
define(Symbol.replace, { [Symbol.replace](string, replaceValue) {
  const rx = thisObject(this, '@@replace');
  const S = toStr(string);
  const functional = typeof replaceValue === 'function';
  if (!functional) replaceValue = toStr(replaceValue);
  const flags = toStr(rx.flags);
  const global = flags.includes('g');
  const fullUnicode = flags.includes('u') || flags.includes('v');
  if (global) set(rx, 'lastIndex', 0);
  const results = [];
  for (;;) {
    const result = exec(rx, S);
    if (result === null) break;
    results.push(result);
    if (!global) break;
    if (toStr(result[0]) === '') set(rx, 'lastIndex', advance(S, toLength(rx.lastIndex), fullUnicode));
  }
  let acc = '';
  let next = 0;
  for (let k = 0; k < results.length; k++) {
    const result = results[k];
    const nCaptures = Math.max(toLength(result.length) - 1, 0);
    const matched = toStr(result[0]);
    const p = Math.trunc(+result.index);
    const position = p !== p || p < 0 ? 0 : Math.min(p, S.length);
    const captures = [];
    for (let n = 1; n <= nCaptures; n++) {
      const cap = result[n];
      captures.push(cap === undefined ? cap : toStr(cap));
    }
    let named = result.groups;
    let replacement;
    if (functional) {
      const args = [matched, ...captures, position, S];
      if (named !== undefined) args.push(named);
      replacement = toStr(apply(replaceValue, undefined, args));
    } else {
      if (named !== undefined) {
        if (named === null) throw new TypeError('Cannot convert null to object');
        named = Object(named);
      }
      replacement = substitution(matched, S, position, captures, named, replaceValue);
    }
    if (position >= next) {
      acc += S.slice(next, position) + replacement;
      next = position + matched.length;
    }
  }
  return next >= S.length ? acc : acc + S.slice(next);
} }[Symbol.replace]);
define(Symbol.split, { [Symbol.split](string, limit) {
  const rx = thisObject(this, '@@split');
  const S = toStr(string);
  const C = speciesConstructor(rx, RegExp);
  const flags = toStr(rx.flags);
  const unicode = flags.includes('u') || flags.includes('v');
  const splitter = new C(rx, flags.includes('y') ? flags : flags + 'y');
  const A = [];
  const lim = limit === undefined ? 4294967295 : limit >>> 0;
  if (lim === 0) return A;
  const size = S.length;
  if (size === 0) {
    if (exec(splitter, S) === null) A.push(S);
    return A;
  }
  let p = 0;
  let q = 0;
  while (q < size) {
    set(splitter, 'lastIndex', q);
    const z = exec(splitter, S);
    const e = z === null ? p : Math.min(toLength(splitter.lastIndex), size);
    if (z === null || e === p) {
      q = advance(S, q, unicode);
      continue;
    }
    A.push(S.slice(p, q));
    if (A.length === lim) return A;
    p = e;
    const n = Math.max(toLength(z.length) - 1, 0);
    for (let i = 1; i <= n; i++) {
      A.push(z[i]);
      if (A.length === lim) return A;
    }
    q = p;
  }
  A.push(S.slice(p, size));
  return A;
} }[Symbol.split]);
const IteratorPrototype = Object.getPrototypeOf(Object.getPrototypeOf([][Symbol.iterator]()));
const state = new WeakMap();
const StringIteratorPrototype = Object.create(IteratorPrototype);
Object.defineProperty(StringIteratorPrototype, 'next', { value: { next() {
  const s = isObject(this) ? state.get(this) : undefined;
  if (s === undefined) throw new TypeError('%RegExpStringIterator%.next called on incompatible receiver');
  if (s.done) return { value: undefined, done: true };
  const match = exec(s.R, s.S);
  if (match === null) {
    s.done = true;
    return { value: undefined, done: true };
  }
  if (!s.global) {
    s.done = true;
    return { value: match, done: false };
  }
  if (toStr(match[0]) === '') set(s.R, 'lastIndex', advance(s.S, toLength(s.R.lastIndex), s.unicode));
  return { value: match, done: false };
} }.next, writable: true, configurable: true });
Object.defineProperty(StringIteratorPrototype, Symbol.toStringTag, { value: 'RegExp String Iterator', configurable: true });
define(Symbol.matchAll, { [Symbol.matchAll](string) {
  const R = thisObject(this, '@@matchAll');
  const S = toStr(string);
  const C = speciesConstructor(R, RegExp);
  const flags = toStr(R.flags);
  const matcher = new C(R, flags);
  set(matcher, 'lastIndex', toLength(R.lastIndex));
  const iterator = Object.create(StringIteratorPrototype);
  state.set(iterator, { R: matcher, S, global: flags.includes('g'), unicode: flags.includes('u') || flags.includes('v'), done: false });
  return iterator;
} }[Symbol.matchAll]);
const punctuators = ',-=<>#&!%:;@~\'`"';
const controls = { '\t': 't', '\n': 'n', '\v': 'v', '\f': 'f', '\r': 'r' };
function hex(c, width) {
  return c.toString(16).padStart(width, '0');
}
function isSpace(c) {
  return c === 0x20 || c === 0xa0 || c === 0xfeff || c === 0x1680 || (c >= 0x2000 && c <= 0x200a) || c === 0x2028 || c === 0x2029 || c === 0x202f || c === 0x205f || c === 0x3000;
}
// spec: https://tc39.es/proposal-regex-escaping/
function escapeChar(ch) {
  if ('^$\\.*+?()[]{}|/'.includes(ch)) return '\\' + ch;
  if (controls[ch] !== undefined) return '\\' + controls[ch];
  const c = ch.codePointAt(0);
  const lone = ch.length === 1 && c >= 0xd800 && c <= 0xdfff;
  if (!punctuators.includes(ch) && !isSpace(c) && !lone) return ch;
  if (c <= 0xff) return '\\x' + hex(c, 2);
  let r = '';
  for (let i = 0; i < ch.length; i++) r += '\\u' + hex(ch.charCodeAt(i), 4);
  return r;
}
Object.defineProperty(RegExp, 'escape', { value: { escape(S) {
  if (typeof S !== 'string') throw new TypeError('RegExp.escape requires a string');
  let r = '';
  let first = true;
  for (const ch of S) {
    r += first && /[0-9A-Za-z]/.test(ch) ? '\\x' + hex(ch.charCodeAt(0), 2) : escapeChar(ch);
    first = false;
  }
  return r;
} }.escape, writable: true, configurable: true });
})();
