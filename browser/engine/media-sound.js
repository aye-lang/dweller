// spec: https://html.spec.whatwg.org/multipage/media.html#playing-the-media-resource
// chromium: media/renderers/audio_renderer_impl.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
const WINDOW_S = 1;
const LATENCY_S = 0.02;
let context = null;

const le = (s, at, n) => { let v = 0; for (let i = n - 1; i >= 0; i--) v = v * 256 + s.charCodeAt(at + i); return v; };
const put = (n, bytes) => { let s = ''; for (let i = 0; i < bytes; i++) s += String.fromCharCode((n / 2 ** (8 * i)) & 255); return s; };

// chromium: media/audio/wav_audio_handler.cc
function format(head) {
  let at = 12, found = null;
  while (at + 8 <= head.length) {
    const size = le(head, at + 4, 4);
    if (head.startsWith('fmt ', at)) found = { fmt: head.slice(at, at + 8 + size), block: le(head, at + 20, 2), rate: le(head, at + 12, 4) };
    else if (head.startsWith('data', at)) return found && Object.assign(found, { data: at + 8 });

    at += 8 + size + (size & 1);
  }

  return null;
}

function slice(m, from, to) {
  let out = '';
  for (const [start, bytes] of m.bytes) {
    const a = Math.max(from, start), b = Math.min(to, start + bytes.length);
    if (b > a && a === from + out.length) out += bytes.slice(a - start, b - start);
  }

  return out.length === to - from ? out : null;
}

function decoded(m, t0, t1) {
  const f = m.format || (m.format = m.bytes.length && m.bytes[0][0] === 0 ? format(m.bytes[0][1]) : null);
  if (!f) return null;

  const pcm = slice(m, f.data + Math.round(t0 * f.rate) * f.block, Math.min(f.data + Math.round(t1 * f.rate) * f.block, m.total));
  if (!pcm) return null;

  const wav = 'RIFF' + put(4 + f.fmt.length + 8 + pcm.length, 4) + 'WAVE' + f.fmt + 'data' + put(pcm.length, 4) + pcm;
  const u = new Uint8Array(wav.length);
  ak.latin1Into(u, wav);

  return context.decodeAudioData(u.buffer);
}

function schedule(el) {
  const m = parts.stateOf(el);
  const s = m.sound;
  if (!s || s.busy || s.at >= parts.playableEnd(m) || s.when - context.currentTime > WINDOW_S / 2) return;

  const t0 = s.at, t1 = Math.min(t0 + WINDOW_S, parts.playableEnd(m));
  const wait = decoded(m, t0, t1);
  if (!wait) return;

  s.busy = true;
  wait.then(buffer => {
    s.busy = false;
    if (m.sound !== s) return;

    const node = context.createBufferSource();
    node.buffer = buffer;
    node.playbackRate.value = m.rate;
    node.connect(s.gain);
    node.start(Math.max(s.when, context.currentTime));
    s.nodes.push(node);
    s.at = t1;
    s.when = Math.max(s.when, context.currentTime) + (t1 - t0) / m.rate;
    schedule(el);
  }, () => { s.busy = false; });
}

parts.soundStop = m => {
  if (!m.sound) return;

  for (const n of m.sound.nodes) n.stop();
  m.sound.gain.disconnect();
  m.sound = null;
};

parts.soundStart = el => {
  const m = parts.stateOf(el);
  parts.soundStop(m);
  if (!m.audio || m.muted || m.volume === 0 || typeof AudioContext !== 'function') return;

  context = context || new AudioContext();
  context.resume();

  const gain = context.createGain();
  gain.gain.value = m.volume;
  gain.connect(context.destination);
  m.sound = { gain, nodes: [], at: m.at, when: context.currentTime + LATENCY_S, busy: false };
  schedule(el);
};

parts.soundMore = el => schedule(el);
parts.soundVolume = m => {
  if (m.sound) m.sound.gain.gain.value = m.muted ? 0 : m.volume;
};
})();
