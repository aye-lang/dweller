(function () {
const ak = globalThis.__canvasHost;
const p = globalThis.__webgl.Context.prototype;
const {WebGLProgram} = globalThis;
const INVALID_OPERATION = 0x502, LINK_STATUS = 0x8b82;
p.getFragDataLocation = function (program, name) {
  if (!this._live(program, WebGLProgram)) return -1;
  const r = ak.webglMore(this._node, 9, program._name, String(name));
  return r ? r[0] | 0 : -1;
};
p.getActiveUniformBlockName = function (program, index) {
  if (!this._live(program, WebGLProgram)) return null;
  if (!ak.webglParam(this._node, 1, program._name, LINK_STATUS)) return (ak.webglRaise(this._node, INVALID_OPERATION), null);
  const r = ak.webglMore(this._node, 10, program._name, index);
  return r && String.fromCharCode(...new Uint8Array(new Uint32Array(r).buffer)).replace(/\0+$/, '');
};
})();
