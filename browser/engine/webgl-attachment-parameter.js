(function () {
const ak = globalThis.__canvasHost;
const webgl = globalThis.__webgl;
const p2 = webgl.Context.prototype, p1 = webgl.Context1.prototype;
const INVALID_ENUM = 0x500, INVALID_OPERATION = 0x502;
const FRAMEBUFFER = 0x8d40, READ_FRAMEBUFFER = 0x8ca8, DRAW_FRAMEBUFFER = 0x8ca9;
const COLOR_ATTACHMENT0 = 0x8ce0, DEPTH_ATTACHMENT = 0x8d00, STENCIL_ATTACHMENT = 0x8d20, DEPTH_STENCIL_ATTACHMENT = 0x821a;
const STENCIL_SIZE = 0x8217, OBJECT_TYPE = 0x8cd0, OBJECT_NAME = 0x8cd1, COMPONENT_TYPE = 0x8211, COLOR_ENCODING = 0x8210, FRAMEBUFFER_DEFAULT = 0x8218;
const DEFAULT_ATTACHMENTS = new Map([[0x405, COLOR_ATTACHMENT0], [0x1801, DEPTH_ATTACHMENT], [0x1802, STENCIL_ATTACHMENT]]);
const WEBGL1_PNAMES = new Set([OBJECT_TYPE, OBJECT_NAME, 0x8cd2, 0x8cd3]);
const WEBGL1_ATTACHMENTS = new Set([COLOR_ATTACHMENT0, DEPTH_ATTACHMENT, STENCIL_ATTACHMENT, DEPTH_STENCIL_ATTACHMENT]);
const fail = (ctx, code) => (ak.webglRaise(ctx._node, code), null);
const enabled = (ctx, name) => !!ctx._exts.get(name);
const attached = (fb, attachment) => (fb._attached && fb._attached.get(attachment)) || null;
function defaultParameter(ctx, target, attachment, pname) {
  const real = DEFAULT_ATTACHMENTS.get(attachment);
  if (!real || pname === OBJECT_NAME) return fail(ctx, INVALID_ENUM);
  const r = ak.webglMore(ctx._node, 2, target, real, pname);
  if (!r) return null;
  return pname === OBJECT_TYPE ? (r[0] ? FRAMEBUFFER_DEFAULT : 0) : r[0];
}
p2.getFramebufferAttachmentParameter = function (target, attachment, pname) {
  target >>>= 0, attachment >>>= 0, pname >>>= 0;
  if (target !== READ_FRAMEBUFFER && target !== FRAMEBUFFER && target !== DRAW_FRAMEBUFFER) return fail(this, INVALID_ENUM);
  const fb = target === READ_FRAMEBUFFER ? this._readFb : this._drawFb;
  if (!fb) return defaultParameter(this, target, attachment, pname);
  const both = attachment === DEPTH_STENCIL_ATTACHMENT;
  const color = attachment >= COLOR_ATTACHMENT0 && attachment < COLOR_ATTACHMENT0 + 16;
  if (!color && attachment !== DEPTH_ATTACHMENT && attachment !== STENCIL_ATTACHMENT && !both) return fail(this, INVALID_ENUM);
  const o = attached(fb, both ? DEPTH_ATTACHMENT : attachment);
  if (both && (o !== attached(fb, STENCIL_ATTACHMENT) || pname === COMPONENT_TYPE)) return fail(this, INVALID_OPERATION);
  const real = both ? (pname === STENCIL_SIZE ? STENCIL_ATTACHMENT : DEPTH_ATTACHMENT) : attachment;
  const type = ak.webglMore(this._node, 2, target, real, OBJECT_TYPE);
  if (!type || pname === OBJECT_TYPE) return type && type[0];
  if (!type[0] && pname !== OBJECT_NAME) return fail(this, INVALID_OPERATION);
  const r = ak.webglMore(this._node, 2, target, real, pname);
  if (!r) return null;
  if (pname !== OBJECT_NAME) return r[0];
  return o && o._name === r[0] ? o : null;
};
p1.getFramebufferAttachmentParameter = function (target, attachment, pname) {
  attachment >>>= 0, pname >>>= 0;
  const color = attachment > COLOR_ATTACHMENT0 && attachment < COLOR_ATTACHMENT0 + 16 && enabled(this, 'WEBGL_draw_buffers');
  if ((target >>> 0) !== FRAMEBUFFER || !(WEBGL1_ATTACHMENTS.has(attachment) || color)) return fail(this, INVALID_ENUM);
  const half = enabled(this, 'EXT_color_buffer_half_float') || enabled(this, 'WEBGL_color_buffer_float');
  const extra = (pname === COMPONENT_TYPE && half) || (pname === COLOR_ENCODING && enabled(this, 'EXT_sRGB'));
  if (!WEBGL1_PNAMES.has(pname) && !extra) return fail(this, INVALID_ENUM);
  if (!this._drawFb) return fail(this, INVALID_OPERATION);
  return p2.getFramebufferAttachmentParameter.call(this, target, attachment, pname);
};
})();
