// spec: https://html.spec.whatwg.org/multipage/media.html
// chromium: third_party/blink/renderer/core/html/media/html_media_element.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
delete G.__mediaParts;
const P = HTMLMediaElement.prototype;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const dispatchTrusted = EventTarget.prototype.dispatchEvent;
const EMPTY = 0, IDLE = 1, LOADING = 2, NO_SOURCE = 3;
const NOTHING = 0, METADATA = 1, ENOUGH = 4;
const TICK_MS = 250;
const later = f => ak.setTimer(f, 0, false, [], false);

function stateOf(el) {
  if (el.__media) return el.__media;

  watch();
  hidden(el, '__media', { network: EMPTY, ready: NOTHING, paused: true, seeking: false, error: null, duration: NaN, at: 0, started: 0, played: [], from: 0, rate: 1, defaultRate: 1, volume: 1, muted: el.hasAttribute('muted'), autoplaying: true, promises: [], load: 0, timer: 0, src: '', audio: false, width: 0, height: 0, seen: null });
  if (el.hasAttribute('src') || el.querySelector('source')) selectResource(el);

  return el.__media;
}

function fire(el, type) {
  const ev = new Event(type);
  ev.isTrusted = true;
  dispatchTrusted.call(el, ev);
}

function queue(el, f) {
  const load = stateOf(el).load;
  later(() => { if (stateOf(el).load === load) f(); });
}

function settle(el, error) {
  const m = stateOf(el);
  const pending = m.promises;
  m.promises = [];
  for (const p of pending) error ? p.reject(new DOMException(error[1], error[0])) : p.resolve();
}

function position(m) {
  if (!m.timer) return m.at;

  return Math.min(m.at + (performance.now() - m.started) / 1000 * m.rate, m.duration);
}

function ended(el) {
  const m = stateOf(el);

  return m.ready >= METADATA && position(m) >= m.duration && !el.loop;
}

function stopClock(m) {
  if (!m.timer) return;

  m.at = position(m);
  m.played = parts.addRange(m.played, m.from, m.at);
  ak.clearTimer(m.timer);
  m.timer = 0;
}

function runClock(el) {
  const m = stateOf(el);
  stopClock(m);
  if (m.paused || m.ready < ENOUGH || m.error || !(m.rate > 0) || m.at >= m.duration) return;

  m.started = performance.now();
  m.from = m.at;
  m.timer = ak.setTimer(() => tick(el), Math.max(1, Math.min(TICK_MS, (m.duration - m.at) / m.rate * 1000)), false, [], false);
}

function tick(el) {
  const m = stateOf(el);
  stopClock(m);
  if (m.at < m.duration) {
    fire(el, 'timeupdate');
    runClock(el);
  } else if (el.loop) {
    seek(el, 0);
  } else {
    fire(el, 'timeupdate');
    fire(el, 'ended');
  }
}

function seek(el, time) {
  const m = stateOf(el);
  stopClock(m);
  m.seeking = true;
  m.at = Math.min(Math.max(time, 0), m.duration);
  queue(el, () => fire(el, 'seeking'));
  queue(el, () => {
    m.seeking = false;
    fire(el, 'timeupdate');
    fire(el, 'seeked');
    runClock(el);
  });
}

function allowed(el) {
  const m = stateOf(el);

  return m.muted || m.volume === 0 || navigator.userActivation.hasBeenActive;
}

function fail(el, source) {
  const m = stateOf(el);
  if (source) {
    fire(source, 'error');
    return trySources(el, source.nextElementSibling);
  }

  m.error = parts.error(4, 'MEDIA_ELEMENT_ERROR: Format error');
  m.network = NO_SOURCE;
  fire(el, 'error');
  settle(el, ['NotSupportedError', 'The element has no supported sources.']);
}

function loaded(el, source, url, r) {
  const m = stateOf(el);
  const found = r.error === undefined && r.status < 400 ? ak.mediaProbe(r.raw !== undefined ? r.raw : r.body) : null;
  if (found === null) return fail(el, source);

  [m.duration, m.audio, , m.width, m.height] = found;
  m.src = url;
  fire(el, 'durationchange');
  m.ready = METADATA;
  fire(el, 'loadedmetadata');
  m.ready = ENOUGH;
  fire(el, 'loadeddata');
  fire(el, 'progress');
  fire(el, 'canplay');
  if (!m.paused) fire(el, 'playing');
  fire(el, 'canplaythrough');
  m.network = IDLE;
  fire(el, 'suspend');
  if (m.autoplaying && m.paused && el.autoplay && allowed(el)) {
    m.paused = false;
    fire(el, 'play');
    fire(el, 'playing');
  }

  if (!m.paused) settle(el, null);
  runClock(el);
}

function fetchResource(el, source, value) {
  const m = stateOf(el);
  const load = m.load;
  let url;
  try {
    url = new URL(value, document.baseURI).href;
  } catch (e) {
    return queue(el, () => fail(el, source));
  }

  m.src = url;
  ak.fetchStart(url, 'GET', '', undefined, el.crossOrigin === 'use-credentials' ? 'include' : 'same-origin', el.crossOrigin === null ? 'no-cors' : 'cors', x => {
    if (stateOf(el).load === load) loaded(el, source, url, typeof x === 'number' ? ak.fetchTake(x) : x);
  }, '');
}

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
function selectResource(el) {
  const m = stateOf(el);
  m.network = NO_SOURCE;
  m.seen = el.getAttribute('src');
  queue(el, () => {
    const value = el.getAttribute('src');
    if (value === null && !el.querySelector(':scope > source')) {
      m.network = EMPTY;
      return;
    }

    m.network = LOADING;
    fire(el, 'loadstart');
    if (value === null) trySources(el, el.firstElementChild);
    else if (value === '') fail(el, null);
    else fetchResource(el, null, value);
  });
}

