(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const {WebGLProgram, WebGLShader} = globalThis;
const INVALID_ENUM = 0x500, INVALID_VALUE = 0x501, INVALID_OPERATION = 0x502;
const TEXTURE_2D = 0xde1, TEXTURE_CUBE_MAP = 0x8513, PIXEL_UNPACK_BUFFER = 0x88ec;
const MAX_ANISOTROPY = 0x84ff, ANISOTROPY = 0x84fe, COMPLETION_STATUS = 0x91b1, COMPRESSED_TEXTURE_FORMATS = 0x86a3;
const TARGETS_3D = new Set([0x806f, 0x8c1a]);
const TEX_PARAMS = new Set([0x2800, 0x2801, 0x2802, 0x2803, 0x8072, 0x813c, 0x813d, 0x884c, 0x884d, 0x82df, 0x813a, 0x813b, 0x912f]);
const DEBUG_STRINGS = new Map([[0x9245, 'Dweller'], [0x9246, 'Dweller WebGL']]);
const ASTC_BLOCKS = [[4, 4], [5, 4], [5, 5], [6, 5], [6, 6], [8, 5], [8, 6], [8, 8], [10, 5], [10, 6], [10, 8], [10, 10], [12, 10], [12, 12]];
const ASTC_NAMES = ASTC_BLOCKS.map(([w, h]) => w + 'x' + h);
function formats(names, first, bytes) {
  const o = {};
  names.forEach((n, i) => (o[n] = [first + i, 4, 4, bytes[i]]));
  return o;
}
const S3TC = formats(['COMPRESSED_RGB_S3TC_DXT1_EXT', 'COMPRESSED_RGBA_S3TC_DXT1_EXT', 'COMPRESSED_RGBA_S3TC_DXT3_EXT', 'COMPRESSED_RGBA_S3TC_DXT5_EXT'], 0x83f0, [8, 8, 16, 16]);
const S3TC_SRGB = formats(['COMPRESSED_SRGB_S3TC_DXT1_EXT', 'COMPRESSED_SRGB_ALPHA_S3TC_DXT1_EXT', 'COMPRESSED_SRGB_ALPHA_S3TC_DXT3_EXT', 'COMPRESSED_SRGB_ALPHA_S3TC_DXT5_EXT'], 0x8c4c, [8, 8, 16, 16]);
const ETC = formats(['COMPRESSED_R11_EAC', 'COMPRESSED_SIGNED_R11_EAC', 'COMPRESSED_RG11_EAC', 'COMPRESSED_SIGNED_RG11_EAC', 'COMPRESSED_RGB8_ETC2', 'COMPRESSED_SRGB8_ETC2', 'COMPRESSED_RGB8_PUNCHTHROUGH_ALPHA1_ETC2', 'COMPRESSED_SRGB8_PUNCHTHROUGH_ALPHA1_ETC2', 'COMPRESSED_RGBA8_ETC2_EAC', 'COMPRESSED_SRGB8_ALPHA8_ETC2_EAC'], 0x9270, [8, 8, 16, 16, 8, 8, 8, 8, 16, 16]);
const BPTC = formats(['COMPRESSED_RGBA_BPTC_UNORM_EXT', 'COMPRESSED_SRGB_ALPHA_BPTC_UNORM_EXT', 'COMPRESSED_RGB_BPTC_SIGNED_FLOAT_EXT', 'COMPRESSED_RGB_BPTC_UNSIGNED_FLOAT_EXT'], 0x8e8c, [16, 16, 16, 16]);
const RGTC = formats(['COMPRESSED_RED_RGTC1_EXT', 'COMPRESSED_SIGNED_RED_RGTC1_EXT', 'COMPRESSED_RED_GREEN_RGTC2_EXT', 'COMPRESSED_SIGNED_RED_GREEN_RGTC2_EXT'], 0x8dbb, [8, 8, 16, 16]);
const ASTC = {};
ASTC_BLOCKS.forEach(([w, h], i) => {
  ASTC['COMPRESSED_RGBA_ASTC_' + ASTC_NAMES[i] + '_KHR'] = [0x93b0 + i, w, h, 16];
  ASTC['COMPRESSED_SRGB8_ALPHA8_ASTC_' + ASTC_NAMES[i] + '_KHR'] = [0x93d0 + i, w, h, 16];
});
const NORM16 = {R16_EXT: 0x822a, RG16_EXT: 0x822c, RGB16_EXT: 0x8054, RGBA16_EXT: 0x805b, R16_SNORM_EXT: 0x8f98, RG16_SNORM_EXT: 0x8f99, RGB16_SNORM_EXT: 0x8f9a, RGBA16_SNORM_EXT: 0x8f9b};
const HALF_FLOAT = {RGBA16F_EXT: 0x881a, RGB16F_EXT: 0x881b, FRAMEBUFFER_ATTACHMENT_COMPONENT_TYPE_EXT: 0x8211, UNSIGNED_NORMALIZED_EXT: 0x8c17};
const EXTENSIONS = new Map([
  ['EXT_color_buffer_float', [['EXT_color_buffer_float'], {}]],
  ['EXT_color_buffer_half_float', [['EXT_color_buffer_half_float'], HALF_FLOAT]],
  ['EXT_float_blend', [['EXT_float_blend'], {}]],
  ['EXT_texture_filter_anisotropic', [['EXT_texture_filter_anisotropic'], {TEXTURE_MAX_ANISOTROPY_EXT: ANISOTROPY, MAX_TEXTURE_MAX_ANISOTROPY_EXT: MAX_ANISOTROPY}]],
  ['OES_texture_float_linear', [['OES_texture_float_linear'], {}]],
  ['EXT_texture_norm16', [['EXT_texture_norm16'], NORM16]],
  ['KHR_parallel_shader_compile', [['KHR_parallel_shader_compile'], {COMPLETION_STATUS_KHR: COMPLETION_STATUS}]],
  ['WEBGL_debug_renderer_info', [[], {UNMASKED_VENDOR_WEBGL: 0x9245, UNMASKED_RENDERER_WEBGL: 0x9246}]],
  ['WEBGL_compressed_texture_s3tc', [['EXT_texture_compression_dxt1', 'ANGLE_texture_compression_dxt3', 'ANGLE_texture_compression_dxt5'], {}, S3TC]],
  ['WEBGL_compressed_texture_s3tc_srgb', [['EXT_texture_compression_s3tc_srgb'], {}, S3TC_SRGB]],
  ['WEBGL_compressed_texture_etc', [['ANGLE_compressed_texture_etc'], {}, ETC]],
  ['WEBGL_compressed_texture_astc', [['KHR_texture_compression_astc_ldr'], {getSupportedProfiles: () => ['ldr']}, ASTC]],
  ['EXT_texture_compression_bptc', [['EXT_texture_compression_bptc'], {}, BPTC]],
  ['EXT_texture_compression_rgtc', [['EXT_texture_compression_rgtc'], {}, RGTC]],
]);
const fail = (ctx, code) => (ak.webglRaise(ctx._node, code), null);
function supported(ctx, name) {
  return EXTENSIONS.get(name)[0].every(n => ak.webglQuery(ctx._node, 2, 0, 'GL_' + n));
}
function makeExtension(ctx, name) {
  const [, members, compressed = {}] = EXTENSIONS.get(name);
  if (!supported(ctx, name)) return null;
  const ext = Object.assign({}, members);
  for (const k in compressed) {
    ext[k] = compressed[k][0];
    ctx._blocks.set(compressed[k][0], compressed[k].slice(1));
  }
  return ext;
}
function enabled(ctx, name) {
  return !!ctx._exts.get(name);
}
p.getSupportedExtensions = function () {
  return this.isContextLost() ? null : [...EXTENSIONS.keys()].filter(n => supported(this, n));
};
p.getExtension = function (name) {
  name = String(name);
  if (this.isContextLost() || !EXTENSIONS.has(name)) return null;
  if (!this._blocks) this._blocks = new Map();
  if (!this._exts.has(name)) this._exts.set(name, makeExtension(this, name));
  return this._exts.get(name);
};
const getParameter = p.getParameter;
p.getParameter = function (pname) {
  pname = pname >>> 0;
  if (pname === MAX_ANISOTROPY && enabled(this, 'EXT_texture_filter_anisotropic')) return ak.webglGet(this._node, pname, 1, 1)[0];
  if (DEBUG_STRINGS.has(pname) && enabled(this, 'WEBGL_debug_renderer_info')) return DEBUG_STRINGS.get(pname);
  if (pname === COMPRESSED_TEXTURE_FORMATS) return new Uint32Array(this._blocks ? [...this._blocks.keys()] : []);
  return getParameter.call(this, pname);
};
function completion(original, cls, kind) {
  return function (o, pname) {
    if (pname >>> 0 !== COMPLETION_STATUS || !enabled(this, 'KHR_parallel_shader_compile')) return original.call(this, o, pname);
    if (!this._live(o, cls)) return null;
    return ak.webglParam(this._node, kind, o._name, COMPLETION_STATUS) !== 0;
  };
}
p.getShaderParameter = completion(p.getShaderParameter, WebGLShader, 0);
p.getProgramParameter = completion(p.getProgramParameter, WebGLProgram, 1);
p.getTexParameter = function (target, pname) {
  target = target >>> 0;
  pname = pname >>> 0;
  if (!this._textureBound(target, new Set([TEXTURE_2D, TEXTURE_CUBE_MAP, ...TARGETS_3D]))) return null;
  const anisotropy = pname === ANISOTROPY && enabled(this, 'EXT_texture_filter_anisotropic');
  if (!TEX_PARAMS.has(pname) && !anisotropy) return fail(this, INVALID_ENUM);
  const v = ak.webglTexParameter(this._node, target, pname);
  return pname === 0x912f ? v !== 0 : v;
};
function boundTarget(ctx, target, dims) {
  const face = target >= 0x8515 && target <= 0x851a;
  const ok = dims === 3 ? TARGETS_3D.has(target) : target === TEXTURE_2D || face;
  if (!ok) return fail(ctx, INVALID_ENUM);
  return ctx._textureBound(face ? TEXTURE_CUBE_MAP : target, new Set([face ? TEXTURE_CUBE_MAP : target]));
}
function imageSize(ctx, format, width, height, depth) {
  const block = ctx._blocks && ctx._blocks.get(format);
  if (!block) return -1;
  const [bw, bh, bytes] = block;
  return Math.ceil(width / bw) * Math.ceil(height / bh) * bytes * depth;
}
function source(ctx, args) {
  const unpack = ctx._bound.get(PIXEL_UNPACK_BUFFER);
  const [data, offset = 0, length = 0] = args;
  if (typeof data === 'number') {
    if (!unpack) return fail(ctx, INVALID_OPERATION);
    return [null, offset, data];
  }
  if (unpack) return fail(ctx, INVALID_OPERATION);
  if (!ArrayBuffer.isView(data)) throw new TypeError('parameter is not an ArrayBufferView');
  const unit = data.BYTES_PER_ELEMENT || 1;
  const count = data.byteLength / unit;
  if (offset > count || length > count - offset) return fail(ctx, INVALID_VALUE);
  const n = length || count - offset;
  return [new Uint8Array(data.buffer, data.byteOffset + offset * unit, n * unit), 0, n * unit];
}
function upload(ctx, op, dims, target, level, ints, format, size, args) {
  if (!boundTarget(ctx, target >>> 0, dims)) return;
  const src = source(ctx, args);
  if (!src) return;
  const [w, h, d] = size;
  const want = imageSize(ctx, format >>> 0, w, h, d);
  if (want < 0) return fail(ctx, INVALID_ENUM);
  if (src[0] && src[2] !== want) return fail(ctx, INVALID_VALUE);
  ak.webglCompressed(ctx._node, src[0], src[1], src[2], op, target, level, ...ints);
}
p.compressedTexImage2D = function (target, level, internalformat, width, height, border, ...args) {
  if (border !== 0) return fail(this, INVALID_VALUE);
  upload(this, 0, 2, target, level, [internalformat, width, height], internalformat, [width, height, 1], args);
};
p.compressedTexImage3D = function (target, level, internalformat, width, height, depth, border, ...args) {
  if (border !== 0) return fail(this, INVALID_VALUE);
  upload(this, 1, 3, target, level, [internalformat, width, height, depth], internalformat, [width, height, depth], args);
};
p.compressedTexSubImage2D = function (target, level, x, y, width, height, format, ...args) {
  upload(this, 2, 2, target, level, [x, y, width, height, format], format, [width, height, 1], args);
};
p.compressedTexSubImage3D = function (target, level, x, y, z, width, height, depth, format, ...args) {
  upload(this, 3, 3, target, level, [x, y, z, width, height, depth, format], format, [width, height, depth], args);
};
})();
