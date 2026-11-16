// spec: https://w3c.github.io/IndexedDB/#cursor-interface
(function (I) {
const what = name => "Failed to execute '" + name + "' on 'IDBCursor': ";

function active(c, name) {
  if (c._from.tx._state !== 'active') throw I.error('TransactionInactiveError', what(name) + 'The transaction is not active.');
}
function alive(c, name) {
  const index = c._source._store ? c._source : null;
  const store = index ? index._store : c._source;

  if ((index && index._meta.deleted) || store._meta.deleted) throw I.error('InvalidStateError', what(name) + 'The cursor\'s source or effective object store has been deleted.');

  return store;
}
function placed(c, name) {
  if (!c._got) throw I.error('InvalidStateError', what(name) + 'The cursor is being iterated or has iterated past its end.');
}
// spec: https://w3c.github.io/IndexedDB/#iterate-a-cursor
function iterate(c, how) {
  const t = I.walk(c._from, c._range, c._dir, 1, Object.assign({ key: c._k, primary: c._p }, how));

  c._key = c._primaryKey = c._value = undefined;
  c._seen = 0;
  c._got = t.length > 0;
  c._k = t[0];
  c._p = t[1];
  c._v = t[2];

  return c._got ? c : null;
}
function move(c, how) {
  c._got = false;
  I.request(c, c._from.tx, () => iterate(c, how), c._request);
}
const before = (c, a, b) => c._dir < 2 ? a < b : a > b;

class IDBCursor {
  constructor(token, source, from, range, dir, request) {
    I.legal(token);
    I.hide(this, { _source: source, _from: from, _range: range, _dir: dir, _request: request, _got: false, _k: '', _p: '', _v: '', _key: undefined, _primaryKey: undefined, _value: undefined, _seen: 0 });
  }
  get source() { return this._source; }
  get direction() { return I.DIRECTIONS[this._dir]; }
  get request() { return this._request; }
  get key() {
    if (!(this._seen & 1) && this._k) this._key = I.decodeKey(this._k);

    this._seen |= 1;

    return this._key;
  }
  get primaryKey() {
    if (!(this._seen & 2) && this._p) this._primaryKey = I.decodeKey(this._p);

    this._seen |= 2;

    return this._primaryKey;
  }
  advance(count) {
    I.need(arguments, 1, "'advance' on 'IDBCursor'");
    count = I.countOf(count);

    if (count === 0) throw new TypeError(what('advance') + 'A count argument with value 0 (zero) was supplied, must be greater than 0.');

    active(this, 'advance');
    alive(this, 'advance');
    placed(this, 'advance');
    move(this, { skip: count - 1 });
  }
  continue(key) {
    active(this, 'continue');
    alive(this, 'continue');
    placed(this, 'continue');

    const to = key === undefined ? '' : I.toKey(key);

    if (to !== '' && !before(this, this._k, to)) throw I.error('DataError', what('continue') + 'The parameter is not after the cursor\'s position.');

    move(this, { toKey: to });
  }
  continuePrimaryKey(key, primaryKey) {
    I.need(arguments, 2, "'continuePrimaryKey' on 'IDBCursor'");
    active(this, 'continuePrimaryKey');
    alive(this, 'continuePrimaryKey');

    if (!this._source._store || this._dir === 1 || this._dir === 3) throw I.error('InvalidAccessError', what('continuePrimaryKey') + 'The cursor\'s source is not an index, or its direction is unique.');

    placed(this, 'continuePrimaryKey');

    const to = I.toKey(key);
    const toPrimary = I.toKey(primaryKey);

    if (before(this, to, this._k) || (to === this._k && !before(this, this._p, toPrimary))) throw I.error('DataError', what('continuePrimaryKey') + 'The parameters are not after the cursor\'s position.');

    move(this, { toKey: to, toPrimary });
  }
  update(value) {
    I.need(arguments, 1, "'update' on 'IDBCursor'");
    active(this, 'update');

    const tx = this._from.tx;

    if (tx._mode === 'readonly') throw I.error('ReadOnlyError', what('update') + 'The transaction is read-only.');

    const meta = alive(this, 'update')._meta;

    placed(this, 'update');

    if (!(this instanceof IDBCursorWithValue)) throw I.error('InvalidStateError', what('update') + 'The cursor is a key cursor.');

    const w = I.cloneFor(tx, value, meta);
    const key = this._p;

    if (meta.keyPath !== null && I.extractKey(w.copy, meta.keyPath) !== key) throw I.error('DataError', what('update') + 'The effective object store of this cursor uses in-line keys and evaluating the key path of the value parameter results in a different value than the cursor\'s effective key.');

    return I.request(this, tx, () => I.putRecord(tx, meta, key, w, true));
  }
  delete() {
    active(this, 'delete');

    const tx = this._from.tx;

    if (tx._mode === 'readonly') throw I.error('ReadOnlyError', what('delete') + 'The transaction is read-only.');

    alive(this, 'delete');
    placed(this, 'delete');

    if (!(this instanceof IDBCursorWithValue)) throw I.error('InvalidStateError', what('delete') + 'The cursor is a key cursor.');

    const key = this._p;
    const store = this._from.store;

    return I.request(this, tx, () => void I.ask('delete', tx._id, store, key, key, 0));
  }
}
I.expose('IDBCursor', IDBCursor);

class IDBCursorWithValue extends IDBCursor {
  get value() {
    if (!(this._seen & 4) && this._got) this._value = I.deserialize(this._v);

    this._seen |= 4;

    return this._value;
  }
}
I.expose('IDBCursorWithValue', IDBCursorWithValue);

I.openCursor = function (source, from, readable, name, query, direction, keyOnly) {
  const dir = I.directionOf(direction);
  const tx = readable(source, name);
  const range = I.rangeOf(query, true);
  const req = new globalThis.IDBRequest(I.TOKEN);
  const c = new (keyOnly ? IDBCursor : IDBCursorWithValue)(I.TOKEN, source, from, range, dir, req);

  return I.request(source, tx, () => iterate(c, {}), req);
};
})(globalThis.__idb);
