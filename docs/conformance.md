# Conformance

Measured 2026-10-17 at Dweller 2a91881. WPT `e58225abb9`, test262 `d53ebc8b19`, WebGL `193020cd37`.
Score is passed over total subtests (WPT, WebGL) or tests (test262), skipped files excluded.
Change is in points since the previous table. Regenerate with `conformance.py run`, then `table`.

| Area | Files | Skipped | Passed | Score % | Threshold % | Status | Change |
|---|---:|---:|---:|---:|---:|---|---:|
| `dom/nodes` | 411 | 6 | 7213/9403 | 76.7 | 75 | done | +0.0 |
| `dom/events` | 180 | 8 | 611/855 | 71.5 | 70 | done | +0.0 |
| `uievents` | 69 | 6 | 25/178 | 14.0 | - | baseline | +0.0 |
| `html/semantics/forms` | 685 | 173 | 1927/5337 | 36.1 | 60 | below | +0.0 |
| `css/cssom` | 292 | 11 | 2025/3753 | 54.0 | - | baseline | +0.0 |
| `fetch/api` | 178 | 13 | 1576/2415 | 65.3 | 50 | done | +0.0 |
| `xhr` | 329 | 3 | 686/1233 | 55.6 | 50 | done | +0.0 |
| `url` | 34 | 0 | 4850/9998 | 48.5 | 80 | below | +0.0 |
| `encoding` | 157 | 5 | 1138861/1139871 | 99.9 | 80 | done | +0.0 |
| `webaudio` | 247 | 63 | 1509/2385 | 63.3 | - | baseline | +0.0 |
| `css/css-flexbox` | 1422 | 469 | 640/1691 | 37.8 | 70 | below | +0.0 |
| `css/css-transitions` | 143 | 59 | 335/3341 | 10.0 | - | baseline | +0.0 |
| `test262/built-ins` | 18373 | 5450 | 12825/18373 | 69.8 | 80 | below | +5.6 |
| `test262/language` | 23210 | 516 | 15080/23210 | 65.0 | 80 | below | +0.0 |
| `WebGL 1: attribs` | 15 | 1 | 0/15 | 0.0 | - | baseline | new |
| `WebGL 1: buffers` | 13 | 0 | 0/13 | 0.0 | - | baseline | new |
| `WebGL 1: canvas` | 18 | 0 | 0/18 | 0.0 | - | baseline | new |
| `WebGL 1: context` | 18 | 3 | 0/37 | 0.0 | - | baseline | new |
| `WebGL 1: extensions` | 24 | 28 | 0/24 | 0.0 | - | baseline | new |
| `WebGL 1: glsl` | 310 | 6 | 6/316 | 1.9 | - | baseline | new |
| `WebGL 1: limits` | 5 | 0 | 0/5 | 0.0 | - | baseline | new |
| `WebGL 1: misc` | 16 | 1 | 0/16 | 0.0 | - | baseline | new |
| `WebGL 1: offscreencanvas` | 12 | 0 | 1/13 | 7.7 | - | baseline | new |
| `WebGL 1: ogles` | 141 | 0 | 0/141 | 0.0 | - | baseline | new |
| `WebGL 1: programs` | 12 | 0 | 0/12 | 0.0 | - | baseline | new |
| `WebGL 1: reading` | 3 | 0 | 0/4 | 0.0 | - | baseline | new |
| `WebGL 1: renderbuffers` | 5 | 2 | 0/5 | 0.0 | - | baseline | new |
| `WebGL 1: rendering` | 43 | 1 | 0/43 | 0.0 | - | baseline | new |
| `WebGL 1: state` | 6 | 3 | 0/6 | 0.0 | - | baseline | new |
| `WebGL 1: textures` | 157 | 7 | 7/162 | 4.3 | - | baseline | new |
| `WebGL 1: typedarrays` | 7 | 0 | 1273/1294 | 98.4 | - | baseline | new |
| `WebGL 1: uniforms` | 30 | 0 | 0/30 | 0.0 | - | baseline | new |
| `WebGL 2: attribs` | 8 | 0 | 1/8 | 12.5 | - | baseline | new |
| `WebGL 2: buffers` | 13 | 0 | 0/13 | 0.0 | - | baseline | new |
| `WebGL 2: canvas` | 2 | 0 | 0/2 | 0.0 | - | baseline | new |
| `WebGL 2: context` | 9 | 0 | 19/27 | 70.4 | - | baseline | new |
| `WebGL 2: extensions` | 30 | 0 | 0/32 | 0.0 | - | baseline | new |
| `WebGL 2: glsl3` | 60 | 0 | 0/60 | 0.0 | - | baseline | new |
| `WebGL 2: misc` | 8 | 0 | 0/8 | 0.0 | - | baseline | new |
| `WebGL 2: offscreencanvas` | 8 | 0 | 2/10 | 20.0 | - | baseline | new |
| `WebGL 2: programs` | 4 | 0 | 0/4 | 0.0 | - | baseline | new |
| `WebGL 2: query` | 3 | 0 | 0/3 | 0.0 | - | baseline | new |
| `WebGL 2: reading` | 5 | 0 | 0/5 | 0.0 | - | baseline | new |
| `WebGL 2: renderbuffers` | 10 | 0 | 0/13 | 0.0 | - | baseline | new |
| `WebGL 2: rendering` | 50 | 0 | 0/53 | 0.0 | - | baseline | new |
| `WebGL 2: samplers` | 3 | 0 | 0/3 | 0.0 | - | baseline | new |
| `WebGL 2: state` | 4 | 0 | 0/4 | 0.0 | - | baseline | new |
| `WebGL 2: sync` | 1 | 0 | 0/1 | 0.0 | - | baseline | new |
| `WebGL 2: textures` | 926 | 0 | 1/932 | 0.1 | - | baseline | new |
| `WebGL 2: transform_feedback` | 10 | 0 | 0/10 | 0.0 | - | baseline | new |
| `WebGL 2: uniforms` | 9 | 0 | 0/9 | 0.0 | - | baseline | new |
| `WebGL 2: vertex_arrays` | 2 | 0 | 0/2 | 0.0 | - | baseline | new |
| `WebGL 2: wasm` | 19 | 0 | 0/19 | 0.0 | - | baseline | new |

## Not yet measured

| Area | Threshold % |
|---|---:|
| HTML parsing | 90 |
| DOM (traversal, ranges, collections, html/dom) | 75 |
| Events and timers (uievents, timers) | 70 |
| CSS selectors | 80 |
| CSS cascade and variables | 75 |
| CSS values, units, color | 70 |
| CSS box model and sizing | 70 |
| CSS floats and tables | 65 |
| CSS inline and text | 60 |
| CSS overflow | 60 |
| CSS grid | 60 |
| CSS positioning | 70 |
| CSS backgrounds and borders | 65 |
| CSS transforms (2D) | 50 |
| CSS fonts | 50 |
| Images (img element, css-images, png, gif) | 60 |
| JS (test262 annexB) | 80 |
