(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const INVALID_VALUE = 0x501, INVALID_OPERATION = 0x502;
const powerOfTwo = n => n > 0 && (n & (n - 1)) === 0;
const fail = (ctx, code) => ak.webglRaise(ctx._node, code);
function levelFormats(ctx, target) {
  const face = target >= 0x8515 && target <= 0x851a;
  const tex = ctx._boundTexture(face ? 0x8513 : target);
  if (tex && !tex._compressedFormats) tex._compressedFormats = new Map();
  return tex && tex._compressedFormats;
}
function image(original) {
  return function (target, level, internalformat, width, height, ...rest) {
    if (!(this instanceof globalThis.WebGL2RenderingContext) && level > 0 && !(powerOfTwo(width) && powerOfTwo(height))) return fail(this, INVALID_VALUE);
    original.call(this, target, level, internalformat, width, height, ...rest);
    const formats = levelFormats(this, target >>> 0);
    if (formats) formats.set((target >>> 0) + ':' + level, internalformat >>> 0);
  };
}
function subImage(original, formatIndex) {
  return function (target, level, ...rest) {
    const formats = levelFormats(this, target >>> 0);
    const known = formats && formats.get((target >>> 0) + ':' + level);
    if (known !== undefined && known !== rest[formatIndex] >>> 0) return fail(this, INVALID_OPERATION);
    original.call(this, target, level, ...rest);
  };
}
p.compressedTexImage2D = image(p.compressedTexImage2D);
p.compressedTexImage3D = image(p.compressedTexImage3D);
p.compressedTexSubImage2D = subImage(p.compressedTexSubImage2D, 4);
p.compressedTexSubImage3D = subImage(p.compressedTexSubImage3D, 6);
})();
