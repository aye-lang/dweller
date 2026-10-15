# WebGL in Dweller (design)

Status: design, 2026-10-15. Cards: P11 in `docs/plans/browser.md`.

**Decision (user, 2026-10-15).** The user builds 3D web games, so
Dweller needs WebGL. The acceptance target is the user's game
(`~/Documents/f1/web`: Vite + TypeScript, `three` 0.170.0). three.js
supports only WebGL 2 since r163, so **WebGL 2 comes first**; WebGL 1
comes later, for other sites.

Today Dweller draws pages with a CPU rasterizer into an SDL window:

- `browser/ui/window.aye` creates an `SDL_Renderer`.
- `browser/ui/pagedraw.aye` keeps the page's pixels between frames.
- `browser/paint/damage.aye` and `rcache.aye` provide the raster cache
  and layers (`browser/engine/PERF.md`, P4.35 and P4.56).
- Canvas 2D (`browser/engine/canvas_api.aye`, `canvas_prelude.aye`)
  draws into an `image.Bitmap` (straight RGBA). The display list shows
  that bitmap as an `Item.Image`, and `Bitmap.version` feeds the damage
  tracking.

Dweller has no GPU code. The strategy note "no GPU" in the plan changes
here: there is a GPU for WebGL, and later for compositing (step 2).
Everything else in the threading model stays the same: GL calls are
made on the window thread, the thread that runs the page's scripts.

## Reference sources

Cards name a reference path in the local checkout
(`/Users/solemnis/Documents/chromium/src`, read-only). Agents read the
known implementation first instead of exploring from scratch. Licenses:

- **V8 and ANGLE are BSD.** Porting code is allowed, with attribution:
  a comment that names the source file, plus the license in
  `browser/licenses/`.
- **Blink has some LGPL files.** Use Blink as an algorithm reference
  only and never copy its code.
- **Khronos tests and three.js are MIT-style.** Copies are allowed, with
  their license file.

Main references:

| Topic | Path |
|---|---|
| ANGLE build, backends | `third_party/angle/doc/DevSetup.md`, `third_party/angle/gni/angle.gni`, `third_party/angle/src/libANGLE/renderer/{metal,vulkan}/` |
| EGL context attributes for WebGL | `ui/gl/gl_context_egl.cc` (the attribute list near `EGL_CONTEXT_WEBGL_COMPATIBILITY_ANGLE`) |
| GL entry points loaded at run time | `ui/gl/generate_bindings.py`, `ui/gl/gl_bindings_autogen_gl.cc` |
| WebGL front end (validation the spec adds on top of GL) | `third_party/blink/renderer/modules/webgl/webgl_rendering_context_base.cc`, `webgl2_rendering_context_base.cc` |
| Drawing buffer (FBO, MSAA resolve, preserveDrawingBuffer) | `third_party/blink/renderer/platform/graphics/gpu/drawing_buffer.cc` |
| Shader validation | `third_party/angle/src/libANGLE/Compiler.cpp` (`SelectShaderSpec`), `third_party/angle/src/compiler/translator/` |
| GPU compositing (idea only) | `docs/how_cc_works.md` |
| Conformance | `third_party/webgl/src/sdk/tests/` |
| WebAssembly | `v8/src/wasm/`, `v8/src/objects/js-array-buffer.h` |
| Web Audio | `third_party/blink/renderer/modules/webaudio/` |
| Workers | `third_party/blink/renderer/core/workers/` |
| Pointer Lock, Fullscreen | `third_party/blink/renderer/core/page/pointer_lock_controller.cc`, `core/fullscreen/` |

## 1. GPU backend: ANGLE

**Why ANGLE.** ANGLE gives OpenGL ES 3.0 and EGL 1.5 on top of Metal
(macOS), Vulkan or desktop GL (Linux). It is the WebGL backend of
Chromium and Firefox, and the WebGL implementations of Safari and Firefox
use its shader translator. Its WebGL compatibility mode does the
WebGL-specific validation and the shader translation. Dweller then does
not need its own GLSL validator. ANGLE's API is plain C (EGL plus GLES),
which fits Aye's `@cimport`. ANGLE is BSD-3-Clause
(`third_party/angle/LICENSE`).

**Rejected alternatives:**

- System GL. Apple deprecated it, and it has no WebGL validation.
- System Mesa GLES on Linux. It has no WebGL validation, and the two
  OSes would behave differently.
- Our own GLSL validator. It is too large, and a weak validator is a
  security hole.

### Where the binaries come from

The vendored copy in the reference checkout (`third_party/angle`) cannot
be built in place. Its `.gn` and `DEPS` apply only to standalone checkouts
with their own `//build` and dependencies (`gclient sync`). Dweller
therefore builds ANGLE standalone, at a pinned revision:

- `browser/gpu/angle.rev` holds one upstream ANGLE commit. To start,
  use the commit that the reference checkout's `DEPS` pins
  (`angle_revision`), so that the code agents read is the code Dweller
  runs.
- `browser/gpu/build_angle.sh` runs `fetch angle` (depot_tools) into
  `~/.cache/dweller/angle-src`, checks out the pinned revision, runs
  `gclient sync`, then `gn gen` and `autoninja` with these args:

