# Conformance

Measured 2026-11-09 at Dweller 4f8516d. WPT `e58225abb9`, test262 `d53ebc8b19`, WebGL `193020cd37`.
Score is passed over total subtests (WPT, WebGL) or tests (test262), skipped files excluded.
Change is in points since the previous table. Regenerate with `conformance.py run`, then `table`.

| Area | Files | Skipped | Passed | Score % | Threshold % | Target % | Status | Change |
|---|---:|---:|---:|---:|---:|---:|---|---:|
| `dom/nodes` | 411 | 6 | 7252/9403 | 77.1 | 75 | - | done | +0.0 |
| `dom/events` | 180 | 8 | 669/877 | 76.3 | 70 | - | done | +0.0 |
| `uievents` | 69 | 6 | 153/201 | 76.1 | 70 | - | done | +0.0 |
| `html/semantics/forms` | 685 | 173 | 2932/5398 | 54.3 | 60 | 82 | below | +0.0 |
| `css/cssom` | 292 | 11 | 2134/3818 | 55.9 | - | - | baseline | +0.0 |
| `fetch/api` | 191 | 0 | 1836/2483 | 73.9 | 50 | - | done | +0.0 |
| `xhr` | 329 | 3 | 687/1233 | 55.7 | 50 | - | done | +0.0 |
| `url` | 34 | 0 | 9384/9998 | 93.9 | 90 | - | done | +3.6 |
| `encoding` | 157 | 5 | 1138861/1139871 | 99.9 | 80 | - | done | +0.0 |
| `webaudio` | 247 | 63 | 1514/2385 | 63.5 | - | - | baseline | +0.0 |
| `css/css-flexbox` | 1422 | 469 | 3653/5816 | 62.8 | 70 | 87 | below | +0.0 |
| `css/css-transforms` | 711 | 156 | 2840/5428 | 52.3 | 50 | 89 | done | +2.2 |
| `css/css-transitions` | 143 | 59 | 1473/3348 | 44.0 | - | - | baseline | +0.0 |
| `css/css-animations` | 206 | 91 | 971/1405 | 69.1 | 65 | - | done | +18.2 |
| `html/syntax` | 98 | 0 | 2560/2732 | 93.7 | 90 | - | done | +0.0 |
| `dom/api` | 467 | 109 | 61380/64996 | 94.4 | 75 | - | done | +0.0 |
| `html/webappapis/timers` | 16 | 0 | 20/25 | 80.0 | 70 | - | done | +0.0 |
| `css/selectors` | 540 | 136 | 5104/6283 | 81.2 | 80 | - | done | +0.0 |
| `css/cascade-variables` | 398 | 31 | 805/1436 | 56.1 | 75 | 89 | below | +0.0 |
| `css/values-color` | 865 | 237 | 13543/23406 | 57.9 | 70 | 70 | below | +0.0 |
| `css/box-sizing` | 1591 | 259 | 4806/8549 | 56.2 | 70 | 76 | below | +4.8 |
| `css/floats-tables` | 709 | 251 | 888/1601 | 55.5 | 65 | 68 | below | +0.0 |
| `css/inline-text` | 2343 | 407 | 4897/8083 | 60.6 | 60 | - | done | +0.0 |
| `css/overflow` | 939 | 189 | 1044/1921 | 54.3 | 60 | 74 | below | +0.0 |
| `css/css-grid` | 2508 | 1085 | 8305/17578 | 47.2 | 60 | 69 | below | +4.1 |
| `css/positioning` | 915 | 239 | 1425/2310 | 61.7 | 70 | 84 | below | +0.0 |
| `css/backgrounds-borders` | 1041 | 295 | 5518/8473 | 65.1 | 65 | - | done | +35.0 |
| `css/css-fonts` | 560 | 325 | 2736/8356 | 32.7 | 50 | 89 | below | +0.0 |
| `images` | 715 | 268 | 4621/5963 | 77.5 | 60 | - | done | +0.8 |
| `intersection-observer` | 144 | 10 | 178/355 | 50.1 | - | - | baseline | +0.0 |
| `mixed-content` | 388 | 0 | 1216/2281 | 53.3 | 50 | 89 | done | +0.0 |
| `upgrade-insecure-requests` | 197 | 0 | 281/1000 | 28.1 | 25 | - | done | +0.0 |
| `html/browsers/sandboxing` | 21 | 1 | 11/32 | 34.4 | 30 | 64 | done | +0.0 |
| `fetch/metadata` | 87 | 0 | 541/1828 | 29.6 | 25 | - | done | +0.0 |
| `referrer-policy` | 1392 | 0 | 4166/8465 | 49.2 | 45 | - | done | +0.0 |
| `content-security-policy/script-src` | 97 | 0 | 127/217 | 58.5 | 55 | - | done | +0.0 |
| `content-security-policy/frame-ancestors` | 34 | 0 | 32/36 | 88.9 | 85 | - | done | +0.0 |
| `x-frame-options` | 6 | 0 | 61/157 | 38.9 | 35 | 90 | done | +0.0 |
| `fetch/nosniff` | 6 | 0 | 65/65 | 100.0 | 95 | - | done | +27.7 |
| `html/cross-origin-opener-policy` | 116 | 0 | 250/615 | 40.7 | 40 | - | done | +39.2 |
| `content-security-policy/style-src` | 42 | 0 | 80/87 | 92.0 | 80 | - | done | new |
| `content-security-policy/img-src` | 15 | 0 | 15/20 | 75.0 | 70 | - | done | new |
| `content-security-policy/connect-src` | 30 | 0 | 14/38 | 36.8 | 35 | - | done | new |
| `content-security-policy/securitypolicyviolation` | 38 | 0 | 30/76 | 39.5 | 35 | - | done | new |
| `content-security-policy` | 894 | 1 | 2323/4076 | 57.0 | 55 | 80 | done | new |
| `html/browsers/origin` | 100 | 1 | 16/668 | 2.4 | 2 | 39 | done | new |
| `test262/built-ins` | 18373 | 5450 | 13352/18373 | 72.7 | 80 | 90 | below | +0.0 |
| `test262/language` | 23210 | 516 | 17032/23210 | 73.4 | 80 | 90 | below | +0.0 |
| `test262/annexB` | 1044 | 42 | 431/1044 | 41.3 | 80 | 90 | below | +0.0 |
| `WebGL 1: attribs` | 15 | 1 | 5694/5701 | 99.9 | - | - | baseline | +0.0 |
| `WebGL 1: buffers` | 13 | 0 | 283/285 | 99.3 | - | - | baseline | +0.0 |
| `WebGL 1: canvas` | 18 | 0 | 842/844 | 99.8 | - | - | baseline | +0.0 |
| `WebGL 1: context` | 18 | 3 | 1006/1028 | 97.9 | - | - | baseline | +0.0 |
| `WebGL 1: extensions` | 24 | 28 | 2710/2722 | 99.6 | - | - | baseline | +0.0 |
| `WebGL 1: glsl` | 310 | 6 | 7720/7724 | 99.9 | - | - | baseline | +0.0 |
| `WebGL 1: limits` | 5 | 0 | 31/31 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 1: misc` | 16 | 1 | 2436/2515 | 96.9 | - | - | baseline | +0.0 |
| `WebGL 1: offscreencanvas` | 12 | 0 | 18/25 | 72.0 | - | - | baseline | +0.0 |
| `WebGL 1: ogles` | 141 | 0 | 1900/1900 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 1: programs` | 12 | 0 | 270/270 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 1: reading` | 3 | 0 | 368/368 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 1: renderbuffers` | 5 | 2 | 125/129 | 96.9 | - | - | baseline | +0.0 |
| `WebGL 1: rendering` | 43 | 1 | 1063/1066 | 99.7 | - | - | baseline | +0.0 |
| `WebGL 1: state` | 6 | 3 | 688/688 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 1: textures` | 157 | 7 | 42279/43803 | 96.5 | - | - | baseline | +0.0 |
| `WebGL 1: typedarrays` | 7 | 0 | 1394/1414 | 98.6 | - | - | baseline | +0.0 |
| `WebGL 1: uniforms` | 30 | 0 | 944/960 | 98.3 | - | - | baseline | +0.0 |
| `WebGL 2: attribs` | 8 | 0 | 3104/3618 | 85.8 | - | - | baseline | +0.0 |
| `WebGL 2: buffers` | 13 | 0 | 1857/1868 | 99.4 | - | - | baseline | +0.0 |
| `WebGL 2: canvas` | 2 | 0 | 7/8 | 87.5 | - | - | baseline | +0.0 |
| `WebGL 2: context` | 9 | 0 | 146/163 | 89.6 | - | - | baseline | +0.0 |
| `WebGL 2: extensions` | 30 | 0 | 747/756 | 98.8 | - | - | baseline | +0.0 |
| `WebGL 2: glsl3` | 60 | 0 | 961/963 | 99.8 | - | - | baseline | +0.0 |
| `WebGL 2: misc` | 8 | 0 | 825/847 | 97.4 | - | - | baseline | +0.0 |
| `WebGL 2: offscreencanvas` | 8 | 0 | 14/15 | 93.3 | - | - | baseline | +0.0 |
| `WebGL 2: programs` | 4 | 0 | 152/152 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 2: query` | 3 | 0 | 60/61 | 98.4 | - | - | baseline | +0.0 |
| `WebGL 2: reading` | 5 | 0 | 519/588 | 88.3 | - | - | baseline | +0.0 |
| `WebGL 2: renderbuffers` | 10 | 0 | 633/683 | 92.7 | - | - | baseline | +0.0 |
| `WebGL 2: rendering` | 50 | 0 | 4676/4700 | 99.5 | - | - | baseline | +0.0 |
| `WebGL 2: samplers` | 3 | 0 | 169/169 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 2: state` | 4 | 0 | 521/531 | 98.1 | - | - | baseline | +0.0 |
| `WebGL 2: sync` | 1 | 0 | 13/14 | 92.9 | - | - | baseline | +0.0 |
| `WebGL 2: textures` | 926 | 0 | 243203/243835 | 99.7 | - | - | baseline | +0.0 |
| `WebGL 2: transform_feedback` | 10 | 0 | 476/481 | 99.0 | - | - | baseline | +0.0 |
| `WebGL 2: uniforms` | 9 | 0 | 7138/7138 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 2: vertex_arrays` | 2 | 0 | 41/49 | 83.7 | - | - | baseline | +0.0 |
| `WebGL 2: wasm` | 19 | 0 | 52/52 | 100.0 | - | - | baseline | +0.0 |

