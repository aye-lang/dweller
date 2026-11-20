// spec: https://w3c.github.io/media-source/
// chromium: third_party/blink/renderer/modules/mediasource/media_source.cc, source_buffer.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
const B = parts.buffers;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const KEY = {};
const later = f => ak.setTimer(f, 0, false, [], false);
const fail = (name, what) => new DOMException(what, name);
const FORMATS = { 'video/webm': 'webm', 'audio/webm': 'webm', 'video/x-matroska': 'webm', 'audio/x-matroska': 'webm', 'video/mp4': 'mp4', 'audio/mp4': 'mp4' };
const CODECS = ['v_uncompressed', 'a_pcm/int/lit', 'a_pcm/float/ieee'];
const urls = new Map();

function handlers(proto, types) {
  for (const type of types) {
    Object.defineProperty(proto, 'on' + type, {
      configurable: true,
      enumerable: true,
      get() { return (this._on && this._on[type]) || null; },
      set(f) {
        if (!this._on) hidden(this, '_on', {});
        if (!(type in this._on)) this.addEventListener(type, e => typeof this._on[type] === 'function' && this._on[type].call(this, e));
        this._on[type] = typeof f === 'function' ? f : null;
      },
    });
  }
}

// chromium: media/base/mime_util_internal.cc
function formatOf(type) {
  const fields = String(type).split(';').map(s => s.trim());
  const format = FORMATS[fields[0].toLowerCase()];
  const named = fields.slice(1).find(p => /^codecs\s*=/i.test(p));
  if (!format || !named) return null;

  const codecs = named.replace(/^codecs\s*=\s*/i, '').replace(/^"|"$/g, '').split(',').map(s => s.trim().toLowerCase());

  return codecs.length && codecs.every(c => CODECS.includes(c)) ? format : null;
}

class SourceBufferList extends EventTarget {
  constructor(key) {
    if (key !== KEY) throw new TypeError('Illegal constructor');

    super();
    hidden(this, '_list', []);
  }
  get length() { return this._list.length; }
  [Symbol.iterator]() { return this._list[Symbol.iterator](); }
  _set(list) {
    for (let i = 0; i < Math.max(list.length, this._list.length); i++) {
      if (i < list.length) Object.defineProperty(this, i, { value: list[i], configurable: true, enumerable: true });
      else delete this[i];
    }

    this._list = list;
  }
}
handlers(SourceBufferList.prototype, ['addsourcebuffer', 'removesourcebuffer']);

