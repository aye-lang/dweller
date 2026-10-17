// https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#textFieldSelection
(function () {
const SELECTABLE = ['text', 'search', 'tel', 'url', 'password'];
const SELECT_ALL = SELECTABLE.concat(['email', 'number', 'date', 'month', 'week', 'time', 'datetime-local', 'color', 'file']);
const directions = new WeakMap();
const pending = new WeakSet();

const applies = el => el instanceof HTMLTextAreaElement || SELECTABLE.includes(el.type);

const check = el => {
  if (!applies(el)) throw new DOMException("The element's type does not support selection", 'InvalidStateError');
};

const direction = el => directions.get(el) || 'none';

const fireSelect = el => {
  if (pending.has(el)) return;

  pending.add(el);
  setTimeout(() => {
    pending.delete(el);

    const ev = new Event('select', { bubbles: true });

    ev.isTrusted = true;
    el.dispatchEvent(ev);
  }, 0);
};

const setRange = (el, start, end, dir) => {
  const length = el.value.length;
  const hi = Math.min(end, length);
  const lo = Math.min(start, hi);
  const next = dir === 'forward' || dir === 'backward' ? dir : 'none';
  const [oldLo, oldHi] = el._selection;
  const oldDir = direction(el);

  el._select(lo, hi);
  directions.set(el, next);

  if (oldLo !== lo || oldHi !== hi || oldDir !== next) fireSelect(el);
};

const unsigned = v => Number(v) >>> 0;

const define = (name, get, set) => {
  for (const I of [HTMLInputElement, HTMLTextAreaElement]) {
    Object.defineProperty(I.prototype, name, { get, set, configurable: true, enumerable: true });
  }
};

define('selectionStart', function () {
  return applies(this) ? this._selection[0] : null;
}, function (v) {
  check(this);

  const start = unsigned(v);

  setRange(this, start, Math.max(start, this._selection[1]), direction(this));
});

define('selectionEnd', function () {
  return applies(this) ? this._selection[1] : null;
}, function (v) {
  check(this);
  setRange(this, this._selection[0], unsigned(v), direction(this));
});

define('selectionDirection', function () {
  return applies(this) ? direction(this) : null;
}, function (v) {
  check(this);

  const [lo, hi] = this._selection;

  setRange(this, lo, hi, String(v));
});

for (const I of [HTMLInputElement, HTMLTextAreaElement]) {
  const value = Object.getOwnPropertyDescriptor(I.prototype, 'value');

  Object.defineProperty(I.prototype, 'value', {
    get: value.get,
    set: function (v) {
      const old = value.get.call(this);
      const [lo, hi] = this._selection;

      value.set.call(this, v);

      if (value.get.call(this) === old) this._select(lo, hi);
      else directions.delete(this);
    },
    configurable: true,
    enumerable: true,
  });

  Object.defineProperty(I.prototype, 'setSelectionRange', {
    value: function setSelectionRange(start, end, dir) {
      check(this);
      setRange(this, unsigned(start), unsigned(end), dir === undefined ? 'none' : String(dir));
    },
    writable: true,
    configurable: true,
  });

  Object.defineProperty(I.prototype, 'select', {
    value: function select() {
      if (this instanceof HTMLInputElement && !SELECT_ALL.includes(this.type)) return;

      if (!applies(this)) return fireSelect(this);

      setRange(this, 0, this.value.length, 'none');
    },
    writable: true,
    configurable: true,
  });

  Object.defineProperty(I.prototype, 'setRangeText', {
    value: function setRangeText(replacement, start, end, selectMode) {
      check(this);
      replaceRange(this, String(replacement), arguments.length > 1, start, end, selectMode);
    },
    writable: true,
    configurable: true,
  });
}

function replaceRange(el, replacement, ranged, start, end, mode) {
  let [lo, hi] = el._selection;
  let from = lo;
  let to = hi;

  if (ranged) {
    from = unsigned(start);
    to = unsigned(end);

    if (from > to) throw new DOMException('The start offset is after the end offset', 'IndexSizeError');
  }

  const old = el.value;

  from = Math.min(from, old.length);
  to = Math.min(to, old.length);

  const added = replacement.length;
  const newEnd = from + added;
  const delta = added - (to - from);

  el.value = old.slice(0, from) + replacement + old.slice(to);

  if (mode === 'select') {
    lo = from;
    hi = newEnd;
  } else if (mode === 'start') {
    lo = hi = from;
  } else if (mode === 'end') {
    lo = hi = newEnd;
  } else {
    if (lo > to) lo += delta;
    else if (lo > from) lo = from;

    if (hi > to) hi += delta;
    else if (hi > from) hi = newEnd;
  }

  el._select(lo, hi);
  directions.set(el, 'none');
  fireSelect(el);
}
})();
