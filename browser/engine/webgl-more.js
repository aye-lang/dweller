(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const {WebGLProgram, WebGLUniformLocation} = globalThis;
const INVALID_ENUM = 0x500, INVALID_VALUE = 0x501, INVALID_OPERATION = 0x502;
const READ_FRAMEBUFFER = 0x8ca8, DEPTH_STENCIL_ATTACHMENT = 0x821a, OBJECT_NAME = 0x8cd1, OBJECT_TYPE = 0x8cd0, FRAMEBUFFER_DEFAULT = 0x8218;
const DEFAULT_ATTACHMENTS = new Map([[0x405, 0x8ce0], [0x1801, 0x8d00], [0x1802, 0x8d20]]);
const ACTIVE_UNIFORMS = 0x8b86, BLOCK_ACTIVE_UNIFORMS = 0x8a42, BLOCK_INDICES = 0x8a43;
const BLOCK_INTS = new Set([0x8a3f, 0x8a40, BLOCK_ACTIVE_UNIFORMS]), BLOCK_BOOLS = new Set([0x8a44, 0x8a46]);
const UNIFORM_INTS = new Set([0x8a37, 0x8a38, 0x8a3a, 0x8a3b, 0x8a3c, 0x8a3d]), ROW_MAJOR = 0x8a3e;
const FLOAT = 0, INT = 1, UINT = 2, BOOL = 3;
const UNIFORM_TYPES = new Map([
  [0x1406, [FLOAT, 1]], [0x8b50, [FLOAT, 2]], [0x8b51, [FLOAT, 3]], [0x8b52, [FLOAT, 4]],
  [0x1404, [INT, 1]], [0x8b53, [INT, 2]], [0x8b54, [INT, 3]], [0x8b55, [INT, 4]],
  [0x1405, [UINT, 1]], [0x8dc6, [UINT, 2]], [0x8dc7, [UINT, 3]], [0x8dc8, [UINT, 4]],
  [0x8b56, [BOOL, 1]], [0x8b57, [BOOL, 2]], [0x8b58, [BOOL, 3]], [0x8b59, [BOOL, 4]],
  [0x8b5a, [FLOAT, 4]], [0x8b5b, [FLOAT, 9]], [0x8b5c, [FLOAT, 16]],
  [0x8b65, [FLOAT, 6]], [0x8b66, [FLOAT, 8]], [0x8b67, [FLOAT, 6]], [0x8b68, [FLOAT, 12]], [0x8b69, [FLOAT, 8]], [0x8b6a, [FLOAT, 12]],
]);
const ARRAYS = [Float32Array, Int32Array, Uint32Array];
const fail = (ctx, code) => (ak.webglRaise(ctx._node, code), null);
function uniformTypes(ctx, program) {
  if (program._types && program._types.link === program._link) return program._types.map;
  const map = new Map();
  const count = ak.webglParam(ctx._node, 1, program._name, ACTIVE_UNIFORMS);
  for (let i = 0; i < count; i++) {
    const [name, size, type] = ak.webglActive(ctx._node, 1, program._name, i);
    const base = name.replace(/\[0\]$/, '');
    for (let j = 0; j < size; j++) map.set(ak.webglLocation(ctx._node, 1, program._name, size > 1 ? base + '[' + j + ']' : name), type);
  }
  program._types = {link: program._link, map};
  return map;
}
p.getUniform = function (program, location) {
  if (!this._live(program, WebGLProgram)) return null;
  if (!(location instanceof WebGLUniformLocation)) throw new TypeError('parameter 2 is not a WebGLUniformLocation');
  if (location._program !== program || location._link !== program._link) return fail(this, INVALID_OPERATION);
  const [kind, count] = UNIFORM_TYPES.get(uniformTypes(this, program).get(location._location)) || [INT, 1];
  const words = ak.webglMore(this._node, 1, program._name, location._location, kind === BOOL ? INT : kind, count);
  if (!words) return null;
  const values = kind === BOOL ? words.map(w => w !== 0) : Array.from(new ARRAYS[kind](new Uint32Array(words).buffer));
  if (count === 1) return values[0];
  return kind === BOOL ? values : new ARRAYS[kind](values);
};
p.copyTexImage2D = function (target, level, internalformat, x, y, width, height, border) {
  if (border !== 0) return fail(this, INVALID_VALUE);
  ak.webglMore(this._node, 0, target, level, internalformat, x, y, width, height);
};
function attach(original, index) {
  return function (target, attachment, ...rest) {
    original.call(this, target, attachment, ...rest);
    const fb = (target >>> 0) === READ_FRAMEBUFFER ? this._readFb : this._drawFb;
    if (!fb) return;
    if (!fb._attached) fb._attached = new Map();
    const both = (attachment >>> 0) === DEPTH_STENCIL_ATTACHMENT;
    for (const a of both ? [0x8d00, 0x8d20, DEPTH_STENCIL_ATTACHMENT] : [attachment >>> 0]) fb._attached.set(a, rest[index] || null);
  };
}
p.framebufferRenderbuffer = attach(p.framebufferRenderbuffer, 1);
p.framebufferTexture2D = attach(p.framebufferTexture2D, 1);
p.framebufferTextureLayer = attach(p.framebufferTextureLayer, 0);
function defaultAttachmentParameter(ctx, target, attachment, pname) {
  const real = DEFAULT_ATTACHMENTS.get(attachment);
  if (!real || pname === OBJECT_NAME) return fail(ctx, INVALID_ENUM);
  const r = ak.webglMore(ctx._node, 2, target, real, pname);
  if (!r) return null;
  return pname === OBJECT_TYPE ? (r[0] ? FRAMEBUFFER_DEFAULT : 0) : r[0];
}
p.getFramebufferAttachmentParameter = function (target, attachment, pname) {
  target = target >>> 0;
  if (target !== READ_FRAMEBUFFER && target !== 0x8d40 && target !== 0x8ca9) return fail(this, INVALID_ENUM);
  const fb = target === READ_FRAMEBUFFER ? this._readFb : this._drawFb;
  if (!fb) return defaultAttachmentParameter(this, target, attachment >>> 0, pname >>> 0);
  const r = ak.webglMore(this._node, 2, target, attachment, pname);
  if (!r) return null;
  if ((pname >>> 0) !== OBJECT_NAME) return r[0];
  const o = fb && fb._attached && fb._attached.get(attachment >>> 0);
  return o && o._name === r[0] ? o : null;
};
p.getActiveUniformBlockParameter = function (program, index, pname) {
  if (!this._live(program, WebGLProgram)) return null;
  pname = pname >>> 0;
  if (pname === BLOCK_INDICES) {
    const n = ak.webglMore(this._node, 3, program._name, index, BLOCK_ACTIVE_UNIFORMS, 1);
    const r = n && ak.webglMore(this._node, 3, program._name, index, BLOCK_INDICES, n[0]);
    return r ? new Uint32Array(r) : null;
  }
  if (!BLOCK_INTS.has(pname) && !BLOCK_BOOLS.has(pname)) return fail(this, INVALID_ENUM);
  const r = ak.webglMore(this._node, 3, program._name, index, pname, 1);
  if (!r) return null;
  return BLOCK_BOOLS.has(pname) ? r[0] !== 0 : r[0];
};
p.getActiveUniforms = function (program, indices, pname) {
  if (!this._live(program, WebGLProgram)) return null;
  pname = pname >>> 0;
  if (!UNIFORM_INTS.has(pname) && pname !== ROW_MAJOR) return fail(this, INVALID_ENUM);
  const out = [];
  for (const index of indices) {
    const r = ak.webglMore(this._node, 4, program._name, index, pname);
    if (!r) return null;
    out.push(pname === ROW_MAJOR ? r[0] !== 0 : pname === 0x8a37 ? r[0] : r[0] | 0);
  }
  return out;
};
p.getUniformIndices = function (program, names) {
  if (!this._live(program, WebGLProgram)) return null;
  if (names === null || typeof names !== 'object') throw new TypeError('parameter 2 is not a sequence');
  const out = [];
  for (const name of names) {
    const r = ak.webglMore(this._node, 5, program._name, String(name));
    if (!r) return null;
    out.push(r[0]);
  }
  return out;
};
})();
