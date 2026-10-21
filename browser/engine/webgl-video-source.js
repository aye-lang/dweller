(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const INVALID_OPERATION = 0x502, PIXEL_UNPACK_BUFFER = 0x88ec;
function videoSource(original) {
  return function (...args) {
    if (!(args[args.length - 1] instanceof HTMLVideoElement)) return original.apply(this, args);
    if (this._bound.get(PIXEL_UNPACK_BUFFER)) ak.webglRaise(this._node, INVALID_OPERATION);
  };
}
p.texImage2D = videoSource(p.texImage2D);
p.texSubImage2D = videoSource(p.texSubImage2D);
})();
