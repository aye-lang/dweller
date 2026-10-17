globalThis.__cssAnimations = function (ak, Animation, KeyframeEffect, timeline, hidden) {
// spec: https://drafts.csswg.org/css-animations-1/
const running = new WeakMap();
const tracked = new Set();
let syncing = false;
let timer = 0;

function watch() {
  if (!timer && Array.from(tracked).some(el => running.get(el).some(a => a.playState === 'running'))) timer = ak.setTimer(resync, 16, false, [], true);
}

function resync() {
  timer = 0;

  for (const el of Array.from(tracked)) sync(el);
}

function items(v) {
  const out = [];
  let depth = 0, start = 0;

  for (let i = 0; i < v.length; i++) {
    if (v[i] === '(') depth++;
    else if (v[i] === ')') depth--;
    else if (v[i] === ',' && depth === 0) {
      out.push(v.slice(start, i).trim());
      start = i + 1;
    }
  }

  out.push(v.slice(start).trim());

  return out;
}

function seconds(v) {
  return v.endsWith('ms') ? parseFloat(v) : parseFloat(v) * 1000;
}

function unquoted(name) {
  return name.startsWith('"') ? name.slice(1, -1) : name;
}

function framesOf(name, easing) {
  const flat = ak.keyframes(name);

  if (flat === null) return null;

  const byOffset = new Map();

  for (let i = 0; i < flat.length; i += 3) {
    const [offset, prop, value] = [flat[i], flat[i + 1], flat[i + 2]];

    if (!byOffset.has(offset)) byOffset.set(offset, { offset, easing });
    if (prop === 'animation-timing-function') byOffset.get(offset).easing = value;
    else if (!prop.startsWith('animation') && !prop.startsWith('transition')) byOffset.get(offset)[prop] = value;
  }

  return Array.from(byOffset.values()).sort((a, b) => a.offset - b.offset);
}

function timingAt(style, i) {
  const at = (n) => { const l = items(style[n]); return l[i % l.length]; };
  const count = at('animation-iteration-count');

  return {
    duration: seconds(at('animation-duration')),
    delay: seconds(at('animation-delay')),
    iterations: count === 'infinite' ? Infinity : Number(count),
    direction: at('animation-direction'),
    fill: at('animation-fill-mode'),
    easing: at('animation-timing-function'),
    paused: at('animation-play-state') === 'paused',
  };
}

function send(anim, events) {
  const el = anim._effect && anim._effect.target;

  for (const [type, elapsed] of events) {
    const ev = new AnimationEvent(type, { bubbles: true, animationName: anim.animationName, elapsedTime: elapsed / 1000 });

    ev.isTrusted = true;
    if (el) el.dispatchEvent(ev);
  }
}

// spec: https://drafts.csswg.org/css-animations-1/#event-dispatch
function eventsOf(was, now, a) {
  const t = a._effect._timing, active = a._effect._times().active;
  const first = Math.max(Math.min(-t.delay, active), 0), last = Math.max(Math.min(a._effect._times().end - t.delay, active), 0);
  const leave = was.phase === 'idle' || was.phase === 'before';

  if (now.phase === 'idle') return was.phase === 'idle' || was.phase === 'after' ? [] : [['animationcancel', a._cancelTime]];
  if (leave && now.phase === 'active') return [['animationstart', first]];
  if (leave && now.phase === 'after') return [['animationstart', first], ['animationend', last]];
  if (was.phase === 'active' && now.phase === 'before') return [['animationend', first]];
  if (was.phase === 'active' && now.phase === 'after') return [['animationend', last]];
  if (was.phase === 'active' && now.phase === 'active' && was.iteration !== now.iteration) return [['animationiteration', now.iteration * t.duration]];
  if (was.phase === 'after' && now.phase === 'active') return [['animationstart', last]];
  if (was.phase === 'after' && now.phase === 'before') return [['animationstart', last], ['animationend', first]];

  return [];
}

class CSSAnimation extends Animation {
  get animationName() { return this._name; }
  _update() {
    const outer = syncing;

    syncing = true;

    try {
      super._update();
    } finally {
      syncing = outer;
    }

    if (!this._name || !this._effect) return;

    if (!this._checking) {
      this._checking = true;
      setTimeout(() => this._sendEvents(), 0);
    }

    this._waitIteration();
  }
  cancel() {
    if (this._name && this._effect && this.playState !== 'idle') this._cancelTime = Math.max(Math.min(this.currentTime - this._effect._timing.delay, this._effect._times().active), 0);

    super.cancel();
  }
  _state() {
    const ct = this.currentTime;

    if (ct === null || this.playState === 'idle') return { phase: 'idle', iteration: null };

    const p = this._effect._progress(ct, this._rate);

    return { phase: p.phase, iteration: p.phase === 'active' ? p.iteration : null };
  }
  _sendEvents() {
    this._checking = false;

    if (!this._effect) return;

    const now = this._state();

    send(this, eventsOf(this._was, now, this));
    this._was = now;
  }
  _waitIteration() {
    clearTimeout(this._iterTimer);

    const t = this._effect._timing, now = this._state();

    if (now.iteration === null || this.playState !== 'running' || this._rate <= 0 || !(t.duration > 0) || now.iteration + 1 >= t.iterations) return;

    const wait = (t.delay + (now.iteration + 1) * t.duration - this.currentTime) / this._rate;

    this._iterTimer = setTimeout(() => this._update(), Math.max(Math.ceil(wait), 0));
  }
}

function start(el, name, frames, timing) {
  const effect = new KeyframeEffect(el, frames, { duration: timing.duration, delay: timing.delay, iterations: timing.iterations, direction: timing.direction, fill: timing.fill === 'none' ? 'auto' : timing.fill });
  const a = new CSSAnimation(effect, timeline);

  hidden(effect, '_implicitEasing', timing.easing);

  hidden(a, '_name', name);
  hidden(a, '_was', { phase: 'idle', iteration: null });
  hidden(a, '_checking', false);
  hidden(a, '_cancelTime', 0);
  hidden(a, '_iterTimer', 0);
  a.play();

  return a;
}

function retime(a, timing) {
  a._effect.updateTiming({ duration: timing.duration, delay: timing.delay, iterations: timing.iterations, direction: timing.direction, fill: timing.fill === 'none' ? 'auto' : timing.fill });

  if (timing.paused && a.playState !== 'paused') a.pause();
  if (!timing.paused && a.playState === 'paused') a.play();
}

// spec: https://drafts.csswg.org/css-animations-1/#animations
function sync(el) {
  if (syncing) return;

  syncing = true;

  try {
    const style = {};

    for (const n of ['animation-name', 'animation-duration', 'animation-delay', 'animation-iteration-count', 'animation-direction', 'animation-fill-mode', 'animation-timing-function', 'animation-play-state', 'display']) style[n] = ak.computedStyle(el, n);

    const old = running.get(el) || [];
    const names = style.display === 'none' ? [] : items(style['animation-name']).map(unquoted);
    const next = [];

    names.forEach((name, i) => {
      if (name === 'none') return;

      const timing = timingAt(style, i);
      const k = old.findIndex(a => a._name === name);

      if (k >= 0) {
        const a = old.splice(k, 1)[0];

        retime(a, timing);
        next.push(a);

        return;
      }

      const frames = framesOf(name, timing.easing);

      if (frames === null) return;

      const a = start(el, name, frames, timing);

      if (timing.paused) a.pause();
      next.push(a);
    });

    for (const a of old) a.cancel();

    if (next.length > 0) running.set(el, next);
    else running.delete(el);

    if (next.length > 0) tracked.add(el);
    else tracked.delete(el);
  } finally {
    syncing = false;
  }

  watch();
}

Object.defineProperty(CSSAnimation.prototype, Symbol.toStringTag, { value: 'CSSAnimation', configurable: true });

return { sync, CSSAnimation };
};
