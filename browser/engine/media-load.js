// spec: https://html.spec.whatwg.org/multipage/media.html#loading-the-media-resource
// chromium: third_party/blink/renderer/platform/media/multi_buffer_data_source.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
delete G.__mediaParts;
const { stateOf, fire, queue, settle, stopClock, runClock, allowed } = parts;
const EMPTY = 0, IDLE = 1, LOADING = 2, NO_SOURCE = 3;
const NOTHING = 0, METADATA = 1, CURRENT = 2, FUTURE = 3, ENOUGH = 4;
const CHUNK = 65536;
const ENOUGH_S = 10;
const later = f => ak.setTimer(f, 0, false, [], false);

const timeOf = (m, byte) => byte >= m.total ? m.duration : byte / m.total * m.duration;
const byteOf = (m, time) => Math.min(Math.floor(time / m.duration * m.total), Math.max(m.total - 1, 0));
const rangeAt = (m, byte) => m.have.find(r => r[0] <= byte && byte < r[1]) || null;

// The range of an attached MediaSource's buffered data that holds the playback position.
const fedAt = m => m.feed.ranges().find(r => r[0] <= m.at && m.at <= r[1]) || null;

parts.playableEnd = m => {
  if (m.feed) return (fedAt(m) || [0, m.at])[1];
  if (!m.total) return m.at;

  const r = rangeAt(m, byteOf(m, m.at));

  return r ? timeOf(m, r[1]) : m.at;
};
parts.bufferedTimes = m => m.feed ? m.feed.ranges() : m.ready >= METADATA ? m.have.map(r => [timeOf(m, r[0]), timeOf(m, r[1])]) : [];

function setReady(el, next) {
  const m = stateOf(el);
  const was = m.ready;
  if (was === next) return;

  m.ready = next;
  if (was < FUTURE && next >= FUTURE) {
    fire(el, 'canplay');
    if (!m.paused) fire(el, 'playing');
  }

  if (was < ENOUGH && next === ENOUGH) {
    fire(el, 'canplaythrough');
    if (m.autoplaying && m.paused && el.autoplay && allowed(el)) {
      m.paused = false;
      fire(el, 'play');
      fire(el, 'playing');
    }
  }

  if (next >= FUTURE) {
    if (!m.paused) settle(el, null);
    runClock(el);
  } else if (was >= FUTURE && !m.paused && !m.seeking && m.at < m.duration) {
    stopClock(m);
    fire(el, 'waiting');
  }
}

function settleReady(el) {
  const m = stateOf(el);
  if (m.ready < METADATA) return;

  if (m.feed) {
    const fed = fedAt(m);
    if (!fed) return setReady(el, METADATA);
    if ((m.feed.ended() && fed[1] >= m.duration) || fed[1] - m.at >= ENOUGH_S) return setReady(el, ENOUGH);

    return setReady(el, fed[1] > m.at + 0.001 ? FUTURE : CURRENT);
  }

  const at = byteOf(m, m.at);
  const r = rangeAt(m, at);
  if (!r) return setReady(el, METADATA);
  if (r[1] >= m.total || timeOf(m, r[1]) - m.at >= ENOUGH_S) return setReady(el, ENOUGH);

  setReady(el, r[1] > at + 1 ? FUTURE : CURRENT);
}

function wants(el) {
  const m = stateOf(el);

  return m.ready < METADATA || el.preload === 'auto' || el.autoplay || !m.paused || m.seeking || m.sought;
}

function pump(el) {
  const m = stateOf(el);
  if (m.feed) return;

  if (m.fetching || m.error || m.network === NO_SOURCE || m.network === EMPTY) return;

  let from = m.total ? byteOf(m, m.at) : 0;
  const r = rangeAt(m, from);
  if (r) from = r[1];
  if ((m.total && from >= m.total) || !wants(el)) {
    if (m.network === LOADING && m.ready >= METADATA) {
      m.network = IDLE;
      fire(el, 'suspend');
      delays(m, false);
    }
    return;
  }

  fetchChunk(el, from);
}

// https://html.spec.whatwg.org/multipage/media.html#delaying-the-load-event-flag
function delays(m, on) {
  if (on && !m.delay) m.delay = ak.mediaDelayLoad(() => {});
  if (!on && m.delay) ak.clearTimer(m.delay);
  if (!on) m.delay = 0;
}