```
is_debug = false
is_component_build = false        # self-contained libEGL + libGLESv2
angle_build_all = false
angle_enable_null = true          # API-only tests
angle_enable_swiftshader = true   # headless pixels (Vulkan on SwiftShader)
angle_enable_vulkan = true
angle_enable_metal = true         # macOS (the default there)
angle_enable_gl = true            # Linux fallback; false on macOS
angle_has_frame_capture = false
```

  The script builds the `libEGL` and `libGLESv2` targets and the
  SwiftShader Vulkan ICD. It copies them to
  `target/angle/<rev>/lib/`: `libEGL.dylib`, `libGLESv2.dylib`,
  `libvk_swiftshader.dylib` and its ICD JSON (`.so` on Linux). It copies
  the headers to `target/angle/<rev>/include/` (`EGL/`, `GLES2/`,
  `GLES3/`, `KHR/`).
- **CI does not build ANGLE on every run.** A separate workflow
  (`.github/workflows/angle.yml`, run by hand or when `angle.rev`
  changes) builds ANGLE on `macos-latest` (arm64) and `ubuntu-24.04`
  (x64). It uploads `angle-<rev>-<os>-<arch>.tar.zst` as assets of a
  GitHub release named `angle-<rev>`. The asset's SHA-256 goes in
  `browser/gpu/angle.lock`. The `browser` job downloads it with
  `browser/gpu/fetch_angle.sh` and caches it with `actions/cache`, keyed
  by `angle.lock`.
  - A release asset is permanent; the `actions/cache` copy alone is
    evicted after 7 days without use.
  - A full ANGLE build takes about an hour on a hosted runner, and
    `gclient sync` downloads several GB.
- Locally, `fetch_angle.sh` downloads the same asset. `build_angle.sh`
  is needed only to bump the revision or to debug ANGLE.
- **Linux backends.** On the desktop, ANGLE uses its Vulkan backend
  first and its desktop-GL backend (Mesa) as the fallback. In CI and
  headless, it uses Vulkan on SwiftShader. System Mesa GLES is never
  used directly: every WebGL call goes through ANGLE's validation.
- **Licenses.** The archive contains `LICENSE` files for ANGLE
  (BSD-3), SwiftShader (Apache-2.0) and the dependencies ANGLE links
  statically: SPIR-V tools and glslang (Apache-2.0 / BSD), the Vulkan
  loader on macOS (Apache-2.0), zlib. `build_angle.sh` collects them
  from the checkout's `third_party/*/LICENSE` for the targets it built.
  Dweller ships them in `licenses/`.

### Loading and the `@cimport` surface

Chromium loads `libEGL` and `libGLESv2` at run time and binds every
entry point through `eglGetProcAddress` (`ui/gl/generate_bindings.py`).
Dweller does the same, in `browser/gpu/gl.aye`:

- `dlopen` loads `libEGL` and `libGLESv2`. The search order is:
  `$DWELLER_ANGLE`, the binary's directory, `../Frameworks` (app
  bundle), then `target/angle/<rev>/lib`.
- Without ANGLE, Dweller still builds and runs: `getContext('webgl2')`
  returns `null`, as a browser without a GPU does. The WebGL tests fail
  with "ANGLE not found" instead of skipping, so CI cannot pass without
  ANGLE.
- The types and constants come from `@cimport("EGL/egl.h")`,
  `@cimport("EGL/eglext.h")` (which includes `eglext_angle.h`) and
  `@cimport("GLES3/gl3.h")` plus `GLES2/gl2ext.h`, built against the
  headers of the pinned ANGLE.
- The functions are pointers taken from `eglGetProcAddress`, using the
  headers' `PFN...PROC` types. If Aye cannot call a C function pointer
  of an `@cimport` typedef, P11.2 records the gap in
  `browser/COMPILER_GAPS.md` and uses `@link("EGL")` and
  `@link("GLESv2")` with an rpath instead.
- **App bundle (P6):** `Dweller.app/Contents/Frameworks/libEGL.dylib`,
  `libGLESv2.dylib` and `libvk_swiftshader.dylib`, ad-hoc signed with
  the app. On Linux: `lib/` next to the binary.

## 2. Contexts and surfaces

- **One EGL display per process.** It is made with
  `eglGetPlatformDisplay(EGL_PLATFORM_ANGLE_ANGLE, ...)` with
  `EGL_PLATFORM_ANGLE_TYPE_ANGLE`, which defaults to Metal on macOS and
  Vulkan on Linux. `DWELLER_GL` selects the backend:
  - `metal`: `EGL_PLATFORM_ANGLE_TYPE_METAL_ANGLE`
  - `vulkan`: `..._VULKAN_ANGLE`
  - `gl`: `..._OPENGL_ANGLE`
  - `swiftshader`: Vulkan plus `EGL_PLATFORM_ANGLE_DEVICE_TYPE_SWIFTSHADER_ANGLE`
  - `null`: `EGL_PLATFORM_ANGLE_TYPE_NULL_ANGLE`, no pixels
  - `off`: no WebGL

  The display is made lazily, on the first `getContext('webgl2')`, so
  pages without WebGL pay nothing.
