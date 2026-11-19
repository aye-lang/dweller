// spec: https://w3c.github.io/IndexedDB/#keyrange
(function (I) {
class IDBKeyRange {
  constructor(token, lower, upper, lowerOpen, upperOpen) {
    I.legal(token);
    I.hide(this, { _lo: lower, _hi: upper, _lowerOpen: lowerOpen, _upperOpen: upperOpen, _lower: undefined, _upper: undefined });

    if (lower !== '') this._lower = I.decodeKey(lower);
    if (upper !== '') this._upper = I.decodeKey(upper);
  }
  get lower() { return this._lower; }
  get upper() { return this._upper; }
  get lowerOpen() { return this._lowerOpen; }
  get upperOpen() { return this._upperOpen; }
  static only(value) {
    I.need(arguments, 1, "'only' on 'IDBKeyRange'");

    const key = I.toKey(value);

    return new IDBKeyRange(I.TOKEN, key, key, false, false);
  }
  static lowerBound(lower, open) {
    I.need(arguments, 1, "'lowerBound' on 'IDBKeyRange'");

    return new IDBKeyRange(I.TOKEN, I.toKey(lower), '', !!open, true);
  }
  static upperBound(upper, open) {
    I.need(arguments, 1, "'upperBound' on 'IDBKeyRange'");

    return new IDBKeyRange(I.TOKEN, '', I.toKey(upper), true, !!open);
  }
  static bound(lower, upper, lowerOpen, upperOpen) {
    I.need(arguments, 2, "'bound' on 'IDBKeyRange'");

    const lo = I.toKey(lower);
    const hi = I.toKey(upper);

    if (lo > hi || (lo === hi && (lowerOpen || upperOpen))) throw I.error('DataError', 'The lower key is greater than the upper key.');

    return new IDBKeyRange(I.TOKEN, lo, hi, !!lowerOpen, !!upperOpen);
  }
  includes(value) {
    I.need(arguments, 1, "'includes' on 'IDBKeyRange'");

    return I.inRange(I.rangeOf(this, false), I.toKey(value));
  }
}
I.expose('IDBKeyRange', IDBKeyRange);

I.inRange = function (r, key) {
  if (r.lo !== '' && (key < r.lo || (key === r.lo && r.flags & 1))) return false;

  return r.hi === '' || key < r.hi || (key === r.hi && !(r.flags & 2));
};
// spec: https://w3c.github.io/IndexedDB/#convert-a-value-to-a-key-range
I.rangeOf = function (query, nullOk) {
  if (query instanceof IDBKeyRange) return { lo: query._lo, hi: query._hi, flags: (query._lowerOpen ? 1 : 0) | (query._upperOpen ? 2 : 0) };

  if (query === undefined || query === null) {
    if (!nullOk) throw I.error('DataError', 'No key or key range specified.');

    return { lo: '', hi: '', flags: 0 };
  }

  const key = I.toKey(query);

  return { lo: key, hi: key, flags: 0 };
};
})(globalThis.__idb);
