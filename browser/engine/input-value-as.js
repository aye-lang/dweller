// https://html.spec.whatwg.org/multipage/input.html#dom-input-valueasnumber
(function () {
const NUMBER_TYPES = ['date', 'month', 'week', 'time', 'datetime-local', 'number', 'range'];
const DATE_TYPES = ['date', 'month', 'week', 'time'];
const DAY = 86400000;

const pad = (n, width) => String(n).padStart(width, '0');

const refuse = () => {
  throw new DOMException("This input element does not support this value", 'InvalidStateError');
};

const dateText = d => pad(d.getUTCFullYear(), 4) + '-' + pad(d.getUTCMonth() + 1, 2) + '-' + pad(d.getUTCDate(), 2);

const timeText = ms => {
  const t = ((ms % DAY) + DAY) % DAY;
  const s = Math.floor(t / 1000) % 60;
  const milli = t % 1000;
  const hm = pad(Math.floor(t / 3600000), 2) + ':' + pad(Math.floor(t / 60000) % 60, 2);

  if (milli) return hm + ':' + pad(s, 2) + '.' + pad(milli, 3);

  return s ? hm + ':' + pad(s, 2) : hm;
};

const weekText = ms => {
  const d = new Date(ms);
  const thursday = new Date(Date.UTC(d.getUTCFullYear(), d.getUTCMonth(), d.getUTCDate() + 3 - (d.getUTCDay() + 6) % 7));
  const year = thursday.getUTCFullYear();
  const week = 1 + Math.floor((thursday - Date.UTC(year, 0, 1)) / (7 * DAY));

  return pad(year, 4) + '-W' + pad(week, 2);
};

const monthText = n => {
  const years = Math.floor(n / 12);

  return pad(1970 + years, 4) + '-' + pad(n - years * 12 + 1, 2);
};

const SERIALIZE = {
  date: ms => dateText(new Date(ms)),
  month: monthText,
  week: weekText,
  time: timeText,
  'datetime-local': ms => dateText(new Date(ms)) + 'T' + timeText(ms),
  number: String,
  range: String,
};

const define = (name, get, set) => Object.defineProperty(HTMLInputElement.prototype, name, { get, set, configurable: true, enumerable: true });

define('valueAsNumber', function () {
  return NUMBER_TYPES.includes(this.type) ? this._scalar : NaN;
}, function (v) {
  const n = Number(v);

  if (!NUMBER_TYPES.includes(this.type)) refuse();
  if (n === Infinity || n === -Infinity) throw new TypeError("The value provided is infinite");

  this.value = Number.isNaN(n) ? '' : SERIALIZE[this.type](this.type === 'month' ? Math.floor(n) : n);
});

define('valueAsDate', function () {
  if (!DATE_TYPES.includes(this.type)) return null;

  const n = this._scalar;

  if (Number.isNaN(n)) return null;

  return new Date(this.type === 'month' ? Date.UTC(1970, n) : n);
}, function (v) {
  if (!DATE_TYPES.includes(this.type)) refuse();
  if (v !== null && !(v instanceof Date)) throw new TypeError("The provided value is not a Date");

  const ms = v === null ? NaN : v.getTime();

  if (Number.isNaN(ms)) {
    this.value = '';

    return;
  }

  const d = new Date(ms);

  this.value = this.type === 'month' ? monthText((d.getUTCFullYear() - 1970) * 12 + d.getUTCMonth()) : SERIALIZE[this.type](ms);
});
})();