- **One EGL context per WebGL canvas.** Each context is ES 3.0 and
  surfaceless (`EGL_KHR_surfaceless_context`), with no share group. It
  uses the same attributes as Chromium (`ui/gl/gl_context_egl.cc`):
  - `EGL_CONTEXT_WEBGL_COMPATIBILITY_ANGLE` = true
  - `EGL_CONTEXT_OPENGL_ROBUST_ACCESS_EXT` = true
  - `EGL_CONTEXT_OPENGL_RESET_NOTIFICATION_STRATEGY_EXT` =
    `EGL_LOSE_CONTEXT_ON_RESET_EXT`
  - `EGL_ROBUST_RESOURCE_INITIALIZATION_ANGLE` = true
  - `EGL_CONTEXT_BIND_GENERATES_RESOURCE_CHROMIUM` = false
  - `EGL_CONTEXT_CLIENT_ARRAYS_ENABLED_ANGLE` = false
  - `EGL_CONTEXT_OPENGL_BACKWARDS_COMPATIBLE_ANGLE` = false

  `eglMakeCurrent` is called whenever the current context changes. A
  page with one canvas switches once.
- **The drawing buffer is an FBO, not a pbuffer.** It is as Blink's
  `DrawingBuffer`: a color texture (RGBA8, or RGB8 when
  `alpha: false`) and a depth/stencil renderbuffer when requested.
  - With `antialias: true` (the default), a multisampled renderbuffer
    (4 samples, at most `GL_MAX_SAMPLES`) is resolved into the color
    texture with `glBlitFramebuffer` when the frame is taken.
  - The page's `bindFramebuffer(null)` binds this FBO.
  - The drawing buffer is resized when `canvas.width` or `height`
    changes, and is limited to `GL_MAX_TEXTURE_SIZE` and the
    `MAX_RENDERBUFFER_SIZE` caps (`drawingBufferWidth` shows the real
    size).
- **Frame end.** After the rAF callbacks of a frame, a context that drew
  is "taken":
  - its MSAA buffer is resolved;
  - the compositor gets the pixels (section 5);
  - with `preserveDrawingBuffer: false`, the buffer is then cleared to
    transparent black (to the clear values the spec gives) before the
    next draw.

  `premultipliedAlpha` says how the compositor reads the texture.
- **Lost context.**
  - `glGetGraphicsResetStatus` is checked after each frame. When it is
    not `GL_NO_ERROR` (a GPU reset on Metal or Vulkan) or a limit is
    hit, the context is marked lost:
    - every call returns its spec value (`null`, `CONTEXT_LOST_WEBGL`
      from `getError` once);
    - `webglcontextlost` fires.
  - If the listener called `preventDefault()`, a new context is made on
    a later task and `webglcontextrestored` fires.
  - `WEBGL_lose_context` drives the same path for tests.
- **Many canvases.** At most 16 live contexts per page (Chromium's
  limit). Making a 17th loses the least recently used one. Contexts of
  removed canvases are freed by the GC finalizer of the canvas wrapper,
  or when the page is torn down.
- **Spaces (P9.2).** A Space is a process, so it makes its own display.
  The backend is the window's backend (Metal works without a window),
  or `swiftshader` when the Space runs with `SDL_VIDEODRIVER=dummy` in
  CI. Agent screenshots of a WebGL page read the canvas as the window
  does.
- **Headless `dump` and `screenshot`.** They default to
  `DWELLER_GL=swiftshader`, which gives the same pixels on every
  machine. The pixel goldens are made with it.
  - API-only tests may use `null`.
  - Chromium disables SwiftShader WebGL on ARM CPUs (an LLVM 10 JIT
    issue noted in `ui/gl/gl_context_egl.cc`), and GitHub's macOS
    runners are arm64. P11.2 measures whether ANGLE's current
    SwiftShader works there.
  - If SwiftShader does not work on macOS arm64, the pixel goldens run
    on Ubuntu only, and macOS CI runs the WebGL tests on `null` (state,
    errors, readback of clears).

## 3. The WebGL API in the JS engine

**Layout.**

- The API is native Aye in `browser/engine/webgl_api.aye`, as
  `canvas_api.aye` is for 2D. Each method is a native function that
  validates its arguments, maps JS objects to GL names and calls GL.
- A small prelude, `browser/engine/webgl_prelude.aye`, holds the
  classes, the constants (generated from `gl3.h` into a table), the
  dictionary arguments and `getContext`.

**Object model.**

- Every `WebGLBuffer`, `WebGLTexture`, `WebGLProgram`, `WebGLShader`,
  `WebGLFramebuffer`, `WebGLRenderbuffer`, `WebGLVertexArrayObject`,
  `WebGLSampler`, `WebGLQuery`, `WebGLSync`, `WebGLTransformFeedback`
  and `WebGLUniformLocation` is a JS object with a native slot of
  (context id, GL name, deleted flag).
- Using an object of another context gives `INVALID_OPERATION`. A
  deleted object gives `INVALID_OPERATION` or is ignored, as the spec
  says per call. ANGLE does not check either rule, because it does not
  know JS objects.
- `WebGLUniformLocation` keeps its program and the program's link
  count, so a location from before a relink is rejected.
- `getParameter` returns the types the spec gives: `Int32Array` for
  viewports and `Float32Array` for ranges.

**Typed arrays.**

- Today `ArrayBuffer` keeps its bytes in a JS array of numbers
  (`browser/js/lib.aye` `_bytes`, `browser/js/typed.aye`). `bufferData`
  of a 1 MB vertex buffer would convert a million values, and
  WebAssembly memory cannot be an array of numbers.
- **P11.3 comes first.** It gives `ArrayBuffer` one contiguous byte
  store (a pointer and a length owned by the heap, freed by the GC, and
  detachable), and the views read and write it directly.
- It first checks the Aye repo's WIP P3.58 ("native `TypedArray`",
  34c2234) and takes it over if it fits.