## Targets

A threshold is a floor: `conformance.py check` fails when an area falls, and the floor was set under a measured score.
A target is where the area is ready: 90% of the subtests that need no deferred feature, over all its subtests.
Nothing checks a target. The patterns and reasons are in `browser/tools/conformance/targets.json`; `conformance.py targets` counts them again.

- `content-security-policy`: 80. 90% of the 3665 of 4076 subtests that need none of: `report-sha256` hash reporting (Chromium only); tentative tests; the `csp` attribute of iframes (Chromium only).
- `mixed-content`: 89. 90% of the 2272 of 2281 subtests that need none of: tentative tests.
- `x-frame-options`: 90. 90% of the 157 of 157 subtests that need none of: tentative tests.
- `html/browsers/sandboxing`: 64. 90% of the 23 of 32 subtests that need none of: tentative tests; the `csp` attribute of iframes (Chromium only).
- `html/browsers/origin`: 39. 90% of the 294 of 668 subtests that need none of: `Origin-Agent-Cluster` (an opt-in finer than the site isolation of Q2; not in Safari); tentative tests.
- `css/css-grid`: 69. 90% of the 13617 of 17578 subtests that need none of: grid-lanes (masonry; in no stable browser); tentative tests.
- `css/css-flexbox`: 87. 90% of the 5686 of 5816 subtests that need none of: tentative tests.
- `css/box-sizing`: 76. 90% of the 7272 of 8549 subtests that need none of: `margin-trim` (Safari only); tentative tests; the `stretch` sizing keyword (Chromium only).
- `css/css-fonts`: 89. 90% of the 8321 of 8356 subtests that need none of: MathML font scaling; tentative tests.
- `css/cascade-variables`: 89. 90% of the 1434 of 1436 subtests that need none of: tentative tests.
- `css/values-color`: 70. 90% of the 17966 of 23406 subtests that need none of: `calc-size()` (in no stable browser); `if()` (Chromium only); `random()` (in no stable browser); `sibling-index()` (Chromium only); tentative tests.
- `css/positioning`: 84. 90% of the 2167 of 2310 subtests that need none of: `overlay` (Chromium only); tentative tests.
- `css/overflow`: 74. 90% of the 1598 of 1921 subtests that need none of: scroll markers (Chromium only); tentative tests.
- `css/floats-tables`: 68. 90% of the 1225 of 1601 subtests that need none of: tentative tests.
- `html/semantics/forms`: 82. 90% of the 4932 of 5398 subtests that need none of: customizable `<select>` (Chromium only); filterable `<select>` (in no stable browser); tentative tests.
- `css/css-writing-modes`: 89. 90% of the 1552 of 1555 subtests that need none of: tentative tests.
- `css/css-transforms`: 89. 90% of the 5418 of 5428 subtests that need none of: tentative tests.
- `test262/built-ins`: 90. 90% of the 18373 of 18373 subtests that need none of: tentative tests.
- `test262/language`: 90. 90% of the 23210 of 23210 subtests that need none of: tentative tests.
- `test262/annexB`: 90. 90% of the 1044 of 1044 subtests that need none of: tentative tests.
