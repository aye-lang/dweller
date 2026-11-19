// spec: https://html.spec.whatwg.org/multipage/media.html#playing-the-media-resource
// chromium: media/renderers/audio_renderer_impl.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
const AHEAD_S = 1;
const LATENCY_S = 0.02;
let context = null;
let sounding = 0;

parts.slice = (m, from, to) => {
  let out = '';
  for (const [start, bytes] of m.bytes) {
    const a = Math.max(from, start), b = Math.min(to, start + bytes.length);
    if (b > a && a === from + out.length) out += bytes.slice(a - start, b - start);
  }

  return out.length === to - from ? out : null;
};

// The tracks of the resource: `a` and `b` are channels and bits (audio), width and height (video).
parts.tracks = (bytes, total) => ak.mediaTracks(bytes, total).map(([kind, codec, config, a, b, rate, flat]) => {
  const samples = [];
  for (let i = 0; i < flat.length; i += 4) samples.push({ offset: flat[i], size: flat[i + 1], time: flat[i + 2], key: flat[i + 3] });

  return { kind, codec, config, a, b, rate, samples };
});

parts.sampleAt = (t, time) => {
  let lo = 0, hi = t.samples.length - 1;
  while (lo < hi) {
    const mid = (lo + hi + 1) >> 1;
    if (t.samples[mid].time <= time) lo = mid;
    else hi = mid - 1;
  }

  return lo;
};

function channelsOf(data, count) {
  const u = new Uint8Array(data.length - data.length % 4);
  ak.latin1Into(u, data.slice(0, u.length));

  const all = new Float32Array(u.buffer);
  const n = Math.floor(all.length / count);
  const out = [];
  for (let c = 0; c < count; c++) {
    const one = new Float32Array(n);
    for (let i = 0; i < n; i++) one[i] = all[i * count + c];
    out.push(one);
  }

  return out;
}

function heard(el, s, seq, ok, data) {
  const m = parts.stateOf(el);
  if (m.sound !== s) return;

  s.waiting--;
  if (!ok) return parts.broke(el);

  const t = m.soundTrack;
  const channels = channelsOf(data, t.a);
  const skip = Math.max(0, s.from - t.samples[seq].time);
  if (channels[0].length / t.rate > skip) {
    const buffer = context.createBuffer(t.a, channels[0].length, t.rate);
    channels.forEach((c, i) => buffer.copyToChannel(c, i));

    const node = context.createBufferSource();
    const when = Math.max(s.when, context.currentTime);
    node.buffer = buffer;
    node.playbackRate.value = m.rate;
    node.connect(s.gain);
    node.start(when, skip);
    s.nodes.push(node);
    s.when = when + (buffer.duration - skip) / m.rate;
  }

  schedule(el);
}

function schedule(el) {
  const m = parts.stateOf(el);
  const s = m.sound;
  if (!s) return;

  const t = m.soundTrack;
  while (s.next < t.samples.length && s.waiting < 2 && s.when - context.currentTime <= AHEAD_S / 2) {
    const sample = t.samples[s.next];
    const bytes = parts.slice(m, sample.offset, sample.offset + sample.size);
    if (bytes === null) return;
    if (!ak.mediaDecoderFeed(s.id, s.next, bytes)) return parts.broke(el);

    s.next++;
    s.waiting++;
  }
}

parts.soundStop = m => {
  const s = m.sound;
  if (!s) return;

  m.sound = null;
  for (const n of s.nodes) n.stop();
  s.gain.disconnect();
  ak.mediaDecoderClose(s.id);
  sounding--;
  if (sounding === 0) context.suspend();
};

parts.soundStart = el => {
  const m = parts.stateOf(el);
  const t = m.soundTrack;
  parts.soundStop(m);
  if (!t || !t.samples.length || m.muted || m.volume === 0 || typeof AudioContext !== 'function') return;

  context = context || new AudioContext();
  context.resume();

  const gain = context.createGain();
  const s = { id: 0, gain, nodes: [], next: parts.sampleAt(t, m.at), from: m.at, when: context.currentTime + LATENCY_S, waiting: 0 };
  gain.gain.value = m.volume;
  gain.connect(context.destination);
  m.sound = s;
  sounding++;
  s.id = ak.mediaDecoderOpen(1, t.codec, t.config, (seq, ok, data) => heard(el, s, seq, ok, data), t.a, t.b, t.rate);
  if (!s.id) return parts.broke(el);

  schedule(el);
};

parts.soundMore = el => schedule(el);
parts.soundVolume = m => {
  if (m.sound) m.sound.gain.gain.value = m.muted ? 0 : m.volume;
};
})();