class SourceBuffer extends EventTarget {
  constructor(key, source, format) {
    if (key !== KEY) throw new TypeError('Illegal constructor');

    super();
    hidden(this, '_s', { source, format, mode: 'segments', offset: 0, windowStart: 0, windowEnd: Infinity, groupStart: null, groupEnd: 0, tracks: new Map(), updating: false, removed: false, pending: '', init: '', base: -1, task: 0, known: false });
  }
  get updating() { return this._s.updating; }
  get mode() { return this._s.mode; }
  set mode(v) {
    const s = this._s;
    if (v !== 'segments' && v !== 'sequence') return;

    ready(this, 'mode');
    if (v === 'sequence') s.groupStart = s.groupEnd;

    s.mode = v;
  }
  get timestampOffset() { return this._s.offset; }
  set timestampOffset(v) {
    const s = this._s;
    v = Number(v);
    if (!Number.isFinite(v)) throw new TypeError("Failed to set the 'timestampOffset' property on 'SourceBuffer': The provided double value is non-finite.");

    ready(this, 'timestampOffset');
    if (s.mode === 'sequence') s.groupStart = v;

    s.offset = v;
  }
  get appendWindowStart() { return this._s.windowStart; }
  set appendWindowStart(v) {
    const s = this._s;
    v = Number(v);
    if (!Number.isFinite(v)) throw new TypeError("Failed to set the 'appendWindowStart' property on 'SourceBuffer': The provided double value is non-finite.");

    usable(this, 'appendWindowStart');
    if (v < 0 || v >= s.windowEnd) throw new TypeError("Failed to set the 'appendWindowStart' property on 'SourceBuffer': The value provided (" + v + ") is outside the range [0, " + s.windowEnd + ").");

    s.windowStart = v;
  }
  get appendWindowEnd() { return this._s.windowEnd; }
  set appendWindowEnd(v) {
    const s = this._s;
    v = Number(v);
    usable(this, 'appendWindowEnd');
    if (Number.isNaN(v) || v <= s.windowStart) throw new TypeError("Failed to set the 'appendWindowEnd' property on 'SourceBuffer': The value provided (" + v + ") is less than or equal to the minimum value (" + s.windowStart + ").");

    s.windowEnd = v;
  }
  get buffered() {
    if (this._s.removed) throw fail('InvalidStateError', "Failed to read the 'buffered' property from 'SourceBuffer': This SourceBuffer has been removed from the parent media source.");

    return parts.ranges(B.ranges(this._s, this._s.source._m.state === 'ended'));
  }
  appendBuffer(data) {
    const s = this._s;
    const view = data instanceof ArrayBuffer ? new Uint8Array(data) : ArrayBuffer.isView(data) ? new Uint8Array(data.buffer, data.byteOffset, data.byteLength) : null;
    if (!view) throw new TypeError("Failed to execute 'appendBuffer' on 'SourceBuffer': parameter 1 is not of type 'ArrayBuffer'.");

    ready(this, 'appendBuffer');
    s.pending += ak.latin1Of(view);
    begin(this, () => {
      if (!parse(this)) return broke(this);

      finish(this);
    });
  }
  abort() {
    const s = this._s;
    if (s.removed) throw fail('InvalidStateError', "Failed to execute 'abort' on 'SourceBuffer': This SourceBuffer has been removed from the parent media source.");
    if (s.source._m.state !== 'open') throw fail('InvalidStateError', "Failed to execute 'abort' on 'SourceBuffer': The parent media source's readyState is not 'open'.");

    if (s.updating) stop(this);

    Object.assign(s, { pending: '', base: -1, windowStart: 0, windowEnd: Infinity });
  }
  remove(start, end) {
    const s = this._s;
    start = Number(start);
    end = Number(end);
    usable(this, 'remove');

    const duration = s.source._m.duration;
    if (Number.isNaN(duration)) throw new TypeError("Failed to execute 'remove' on 'SourceBuffer': The duration is NaN.");
    if (!(start >= 0 && start <= duration)) throw new TypeError("Failed to execute 'remove' on 'SourceBuffer': The start provided (" + start + ") is outside the range (0, " + duration + ").");
    if (!(end > start)) throw new TypeError("Failed to execute 'remove' on 'SourceBuffer': The end provided (" + end + ") must be greater than the start provided (" + start + ").");

    reopen(s.source);
    begin(this, () => {
      B.remove(s, start, end);
      finish(this);
    });
  }
  changeType(type) {
    const s = this._s;
    if (String(type) === '') throw new TypeError("Failed to execute 'changeType' on 'SourceBuffer': The type provided is empty");

    usable(this, 'changeType');

    const format = formatOf(type);
    if (!format) throw fail('NotSupportedError', "Failed to execute 'changeType' on 'SourceBuffer': The type provided ('" + type + "') is not supported.");

    reopen(s.source);
    Object.assign(s, { format, pending: '', base: -1 });
  }
}
handlers(SourceBuffer.prototype, ['updatestart', 'update', 'updateend', 'error', 'abort']);

function usable(sb, what) {
  const s = sb._s;
  const access = what in SourceBuffer.prototype && typeof Object.getOwnPropertyDescriptor(SourceBuffer.prototype, what).value === 'function' ? "execute '" + what + "' on" : "set the '" + what + "' property on";
  if (s.removed) throw fail('InvalidStateError', 'Failed to ' + access + " 'SourceBuffer': This SourceBuffer has been removed from the parent media source.");
  if (s.updating) throw fail('InvalidStateError', 'Failed to ' + access + " 'SourceBuffer': This SourceBuffer is still processing an 'appendBuffer' or 'remove' operation.");
}

// https://w3c.github.io/media-source/#sourcebuffer-prepare-append
function ready(sb, what) {
  usable(sb, what);
  reopen(sb._s.source);
}

function reopen(source) {
  if (source._m.state !== 'ended') return;

  source._m.state = 'open';
  later(() => parts.fire(source, 'sourceopen'));
}

function begin(sb, work) {
  const s = sb._s;
  s.updating = true;
  later(() => parts.fire(sb, 'updatestart'));
  s.task = later(() => { s.task = 0; work(); });
}

function finish(sb) {
  const s = sb._s;
  s.updating = false;
  later(() => parts.fire(sb, 'update'));
  later(() => parts.fire(sb, 'updateend'));
  fed(s.source);
}

function stop(sb) {
  const s = sb._s;
  if (s.task) ak.clearTimer(s.task);

  s.task = 0;
  s.updating = false;
  later(() => parts.fire(sb, 'abort'));
  later(() => parts.fire(sb, 'updateend'));
}

// https://w3c.github.io/media-source/#sourcebuffer-append-error
function broke(sb) {
  const s = sb._s;
  Object.assign(s, { pending: '', base: -1, updating: false });
  later(() => parts.fire(sb, 'error'));
  later(() => parts.fire(sb, 'updateend'));
  end(s.source, 'decode');
}