function fail(el, source) {
  const m = stateOf(el);
  if (source) {
    fire(source, 'error');
    return trySources(el, source.nextElementSibling);
  }

  delays(m, false);

  m.error = parts.error(m.ready >= METADATA ? 2 : 4, m.ready >= METADATA ? 'MEDIA_ELEMENT_ERROR: Network error' : 'MEDIA_ELEMENT_ERROR: Format error');
  m.network = m.ready >= METADATA ? IDLE : NO_SOURCE;
  stopClock(m);
  fire(el, 'error');
  if (m.ready < METADATA) settle(el, ['NotSupportedError', 'The element has no supported sources.']);
}

// https://html.spec.whatwg.org/multipage/media.html#media-data-processing-steps-list : the metadata is known.
function known(el, tracks, duration) {
  const m = stateOf(el);
  for (const t of tracks) m[t.kind === 1 ? 'soundTrack' : 'pictureTrack'] = m[t.kind === 1 ? 'soundTrack' : 'pictureTrack'] || t;

  Object.assign(m, { duration, audio: m.soundTrack ? m.soundTrack.a : 0, width: m.pictureTrack ? m.pictureTrack.a : 0, height: m.pictureTrack ? m.pictureTrack.b : 0 });
  if (m.pictureTrack) ak.mediaFrame(el, m.width, m.height, '');

  parts.pictureMirror(el);
  fire(el, 'durationchange');
  m.ready = METADATA;
  fire(el, 'loadedmetadata');
}

function took(el, source, from, r) {
  const m = stateOf(el);
  const bytes = r.raw !== undefined ? r.raw : r.body;
  const said = /^content-range:\s*bytes\s+(\d+)-(\d+)\/(\d+)/im.exec(r.headers || '');
  if (r.error !== undefined || r.status >= 400 || (r.status === 206 && (!said || Number(said[1]) !== from))) return fail(el, m.ready < METADATA ? source : null);

  const start = r.status === 206 ? from : 0;
  const total = r.status === 206 ? Number(said[3]) : bytes.length;
  if (m.ready === NOTHING) {
    const found = ak.mediaProbe(bytes, total);
    if (found === null) return fail(el, source);

    m.total = total;
    known(el, parts.tracks(bytes, total), found[0]);
  }

  m.have = parts.addRange(m.have, start, start + bytes.length);
  m.bytes.push([start, bytes]);
  if (!m.first && rangeAt(m, byteOf(m, m.at))) {
    m.first = true;
    m.ready = CURRENT;
    fire(el, 'loadeddata');
    delays(m, false);
  }

  fire(el, 'progress');
  settleReady(el);
  parts.pictureShow(el);
  if (m.seekDone && rangeAt(m, byteOf(m, m.at))) {
    const done = m.seekDone;
    m.seekDone = null;
    done();
  }

  pump(el);
}

function fetchChunk(el, from) {
  const m = stateOf(el);
  const load = m.load;
  const chunk = ++m.chunk;
  m.fetching = true;
  m.network = LOADING;
  ak.fetchStart(m.src, 'GET', 'Range: bytes=' + from + '-' + (from + CHUNK - 1), undefined, el.crossOrigin === 'use-credentials' ? 'include' : 'same-origin', el.crossOrigin === null ? 'no-cors' : 'cors', x => {
    if (m.load !== load || m.chunk !== chunk) return;

    m.fetching = false;
    took(el, m.source, from, typeof x === 'number' ? ak.fetchTake(x) : x);
  }, '', false, el.localName);
}

function fetchResource(el, source, value) {
  const m = stateOf(el);
  try {
    m.src = new URL(value, document.baseURI).href;
  } catch (e) {
    return queue(el, () => fail(el, source));
  }

  m.source = source;
  m.fetching = false;
  if (parts.sourceFor(m.src)) return attach(el, source, parts.sourceFor(m.src));

  fetchChunk(el, 0);
}

// https://w3c.github.io/media-source/#mediasource-attach
function attach(el, source, provider) {
  const m = stateOf(el);
  m.source = source;
  m.feed = parts.sourceAttach(el, provider);
  delays(m, false);
  if (!m.feed) queue(el, () => fail(el, source));
}