function load(el) {
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

  Object.assign(m, { rate: m.defaultRate, error: null, autoplaying: true });
  selectResource(el);
}

const methods = {
  load() { load(this); },
  canPlayType(type) { return parts.canPlay(type); },
  play() {
    const m = stateOf(this);
    if (!allowed(this)) return Promise.reject(new DOMException("play() failed because the user didn't interact with the document first.", 'NotAllowedError'));
    if (m.error && m.error.code === 4) return Promise.reject(new DOMException('The element has no supported sources.', 'NotSupportedError'));

    const promise = new Promise((resolve, reject) => m.promises.push({ resolve, reject }));
    if (m.network === EMPTY) selectResource(this);
    if (ended(this)) seek(this, 0);
    if (m.paused) {
      m.paused = false;
      queue(this, () => fire(this, 'play'));
      queue(this, () => fire(this, m.ready >= ENOUGH ? 'playing' : 'waiting'));
    }

    if (m.ready >= ENOUGH) queue(this, () => { settle(this, null); runClock(this); });
    m.autoplaying = false;

    return promise;
  },
  pause() {
    const m = stateOf(this);
    if (m.network === EMPTY) selectResource(this);
    m.autoplaying = false;
    if (m.paused) return;

    stopClock(m);
    m.paused = true;
    queue(this, () => {
      fire(this, 'timeupdate');
      fire(this, 'pause');
      settle(this, ['AbortError', 'The play() request was interrupted by a call to pause().']);
    });
  },
};
for (const [k, f] of Object.entries(methods)) hidden(P, k, f);

const whole = el => parts.ranges(stateOf(el).ready >= METADATA ? [[0, stateOf(el).duration]] : []);
const getters = {
  networkState: el => stateOf(el).network,
  readyState: el => stateOf(el).ready,
  paused: el => stateOf(el).paused,
  seeking: el => stateOf(el).seeking,
  ended: el => ended(el),
  error: el => stateOf(el).error,
  duration: el => stateOf(el).duration,
  currentSrc: el => stateOf(el).src,
  buffered: whole,
  seekable: whole,
  played: el => parts.ranges(parts.addRange(stateOf(el).played, stateOf(el).from, stateOf(el).timer ? position(stateOf(el)) : stateOf(el).from)),
  currentTime: el => position(stateOf(el)),
  playbackRate: el => stateOf(el).rate,
  defaultPlaybackRate: el => stateOf(el).defaultRate,
  volume: el => stateOf(el).volume,
  muted: el => stateOf(el).muted,
};
const setters = {
  currentTime(el, v) { stateOf(el).ready === NOTHING ? (stateOf(el).at = Number(v)) : seek(el, Number(v)); },
  playbackRate(el, v) { change(el, 'rate', Number(v), 'ratechange'); runClock(el); },
  defaultPlaybackRate(el, v) { change(el, 'defaultRate', Number(v), 'ratechange'); },
  muted(el, v) { change(el, 'muted', !!v, 'volumechange'); },
  volume(el, v) {
    if (!(v >= 0 && v <= 1)) throw new DOMException("Failed to set the 'volume' property on 'HTMLMediaElement': The volume provided (" + v + ") is outside the range [0, 1].", 'IndexSizeError');
    change(el, 'volume', Number(v), 'volumechange');
  },
};

function change(el, key, value, type) {
  const m = stateOf(el);
  if (m[key] === value) return;

  m[key] = value;
  queue(el, () => fire(el, type));
}

for (const [k, get] of Object.entries(getters)) {
  const set = setters[k];
  Object.defineProperty(P, k, { configurable: true, enumerable: true, get() { return get(this); }, set: set ? function (v) { set(this, v); } : undefined });
}
for (const [k, v] of [['NETWORK_EMPTY', 0], ['NETWORK_IDLE', 1], ['NETWORK_LOADING', 2], ['NETWORK_NO_SOURCE', 3], ['HAVE_NOTHING', 0], ['HAVE_METADATA', 1], ['HAVE_CURRENT_DATA', 2], ['HAVE_FUTURE_DATA', 3], ['HAVE_ENOUGH_DATA', 4]]) {
  Object.defineProperty(HTMLMediaElement, k, { value: v, enumerable: true });
  Object.defineProperty(P, k, { value: v, enumerable: true });
}
for (const k of ['videoWidth', 'videoHeight']) {
  Object.defineProperty(HTMLVideoElement.prototype, k, { configurable: true, enumerable: true, get() { return stateOf(this)[k === 'videoWidth' ? 'width' : 'height']; } });
}

function noticed(node) {
  if (!(node instanceof HTMLElement)) return;

  const media = node instanceof HTMLMediaElement ? [node] : Array.from(node.querySelectorAll('audio, video'));
  if (node.localName === 'source' && node.parentNode instanceof HTMLMediaElement) media.push(node.parentNode);
  for (const el of media) {
    const m = stateOf(el);
    if (m.seen !== el.getAttribute('src')) load(el);
    else if (m.network === EMPTY || (m.network === NO_SOURCE && !el.hasAttribute('src'))) selectResource(el);
  }
}

let watching = false;
function watch() {
  if (watching) return;

  watching = true;
  new MutationObserver(records => {
    for (const r of records) {
      if (r.type === 'attributes') noticed(r.target);
      else r.addedNodes.forEach(noticed);
    }
  }).observe(document, { subtree: true, childList: true, attributes: true, attributeFilter: ['src'] });
}
later(() => document.querySelectorAll('audio, video').forEach(stateOf));
})();
