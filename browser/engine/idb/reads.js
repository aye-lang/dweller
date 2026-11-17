// spec: https://w3c.github.io/IndexedDB/#retrieve-multiple-items-from-an-object-store
(function (I) {
const DIRECTIONS = ['next', 'nextunique', 'prev', 'prevunique'];

class IDBRecord {
  constructor(token, key, primaryKey, value) {
    I.legal(token);
    I.hide(this, { _key: key, _primaryKey: primaryKey, _value: value });
  }
  get key() { return this._key; }
  get primaryKey() { return this._primaryKey; }
  get value() { return this._value; }
}
I.expose('IDBRecord', IDBRecord);

I.DIRECTIONS = DIRECTIONS;
I.directionOf = function (d) {
  const at = d === undefined ? 0 : DIRECTIONS.indexOf(String(d));

  if (at < 0) throw new TypeError("The provided value '" + d + "' is not a valid enum value of type IDBCursorDirection.");

  return at;
};
I.countOf = function (count) {
  if (count === undefined) return 0;

  const n = I.enforceRange(count);

  if (n > 4294967295) throw new TypeError('Value is outside the range of unsigned long.');

  return n;
};
I.pairs = function (reply) {
  return reply === '' ? [] : reply.split('\n');
};
// spec: https://w3c.github.io/IndexedDB/#dictdef-idbgetalloptions
function isOptions(q) {
  return q !== null && typeof q === 'object' && !(q instanceof globalThis.IDBKeyRange) && !Array.isArray(q) && !(q instanceof ArrayBuffer) && !ArrayBuffer.isView(q) && Object.prototype.toString.call(q) !== '[object Date]';
}
I.allArgs = function (query, count) {
  if (!isOptions(query)) return { r: I.rangeOf(query, true), limit: I.countOf(count), dir: 0 };

  return { r: I.rangeOf(query.query, true), limit: I.countOf(query.count), dir: I.directionOf(query.direction) };
};
// What a store or an index reads from: { tx, store, index } with the ids the browser knows.
I.walk = function (from, r, dir, limit, after) {
  after = after || {};

  return I.pairs(I.ask('walk', from.tx._id, from.store, from.index, r.lo, r.hi, r.flags, dir, after.key || '', after.primary || '', after.toKey || '', after.toPrimary || '', after.skip || 0, limit));
};
I.picks = {
  value: (t, i) => I.deserialize(t[i + 2]),
  primary: (t, i) => I.decodeKey(t[i + 1]),
  record: (t, i) => new IDBRecord(I.TOKEN, I.decodeKey(t[i]), I.decodeKey(t[i + 1]), I.deserialize(t[i + 2])),
};
I.readOne = function (handle, from, r, pick) {
  return I.request(handle, from.tx, () => {
    const t = I.walk(from, r, 0, 1);

    return t.length ? pick(t, 0) : undefined;
  });
};
I.readAll = function (handle, from, a, pick) {
  return I.request(handle, from.tx, () => {
    const t = I.walk(from, a.r, a.dir, a.limit);
    const out = [];

    for (let i = 0; i < t.length; i += 3) out.push(pick(t, i));

    return out;
  });
};
I.count = function (handle, from, r) {
  return I.request(handle, from.tx, () => Number(I.ask('count', from.tx._id, from.store, from.index, r.lo, r.hi, r.flags)));
};
})(globalThis.__idb);