// https://w3c.github.io/media-source/#sourcebuffer-segment-parser-loop
function parse(sb) {
  const s = sb._s;
  for (;;) {
    const [used, bad, initAt, initEnd, base, flat] = ak.mediaStream(s.format, s.init, s.pending, s.base);
    if (bad) return false;

    if (initEnd > 0 && !started(sb, s.pending.slice(initAt, initEnd))) return false;

    const frames = [];
    for (let i = 0; i < flat.length; i += 6) frames.push({ track: flat[i], time: flat[i + 1], duration: flat[i + 2], key: flat[i + 3], data: s.pending.slice(flat[i + 4], flat[i + 4] + flat[i + 5]) });
    if (frames.length && !s.known) return false;

    B.add(s, frames);
    s.base = base;
    s.pending = s.pending.slice(used);
    if (!used) break;
  }

  if (B.end(s) > s.source._m.duration) lasts(s.source, B.end(s));

  return true;
}

// https://w3c.github.io/media-source/#sourcebuffer-init-segment-received
function started(sb, init) {
  const s = sb._s;
  const tracks = parts.tracks(init, init.length);
  if (!tracks.length || !tracks.every(t => parts.decodable(t))) return false;

  if (s.known) {
    if (tracks.length !== s.tracks.size || !tracks.every(t => s.tracks.has(t.number) && s.tracks.get(t.number).track.codec === t.codec)) return false;

    for (const t of tracks) Object.assign(s.tracks.get(t.number), { needKey: true });
  } else {
    for (const t of tracks) s.tracks.set(t.number, B.track(t));
  }

  s.init = init;
  if (Number.isNaN(s.source._m.duration)) lasts(s.source, (ak.mediaProbe(init, init.length) || [0])[0] || Infinity);

  if (!s.known) {
    s.known = true;
    active(s.source);
  }

  return true;
}

class MediaSource extends EventTarget {
  constructor() {
    super();
    hidden(this, '_m', { state: 'closed', duration: NaN, buffers: new SourceBufferList(KEY), active: new SourceBufferList(KEY), el: null, live: null });
  }
  static isTypeSupported(type) { return formatOf(type) !== null; }
  static get canConstructInDedicatedWorker() { return false; }
  get readyState() { return this._m.state; }
  get sourceBuffers() { return this._m.buffers; }
  get activeSourceBuffers() { return this._m.active; }
  get duration() { return this._m.state === 'closed' ? NaN : this._m.duration; }
  set duration(v) {
    const m = this._m;
    v = Number(v);
    if (Number.isNaN(v) || v < 0) throw new TypeError("Failed to set the 'duration' property on 'MediaSource': The value provided (" + v + ") is invalid.");

    open(this, "set the 'duration' property on");

    const highest = Math.max(0, ...m.buffers._list.map(sb => B.end(sb._s)));
    if (v < highest) throw fail('InvalidStateError', "Failed to set the 'duration' property on 'MediaSource': Setting duration below highest presentation timestamp of any buffered coded frames is disallowed. Instead, first do asynchronous remove(newDuration, oldDuration) on all sourceBuffers, where newDuration < oldDuration.");

    lasts(this, v);
  }
  addSourceBuffer(type) {
    const m = this._m;
    if (String(type) === '') throw new TypeError("Failed to execute 'addSourceBuffer' on 'MediaSource': The type provided is empty.");

    const format = formatOf(type);
    if (!format) throw fail('NotSupportedError', "Failed to execute 'addSourceBuffer' on 'MediaSource': The type provided ('" + type + "') is unsupported.");
    if (m.state !== 'open') throw fail('InvalidStateError', "Failed to execute 'addSourceBuffer' on 'MediaSource': The MediaSource's readyState is not 'open'.");

    const sb = new SourceBuffer(KEY, this, format);
    m.buffers._set(m.buffers._list.concat([sb]));
    later(() => parts.fire(m.buffers, 'addsourcebuffer'));

    return sb;
  }
  removeSourceBuffer(sb) {
    const m = this._m;
    if (!(sb instanceof SourceBuffer)) throw new TypeError("Failed to execute 'removeSourceBuffer' on 'MediaSource': parameter 1 is not of type 'SourceBuffer'.");
    if (!m.buffers._list.includes(sb)) throw fail('NotFoundError', "Failed to execute 'removeSourceBuffer' on 'MediaSource': The SourceBuffer provided is not contained in this MediaSource.");

    if (sb._s.updating) stop(sb);

    gone(this, sb);
    fed(this);
  }
  endOfStream(error) {
    const m = this._m;
    if (error !== undefined && error !== 'network' && error !== 'decode') throw new TypeError("Failed to execute 'endOfStream' on 'MediaSource': The provided value '" + error + "' is not a valid enum value of type EndOfStreamError.");

    open(this, "execute 'endOfStream' on");
    end(this, error);
  }
  setLiveSeekableRange(start, end) {
    start = Number(start);
    end = Number(end);
    if (this._m.state !== 'open') throw fail('InvalidStateError', "Failed to execute 'setLiveSeekableRange' on 'MediaSource': The MediaSource's readyState is not 'open'.");
    if (!(start >= 0) || !(end >= start)) throw new TypeError("Failed to execute 'setLiveSeekableRange' on 'MediaSource': The start and end values are invalid.");

    this._m.live = [start, end];
  }
  clearLiveSeekableRange() {
    if (this._m.state !== 'open') throw fail('InvalidStateError', "Failed to execute 'clearLiveSeekableRange' on 'MediaSource': The MediaSource's readyState is not 'open'.");

    this._m.live = null;
  }
}
handlers(MediaSource.prototype, ['sourceopen', 'sourceended', 'sourceclose']);

