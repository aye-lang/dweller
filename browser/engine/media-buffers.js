// spec: https://w3c.github.io/media-source/#sourcebuffer-coded-frame-processing
// chromium: media/filters/frame_processor.cc, media/filters/source_buffer_range.cc
(function () {
const G = globalThis;
const parts = G.__mediaParts;
const PCM = { 'A_PCM/INT/LIT': true, 'A_PCM/FLOAT/IEEE': true };

// The first frame at or after `time`.
function lower(frames, time) {
  let lo = 0, hi = frames.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (frames[mid].time < time) lo = mid + 1;
    else hi = mid;
  }

  return lo;
}

// Frames from `start` to before `end` go, and after them the frames that needed them (up to the next key frame).
function drop(tb, start, end) {
  const frames = tb.frames;
  const i = lower(frames, start);
  let j = i;
  while (j < frames.length && frames[j].time < end) j++;
  if (j === i) return false;

  while (j < frames.length && !frames[j].key) j++;
  frames.splice(i, j - i);

  return true;
}

// A container that gives no duration: PCM by its bytes, the track's default, the distance to the next frame, else the last one's.
function durations(tb, batch) {
  const t = tb.track;
  batch.forEach((f, i) => {
    if (f.duration > 0) return;

    if (t.kind === 1 && PCM[t.codec]) f.duration = f.data.length / (t.a * t.b / 8) / t.rate;
    else if (t.frame > 0) f.duration = t.frame;
    else if (i + 1 < batch.length) f.duration = batch[i + 1].time - f.time;
    else f.duration = (i > 0 ? batch[i - 1].duration : tb.lastDuration) || 1 / 30;
  });
}

function restart(sb) {
  for (const tb of sb.tracks.values()) Object.assign(tb, { lastTime: null, lastDuration: 0, needKey: true });
}

function place(sb, tb, f) {
  let time = 0;
  for (let again = 0; again < 2; again++) {
    if (sb.mode === 'sequence' && sb.groupStart !== null) {
      sb.offset = sb.groupStart - f.time;
      sb.groupEnd = Math.max(sb.groupStart, sb.groupEnd);
      sb.groupStart = null;
      for (const other of sb.tracks.values()) other.needKey = true;
    }

    time = f.time + sb.offset;
    if (tb.lastTime === null || (time >= tb.lastTime && time - tb.lastTime <= 2 * tb.lastDuration)) break;

    if (sb.mode === 'segments') sb.groupEnd = time;
    else sb.groupStart = sb.groupEnd;

    restart(sb);
  }

  const end = time + f.duration;
  if (time < sb.windowStart || end > sb.windowEnd) {
    tb.needKey = true;

    return;
  }

  if (tb.needKey && !f.key) return;

  tb.needKey = false;
  drop(tb, time, end);
  tb.frames.splice(lower(tb.frames, time), 0, { time, duration: f.duration, key: f.key, data: f.data, size: f.data.length });
  Object.assign(tb, { lastTime: time, lastDuration: f.duration, span: Math.max(tb.span, f.duration) });
  sb.groupEnd = Math.max(sb.groupEnd, end);
}

function intersect(a, b) {
  const out = [];
  let i = 0, j = 0;
  while (i < a.length && j < b.length) {
    const from = Math.max(a[i][0], b[j][0]), to = Math.min(a[i][1], b[j][1]);
    if (to > from) out.push([from, to]);

    if (a[i][1] < b[j][1]) i++;
    else j++;
  }

  return out;
}

function trackRanges(tb) {
  const out = [];
  const near = Math.max(0.001, 2 * tb.span);
  for (const f of tb.frames) {
    const last = out[out.length - 1];
    if (last && f.time <= last[1] + near) last[1] = Math.max(last[1], f.time + f.duration);
    else out.push([f.time, f.time + f.duration]);
  }

  return out;
}

// What every one of `lists` has; when the stream ended, each list reaches the highest end among them.
function common(lists, ended) {
  const highest = Math.max(0, ...lists.map(l => l.length ? l[l.length - 1][1] : 0));
  let out = null;
  for (const list of lists) {
    const l = list.map(r => r.slice());
    if (ended && l.length) l[l.length - 1][1] = highest;

    out = out === null ? l : intersect(out, l);
  }

  return out || [];
}

parts.buffers = {
  track: t => ({ track: t, frames: t.samples, lastTime: null, lastDuration: 0, needKey: true, span: 0 }),
  // The frames of one append, in the order of the stream.
  add(sb, frames) {
    const by = new Map();
    for (const f of frames) {
      const tb = sb.tracks.get(f.track);
      if (tb) by.set(tb, (by.get(tb) || []).concat([f]));
    }

    for (const [tb, batch] of by) durations(tb, batch);
    for (const f of frames) if (sb.tracks.has(f.track)) place(sb, sb.tracks.get(f.track), f);
  },
  remove(sb, start, end) {
    for (const tb of sb.tracks.values()) {
      if (drop(tb, start, end)) tb.needKey = true;
    }
  },
  restart,
  ranges: (sb, ended) => common(Array.from(sb.tracks.values(), trackRanges), ended),
  common,
  end: sb => Math.max(0, ...Array.from(sb.tracks.values(), tb => tb.frames.length ? tb.frames[tb.frames.length - 1].time + tb.frames[tb.frames.length - 1].duration : 0)),
};
})();
