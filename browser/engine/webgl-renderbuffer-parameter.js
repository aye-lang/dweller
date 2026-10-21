(function () {
const ak = globalThis.__canvasHost;
const webgl = globalThis.__webgl;
const p2 = webgl.Context.prototype, p1 = webgl.Context1.prototype;
const INVALID_ENUM = 0x500, INVALID_OPERATION = 0x502, RENDERBUFFER = 0x8d41, SAMPLES = 0x8cab;
const PNAMES = new Set([0x8d42, 0x8d43, 0x8d44, 0x8d50, 0x8d51, 0x8d52, 0x8d53, 0x8d54, 0x8d55]);
const fail = (ctx, code) => (ak.webglRaise(ctx._node, code), null);
function renderbufferParameter(samples) {
  return function (target, pname) {
    pname >>>= 0;
    if ((target >>> 0) !== RENDERBUFFER || !(PNAMES.has(pname) || (samples && pname === SAMPLES))) return fail(this, INVALID_ENUM);
    if (!this._rb) return fail(this, INVALID_OPERATION);
    const r = ak.webglMore(this._node, 6, target, pname);
    return r && r[0];
  };
}
p2.getRenderbufferParameter = renderbufferParameter(true);
p1.getRenderbufferParameter = renderbufferParameter(false);
})();