function open(source, access) {
  const m = source._m;
  if (m.state !== 'open') throw fail('InvalidStateError', 'Failed to ' + access + " 'MediaSource': The MediaSource's readyState is not 'open'.");
  if (m.buffers._list.some(sb => sb._s.updating)) throw fail('InvalidStateError', 'Failed to ' + access + " 'MediaSource': The 'updating' attribute is true on one or more of this MediaSource's SourceBuffers.");
}

function gone(source, sb) {
  const m = source._m;
  sb._s.removed = true;
  if (m.active._list.includes(sb)) {
    m.active._set(m.active._list.filter(x => x !== sb));
    later(() => parts.fire(m.active, 'removesourcebuffer'));
  }

  m.buffers._set(m.buffers._list.filter(x => x !== sb));
  later(() => parts.fire(m.buffers, 'removesourcebuffer'));
}

// https://w3c.github.io/media-source/#duration-change-algorithm
function lasts(source, duration) {
  const m = source._m;
  if (m.duration === duration) return;

  m.duration = duration;
  if (m.el) parts.sourceLasts(m.el, duration);
}

// https://w3c.github.io/media-source/#end-of-stream-algorithm
function end(source, error) {
  const m = source._m;
  m.state = 'ended';
  later(() => parts.fire(source, 'sourceended'));
  if (error === undefined) lasts(source, Math.max(0, ...m.buffers._list.map(sb => B.end(sb._s))));

  if (m.el) parts.sourceEnded(m.el, error);
}

// A buffer that got its first initialization segment is active; the element learns its tracks.
function active(source) {
  const m = source._m;
  m.active._set(m.buffers._list.filter(sb => sb._s.known));
  later(() => parts.fire(m.active, 'addsourcebuffer'));
}

function fed(source) {
  if (source._m.el) parts.sourceFed(source._m.el);
}

// What the element reads: the tracks, the buffered ranges, whether the stream ended.
function feedOf(source) {
  const m = source._m;

  return {
    source,
    tracks: () => m.active._list.flatMap(sb => Array.from(sb._s.tracks.values(), tb => tb.track)),
    ranges: () => B.common(m.active._list.map(sb => B.ranges(sb._s, m.state === 'ended')), m.state === 'ended'),
    ended: () => m.state === 'ended',
    duration: () => m.duration,
    complete: () => m.buffers._list.length > 0 && m.buffers._list.every(sb => sb._s.known),
  };
}

// https://w3c.github.io/media-source/#mediasource-attach
parts.sourceAttach = (el, source) => {
  const m = source._m;
  if (m.state !== 'closed') return null;

  m.state = 'open';
  m.el = el;
  later(() => parts.fire(source, 'sourceopen'));

  return feedOf(source);
};

// https://w3c.github.io/media-source/#mediasource-detach
parts.sourceDetach = feed => {
  const source = feed.source;
  const m = source._m;
  m.state = 'closed';
  m.duration = NaN;
  m.el = null;
  for (const sb of m.buffers._list.slice()) {
    if (sb._s.updating) stop(sb);

    gone(source, sb);
  }

  later(() => parts.fire(source, 'sourceclose'));
};

parts.sourceFor = url => urls.get(String(url).split('#')[0]) || null;
parts.isSource = v => v instanceof MediaSource;

const createObjectURL = URL.createObjectURL, revokeObjectURL = URL.revokeObjectURL;
hidden(URL, 'createObjectURL', function (object) {
  if (!(object instanceof MediaSource)) return createObjectURL.apply(this, arguments);

  const url = 'blob:' + ak.urlPart('origin') + '/' + crypto.randomUUID();
  urls.set(url, object);

  return url;
});
hidden(URL, 'revokeObjectURL', function (url) {
  urls.delete(String(url).split('#')[0]);

  return revokeObjectURL.apply(this, arguments);
});

hidden(G, 'MediaSource', MediaSource);
hidden(G, 'SourceBuffer', SourceBuffer);
hidden(G, 'SourceBufferList', SourceBufferList);
})();
