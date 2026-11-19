# Conformance

Measured 2026-11-19 at Dweller 2cc08106. WPT `e58225abb9`, test262 `d53ebc8b19`, WebGL `193020cd37`.
Score is passed over total subtests (WPT, WebGL) or tests (test262), skipped files excluded.
Change is in points since the previous table. Regenerate with `conformance.py run`, then `table`.

| Area | Files | Skipped | Passed | Score % | Threshold % | Target % | Status | Change |
|---|---:|---:|---:|---:|---:|---:|---|---:|
| `dom/nodes` | 414 | 3 | 7645/9684 | 78.9 | 75 | - | done | +1.8 |
| `dom/events` | 181 | 7 | 669/878 | 76.2 | 70 | - | done | -0.1 |
| `uievents` | 69 | 6 | 153/201 | 76.1 | 70 | - | done | +0.0 |
| `html/semantics/forms` | 686 | 172 | 3441/5413 | 63.6 | 60 | 82 | done | +9.3 |
| `css/cssom` | 292 | 11 | 2322/3821 | 60.8 | - | - | baseline | +5.0 |
| `fetch/api` | 191 | 0 | 1842/2478 | 74.3 | 50 | - | done | +0.4 |
| `xhr` | 329 | 3 | 732/1234 | 59.3 | 50 | - | done | +3.6 |
| `url` | 34 | 0 | 9558/9998 | 95.6 | 90 | - | done | +1.7 |
| `encoding` | 158 | 4 | 1138971/1139872 | 99.9 | 80 | - | done | +0.0 |
| `webaudio` | 305 | 5 | 1571/2685 | 58.5 | - | - | baseline | -5.0 |
| `css/css-flexbox` | 1422 | 469 | 4610/5816 | 79.3 | 70 | 87 | done | +16.4 |
| `css/css-transforms` | 711 | 156 | 3733/5428 | 68.8 | 50 | 89 | done | +16.4 |
| `css/css-transitions` | 143 | 59 | 1687/3374 | 50.0 | - | - | baseline | +6.0 |
| `css/css-animations` | 206 | 91 | 985/1405 | 70.1 | 65 | - | done | +1.0 |
| `html/syntax` | 98 | 0 | 2560/2772 | 92.4 | 90 | - | done | -1.4 |
| `dom/api` | 469 | 107 | 61484/65091 | 94.5 | 75 | - | done | +0.0 |
| `html/webappapis/timers` | 16 | 0 | 20/25 | 80.0 | 70 | - | done | +0.0 |
| `css/selectors` | 540 | 136 | 5190/6280 | 82.6 | 80 | - | done | +1.4 |
| `css/cascade-variables` | 398 | 31 | 1118/1436 | 77.9 | 75 | 89 | done | +21.8 |
| `css/values-color` | 865 | 237 | 16768/23417 | 71.6 | 70 | 70 | done | +13.7 |
| `css/box-sizing` | 1591 | 259 | 6343/8549 | 74.2 | 70 | 76 | done | +18.0 |
| `css/floats-tables` | 709 | 251 | 1139/1603 | 71.1 | 65 | 68 | done | +15.6 |
| `css/inline-text` | 2343 | 407 | 5070/8112 | 62.5 | 60 | - | done | +1.9 |
| `css/overflow` | 939 | 189 | 1171/1924 | 60.9 | 60 | 74 | done | +6.5 |
| `css/css-grid` | 2508 | 1085 | 11104/17578 | 63.2 | 60 | 69 | done | +15.9 |
| `css/positioning` | 915 | 239 | 1673/2310 | 72.4 | 70 | 84 | done | +10.7 |
| `css/backgrounds-borders` | 1041 | 295 | 5592/8473 | 66.0 | 65 | - | done | +0.9 |
| `css/css-fonts` | 560 | 325 | 7008/8396 | 83.5 | 50 | 89 | done | +50.7 |
| `images` | 715 | 268 | 4811/5965 | 80.7 | 60 | - | done | +3.2 |
| `intersection-observer` | 144 | 10 | 179/355 | 50.4 | - | - | baseline | +0.3 |
| `mixed-content` | 388 | 0 | 1279/2281 | 56.1 | 50 | 89 | done | +2.8 |
| `upgrade-insecure-requests` | 197 | 0 | 283/1000 | 28.3 | 25 | - | done | +0.2 |
| `html/browsers/sandboxing` | 21 | 1 | 20/32 | 62.5 | 30 | 64 | done | +28.1 |
| `fetch/metadata` | 87 | 0 | 666/1928 | 34.5 | 25 | - | done | +4.9 |
| `referrer-policy` | 1392 | 0 | 4166/8465 | 49.2 | 45 | - | done | +0.0 |
| `content-security-policy/script-src` | 97 | 0 | 128/217 | 59.0 | 55 | - | done | +0.9 |
| `content-security-policy/frame-ancestors` | 34 | 0 | 33/36 | 91.7 | 85 | - | done | +2.8 |
| `x-frame-options` | 6 | 0 | 157/157 | 100.0 | 35 | 90 | done | +61.1 |
| `fetch/nosniff` | 6 | 0 | 65/65 | 100.0 | 95 | - | done | +27.7 |
| `html/cross-origin-opener-policy` | 116 | 0 | 250/615 | 40.7 | 40 | - | done | +39.2 |
| `content-security-policy/style-src` | 42 | 0 | 80/87 | 92.0 | 80 | - | done | +0.0 |
| `content-security-policy/img-src` | 15 | 0 | 15/20 | 75.0 | 70 | - | done | +0.0 |
| `content-security-policy/connect-src` | 30 | 0 | 17/38 | 44.7 | 35 | - | done | +7.9 |
| `content-security-policy/securitypolicyviolation` | 38 | 0 | 30/76 | 39.5 | 35 | - | done | +0.0 |
| `content-security-policy` | 894 | 1 | 2379/4077 | 58.4 | 55 | 80 | done | +1.4 |
| `html/browsers/origin` | 100 | 1 | 100/668 | 15.0 | 2 | 39 | done | +12.6 |
| `css/css-writing-modes` | 1234 | 353 | 950/1555 | 61.1 | 40 | 89 | done | new |
| `websockets` | 214 | 0 | 517/610 | 84.8 | 80 | - | done | new |
| `IndexedDB` | 226 | 4 | 1296/1370 | 94.6 | - | - | baseline | new |
| `service-workers/service-worker` | 272 | 10 | 229/1528 | 15.0 | - | - | baseline | new |
| `service-workers/cache-storage` | 18 | 0 | 133/162 | 82.1 | - | - | baseline | new |
| `eventsource` | 63 | 0 | 87/102 | 85.3 | - | - | baseline | new |
| `css/css-color-adjust` | 27 | 21 | 149/171 | 87.1 | - | - | baseline | new |
| `selection` | 178 | 15 | 246/690 | 35.7 | - | - | baseline | new |
| `test262/built-ins` | 18373 | 5450 | 14475/18373 | 78.8 | 80 | 90 | below | +6.1 |
| `test262/language` | 23210 | 516 | 19664/23210 | 84.7 | 80 | 90 | done | +11.3 |
| `test262/annexB` | 1044 | 42 | 856/1044 | 82.0 | 80 | 90 | done | +40.7 |
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

