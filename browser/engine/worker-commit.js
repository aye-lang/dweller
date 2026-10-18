(function () {
const P = OffscreenCanvas.prototype;
const getContext = P.getContext;
const linked = new Set();
P.getContext = function (kind, options) {
  const context = getContext.call(this, kind, options);
  if (context && this._placeholder) linked.add(this);
  return context;
};
P.commit = function () {
  if (this._placeholder) frame(this._node, this._placeholder);
};
const frames = new Map();
let nextFrame = 1;
globalThis.requestAnimationFrame = function requestAnimationFrame(callback) {
  const id = nextFrame++;
  frames.set(id, setTimeout(() => {
    frames.delete(id);
    callback(performance.now());
    for (const canvas of linked) canvas.commit();
  }, 16));
  return id;
};
globalThis.cancelAnimationFrame = function cancelAnimationFrame(id) {
  clearTimeout(frames.get(id));
  frames.delete(id);
};
})();
