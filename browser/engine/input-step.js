// https://html.spec.whatwg.org/multipage/input.html#dom-input-stepup
(function () {
const SCALE = { date: 86400000, month: 1, week: 604800000, time: 1000, 'datetime-local': 1000, number: 1, range: 1 };
const DEFAULT_STEP = { date: 1, month: 1, week: 1, time: 60, 'datetime-local': 60, number: 1, range: 1 };
const probe = document.createElement('input');

const refuse = message => {
  throw new DOMException(message, 'InvalidStateError');
};

const parse = (type, s) => {
  if (s === null) return NaN;

  probe.type = type === 'range' ? 'number' : type;
  probe.value = s;

  return probe._scalar;
};

const stepOf = el => {
  const attr = el.getAttribute('step');

  if (attr !== null && attr.trim().toLowerCase() === 'any') return null;

  const n = attr === null ? NaN : parse('number', attr.trim());

  return (n > 0 ? n : DEFAULT_STEP[el.type]) * SCALE[el.type];
};

const baseOf = el => {
  for (const name of ['min', 'value']) {
    const n = parse(el.type, el.getAttribute(name));

    if (!Number.isNaN(n)) return n;
  }

  return el.type === 'week' ? -259200000 : 0;
};

const bound = (el, name, fallback) => {
  const n = parse(el.type, el.getAttribute(name));

  return Number.isNaN(n) ? fallback : n;
};

function step(el, n, sign) {
  if (!(el.type in SCALE)) refuse("This form element is not steppable");

  const size = stepOf(el);

  if (size === null) refuse("This form element does not have an allowed value step");

  const base = baseOf(el);
  const min = bound(el, 'min', el.type === 'range' ? 0 : -Infinity);
  const max = bound(el, 'max', el.type === 'range' ? 100 : Infinity);

  if (min > max) return;

  const ceil = v => base + Math.ceil((v - base) / size) * size;
  const floor = v => base + Math.floor((v - base) / size) * size;

  if (ceil(min) > max) return;

  const current = el.valueAsNumber;
  const before = Number.isNaN(current) ? 0 : current;
  const aligned = Math.abs((before - base) / size - Math.round((before - base) / size)) < 1e-9;
  let value = before + sign * size * n;

  if (!aligned) value = sign > 0 ? ceil(before) : floor(before);
  if (value < min) value = ceil(min);
  if (value > max) value = floor(max);
  if (sign * (value - before) < 0) return;

  el.valueAsNumber = value;
}

Object.defineProperty(HTMLInputElement.prototype, 'stepUp', {
  value: function stepUp(n = 1) { step(this, Math.trunc(Number(n)) || 0, 1); },
  writable: true,
  configurable: true,
});

Object.defineProperty(HTMLInputElement.prototype, 'stepDown', {
  value: function stepDown(n = 1) { step(this, Math.trunc(Number(n)) || 0, -1); },
  writable: true,
  configurable: true,
});
})();