- With the byte store, `bufferData`, `texImage2D` and `readPixels`
  pass a pointer and a byte length, after checking the view's type
  against the GL type, as the spec requires.

**Texture sources.**

- `texImage2D` and `texSubImage2D` take an `ArrayBufferView`,
  `ImageData`, `HTMLImageElement`, `HTMLCanvasElement`, `ImageBitmap`
  and `OffscreenCanvas` (all via `image.Bitmap`, straight RGBA8).
- `UNPACK_FLIP_Y_WEBGL`, `UNPACK_PREMULTIPLY_ALPHA_WEBGL` and
  `UNPACK_COLORSPACE_CONVERSION_WEBGL` are done in Aye on a copy before
  the upload. The other unpack parameters (alignment, row length, skip)
  go to GL.
- An image or canvas that is not origin-clean throws `SecurityError`
  (WebGL forbids reading cross-origin pixels into a texture).
- `HTMLVideoElement` sources wait for video decoding, which Dweller
  does not have (P11.28).

**Extensions.**

- `getSupportedExtensions` lists what Dweller implements, provided ANGLE
  can enable the extension through `glRequestExtensionANGLE`
  (`GL_ANGLE_request_extension`; a WebGL context starts with none).
- WebGL 2 set, first:
  - `EXT_color_buffer_float`, `EXT_color_buffer_half_float`
  - `EXT_texture_filter_anisotropic`, `OES_texture_float_linear`
  - `EXT_float_blend`, `KHR_parallel_shader_compile`
  - `WEBGL_lose_context`, `WEBGL_debug_renderer_info` (with a generic
    renderer string, as Firefox does)
  - `EXT_texture_norm16`
  - the compressed formats ANGLE has on the device:
    `WEBGL_compressed_texture_s3tc` (and `_srgb`),
    `WEBGL_compressed_texture_etc`, `WEBGL_compressed_texture_astc`,
    `EXT_texture_compression_bptc` and `_rgtc`.
- The WebGL 1 set comes with P11.29:
  - `OES_vertex_array_object`, `ANGLE_instanced_arrays`
  - `WEBGL_depth_texture`, `OES_texture_float`, `OES_texture_half_float`
  - `OES_element_index_uint`, `OES_standard_derivatives`
  - `EXT_shader_texture_lod`, `EXT_frag_depth`, `EXT_blend_minmax`
  - `WEBGL_draw_buffers`, `EXT_sRGB`
- Not exposed: `EXT_disjoint_timer_query_webgl2` (a precise GPU timer,
  which browsers gate for timing attacks).

**Errors and attributes.**

- `getError` returns the first error recorded since the last call: GL's
  error, or one the front end raised before calling GL. It is not
  called after every GL call, because that would stall Metal.
- The front end raises the WebGL-only errors itself.
- `getContextAttributes` returns what was granted: `alpha`, `depth`,
  `stencil`, `antialias`, `premultipliedAlpha`,
  `preserveDrawingBuffer`, `powerPreference`,
  `failIfMajorPerformanceCaveat` (true fails on SwiftShader) and
  `desynchronized` (false).

## 4. Shaders and security

- **Shader source is never passed raw to a driver.** `shaderSource` and
  `compileShader` go to ANGLE's `glShaderSource` and `glCompileShader`
  on a WebGL compatibility context. ANGLE then picks `SH_WEBGL2_SPEC`
  (or `SH_WEBGL_SPEC`) itself (`Compiler.cpp`, `SelectShaderSpec`) and
  does the rest:
  - it validates against WebGL GLSL ES (identifier length, no
    `#extension` that was not enabled, loop and index rules for WebGL
    1);
  - it translates to MSL, SPIR-V or GLSL for the backend;
  - it adds its workarounds (array bounds clamping, variable
    initialization).

  Dweller does not call `ShCompile` itself.
- `getShaderInfoLog` returns ANGLE's log. `getTranslatedShaderSource`
  (`WEBGL_debug_shaders`) is not exposed.
- **Robust access.**
  - Out-of-range buffer reads in shaders return 0 or a value inside the
    buffer (`EGL_CONTEXT_OPENGL_ROBUST_ACCESS_EXT`).
  - `drawElements` index ranges are checked by ANGLE's WebGL validation.
  - New textures, renderbuffers and buffers are zeroed
    (`EGL_ROBUST_RESOURCE_INITIALIZATION_ANGLE`), so a page never reads
    another page's or process's GPU memory.
  - `readPixels` outside the framebuffer returns zeros for those pixels
    (spec, done by ANGLE in WebGL mode).
- **Limits.**
  - `MAX_TEXTURE_SIZE` and the renderbuffer limits are ANGLE's caps,
    clamped to 16384.
  - Drawing buffers are at most 16384 on a side and 64 Mpx, scaled
    down as the spec allows.
  - At most 16 contexts per page.
  - A shader source over 512 KB is rejected with `INVALID_VALUE`.
  - GPU memory is not counted at first. If a page exhausts it, ANGLE
    reports `OUT_OF_MEMORY` or the context is lost, and the window
    survives either way.
- **Hangs.** Metal and Vulkan reset a hung GPU. ANGLE reports it
  through robustness, and the context is lost (section 2). The window
  stays responsive because the window thread waits on no GPU fence
  longer than a frame: readback uses a fence (section 5).

## 5. Compositing

**Step 1: readback into the existing pipeline (P11.7).**

- At frame end, a canvas whose context drew is resolved, then read into
  a pixel-pack buffer (`GL_PIXEL_PACK_BUFFER`) with `glReadPixels`, and
  a fence is set.
