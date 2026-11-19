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

  p.ready.set(p.sent.get(seq), data);
  p.sent.delete(seq);
  show(el);
}

function open(el) {
  const m = parts.stateOf(el);
  const t = m.pictureTrack;
  const p = { id: 0, fed: -Infinity, shown: -Infinity, seq: 0, sent: new Map(), done: new Set(), ready: new Map(), timer: 0, last: '' };
  m.picture = p;
  p.id = ak.mediaDecoderOpen(2, t.codec, t.config, (seq, ok, data) => seen(el, p, seq, ok, data), t.a, t.b, 0);
  if (!p.id) parts.broke(el);

  return p;
}

// Feeds the decoder up to a few frames past the playback position (from the key frame before it after a seek) and shows the newest decoded frame that is due. Frames are known by their times: a source buffer's list changes under the sink.
function show(el) {
  const m = parts.stateOf(el);
  const t = m.pictureTrack;
  if (!t || !t.samples.length || m.error || el.localName !== 'video') return;

  const p = m.picture || open(el);
  if (!p.id) return;

  const at = parts.position(m);
  const i = parts.sampleAt(t, at);
  const want = t.samples[i];
  if (want.time > at + 1e-9 || (want.duration > 0 && at > want.time + want.duration + 1e-6 && at < m.duration)) return;

  let key = i;
  while (key > 0 && !t.samples[key].key) key--;

  const back = want.time < p.shown;
  const ahead = t.samples[key].time > p.fed && parts.sampleAfter(t, p.fed) < key;
  const changed = want.time <= p.fed && !p.done.has(want.time);
  if (back || ahead || changed) Object.assign(p, { fed: key > 0 ? t.samples[key - 1].time : -Infinity, shown: -Infinity, ready: new Map(), done: new Set() });

  for (let j = parts.sampleAfter(t, p.fed); j <= Math.min(i + AHEAD, t.samples.length - 1); j++) {
    const bytes = parts.sampleBytes(m, t.samples[j]);
    if (bytes === null) break;
    if (!ak.mediaDecoderFeed(p.id, ++p.seq, bytes)) return parts.broke(el);

    p.sent.set(p.seq, t.samples[j].time);
    p.done.add(t.samples[j].time);
    p.fed = t.samples[j].time;
  }

  let best = -Infinity;
  for (const time of p.ready.keys()) if (time <= want.time && time > best) best = time;
  if (best > p.shown) {
    p.last = p.ready.get(best);
    p.shown = best;
    ak.mediaFrame(el, t.a, t.b, p.last);
    if (m.controls) ak.mediaFrame(m.controls.screen, t.a, t.b, p.last);
  }

  for (const time of [...p.ready.keys()]) if (time <= best) p.ready.delete(time);
  if (p.done.size > 512) p.done = new Set([...p.done].filter(time => time >= want.time));
}

parts.pictureShow = show;
parts.pictureMirror = el => {
  const m = parts.stateOf(el);
  if (m.controls && m.controls.screen && m.pictureTrack) ak.mediaFrame(m.controls.screen, m.width, m.height, m.picture ? m.picture.last : '');
};
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

  const next = t.samples[parts.sampleAfter(t, parts.position(m))];
  if (!m.timer || !next) return;

  p.timer = ak.setTimer(() => { p.timer = 0; parts.pictureRun(el); }, Math.max(1, (next.time - parts.position(m)) / m.rate * 1000), false, [], false);
};
})();
