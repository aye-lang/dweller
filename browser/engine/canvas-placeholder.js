(function (G) {
const P = HTMLCanvasElement.prototype;
const getContext = P.getContext;
const placeholders = new Map();
const ids = new WeakMap();
let next = 1;
const invalid = message => new DOMException(message, 'InvalidStateError');
P.getContext = function (kind, options) {
  if (ids.has(this)) throw invalid("Failed to execute 'getContext' on 'HTMLCanvasElement': Cannot get context from a canvas that has transferred its control to offscreen.");
  return getContext.call(this, kind, options);
};
P.transferControlToOffscreen = function () {
  if (ids.has(this)) throw invalid("Failed to execute 'transferControlToOffscreen' on 'HTMLCanvasElement': Cannot transfer control from a canvas for more than one time.");
  const id = next++;
  ids.set(this, id);
  placeholders.set(id, this);
  const offscreen = new OffscreenCanvas(this.width, this.height);
  offscreen._placeholder = id;
  return offscreen;
};
function show(id, w, h) {
  const el = placeholders.get(id);
  if (!el) return null;
  if (el.width !== w) el.width = w;
  if (el.height !== h) el.height = h;
  return getContext.call(el, '2d');
}
OffscreenCanvas.prototype.commit = function () {
  const context = show(this._placeholder, this.width, this.height);
  if (!context) return;
  context.clearRect(0, 0, this.width, this.height);
  context.drawImage(this._node, 0, 0);
};
Object.defineProperty(G, '__placeholderFrame', {value: function (id, w, h, pixels) {
  const context = show(id, w, h);
  if (context) context.putImageData(new ImageData(new Uint8ClampedArray(pixels), w, h), 0, 0);
}, configurable: true});
})(globalThis);
