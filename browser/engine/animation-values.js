globalThis.__animationValues = function (ak, AnimationEffect, KeyframeEffect, hidden, cssName, fills) {
// spec: https://drafts.csswg.org/web-animations-1/#calculating-the-transformed-progress
const progressOf = AnimationEffect.prototype._progress;
hidden(AnimationEffect.prototype, '_progress', function (lt, rate) {
  const p = progressOf.call(this, lt, rate);

  if (p && p.progress !== null) p.progress = ak.ease(this._timing.easing, p.progress);

  return p;
});

function computedWith(el, name, value) {
  const old = el.style.getPropertyValue(name);

  el.style.setProperty(name, value);

  const v = getComputedStyle(el).getPropertyValue(name);

  el.style.setProperty(name, old);

  return v;
}

function underlying(el, name) {
  const rec = fills.get(el) && fills.get(el).get(name);

  return rec ? rec.base : el.style.getPropertyValue(name);
}

// spec: https://drafts.csswg.org/web-animations-1/#the-effect-value-of-a-keyframe-animation-effect
function valueAt(el, name, frames, progress, easing) {
  if (frames[0].offset !== 0) frames.unshift({ offset: 0, easing, value: null });
  if (frames[frames.length - 1].offset !== 1) frames.push({ offset: 1, easing, value: null });

  let i = 0;

  while (i < frames.length - 2 && frames[i + 1].offset <= progress) i++;

  const a = frames[i], b = frames[i + 1];
  const va = a.value === null ? underlying(el, name) : a.value;
  const vb = b.value === null ? underlying(el, name) : b.value;
  const span = b.offset - a.offset;
  const local = span === 0 ? 1 : ak.ease(a.easing, (progress - a.offset) / span);

  if (local === 0) return va;
  if (local === 1) return vb;

  const r = ak.interpolate(name, computedWith(el, name, va), computedWith(el, name, vb), local);

  if (r !== null) return r;

  return local < 0.5 ? va : vb;
}

const valuesOf = KeyframeEffect.prototype._valuesAt;
hidden(KeyframeEffect.prototype, '_valuesAt', function (progress) {
  const el = this.target;

  if (!el || !el.style || !el.isConnected) return valuesOf.call(this, progress);

  const byName = new Map();

  for (const f of this._frames) {
    for (const p of Object.keys(f.props)) {
      const name = cssName(p);

      if (!byName.has(name)) byName.set(name, []);
      byName.get(name).push({ offset: f.computed, easing: f.easing, value: f.props[p] });
    }
  }

  const out = {};

  for (const [name, frames] of byName) out[name] = valueAt(el, name, frames, progress, this._implicitEasing || 'linear');

  return out;
});
};
