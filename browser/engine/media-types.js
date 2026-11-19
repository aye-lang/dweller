// spec: https://html.spec.whatwg.org/multipage/media.html#time-ranges
// chromium: third_party/blink/renderer/core/html/time_ranges.cc, media/base/mime_util_internal.cc
(function () {
const G = globalThis;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const KEY = {};

class TimeRanges {
  constructor(key, ranges) {
    if (key !== KEY) throw new TypeError('Illegal constructor');
    hidden(this, '_ranges', ranges);
  }
  get length() { return this._ranges.length; }
  start(i) { return this._at('start', i)[0]; }
  end(i) { return this._at('end', i)[1]; }
  _at(what, i) {
    if (arguments.length < 2 || !(i >>> 0 < this._ranges.length)) throw new DOMException("Failed to execute '" + what + "' on 'TimeRanges': The index provided (" + i + ") is greater than or equal to the maximum bound (" + this._ranges.length + ").", 'IndexSizeError');

    return this._ranges[i >>> 0];
  }
}

class MediaError {
  constructor(key, code, message) {
    if (key !== KEY) throw new TypeError('Illegal constructor');
    hidden(this, '_code', code);
    hidden(this, '_message', message);
  }
  get code() { return this._code; }
  get message() { return this._message; }
}
for (const [k, v] of [['MEDIA_ERR_ABORTED', 1], ['MEDIA_ERR_NETWORK', 2], ['MEDIA_ERR_DECODE', 3], ['MEDIA_ERR_SRC_NOT_SUPPORTED', 4]]) {
  Object.defineProperty(MediaError, k, { value: v, enumerable: true });
  Object.defineProperty(MediaError.prototype, k, { value: v, enumerable: true });
}

function addRange(ranges, from, to) {
  if (!(to > from)) return ranges;

  const all = ranges.concat([[from, to]]).sort((a, b) => a[0] - b[0]);
  const out = [all[0].slice()];
  for (const r of all.slice(1)) {
    const last = out[out.length - 1];
    if (r[0] <= last[1]) last[1] = Math.max(last[1], r[1]);
    else out.push(r.slice());
  }

  return out;
}

// https://html.spec.whatwg.org/multipage/media.html#dom-navigator-canplaytype
const CONTAINERS = { 'audio/wav': ['1'], 'audio/wave': ['1'], 'audio/x-wav': ['1'], 'audio/vnd.wave': ['1'] };
function canPlay(type) {
  const parts = String(type).split(';').map(s => s.trim());
  const codecs = CONTAINERS[parts[0].toLowerCase()];
  if (!codecs) return '';

  const named = parts.slice(1).find(p => /^codecs\s*=/i.test(p));
  if (!named) return 'maybe';

  const wanted = named.replace(/^codecs\s*=\s*/i, '').replace(/^"|"$/g, '').split(',').map(s => s.trim());

  return wanted.every(c => codecs.includes(c)) ? 'probably' : '';
}

hidden(G, 'TimeRanges', TimeRanges);
hidden(G, 'MediaError', MediaError);
hidden(G, '__mediaParts', {
  ranges: list => new TimeRanges(KEY, list),
  error: (code, message) => new MediaError(KEY, code, message),
  addRange,
  canPlay,
});
})();
