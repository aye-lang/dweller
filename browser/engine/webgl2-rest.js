(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const later = globalThis.setTimeout;
const {WebGLQuery, WebGLSync, WebGLTransformFeedback, WebGLBuffer, WebGLProgram, WebGLActiveInfo} = globalThis;
const INVALID_ENUM = 0x500, INVALID_VALUE = 0x501, INVALID_OPERATION = 0x502;
const ANY_SAMPLES_PASSED = 0x8c2f, PRIMITIVES_WRITTEN = 0x8c88, PIXEL_PACK_BUFFER = 0x88eb, ELEMENT_ARRAY_BUFFER = 0x8893;
const QUERY_TARGETS = new Set([ANY_SAMPLES_PASSED, 0x8d6a, PRIMITIVES_WRITTEN]);
const BUFFER_TARGETS = new Set([0x8892, 0x8893, 0x8f36, 0x8f37, 0x88eb, 0x88ec, 0x8c8e, 0x8a11]);
const INDEXED = new Map([[0x8c8f, 0x8c8e], [0x8a28, 0x8a11]]);
const INDEXED_RANGES = new Set([0x8c84, 0x8c85, 0x8a29, 0x8a2a]);
const INTEGER_FORMATS = new Set([0x8232, 0x8231, 0x8234, 0x8233, 0x8236, 0x8235, 0x8238, 0x8237, 0x823a, 0x8239, 0x823c, 0x823b, 0x8d7c, 0x8d8e, 0x906f, 0x8d76, 0x8d88, 0x8d70, 0x8d82]);
const RENDERABLE_FORMATS = new Set([0x8229, 0x822b, 0x8051, 0x8d62, 0x8058, 0x8c43, 0x8057, 0x8056, 0x8059, 0x81a5, 0x81a6, 0x8cac, 0x88f0, 0x8cad, 0x8d48]);
const HALF_FLOAT_FORMATS = new Set([0x822d, 0x822f, 0x881a]);
const FLOAT_FORMATS = new Set([0x822e, 0x8230, 0x8814, 0x8c3a]);
const ATTRIB_BOOLEANS = new Set([0x8622, 0x886a, 0x88fd]);
const ATTRIB_INTEGERS = new Set([0x8623, 0x8624, 0x8625, 0x88fe]);
const DEFAULT_ATTACHMENTS = new Map([[0x1800, 0x8ce0], [0x1801, 0x8d00], [0x1802, 0x8d20]]);
const rest = (ctx, ...args) => ak.webglRest(ctx._node, ...args);
const raise = (ctx, code) => ak.webglRaise(ctx._node, code);
const fail = (ctx, code, value = null) => (raise(ctx, code), value);
function check(ctx, o, cls) {
  if (!(o instanceof cls)) throw new TypeError('parameter is not a ' + cls.name);
  return ctx._use(o);
}
function drop(ctx, o, cls, op) {
  if (o === null || o === undefined) return false;
  if (!(o instanceof cls)) throw new TypeError('parameter is not a ' + cls.name);
  if (o._ctx !== ctx) return fail(ctx, INVALID_OPERATION, false);
  if (o._deleted) return false;
  o._deleted = true;
  rest(ctx, op, o._name);
  return true;
}
function allowLater(o) {
  if (o._pending) return;
  o._pending = true;
  later(() => {
    o._pending = false;
    o._allowed = true;
  }, 0);
}
function active(ctx) {
  return ctx._queries || (ctx._queries = new Map());
}
const slot = target => (target === PRIMITIVES_WRITTEN ? PRIMITIVES_WRITTEN : ANY_SAMPLES_PASSED);
function updateQuery(ctx, q) {
  if (q._available || !q._allowed) return;
  q._allowed = false;
  q._available = !!rest(ctx, 4, q._name);
  if (q._available) q._result = rest(ctx, 5, q._name);
  else allowLater(q);
}
function updateSync(ctx, s) {
  if (s._signaled || !s._allowed) return;
  s._allowed = false;
  s._signaled = rest(ctx, 23, s._name);
  if (!s._signaled) allowLater(s);
}
function named(ctx, buffer) {
  if (buffer instanceof WebGLBuffer && buffer._ctx === ctx) (ctx._names || (ctx._names = new Map())).set(buffer._name, buffer);
}
const byName = (ctx, name) => (ctx._names && ctx._names.get(name)) || null;

p.createQuery = function () {
  const n = rest(this, 0);
  return n ? new WebGLQuery(this, n) : null;
};
p.isQuery = function (q) {
  return q instanceof WebGLQuery && q._ctx === this && !q._deleted && !!q._target;
};
p.deleteQuery = function (q) {
  if (q instanceof WebGLQuery && q._ctx === this && !q._deleted && q._target && active(this).get(slot(q._target)) === q) {
    active(this).delete(slot(q._target));
    rest(this, 3, q._target);
  }
  drop(this, q, WebGLQuery, 1);
};
p.beginQuery = function (target, q) {
  target = target >>> 0;
  if (!check(this, q, WebGLQuery)) return;
  if (q._target && q._target !== target) return fail(this, INVALID_OPERATION);
  if (!QUERY_TARGETS.has(target)) return fail(this, INVALID_ENUM);
  if (active(this).get(slot(target))) return fail(this, INVALID_OPERATION);
  active(this).set(slot(target), q);
  q._target = target;
  rest(this, 2, target, q._name);
};
p.endQuery = function (target) {
  target = target >>> 0;
  if (!QUERY_TARGETS.has(target)) return fail(this, INVALID_ENUM);
  const q = active(this).get(slot(target));
  if (!q || q._target !== target) return fail(this, INVALID_OPERATION);
  active(this).delete(slot(target));
  q._available = false;
  q._result = 0;
  q._allowed = false;
  allowLater(q);
  rest(this, 3, target);
};
p.getQuery = function (target, pname) {
  target = target >>> 0;
  if ((pname >>> 0) !== 0x8865 || !QUERY_TARGETS.has(target)) return fail(this, INVALID_ENUM);
  const q = active(this).get(slot(target));
  return q && q._target === target ? q : null;
};
p.getQueryParameter = function (q, pname) {
  if (!check(this, q, WebGLQuery)) return null;
  if (!q._target || active(this).get(slot(q._target)) === q) return fail(this, INVALID_OPERATION);
  pname = pname >>> 0;
  if (pname !== 0x8866 && pname !== 0x8867) return fail(this, INVALID_ENUM);
  updateQuery(this, q);
  return pname === 0x8866 ? q._result : q._available;
};

p.fenceSync = function (condition, flags) {
  if ((condition >>> 0) !== 0x9117) return fail(this, INVALID_ENUM);
  if ((flags >>> 0) !== 0) return fail(this, INVALID_VALUE);
  const id = rest(this, 22);
  if (!id) return null;
  const s = new WebGLSync(this, id);
  s._signaled = false;
  allowLater(s);
  return s;
};
p.isSync = function (s) {
  return s instanceof WebGLSync && s._ctx === this && !s._deleted;
};
p.deleteSync = function (s) {
  drop(this, s, WebGLSync, 24);
};
p.clientWaitSync = function (s, flags, timeout) {
  if (!check(this, s, WebGLSync)) return 0x911d;
  if (Number(timeout) > 0) return fail(this, INVALID_OPERATION, 0x911d);
  flags = flags >>> 0;
  if (flags !== 0 && flags !== 1) return fail(this, INVALID_VALUE, 0x911d);
  if (s._signaled) return 0x911a;
  updateSync(this, s);
  return s._signaled ? 0x911c : 0x911b;
};
p.waitSync = function (s, flags, timeout) {
  if (!check(this, s, WebGLSync)) return;
  if ((flags >>> 0) !== 0 || Number(timeout) !== -1) raise(this, INVALID_VALUE);
};
p.getSyncParameter = function (s, pname) {
  if (!check(this, s, WebGLSync)) return null;
  pname = pname >>> 0;
  if (pname === 0x9112) return 0x9116;
  if (pname === 0x9113) return 0x9117;
  if (pname === 0x9115) return 0;
  if (pname !== 0x9114) return fail(this, INVALID_ENUM);
  updateSync(this, s);
  return s._signaled ? 0x9119 : 0x9118;
};

const feedback = ctx => ctx._tf || ctx._defaultTf || (ctx._defaultTf = {});
function feedbackState(ctx) {
  const t = feedback(ctx);
  t._active = ak.webglGet(ctx._node, 0x8e24, 2, 1)[0] !== 0;
  t._paused = ak.webglGet(ctx._node, 0x8e23, 2, 1)[0] !== 0;
}
p.createTransformFeedback = function () {
  const n = rest(this, 6);
  return n ? new WebGLTransformFeedback(this, n) : null;
};
p.isTransformFeedback = function (t) {
  return t instanceof WebGLTransformFeedback && t._ctx === this && !t._deleted && !!t._used;
};
p.deleteTransformFeedback = function (t) {
  if (t instanceof WebGLTransformFeedback && t._ctx === this && t._active) return fail(this, INVALID_OPERATION);
  if (drop(this, t, WebGLTransformFeedback, 7) && this._tf === t) this._tf = null;
};
p.bindTransformFeedback = function (target, t) {
  if ((target >>> 0) !== 0x8e22) return fail(this, INVALID_ENUM);
  if (t !== null && t !== undefined) {
    if (!check(this, t, WebGLTransformFeedback)) return;
  } else {
    t = null;
  }
  const now = feedback(this);
  if (now._active && !now._paused) return fail(this, INVALID_OPERATION);
  this._tf = t;
  if (t) t._used = true;
  rest(this, 8, t ? t._name : 0);
};
p.beginTransformFeedback = function (mode) {
  rest(this, 9, mode);
  feedbackState(this);
};
p.endTransformFeedback = function () {
  rest(this, 10);
  feedbackState(this);
};
p.pauseTransformFeedback = function () {
  rest(this, 11);
  feedbackState(this);
};
p.resumeTransformFeedback = function () {
  rest(this, 12);
  feedbackState(this);
};
p.transformFeedbackVaryings = function (program, varyings, bufferMode) {
  if (!this._live(program, WebGLProgram)) return;
  if (varyings === null || typeof varyings !== 'object') throw new TypeError('parameter 2 is not a sequence');
  const names = Array.from(varyings, String);
  const bytes = new Uint8Array(names.reduce((n, s) => n + s.length + 1, 0));
  let i = 0;
  for (const s of names) {
    for (let k = 0; k < s.length; k++) bytes[i++] = s.charCodeAt(k) & 0xff;
    i++;
  }
  ak.webglRestView(this._node, 4, bytes, program._name, bufferMode);
};
p.getTransformFeedbackVarying = function (program, index) {
  if (!this._live(program, WebGLProgram)) return null;
  const v = ak.webglFeedbackVarying(this._node, program._name, index);
  return v ? new WebGLActiveInfo(v[0], v[1], v[2]) : null;
};

const getParameter = p.getParameter;
p.getParameter = function (pname) {
  if ((pname >>> 0) === 0x8e25) return this._tf || null;
  return getParameter.call(this, pname);
};
const bindBuffer = p.bindBuffer;
p.bindBuffer = function (target, buffer) {
  named(this, buffer);
  bindBuffer.call(this, target, buffer);
};
const bindBufferRange = p.bindBufferRange;
p.bindBufferRange = function (target, index, buffer, offset, size, whole) {
  named(this, buffer);
  bindBufferRange.call(this, target, index, buffer, offset, size, whole);
};
p.getIndexedParameter = function (target, index) {
  target = target >>> 0;
  if (INDEXED.has(target)) return byName(this, rest(this, 18, target, index));
  if (INDEXED_RANGES.has(target)) return rest(this, 18, target, index);
  return fail(this, INVALID_ENUM);
};
p.getBufferParameter = function (target, pname) {
  target = target >>> 0;
  pname = pname >>> 0;
  if (!BUFFER_TARGETS.has(target) || (pname !== 0x8764 && pname !== 0x8765)) return fail(this, INVALID_ENUM);
  if (!this._bound.get(target)) return fail(this, INVALID_OPERATION);
  return rest(this, 19, target, pname);
};
p.getVertexAttrib = function (index, pname) {
  index = index >>> 0;
  pname = pname >>> 0;
  if (index >= this.getParameter(0x8869)) return fail(this, INVALID_VALUE);
  if (pname === 0x889f) return byName(this, rest(this, 20, index, pname));
  if (ATTRIB_BOOLEANS.has(pname)) return rest(this, 20, index, pname) !== 0;
  if (ATTRIB_INTEGERS.has(pname)) return rest(this, 20, index, pname);
  if (pname === 0x8626) {
    const v = new Float32Array(4);
    ak.webglRestView(this._node, 5, v, index);
    return v;
  }
  return fail(this, INVALID_ENUM);
};
p.getVertexAttribOffset = function (index, pname) {
  if ((pname >>> 0) !== 0x8645) return fail(this, INVALID_ENUM, 0);
  return rest(this, 21, index);
};
p.getInternalformatParameter = function (target, internalformat, pname) {
  internalformat = internalformat >>> 0;
  if ((target >>> 0) !== 0x8d41) return fail(this, INVALID_ENUM);
  if (INTEGER_FORMATS.has(internalformat)) return new Int32Array(0);
  const half = HALF_FLOAT_FORMATS.has(internalformat) && (this._exts.get('EXT_color_buffer_float') || this._exts.get('EXT_color_buffer_half_float'));
  const full = FLOAT_FORMATS.has(internalformat) && this._exts.get('EXT_color_buffer_float');
  if (!RENDERABLE_FORMATS.has(internalformat) && !half && !full) return fail(this, INVALID_ENUM);
  if ((pname >>> 0) !== 0x80a9) return fail(this, INVALID_ENUM);
  const samples = new Int32Array(rest(this, 25, internalformat));
  ak.webglRestView(this._node, 3, samples, internalformat);
  return samples;
};

p.copyBufferSubData = function (readTarget, writeTarget, readOffset, writeOffset, size) {
  if (readOffset < 0 || writeOffset < 0 || size < 0) return fail(this, INVALID_VALUE);
  readTarget = readTarget >>> 0;
  writeTarget = writeTarget >>> 0;
  if (!BUFFER_TARGETS.has(readTarget) || !BUFFER_TARGETS.has(writeTarget)) return fail(this, INVALID_ENUM);
  const from = this._bound.get(readTarget), to = this._bound.get(writeTarget);
  if (!from || !to || (from._target === ELEMENT_ARRAY_BUFFER) !== (to._target === ELEMENT_ARRAY_BUFFER)) return fail(this, INVALID_OPERATION);
  rest(this, 13, readTarget, writeTarget, readOffset, writeOffset, size);
};
p.getBufferSubData = function (target, srcByteOffset, dst, dstOffset = 0, length = 0) {
  if (srcByteOffset < 0) return fail(this, INVALID_VALUE);
  target = target >>> 0;
  if (!BUFFER_TARGETS.has(target)) return fail(this, INVALID_ENUM);
  if (!ArrayBuffer.isView(dst)) throw new TypeError('parameter 3 is not an ArrayBufferView');
  if (!this._bound.get(target) || (target === 0x8c8e && feedback(this)._active)) return fail(this, INVALID_OPERATION);
  const count = dst.length === undefined ? dst.byteLength : dst.length;
  dstOffset = dstOffset >>> 0;
  length = length >>> 0;
  if (dstOffset > count) return fail(this, INVALID_VALUE);
  if (length === 0) length = count - dstOffset;
  if (dstOffset + length > count) return fail(this, INVALID_VALUE);
  const view = dst instanceof DataView ? new Uint8Array(dst.buffer, dst.byteOffset + dstOffset, length) : dst.subarray(dstOffset, dstOffset + length);
  ak.webglRestView(this._node, 0, view, target, srcByteOffset);
};
const readPixels = p.readPixels;
p.readPixels = function (x, y, width, height, format, type, dst, offset) {
  if (!this._bound.get(PIXEL_PACK_BUFFER)) return readPixels.call(this, x, y, width, height, format, type, dst, offset);
  if (typeof dst !== 'number') return fail(this, INVALID_OPERATION);
  if (dst < 0) return fail(this, INVALID_VALUE);
  rest(this, 17, x, y, width, height, format, type, dst);
};
p.copyTexSubImage2D = function (target, level, xoffset, yoffset, x, y, width, height) {
  rest(this, 14, target, level, xoffset, yoffset, x, y, width, height);
};
p.copyTexSubImage3D = function (target, level, xoffset, yoffset, zoffset, x, y, width, height) {
  rest(this, 15, target, level, xoffset, yoffset, zoffset, x, y, width, height);
};
p.clearBufferfi = function (buffer, drawbuffer, depth, stencil) {
  rest(this, 16, buffer, drawbuffer, depth, stencil);
};
function invalidate(ctx, target, attachments, op, ...rect) {
  target = target >>> 0;
  if (attachments === null || typeof attachments !== 'object') throw new TypeError('parameter 2 is not a sequence');
  const list = Uint32Array.from(attachments, a => a >>> 0);
  const fb = target === 0x8ca8 ? ctx._readFb : ctx._drawFb;
  if (!fb && (target === 0x8d40 || target === 0x8ca8 || target === 0x8ca9)) {
    for (let i = 0; i < list.length; i++) {
      const a = DEFAULT_ATTACHMENTS.get(list[i]);
      if (a === undefined) return fail(ctx, INVALID_ENUM);
      list[i] = a;
    }
  }
  ak.webglRestView(ctx._node, op, list, target, ...rect);
}
p.invalidateFramebuffer = function (target, attachments) {
  invalidate(this, target, attachments, 1);
};
p.invalidateSubFramebuffer = function (target, attachments, x, y, width, height) {
  invalidate(this, target, attachments, 2, x, y, width, height);
};
})();
