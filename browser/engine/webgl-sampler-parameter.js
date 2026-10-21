(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const {WebGLSampler} = globalThis;
const INVALID_ENUM = 0x500;
const INTS = new Set([0x2800, 0x2801, 0x2802, 0x2803, 0x8072, 0x884c, 0x884d]), FLOATS = new Set([0x813a, 0x813b]);
p.getSamplerParameter = function (sampler, pname) {
  if (!(sampler instanceof WebGLSampler)) throw new TypeError('parameter 1 is not a WebGLSampler');
  if (!this._use(sampler)) return null;
  pname >>>= 0;
  if (!INTS.has(pname) && !FLOATS.has(pname)) return (ak.webglRaise(this._node, INVALID_ENUM), null);
  const r = ak.webglMore(this._node, INTS.has(pname) ? 7 : 8, sampler._name, pname);
  if (!r) return null;
  return INTS.has(pname) ? r[0] : new Float32Array(new Uint32Array(r).buffer)[0];
};
})();
