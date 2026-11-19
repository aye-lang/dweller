// spec: https://html.spec.whatwg.org/multipage/media.html#the-video-element
// chromium: media/renderers/video_renderer_impl.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
const AHEAD = 2;

function seen(el, p, seq, ok, data) {
  const m = parts.stateOf(el);
  if (m.picture !== p) return;
  if (!ok) return parts.broke(el);

  p.ready.set(seq, data);
  show(el);
}

function open(el) {
  const m = parts.stateOf(el);
  const t = m.pictureTrack;
  const p = { id: 0, fed: -1, shown: -1, ready: new Map(), timer: 0 };
  m.picture = p;
  p.id = ak.mediaDecoderOpen(2, t.codec, t.config, (seq, ok, data) => seen(el, p, seq, ok, data), t.a, t.b, 0);
  if (!p.id) parts.broke(el);

  return p;
}

// Feeds the decoder up to a few frames past the playback position (from the key frame before it after a seek) and shows the newest decoded frame that is due.
function show(el) {
  const m = parts.stateOf(el);
  const t = m.pictureTrack;
  if (!t || !t.samples.length || m.error || el.localName !== 'video') return;

  const p = m.picture || open(el);
  if (!p.id) return;

  const want = parts.sampleAt(t, parts.position(m));
  let key = want;
  while (key > 0 && !t.samples[key].key) key--;
  if (want < p.shown || p.fed < key - 1) Object.assign(p, { fed: key - 1, shown: -1, ready: new Map() });

  while (p.fed < Math.min(want + AHEAD, t.samples.length - 1)) {
    const sample = t.samples[p.fed + 1];
    const bytes = parts.slice(m, sample.offset, sample.offset + sample.size);
    if (bytes === null) break;
    if (!ak.mediaDecoderFeed(p.id, p.fed + 1, bytes)) return parts.broke(el);

    p.fed++;
  }

  let best = -1;
  for (const seq of p.ready.keys()) if (seq <= want && seq > best) best = seq;
  if (best > p.shown) {
    ak.mediaFrame(el, t.a, t.b, p.ready.get(best));
    p.shown = best;
  }

  for (const seq of [...p.ready.keys()]) if (seq <= best) p.ready.delete(seq);
}

parts.pictureShow = show;
parts.pictureStop = m => {
  const p = m.picture;
  if (!p) return;

  m.picture = null;
  if (p.timer) ak.clearTimer(p.timer);
  if (p.id) ak.mediaDecoderClose(p.id);
};

// While the clock runs, wakes at each frame's time.
parts.pictureRun = el => {
  const m = parts.stateOf(el);
  const t = m.pictureTrack;
  show(el);

  const p = m.picture;
  if (!p) return;

  if (p.timer) ak.clearTimer(p.timer);
  p.timer = 0;

  const next = t.samples[parts.sampleAt(t, parts.position(m)) + 1];
  if (!m.timer || !next) return;

  p.timer = ak.setTimer(() => { p.timer = 0; parts.pictureRun(el); }, Math.max(1, (next.time - parts.position(m)) / m.rate * 1000), false, [], false);
};
})();
