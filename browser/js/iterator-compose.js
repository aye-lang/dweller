(function () {
'use strict';
// spec: https://tc39.es/proposal-joint-iteration/ and https://tc39.es/proposal-iterator-sequencing/
const helper = globalThis.__iteratorHelper;
delete globalThis.__iteratorHelper;
const apply = Reflect.apply;
function isObject(o) {
  return o !== null && (typeof o === 'object' || typeof o === 'function');
}
function object(o, what) {
  if (!isObject(o)) throw new TypeError(what + ' is not an object');
  return o;
}
function done() {
  return { value: undefined, done: true };
}
function flattenable(v) {
  object(v, String(typeof v === 'symbol' ? 'Symbol()' : v));
  const m = v[Symbol.iterator];
  const iter = m === undefined || m === null ? v : object(apply(m, v, []), 'iterator');
  return { iter, next: iter.next };
}
function step(r) {
  const res = object(apply(r.next, r.iter, []), 'iterator result');
  return res.done ? undefined : { value: res.value };
}
function close(r, error) {
  if (error !== undefined) {
    try {
      const ret = r.iter.return;
      if (ret !== undefined && ret !== null) apply(ret, r.iter, []);
    } catch (e) {}
    return;
  }
  const ret = r.iter.return;
  if (ret === undefined || ret === null) return;
  if (typeof ret !== 'function') throw new TypeError('iterator.return is not a function');
  object(apply(ret, r.iter, []), 'iterator.return() result');
}
function closeAll(records, error) {
  for (let i = records.length - 1; i >= 0; i--) {
    try {
      close(records[i], error);
    } catch (e) {
      if (error === undefined) error = { e };
    }
  }
  if (error !== undefined) throw error.e;
}
function remove(list, r) {
  list.splice(list.indexOf(r), 1);
}
function options(o) {
  if (o === undefined) return { mode: 'shortest', padding: undefined };
  object(o, 'options');
  let mode = o.mode;
  if (mode === undefined) mode = 'shortest';
  if (mode !== 'shortest' && mode !== 'longest' && mode !== 'strict') throw new TypeError('mode must be "shortest", "longest" or "strict"');
  let padding;
  if (mode === 'longest') {
    padding = o.padding;
    if (padding !== undefined) object(padding, 'padding');
  }
  return { mode, padding };
}
function zipStep(iters, open, mode, padding, finish) {
  return () => {
    const n = iters.length;
    if (n === 0) return done();
    const results = [];
    for (let i = 0; i < n; i++) {
      const r = iters[i];
      if (r === null) {
        results.push(padding[i]);
        continue;
      }
      let res;
      try {
        res = step(r);
      } catch (e) {
        remove(open, r);
        closeAll(open, { e });
      }
      if (res !== undefined) {
        results.push(res.value);
        continue;
      }
      remove(open, r);
      if (mode === 'shortest') {
        closeAll(open);
        return done();
      }
      if (mode === 'strict') {
        if (i !== 0) closeAll(open, { e: new TypeError('Iterator.zip: iterators have different lengths') });
        for (let k = 1; k < n; k++) {
          const other = iters[k];
          let more;
          try {
            more = step(other);
          } catch (e) {
            remove(open, other);
            closeAll(open, { e });
          }
          if (more !== undefined) closeAll(open, { e: new TypeError('Iterator.zip: iterators have different lengths') });
          remove(open, other);
        }
        return done();
      }
      if (open.length === 0) return done();
      iters[i] = null;
      results.push(padding[i]);
    }
    return { value: finish(results), done: false };
  };
}
function zipHelper(iters, mode, padding, finish) {
  const open = iters.slice();
  return helper(null, zipStep(iters, open, mode, padding, finish), () => closeAll(open));
}
function paddingFrom(option, iters) {
  const padding = [];
  if (option === undefined) {
    for (let i = 0; i < iters.length; i++) padding.push(undefined);
    return padding;
  }
  let p;
  try {
    p = flattenable(option);
  } catch (e) {
    closeAll(iters, { e });
  }
  let more = true;
  for (let i = 0; i < iters.length; i++) {
    let res;
    if (more) {
      try {
        res = step(p);
      } catch (e) {
        closeAll(iters, { e });
      }
      if (res === undefined) more = false;
    }
    padding.push(res === undefined ? undefined : res.value);
  }
  if (more) {
    try {
      close(p);
    } catch (e) {
      closeAll(iters, { e });
    }
  }
  return padding;
}
const zip = { zip(iterables) {
  const opts = arguments[1];
  object(iterables, 'iterables');
  const { mode, padding } = options(opts);
  const iters = [];
  const input = flattenable(iterables);
  for (;;) {
    let res;
    try {
      res = step(input);
    } catch (e) {
      closeAll(iters, { e });
    }
    if (res === undefined) break;
    try {
      iters.push(flattenable(res.value));
    } catch (e) {
      closeAll([input, ...iters], { e });
    }
  }
  const pad = mode === 'longest' ? paddingFrom(padding, iters) : [];
  return zipHelper(iters, mode, pad, results => results);
} }.zip;
const zipKeyed = { zipKeyed(iterables) {
  const opts = arguments[1];
  object(iterables, 'iterables');
  const { mode, padding } = options(opts);
  const keys = [];
  const iters = [];
  for (const key of Reflect.ownKeys(iterables)) {
    try {
      const desc = Reflect.getOwnPropertyDescriptor(iterables, key);
      if (desc === undefined || !desc.enumerable) continue;
      const value = iterables[key];
      if (value === undefined) continue;
      iters.push(flattenable(value));
      keys.push(key);
    } catch (e) {
      closeAll(iters, { e });
    }
  }
  const pad = [];
  if (mode === 'longest') {
    for (const key of keys) {
      try {
        pad.push(padding === undefined ? undefined : padding[key]);
      } catch (e) {
        closeAll(iters, { e });
      }
    }
  }
  return zipHelper(iters, mode, pad, results => {
    const o = Object.create(null);
    for (let i = 0; i < keys.length; i++) Object.defineProperty(o, keys[i], { value: results[i], writable: true, enumerable: true, configurable: true });
    return o;
  });
} }.zipKeyed;
const concat = { concat(...items) {
  const sources = [];
  for (let i = 0; i < items.length; i++) {
    const item = object(items[i], 'Iterator.concat argument');
    const method = item[Symbol.iterator];
    if (typeof method !== 'function') throw new TypeError('Iterator.concat argument is not iterable');
    sources.push({ item, method });
  }
  let index = 0;
  let inner = null;
  return helper(null, () => {
    for (;;) {
      if (inner === null) {
        if (index >= sources.length) return done();
        const { item, method } = sources[index++];
        const iter = object(apply(method, item, []), 'iterator');
        inner = { iter, next: iter.next };
      }
      const res = step(inner);
      if (res !== undefined) return { value: res.value, done: false };
      inner = null;
    }
  }, () => {
    if (inner !== null) close(inner);
  });
} }.concat;
for (const f of [zip, zipKeyed, concat]) Object.defineProperty(Iterator, f.name, { value: f, writable: true, configurable: true });
})();