- `content-security-policy`: 80. 90% of the 3666 of 4077 subtests that need none of: `report-sha256` hash reporting (Chromium only); tentative tests; the `csp` attribute of iframes (Chromium only).
- `mixed-content`: 89. 90% of the 2272 of 2281 subtests that need none of: tentative tests.
- `x-frame-options`: 90. 90% of the 157 of 157 subtests that need none of: tentative tests.
- `html/browsers/sandboxing`: 64. 90% of the 23 of 32 subtests that need none of: tentative tests; the `csp` attribute of iframes (Chromium only).
- `html/browsers/origin`: 39. 90% of the 294 of 668 subtests that need none of: `Origin-Agent-Cluster` (an opt-in finer than the site isolation of Q2; not in Safari); tentative tests.
- `css/css-grid`: 69. 90% of the 13617 of 17578 subtests that need none of: grid-lanes (masonry; in no stable browser); tentative tests.
- `css/css-flexbox`: 87. 90% of the 5686 of 5816 subtests that need none of: tentative tests.
- `css/box-sizing`: 76. 90% of the 7272 of 8549 subtests that need none of: `margin-trim` (Safari only); tentative tests; the `stretch` sizing keyword (Chromium only).
- `css/css-fonts`: 89. 90% of the 8361 of 8396 subtests that need none of: MathML font scaling; tentative tests.
- `css/cascade-variables`: 89. 90% of the 1434 of 1436 subtests that need none of: tentative tests.
- `css/values-color`: 70. 90% of the 17974 of 23417 subtests that need none of: `calc-size()` (in no stable browser); `if()` (Chromium only); `random()` (in no stable browser); `sibling-index()` (Chromium only); tentative tests.
- `css/positioning`: 84. 90% of the 2167 of 2310 subtests that need none of: `overlay` (Chromium only); tentative tests.
- `css/overflow`: 74. 90% of the 1600 of 1924 subtests that need none of: scroll markers (Chromium only); tentative tests.
- `css/floats-tables`: 68. 90% of the 1227 of 1603 subtests that need none of: tentative tests.
- `html/semantics/forms`: 82. 90% of the 4933 of 5413 subtests that need none of: customizable `<select>` (Chromium only); filterable `<select>` (in no stable browser); tentative tests.
- `css/css-writing-modes`: 89. 90% of the 1552 of 1555 subtests that need none of: tentative tests.
- `css/css-transforms`: 89. 90% of the 5418 of 5428 subtests that need none of: tentative tests.
- `test262/built-ins`: 90. 90% of the 18373 of 18373 subtests that need none of: tentative tests.
- `test262/language`: 90. 90% of the 23210 of 23210 subtests that need none of: tentative tests.
- `test262/annexB`: 90. 90% of the 1044 of 1044 subtests that need none of: tentative tests.
