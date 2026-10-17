(function () {
const ak = globalThis.__canvasHost;
const webgl = globalThis.__webgl;
const Context2 = webgl.Context;
const INVALID_ENUM = 0x500, INVALID_OPERATION = 0x502;
const ES3_ERROR = '#error WebGL 1 has no GLSL ES 3.00\n';
const STRINGS = new Map([[0x1f02, 'WebGL 1.0'], [0x8b8c, 'WebGL GLSL ES 1.0']]);
const FORMATS = new Set([0x1906, 0x1907, 0x1908, 0x1909, 0x190a]);
const TYPES = new Set([0x1401, 0x8033, 0x8034, 0x8363]);
const EXTENSIONS = new Set(['EXT_texture_filter_anisotropic', 'OES_texture_float_linear', 'WEBGL_lose_context', 'WEBGL_debug_renderer_info', 'KHR_parallel_shader_compile', 'WEBGL_compressed_texture_s3tc', 'WEBGL_compressed_texture_s3tc_srgb', 'WEBGL_compressed_texture_etc', 'WEBGL_compressed_texture_astc', 'EXT_texture_compression_bptc', 'EXT_texture_compression_rgtc']);
const METHODS = 'getContextAttributes isContextLost getSupportedExtensions getExtension activeTexture attachShader bindAttribLocation bindBuffer bindFramebuffer bindRenderbuffer bindTexture blendColor blendEquation blendEquationSeparate blendFunc blendFuncSeparate bufferData bufferSubData checkFramebufferStatus clear clearColor clearDepth clearStencil colorMask compileShader compressedTexImage2D compressedTexSubImage2D copyTexImage2D copyTexSubImage2D createBuffer createFramebuffer createProgram createRenderbuffer createShader createTexture cullFace deleteBuffer deleteFramebuffer deleteProgram deleteRenderbuffer deleteShader deleteTexture depthFunc depthMask depthRange detachShader disable disableVertexAttribArray drawArrays drawElements enable enableVertexAttribArray finish flush framebufferRenderbuffer framebufferTexture2D frontFace generateMipmap getActiveAttrib getActiveUniform getAttachedShaders getAttribLocation getBufferParameter getParameter getError getFramebufferAttachmentParameter getProgramParameter getProgramInfoLog getRenderbufferParameter getShaderParameter getShaderPrecisionFormat getShaderInfoLog getShaderSource getTexParameter getUniform getUniformLocation getVertexAttrib getVertexAttribOffset hint isBuffer isEnabled isFramebuffer isProgram isRenderbuffer isShader isTexture lineWidth linkProgram pixelStorei polygonOffset readPixels renderbufferStorage sampleCoverage scissor shaderSource stencilFunc stencilFuncSeparate stencilMask stencilMaskSeparate stencilOp stencilOpSeparate texImage2D texParameterf texParameteri texSubImage2D uniform1f uniform2f uniform3f uniform4f uniform1i uniform2i uniform3i uniform4i uniform1fv uniform2fv uniform3fv uniform4fv uniform1iv uniform2iv uniform3iv uniform4iv uniformMatrix2fv uniformMatrix3fv uniformMatrix4fv useProgram validateProgram vertexAttrib1f vertexAttrib2f vertexAttrib3f vertexAttrib4f vertexAttrib1fv vertexAttrib2fv vertexAttrib3fv vertexAttrib4fv vertexAttribPointer viewport'.split(' ');
const EXTRA_CONSTANTS = ['DEPTH_STENCIL', 'DEPTH_STENCIL_ATTACHMENT', 'UNPACK_FLIP_Y_WEBGL', 'UNPACK_PREMULTIPLY_ALPHA_WEBGL', 'CONTEXT_LOST_WEBGL', 'UNPACK_COLORSPACE_CONVERSION_WEBGL', 'BROWSER_DEFAULT_WEBGL'];
class WebGLRenderingContext {
  constructor() {
    throw new TypeError('Illegal constructor');
  }
}
const p = WebGLRenderingContext.prototype;
const p2 = Context2.prototype;
for (const k of ['canvas', 'drawingBufferWidth', 'drawingBufferHeight', 'drawingBufferColorSpace']) Object.defineProperty(p, k, Object.getOwnPropertyDescriptor(p2, k));
for (const k of Object.getOwnPropertyNames(p2)) if (k.startsWith('_')) p[k] = p2[k];
for (const k of METHODS) if (p2[k]) p[k] = p2[k];
// The constants are installed at first use, as WebGL 2's are: every page
// runs this prelude.
let installed = false;
function install() {
  if (installed) return;
  installed = true;
  webgl.install();
  const all = Object.keys(Context2);
  for (const k of all.slice(0, all.indexOf('INVALID_FRAMEBUFFER_OPERATION') + 1).concat(EXTRA_CONSTANTS)) {
    Object.defineProperty(WebGLRenderingContext, k, {value: Context2[k], enumerable: true});
    Object.defineProperty(p, k, {value: Context2[k], enumerable: true});
  }
}
p.getParameter = function (pname) {
  return STRINGS.has(pname >>> 0) ? STRINGS.get(pname >>> 0) : p2.getParameter.call(this, pname);
};
p.getSupportedExtensions = function () {
  const all = p2.getSupportedExtensions.call(this);
  return all && all.filter(n => EXTENSIONS.has(n));
};
p.getExtension = function (name) {
  return EXTENSIONS.has(String(name)) ? p2.getExtension.call(this, name) : null;
};
p.shaderSource = function (shader, source) {
  const es3 = /^\s*#\s*version\s+300/.test(String(source));
  p2.shaderSource.call(this, shader, es3 ? ES3_ERROR : source);
  if (es3 && shader._source === ES3_ERROR) shader._source = String(source);
};
p.texImage2D = function (...args) {
  if (args.length >= 6) {
    const internalformat = args[2] >>> 0, format = args[args.length >= 9 ? 6 : 3] >>> 0, type = args[args.length >= 9 ? 7 : 4] >>> 0;
    if (!FORMATS.has(format) || !TYPES.has(type)) return ak.webglRaise(this._node, INVALID_ENUM);
    if (internalformat !== format) return ak.webglRaise(this._node, INVALID_OPERATION);
  }
  p2.texImage2D.apply(this, args);
};
p.texSubImage2D = function (...args) {
  if (args.length >= 7) {
    const format = args[args.length >= 9 ? 6 : 4] >>> 0, type = args[args.length >= 9 ? 7 : 5] >>> 0;
    if (!FORMATS.has(format) || !TYPES.has(type)) return ak.webglRaise(this._node, INVALID_ENUM);
  }
  p2.texSubImage2D.apply(this, args);
};
Object.defineProperty(globalThis, 'WebGLRenderingContext', {
  configurable: true,
  get() {
    install();
    Object.defineProperty(globalThis, 'WebGLRenderingContext', {value: WebGLRenderingContext, writable: true, configurable: true});
    return WebGLRenderingContext;
  },
  set(v) {
    Object.defineProperty(globalThis, 'WebGLRenderingContext', {value: v, writable: true, configurable: true});
  },
});
webgl.create1 = (canvas, node, options) => {
  const ctx = webgl.create(canvas, node, options);
  install();
  return ctx && Object.setPrototypeOf(ctx, p);
};
webgl.Context1 = WebGLRenderingContext;
webgl.is = ctx => ctx instanceof Context2 || ctx instanceof WebGLRenderingContext;
})();
