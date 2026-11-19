// spec: https://html.spec.whatwg.org/multipage/media.html#user-interface
// chromium: third_party/blink/renderer/modules/media_controls/media_controls_impl.cc, resources/mediaControls.css
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const parts = G.__mediaParts;
const P = HTMLMediaElement.prototype;
const play = P.play, pause = P.pause;
const set = name => Object.getOwnPropertyDescriptor(P, name).set;
const setTime = set('currentTime'), setMuted = set('muted'), setVolume = set('volume');
const listen = EventTarget.prototype.addEventListener;
const SVG = 'http://www.w3.org/2000/svg';
const ICONS = {
  play: 'M8 5v14l11-7z',
  pause: 'M6 19h4V5H6v14zm8-14v14h4V5h-4z',
  sound: 'M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02z',
  muted: 'M3 9v6h4l5 5V4L7 9H3zm18 .41L19.59 8 17 10.59 14.41 8 13 9.41 15.59 12 13 14.59 14.41 16 17 13.41 19.59 16 21 14.59 18.41 12z',
};
const STYLE = `
.panel { position: absolute; left: 0; top: 0; right: 0; bottom: 0; overflow: hidden; font: 12px sans-serif; }
.video { background: black; color: white; }
.audio { background: #f1f3f4; color: #202124; border-radius: 27px; }
canvas { position: absolute; left: 0; top: 0; width: 100%; height: 100%; object-fit: contain; }
.bar { position: absolute; left: 0; right: 0; bottom: 0; height: 32px; display: flex; align-items: center; padding: 0 8px; }
.video .bar { background: rgba(0, 0, 0, 0.6); }
.audio .bar { top: 0; height: 54px; }
button { width: 24px; height: 24px; padding: 0; margin: 0 2px; border: none; background: transparent; color: inherit; }
svg { width: 24px; height: 24px; fill: currentColor; }
.time { margin: 0 6px; white-space: nowrap; }
input { margin: 0 4px; }
.seek { flex: 1 1 0; min-width: 0; }
.volume { flex: 0 0 40px; width: 40px; }
`;

const clock = s => !(s >= 0) || s === Infinity ? '0:00' : Math.floor(s / 60) + ':' + String(Math.floor(s % 60)).padStart(2, '0');

function make(root, name, cls, label) {
  const el = name === 'svg' || name === 'path' ? document.createElementNS(SVG, name) : document.createElement(name);
  if (cls) el.setAttribute('class', cls);
  if (label) el.setAttribute('aria-label', label);

  root.appendChild(el);

  return el;
}

function icon(button) {
  const svg = make(button, 'svg');
  svg.setAttribute('viewBox', '0 0 24 24');

  return make(svg, 'path');
}

function build(el) {
  const m = parts.stateOf(el);
  const root = ak.mediaControlsRoot(el);
  const video = el.localName === 'video';
  const size = make(root, 'style');
  make(root, 'style').textContent = STYLE;

  const panel = make(root, 'div', 'panel ' + el.localName);
  const screen = video ? make(panel, 'canvas') : null;
  const bar = make(panel, 'div', 'bar');
  const toggle = make(bar, 'button', 'play', 'play');
  const time = make(bar, 'span', 'time');
  const seek = make(bar, 'input', 'seek', 'seek');
  const mute = make(bar, 'button', 'mute', 'mute');
  const volume = make(bar, 'input', 'volume', 'volume');
  const c = { root, size, screen, time, seek, volume, toggleIcon: icon(toggle), muteIcon: icon(mute), toggle, mute };
  for (const [input, max] of [[seek, 1000], [volume, 100]]) Object.assign(input, { type: 'range', min: 0, max, step: 1 });

  const flip = () => { (m.paused || el.ended ? play.call(el) : Promise.resolve(pause.call(el))).catch(() => {}); };
  listen.call(toggle, 'click', flip);
  if (screen) listen.call(screen, 'click', flip);
  listen.call(mute, 'click', () => setMuted.call(el, !m.muted));
  listen.call(seek, 'input', () => { if (m.duration > 0) setTime.call(el, seek.value / 1000 * m.duration); });
  listen.call(volume, 'input', () => { setMuted.call(el, false); setVolume.call(el, volume.value / 100); });
  for (const type of ['play', 'pause', 'ended', 'timeupdate', 'seeking', 'seeked', 'durationchange', 'volumechange', 'loadedmetadata', 'emptied']) listen.call(el, type, () => update(el));

  m.controls = c;
  update(el);
  parts.pictureMirror(el);
}

function update(el) {
  const m = parts.stateOf(el);
  const c = m.controls;
  if (!c) return;

  const video = el.localName === 'video';
  const at = parts.position(m);
  const playing = !m.paused && !el.ended;
  const quiet = m.muted || m.volume === 0;
  const shown = [':host { display: inline-block; position: relative; width: ' + (video ? m.width || 300 : 300) + 'px; height: ' + (video ? m.height || 150 : 54) + 'px; }', playing, quiet, clock(at) + ' / ' + clock(m.duration)];
  const was = c.shown || [];
  c.shown = shown;
  if (shown[0] !== was[0]) c.size.textContent = shown[0];
  if (shown[1] !== was[1]) {
    c.toggleIcon.setAttribute('d', playing ? ICONS.pause : ICONS.play);
    c.toggle.setAttribute('aria-label', playing ? 'pause' : 'play');
  }

  if (shown[2] !== was[2]) {
    c.muteIcon.setAttribute('d', quiet ? ICONS.muted : ICONS.sound);
    c.mute.setAttribute('aria-label', quiet ? 'unmute' : 'mute');
  }

  if (shown[3] !== was[3]) c.time.textContent = shown[3];

  c.seek.value = m.duration > 0 ? Math.round(at / m.duration * 1000) : 0;
  c.volume.value = m.muted ? 0 : Math.round(m.volume * 100);
}

// The controls exist from the first time the element has the attribute; without it the layout shows the element as before.
parts.controls = el => {
  if (el.hasAttribute('controls') && !parts.stateOf(el).controls) build(el);
};
})();
