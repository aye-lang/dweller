(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const texImage3D = p._texImage3D;
const INVALID_ENUM = 0x500, INVALID_VALUE = 0x501, INVALID_OPERATION = 0x502;

function domSource(source, unpack) {
  const fromUnpack = {kind: 1, node: source, flip: unpack[0x9240], premul: unpack[0x9241], sw: 0, sh: 0};
  if (source instanceof ImageData) return {...fromUnpack, kind: 2, node: source.data, sw: source.width, sh: source.height};
  if (source instanceof ImageBitmap) return {...fromUnpack, node: source._node, flip: false, premul: source._premultiplied};
  if (source instanceof OffscreenCanvas) return {...fromUnpack, node: source._node};
  if (source instanceof HTMLImageElement || source instanceof HTMLCanvasElement) return fromUnpack;
  throw new TypeError('The provided value is not a valid texture source');
}

p._texImage3D = function (sub, a, border) {
  const [target, level, internalformat, xo, yo, zo, width, height, depth, format, type, source] = a;
  if (source === null || source === undefined || typeof source === 'number' || ArrayBuffer.isView(source)) {
    texImage3D.call(this, sub, a, border);
    return;
  }
  const src = domSource(source, this._unpack);
  const t = target >>> 0, w = width | 0, h = height | 0, d = depth | 0, u = this._unpack;
  if (t !== 0x806f && t !== 0x8c1a) {
    ak.webglRaise(this._node, INVALID_ENUM);
    return;
  }
  if (!this._boundTexture(t) || this._bound.get(0x88ec)) {
    ak.webglRaise(this._node, INVALID_OPERATION);
    return;
  }
  if (w < 0 || h < 0 || d < 1 || (!sub && (border | 0) !== 0)) {
    ak.webglRaise(this._node, INVALID_VALUE);
    return;
  }
  ak.webglTex(this._node, [sub ? 1 : 0, t, level | 0, internalformat | 0, xo | 0, yo | 0, w, h, format >>> 0, type >>> 0, src.flip ? 1 : 0, src.premul ? 1 : 0, src.kind, 0, u[0xcf5], u[0xcf2], u[0xcf3], u[0xcf4], src.sw, src.sh, zo | 0, d, u[0x806e]], src.node);
};
})();