- On the next frame the buffer is mapped (`glMapBufferRange`), flipped
  in Y, and un-premultiplied when `alpha && premultipliedAlpha`. It is
  then copied into the canvas's `image.Bitmap`, and `Bitmap.version` is
  bumped.
- The display list, the damage tracking and the raster cache do the
  rest unchanged: the canvas is an `Item.Image`, as a 2D canvas is.
- `toDataURL`, `drawImage(webglCanvas)`, `texImage2D(webglCanvas)`,
  `screenshot` and `dump` read the same bitmap. When they come before
  frame end, they take the frame synchronously, as the spec requires
  for `preserveDrawingBuffer: false`.
- One frame of latency: the canvas shows the frame drawn in the
  previous rAF. Chromium has the same latency with its compositor.

Cost at 1280x720 and 60 fps:

- One frame is 3.7 MB. Per frame: readback (GPU to shared memory; on
  Apple Silicon the memory is unified), Y flip into the bitmap, copy
  into the page canvas (`pagedraw`), and upload to the SDL texture
  (`SDL_UpdateTexture`). That is about 4 x 3.7 = 15 MB of memory
  traffic, or about 0.9 GB/s.
- Estimate: 3 to 5 ms of the 16.7 ms frame on an M-series Mac. Most of
  it is the copies, so the fence keeps it from becoming a GPU stall. Un-
  premultiplying adds about 2 ms; three.js uses `alpha: false` by
  default, so it is skipped.
- At device pixel ratio 2 (2560x1440, 14.7 MB) the estimate is 12 to
  20 ms. That is too slow, which is why step 2 exists. Until then, a
  game should cap its pixel ratio.
- P11.7 measures the real numbers with `DWELLER_TIMING`
  (`webgl_readback`, `ui_draw`).

**Step 2: GPU compositing of the page (P11.22, P11.23).** The idea is
from `docs/how_cc_works.md` (layers rasterized into textures,
composited into a frame of quads, damage limiting what is drawn).
Dweller does not take property trees, the pending/active trees or a
compositor thread.

- **The window presents through ANGLE, not `SDL_Renderer`.**
  - On macOS, the SDL window is made with `SDL_WINDOW_METAL`, and
    `SDL_Metal_GetLayer` gives the `CAMetalLayer` that ANGLE's Metal
    `eglCreateWindowSurface` takes. P11.22 checks this against the
    sample windows in ANGLE's `util/` directory.
  - On Linux, the X11 window id or a `wl_egl_window`.
  - `eglSwapInterval(1)` gives vsync.