// The attached source got or lost data, or a buffer.
parts.sourceFed = el => {
  const m = stateOf(el);
  if (!m.feed) return;

  if (m.ready === NOTHING) {
    if (!m.feed.complete()) return;

    known(el, m.feed.tracks(), Number.isNaN(m.feed.duration()) ? Infinity : m.feed.duration());
  }

  if (!m.first && fedAt(m)) {
    m.first = true;
    m.ready = CURRENT;
    fire(el, 'loadeddata');
  }

  settleReady(el);
  parts.pictureShow(el);
  parts.soundMore(el);
  if (m.seekDone && fedAt(m)) {
    const done = m.seekDone;
    m.seekDone = null;
    done();
  }
};

parts.sourceLasts = (el, duration) => {
  const m = stateOf(el);
  if (!m.feed || m.ready < METADATA) return;

  m.duration = duration;
  fire(el, 'durationchange');
};

// https://w3c.github.io/media-source/#end-of-stream-algorithm
parts.sourceEnded = (el, error) => {
  const m = stateOf(el);
  if (!m.feed) return;
  if (error === undefined) return parts.sourceFed(el);
  if (m.ready < METADATA) return fail(el, m.source);
  if (error === 'decode') return parts.broke(el);

  fail(el, null);
};

function trySources(el, from) {
  const m = stateOf(el);
  for (let s = from; s; s = s.nextElementSibling) {
    if (s.localName !== 'source') continue;

    const value = s.getAttribute('src');
    const type = s.getAttribute('type');
    if (value && (type === null || parts.canPlay(type) !== '')) return fetchResource(el, s, value);

    fire(s, 'error');
  }

  m.network = NO_SOURCE;
}

// https://html.spec.whatwg.org/multipage/media.html#concept-media-load-algorithm
parts.selectResource = el => {
  const m = stateOf(el);
  m.network = NO_SOURCE;
  m.seen = el.getAttribute('src');
  delays(m, true);
  queue(el, () => {
    const value = el.getAttribute('src');
    if (!m.provider && value === null && !el.querySelector(':scope > source')) {
      m.network = EMPTY;
      delays(m, false);
      return;
    }

    m.network = LOADING;
    fire(el, 'loadstart');
    if (m.provider) return attach(el, null, m.provider);
    if (value === null) trySources(el, el.firstElementChild);
    else if (value === '') fail(el, null);
    else fetchResource(el, null, value);
  });
};

parts.load = el => {
  const m = stateOf(el);
  const was = m.network;
  stopClock(m);
  settle(el, ['AbortError', 'The play() request was interrupted by a new load request.']);
  m.load++;
  if (was === LOADING || was === IDLE) later(() => fire(el, 'abort'));
  if (was !== EMPTY) {
    later(() => fire(el, 'emptied'));
    Object.assign(m, { ready: NOTHING, paused: true, seeking: false, duration: NaN, at: 0, played: [] });
  }

  Object.assign(m, { rate: m.defaultRate, error: null, autoplaying: true, total: 0, have: [], fetching: false, first: false, sought: false, seekDone: null, bytes: [], soundTrack: null, pictureTrack: null });
  parts.pictureStop(m);
  if (m.feed) parts.sourceDetach(m.feed);

  m.feed = null;
  parts.selectResource(el);
};

// chromium: media/base/pipeline_status.h (PIPELINE_ERROR_DECODE)
parts.broke = el => {
  const m = stateOf(el);
  if (m.error) return;

  m.error = parts.error(3, 'PIPELINE_ERROR_DECODE');
  m.network = IDLE;
  stopClock(m);
  parts.pictureStop(m);
  fire(el, 'error');
};

// https://html.spec.whatwg.org/multipage/media.html#dom-media-srcobject
Object.defineProperty(HTMLMediaElement.prototype, 'srcObject', {
  configurable: true,
  enumerable: true,
  get() { return stateOf(this).provider; },
  set(v) {
    if (v !== null && !parts.isSource(v)) throw new TypeError("Failed to set the 'srcObject' property on 'HTMLMediaElement': The provided value is not of type 'MediaSourceHandle or MediaSource'.");

    stateOf(this).provider = v;
    parts.load(this);
  },
});

parts.ahead = el => {
  settleReady(el);
  pump(el);
};

parts.seekTo = (el, done) => {
  const m = stateOf(el);
  m.sought = true;
  settleReady(el);
  if (m.feed ? fedAt(m) : rangeAt(m, byteOf(m, m.at))) return done();

  m.seekDone = done;
  if (m.feed) return;

  m.chunk++;
  m.fetching = false;
  pump(el);
};
})();