- **The page stays CPU-rasterized.** `pagedraw.aye`'s canvas becomes
  texture tiles (256x256, as cc's software raster). Only damaged tiles
  are uploaded (`glTexSubImage2D`).
- The chrome (tab strip, address bar, overlays) is its own texture.
- The compositor context gets each WebGL canvas's color texture as an
  `EGLImage` (`EGL_KHR_gl_texture_2D_image`) and draws it as a quad: no
  readback and no copies. It needs `preserveDrawingBuffer: false` to
  swap between two color textures.
- **Promoted layers (P4.56)** become textures. Their transform and
  opacity are applied in the quad shader, so a transform animation costs
  one draw call, not a CPU blend.
- **Scrolling** moves the tile quads; only newly exposed tiles are
  rasterized. That replaces the "move the pixels" copy of P4.35 and
  P4.36 on the GPU path.
- **What changes in `ui/`:**
  - `window.aye`: the present path, and context setup at window
    creation.
  - `pagedraw.aye`: tiles instead of one canvas; the damage marks tiles
    dirty.
  - `chrome.aye`: draws into its own bitmap.
  - New: `ui/compositor.aye` (quads, one shader, tile pool).
  - `DWELLER_DAMAGE_CHECK` compares the composited frame with a whole
    CPU frame, read back.
- **What it helps beyond WebGL:** scroll frames no longer copy or upload
  the whole page (W7 scroll p95), opacity and transform animations do
  not blend on the CPU (mistral.ai-style carousels), and Retina windows
  do not upload 4x the pixels per frame.
- **Fallback.** `DWELLER_GL=off`, or no ANGLE, keeps the
  `SDL_Renderer` path. The tests run both until step 2 has run on
  `main` for a while.

## 6. Frame timing and the APIs games need

**Frame timing (P11.10).**

- **Today:**
  - `requestAnimationFrame` is a 16 ms timer
    (`browser/engine/prelude.aye`), so it is not aligned to the
    display. Each callback gets its own time.
  - `performance.now()` has whole-millisecond resolution
    (`time.monotonicMs` in `engine/script.aye` `tick`).
  - The renderer is created without `SDL_RENDERER_PRESENTVSYNC`.
- **Target:**
  - rAF callbacks run once per display frame, as the "update the
    rendering" step of the window's frame, before style, layout and
    paint. All callbacks of a frame get the same timestamp: the frame's
    start time.
  - The frame interval is the display's refresh rate
    (`SDL_GetWindowDisplayMode`): 60 or 120 Hz on ProMotion. Hidden
    windows are throttled (P4.55).
  - Present with vsync (`SDL_RENDERER_PRESENTVSYNC`, then
    `eglSwapInterval` in step 2) and schedule the next frame from the
    vsync time, not from a timer.
  - `performance.now()` has 100 µs resolution, from a monotonic
    microsecond clock rounded down (the coarsening browsers apply
    without cross-origin isolation). The virtual clock of `dump` stays
    as it is.
- **Input latency.** Input events are dispatched as tasks before the
  frame's rAF. A key pressed during frame N is seen by the rAF of frame
  N+1, and its pixels show at N+2 with step 1 (readback latency), or at
  N+1 with step 2. P11.10 measures the path from an SDL event timestamp
  to the present of a frame that used it.

**Adjacent APIs.** The table below shows what the game uses (grep of
`~/Documents/f1/web/src`, tests excluded) and what Dweller has today
(grep of `browser/`).

| API | Game use | Dweller today | Card |
|---|---|---|---|
| WebGL 2 (three r170) | `WebGLRenderer` in 5 entry points | none | P11.4–P11.15 |
| WebAssembly | `sim_wasm_bg.wasm` (2.0 MB, wasm-bindgen; Rust sim at a fixed 200 Hz step); `instantiate`, `instantiateStreaming`, `Module`, `Instance` | none | P11.16, P11.17 |
| Web Audio | `AudioContext` in 4 files; `createGain` (12), `createBufferSource` (6), `createBiquadFilter` (6), `createStereoPanner` (2), `createBuffer` (2), `createDynamicsCompressor`, `createConvolver`, `decodeAudioData` (2, WAV files) | none | P11.19, P11.20 |
| Workers | module workers: `env/cloudNoise.worker.ts` (game), `tunnel/worker.ts`; a blob-URL worker in the recorder | none (`URL.createObjectURL` returns a URL that resolves to nothing) | P11.18 |
| Pointer Lock | `requestPointerLock` (2 pages), `pointerlockchange`, `movementX` | none | P11.11 |
| Pointer events | `pointerdown` (5), `pointermove` (4), `pointerup`, `pointercancel`, `pointerleave` | the `PointerEvent` class exists; the window never dispatches pointer events | P11.11 |
| `KeyboardEvent.code` | 30 uses | trusted key events carry only `key` | P11.11 |
| `structuredClone` | 8 uses | none | P11.11 |
| `visibilitychange` | 1 | none | P11.11 |
| Fullscreen | none | none | P11.11 (small, games expect it) |
| OffscreenCanvas | 2D, in the recorder only (`recordUpload.ts`, with insertable streams) | main-thread 2D (`canvas_prelude.aye`) | P11.25 if needed |
| KTX2 / Basis textures | none found (`KTX2Loader` absent; textures are WebP, which Dweller decodes) | none | P11.26 if needed |
| Gamepad | none | none | P11.27 if needed |
| `requestVideoFrameCallback`, video textures | none | `HTMLVideoElement` without decoding | P11.28 if needed |
| `FinalizationRegistry`, `TextDecoder`/`TextEncoder` (wasm-bindgen glue) | yes | present | – |
| `devicePixelRatio`, `ResizeObserver` | yes | present | – |

**WebAssembly runtime: wasmtime (Cranelift), chosen by P11.16.**

- An interpreter in Aye would run the sim's 200 Hz step far too slowly.
  The budget is at most 0.5 ms per step (about 3.3 steps per 60 Hz
  frame, under 2 ms of the frame).
- **Measurement (P11.16, 2026-10-15).** The user's Mac (Apple Silicon,
  arm64, macOS 26), with an ANGLE build running on the same machine.
  - The bench is a `sim-bench` crate added to a scratch copy of the
    game. It wraps `sim_wasm::SimHandle` exactly as `web/src/main.ts`
    uses it: `new SimHandle('sepang')`, `set_autopilot(true, true)`,
    then `step(1)` 20,000 times (100 s of race time), and reads
    `snapshot().speed_kmh` as a checksum.
  - It exports three C-ABI functions (`bench_init`, `bench_step`,
    `bench_speed`). It is built for `wasm32-unknown-unknown` with the
    workspace's release profile, then optimized with `wasm-opt -O`, as
    `wasm-pack` builds the shipped `sim_wasm_bg.wasm` (2.1 MB).
  - The five wasm-bindgen imports are never called on this path. Each
    harness turns them into traps.
  - Every wasm runtime gives the same checksum (268.5856). Native gives
    269.8566 because arm64 code contracts to FMA.
  - Each number is the median of 5 runs. "Init" is instantiation plus
    `new SimHandle`, which builds the racing line (about 0.4 s native).
    It is the load-time cost.

| Runtime (version, mode) | µs per step | × native | steps/s | init | peak RSS | library size |
|---|---|---|---|---|---|---|
| native (`aarch64-apple-darwin`, LTO) | 9.0 | 1.0 | 111,000 | 0.40 s | 5 MB | – |
| node 22.22 (V8: Liftoff, then TurboFan) | 32.3 | 3.6 | 31,000 | 1.22 s | 89 MB | – |
| node 22.22, the game's own glue and `sim_wasm_bg.wasm` | 31.0 | 3.4 | 32,000 | 1.28 s | 99 MB | – |
| node 22.22 `--liftoff-only` (V8 baseline) | 60.4 | 6.7 | 16,600 | 2.16 s | 57 MB | – |
| **wasmtime 49.0.3, Cranelift** (C API) | **29.1** | **3.2** | **34,300** | 1.66 s (compile 0.14 s) | 165 MB | 25.5 MB dylib, 19.6 MB static (macOS); 31.6 MB `.so` (Linux x64) |
| wasmtime 49.0.3, Winch (baseline JIT) | 93.5 | 10.4 | 10,700 | 3.20 s (compile 0.04 s) | 62 MB | (same library) |
| wasmtime 49.0.3, Pulley (interpreter) | 892 | 99 | 1,100 | 34.0 s | 192 MB | (same library) |
| wasm3 (main at 28ecb9a, interpreter) | 362 | 40 | 2,800 | 12.2 s | 8 MB | 0.28 MB |
| WAMR 2.4.5, fast interpreter | 500 | 56 | 2,000 | 18.9 s | 20 MB | 0.33 MB (with AOT loader) |

- **Not measured:**
  - WAMR AOT and LLVM JIT. `wamrc` does not build against the
    installed LLVM 23 (`LLVMOrcThreadSafeContextGetContext` was
    removed). WAMR's pinned LLVM is a source build of over an hour on
    half the cores.
  - WAMR Fast JIT, which exists only on x86-64.
  - `node --jitless`, which has no `WebAssembly` object.
  - AOT does not fit wasm fetched at run time anyway. An LLVM JIT
    would ship LLVM, which is larger than wasmtime.
- **Decision: wasmtime with Cranelift.**
  - It is the fastest runtime measured, slightly ahead of V8. A step
    takes 29 µs, 6% of the 0.5 ms budget and 0.6% of the 5 ms step
    period. Three steps per frame take under 0.1 ms.
  - Load time is the same as in a browser (1.7 s, against V8's 1.2 s).
  - The interpreters fail the "comfortably" criterion:
    - wasm3 at 362 µs and WAMR at 500 µs pass or touch the 0.5 ms line
      with one car. A race steps every AI car too, which scales the
      cost with the field.
    - `new SimHandle` takes 12 to 19 s instead of 1.2 s.
  - The simpler embeddings (wasm3, WAMR: C, under 0.4 MB) therefore
    lose on the game's own numbers, not only on peak speed.
  - Winch and Pulley stay available in the same library through
    `wasmtime_config_strategy_set` and `wasmtime_config_target_set`.
    Pulley is the fallback for a platform without a Cranelift backend.
- **Embedding.**
  - The C API is `wasm.h` plus `wasmtime.h`. They are plain C (C11
    `stdint`/`stdbool`). The `.hh` files are an optional C++ wrapper
    that Dweller does not include.
  - The library is Rust with a C ABI. It needs no C++ runtime: the
    macOS dylib links only `libSystem` and `libiconv`; Linux needs
    `-lpthread -ldl -lm`.
  - `wasm.h` declares its vector and reference types through
    function-like macros (`WASM_DECLARE_VEC`, `WASM_DECLARE_REF`).
    `@cimport` reads the header through libclang, which expands them:
    P11.17 declares the whole C API with `@cimport("wasmtime.h")` and
    writes nothing by hand. The headers are vendored in
    `browser/wasm/include`, so the browser builds without the library.
  - Calls the JS API needs: `wasmtime_module_new` and
    `wasmtime_module_validate`; `wasmtime_linker_*` or
    `wasmtime_instance_new` with host functions from
    `wasmtime_func_new` (imports call JS); `wasmtime_func_call`
    (exports); `wasmtime_memory_data` and `wasmtime_memory_grow`
    (`Memory.buffer`); `wasmtime_table_*` and `wasmtime_global_*`.
    Traps come back as `wasm_trap_t`, which becomes `RuntimeError`.
- **Shipping.**
  - Bytecode Alliance publishes the C API as release assets for both
    platforms (`wasmtime-v<ver>-aarch64-macos-c-api.tar.xz`,
    `wasmtime-v<ver>-x86_64-linux-c-api.tar.xz`). Unlike ANGLE, Dweller
    builds nothing.
  - `browser/wasm/wasmtime.lock` holds the version and each asset's
    SHA-256. `browser/wasm/fetch_wasmtime.sh` downloads and checks them
    into `target/wasmtime/<ver>/` (`lib/`, `include/`). CI caches that
    directory, keyed by the lock file.
  - Dweller links `libwasmtime.dylib` (`.so` on Linux) with an rpath,
    as P11.2 does for ANGLE. Without it, `WebAssembly` is undefined.
  - App bundle (P6): `Contents/Frameworks/libwasmtime.dylib`. A
    hardened-runtime signature needs the
    `com.apple.security.cs.allow-jit` entitlement for Cranelift's
    executable memory. Ad-hoc signing without the hardened runtime
    does not.
  - The full asset also carries WASI, the component model, GC and
    Winch. A build from source of `wasmtime-c-api` with only
    `cranelift` (Rust is already in CI) is the way to shrink it, if
    25 MB matters.
- **Licenses.**
  - wasmtime: Apache-2.0 WITH LLVM-exception.
  - WAMR: Apache-2.0 WITH LLVM-exception.
  - wasm3: MIT.
  - The wasmtime `LICENSE` goes in `licenses/`.
- **Platforms.**
  - wasmtime has Cranelift backends and C API release builds for
    macOS arm64 and Linux x86-64.
  - WAMR's interpreters run on both. Its Fast JIT is x86-64 only.
  - wasm3 runs on both.
  - wasm3 is maintained again: over 100 commits in the past year
    (atomics and exception handling in 2026-09).
- **Security.**
  - wasmtime:
    - On 64-bit hosts it reserves 4 GiB plus guard pages per 32-bit
      memory, so an out-of-bounds access faults into a trap with no
      check per access.
    - Indirect calls are checked by signature, and table accesses
      have Spectre hardening.
    - It is fuzzed continuously (OSS-Fuzz) and has a published
      security-advisory process.
  - WAMR's interpreters check every access in software. On 64-bit
    hosts its AOT and JIT use guard pages too.
  - wasm3 checks bounds in software. Of the three, it has the smallest
    fuzzing and audit history.
- **Rerun.**
  - The harness sources and commands are in the P11.16 pull request.
  - P11.17's `wasm_sim_bench` compares against wasmtime's 29 µs.
- V8's `src/wasm/` is the reference for the JS API's semantics (memory
  growth detaching `buffer`, import and export wrapping), not for code
  to port.

**Workers (P11.18).**

- One OS thread per dedicated worker, each with its own `Interp` and
  event loop.
- `postMessage` uses structured clone; `transfer` moves an
  `ArrayBuffer`'s byte store (P11.3).
- The JS engine's module-level state is small: one top-level `var` in
  `browser/js/`, 15 in `browser/engine/`. P11.18 audits it, then either
  makes it thread-local or keeps it for the window thread only.
- `URL.createObjectURL` gets a real registry of blob URLs, so a blob
  worker and a `blob:` fetch resolve.

**Web Audio (P11.19, P11.20).**

- The graph renders on the window thread in 128-frame quanta, about
  30 ms ahead, into a ring buffer.
- An SDL audio device (`SDL_OpenAudioDevice`, float32 stereo at
  48 kHz; SDL2 is already linked) pulls from that buffer on its own
  thread. No JS runs on the audio thread, and parameter changes take
  effect at the next quantum.
- `decodeAudioData` handles WAV (PCM 8, 16 and 24 bit, and float) first,
  which is all the game ships. MP3, AAC and Opus come later, if needed.
- The node algorithms follow the Web Audio spec (biquad coefficients
  from the Audio EQ Cookbook, as the spec gives them), with Blink's
  `modules/webaudio/` as the algorithm reference.

## 7. Conformance

- **Suite.** The Khronos WebGL conformance tests: in the reference
  checkout at `third_party/webgl/src/sdk/tests/`, upstream
  `KhronosGroup/WebGL`.
  - `00_test_list.txt` lists `conformance/` (WebGL 1) and, with
    `--min-version 2.0.0`, `conformance2/`. Each directory has its own
    list.
  - Harness: `js/js-test-pre.js` and `js/webgl-test-utils.js`.
- **Fetched at a pinned commit, not vendored**, as WPT and test262 are
  (decision 2026-10-15): `browser/tools/webgl_cts.rev`, cached in CI.
- **Runner (P11.14).** `browser/tools/webgl_conformance.py`:
  - It serves the suite with the WPT static server (P10.1).
  - It runs each test page in `dweller dump --js` with
    `DWELLER_GL=swiftshader` and a prelude hook that defines
    `window.webglTestHarness`. `js-test-pre.js` calls
    `webglTestHarness.reportResults(path, success, msg)` and
    `notifyFinished(path)`, so the runner needs no screen-scraping.
  - It writes a per-directory table into `docs/conformance.md`, rows
    "WebGL 2: buffers", and so on.
  - Each page has a 30 s timeout. A crash counts as fail.
- **The first subset is `conformance2/` in the directories three.js
  exercises:** `buffers`, `context`, `programs`, `glsl3`, `state`,
  `textures/misc`, `textures/image` (2D only), `rendering`,
  `vertex_arrays`, `uniforms`, `renderbuffers`, `reading`, `attribs`.
  - `extensions`, `query`, `sync` and `transform_feedback` follow
    P11.12 and P11.13.
  - The large `textures/*` format matrices and `deqp/` come last.
  - `conformance/` (WebGL 1) comes with P11.29.
- **Thresholds.** The first run is a baseline. The area threshold is
  set at 90% of test pages passing for the first subset once P11.13 has
  landed, by a recorded decision as with the WPT areas.
  `conformance.py check` (P10.4) includes the WebGL rows, and the weekly
  CI run executes them on Ubuntu (SwiftShader).
- **Pixel tests** (Dweller's own, `browser/tests/webgl/`) compare
  `screenshot` PNGs made with SwiftShader against goldens. Upstream
  pages check pixels themselves through `readPixels`.

## 8. Order

The cards are in `docs/plans/browser.md`, phase P11. Each is PR-sized,
about 45 minutes of agent work, with a verify command.

- **The game's path:**
  - P11.1–P11.3 (ANGLE, smoke test, byte store);
  - P11.4–P11.9 (WebGL 2 up to a three.js cube in the window);
  - P11.10–P11.11 (frame timing, input);
  - P11.12–P11.15 (the rest of WebGL 2, extensions, conformance, loss);
  - P11.16–P11.20 (WASM, Workers, audio);
  - P11.21 (frame budget);
  - P11.22–P11.23 (GPU compositing);
  - P11.24 (the game, accepted).
- P11.16 (the WASM measurement) touches no browser code and can run at
  any time.
- P11.25–P11.28 are needed only if the game needs them. P11.29
  (WebGL 1) is for other sites.

**Risk: JS speed.** three.js does a lot of JS per frame: matrix updates,
uniform uploads and the render list. Dweller's interpreter may cost more
than the GPU does. P11.9 records the JS time of a frame of the cube.
P11.21 measures the game's scene. If JS dominates, the lever is the
bytecode compiler (`browser/js/PERF.md`, P4.30), not WebGL.
