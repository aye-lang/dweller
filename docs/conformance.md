# Conformance

Measured 2026-11-23 at Dweller 8e2643a6. WPT `e58225abb9`, test262 `d53ebc8b19`, WebGL `193020cd37`.
Score is passed over total subtests (WPT, WebGL) or tests (test262), skipped files excluded.
Change is in points since the previous table. Regenerate with `conformance.py run`, then `table`.

| Area | Files | Skipped | Passed | Score % | Threshold % | Target % | Status | Change |
|---|---:|---:|---:|---:|---:|---:|---|---:|
| `dom/nodes` | 414 | 3 | 7662/9684 | 79.1 | 75 | - | done | +0.2 |
| `dom/events` | 181 | 7 | 670/878 | 76.3 | 70 | - | done | +0.1 |
| `uievents` | 69 | 6 | 154/201 | 76.6 | 70 | - | done | +0.5 |
| `html/semantics/forms` | 686 | 172 | 3868/5413 | 71.5 | 60 | 82 | done | +7.9 |
| `css/cssom` | 292 | 11 | 2341/3821 | 61.3 | - | - | baseline | +0.5 |
| `fetch/api` | 191 | 0 | 1904/2481 | 76.7 | 50 | - | done | +2.4 |
| `xhr` | 329 | 3 | 767/1250 | 61.4 | 50 | - | done | +2.0 |
| `url` | 34 | 0 | 9754/9998 | 97.6 | 90 | - | done | +2.0 |
| `encoding` | 158 | 4 | 1139009/1139872 | 99.9 | 80 | - | done | +0.0 |
| `webaudio` | 305 | 5 | 1592/2706 | 58.8 | - | - | baseline | +0.3 |
| `css/css-flexbox` | 1422 | 469 | 5083/5816 | 87.4 | 70 | 87 | done | +8.1 |
| `css/css-transforms` | 711 | 156 | 4362/5428 | 80.4 | 50 | 89 | done | +28.0 |
| `css/css-transitions` | 143 | 59 | 1690/3374 | 50.1 | - | - | baseline | +0.1 |
| `css/css-animations` | 206 | 91 | 995/1405 | 70.8 | 65 | - | done | +0.7 |
| `html/syntax` | 98 | 0 | 2560/2772 | 92.4 | 90 | - | done | +0.0 |
| `dom/api` | 469 | 107 | 61458/65091 | 94.4 | 75 | - | done | -0.0 |
| `html/webappapis/timers` | 16 | 0 | 20/25 | 80.0 | 70 | - | done | +0.0 |
| `css/selectors` | 540 | 136 | 5190/6280 | 82.6 | 80 | - | done | +0.0 |
| `css/cascade-variables` | 398 | 31 | 1122/1436 | 78.1 | 75 | 89 | done | +0.3 |
| `css/values-color` | 865 | 237 | 16782/23417 | 71.7 | 70 | 70 | done | +0.1 |
| `css/box-sizing` | 1591 | 259 | 6349/8549 | 74.3 | 70 | 76 | done | +18.1 |
| `css/floats-tables` | 709 | 251 | 1142/1603 | 71.2 | 65 | 68 | done | +15.8 |
| `css/inline-text` | 2343 | 407 | 5139/8112 | 63.4 | 60 | - | done | +0.8 |
| `css/overflow` | 939 | 189 | 1206/1924 | 62.7 | 60 | 74 | done | +8.3 |
| `css/css-grid` | 2508 | 1085 | 11189/17578 | 63.7 | 60 | 69 | done | +0.5 |
| `css/positioning` | 915 | 239 | 1673/2310 | 72.4 | 70 | 84 | done | +10.7 |
| `css/backgrounds-borders` | 1041 | 295 | 5592/8473 | 66.0 | 65 | - | done | +0.0 |
| `css/css-fonts` | 560 | 325 | 7469/8390 | 89.0 | 50 | 89 | done | +56.3 |
| `images` | 715 | 268 | 4811/5965 | 80.7 | 60 | - | done | +0.0 |
| `intersection-observer` | 144 | 10 | 179/355 | 50.4 | - | - | baseline | +0.0 |
| `mixed-content` | 388 | 0 | 2196/2281 | 96.3 | 50 | 89 | done | +40.2 |
| `upgrade-insecure-requests` | 197 | 0 | 724/993 | 72.9 | 25 | - | done | +44.6 |
| `html/browsers/sandboxing` | 21 | 1 | 21/32 | 65.6 | 30 | 64 | done | +31.2 |
| `fetch/metadata` | 87 | 0 | 820/2001 | 41.0 | 25 | - | done | +6.4 |
| `referrer-policy` | 1392 | 0 | 6152/8465 | 72.7 | 45 | - | done | +23.5 |
| `content-security-policy/script-src` | 97 | 0 | 142/217 | 65.4 | 55 | - | done | +6.5 |
| `content-security-policy/frame-ancestors` | 34 | 0 | 34/36 | 94.4 | 85 | - | done | +2.8 |
| `x-frame-options` | 6 | 0 | 157/157 | 100.0 | 35 | 90 | done | +0.0 |
| `fetch/nosniff` | 6 | 0 | 65/65 | 100.0 | 95 | - | done | +0.0 |
| `html/cross-origin-opener-policy` | 116 | 0 | 276/651 | 42.4 | 40 | - | done | +1.8 |
| `content-security-policy/style-src` | 42 | 0 | 80/87 | 92.0 | 80 | - | done | +0.0 |
| `content-security-policy/img-src` | 15 | 0 | 15/20 | 75.0 | 70 | - | done | +0.0 |
| `content-security-policy/connect-src` | 30 | 0 | 20/38 | 52.6 | 35 | - | done | +7.9 |
| `content-security-policy/securitypolicyviolation` | 38 | 0 | 32/80 | 40.0 | 35 | - | done | +0.5 |
| `content-security-policy` | 894 | 1 | 3341/4127 | 81.0 | 55 | 81 | done | +22.6 |
| `html/browsers/origin` | 100 | 1 | 312/676 | 46.2 | 2 | 40 | done | +31.2 |
| `css/css-writing-modes` | 1234 | 353 | 973/1555 | 62.6 | 40 | 89 | done | +1.5 |
| `websockets` | 214 | 0 | 517/610 | 84.8 | 80 | - | done | +0.0 |
| `IndexedDB` | 226 | 4 | 1307/1370 | 95.4 | - | - | baseline | +0.8 |
| `service-workers/service-worker` | 272 | 10 | 232/1442 | 16.1 | - | - | baseline | +1.1 |
| `service-workers/cache-storage` | 18 | 0 | 136/162 | 84.0 | - | - | baseline | +1.9 |
| `eventsource` | 63 | 0 | 97/102 | 95.1 | - | - | baseline | new |
| `css/css-color-adjust` | 27 | 21 | 163/171 | 95.3 | - | - | baseline | new |
| `selection` | 178 | 15 | 250/690 | 36.2 | - | - | baseline | new |
| `html/semantics/embedded-content/media-elements` | 323 | 19 | 367/830 | 44.2 | - | - | baseline | new |
| `media-source` | 85 | 0 | 75/363 | 20.7 | - | - | baseline | new |
| `test262/built-ins` | 18373 | 5450 | 15837/18373 | 86.2 | 80 | 90 | done | +7.4 |
| `test262/language` | 23210 | 516 | 20047/23210 | 86.4 | 80 | 90 | done | +1.7 |
| `test262/annexB` | 1044 | 42 | 860/1044 | 82.4 | 80 | 90 | done | +0.4 |
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
| `WebGL 1: rendering` | 43 | 1 | 1088/1091 | 99.7 | - | - | baseline | +0.0 |
| `WebGL 1: state` | 6 | 3 | 688/688 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 1: textures` | 157 | 7 | 42279/43803 | 96.5 | - | - | baseline | +0.0 |
| `WebGL 1: typedarrays` | 7 | 0 | 1414/1414 | 100.0 | - | - | baseline | +1.4 |
| `WebGL 1: uniforms` | 30 | 0 | 944/960 | 98.3 | - | - | baseline | +0.0 |
| `WebGL 2: attribs` | 8 | 0 | 3168/3618 | 87.6 | - | - | baseline | +1.8 |
| `WebGL 2: buffers` | 13 | 0 | 1857/1868 | 99.4 | - | - | baseline | +0.0 |
| `WebGL 2: canvas` | 2 | 0 | 7/8 | 87.5 | - | - | baseline | +0.0 |
| `WebGL 2: context` | 9 | 0 | 146/163 | 89.6 | - | - | baseline | +0.0 |
| `WebGL 2: extensions` | 30 | 0 | 747/756 | 98.8 | - | - | baseline | +0.0 |
| `WebGL 2: glsl3` | 60 | 0 | 961/963 | 99.8 | - | - | baseline | +0.0 |
| `WebGL 2: misc` | 8 | 0 | 825/847 | 97.4 | - | - | baseline | +0.0 |
| `WebGL 2: offscreencanvas` | 8 | 0 | 14/15 | 93.3 | - | - | baseline | +0.0 |
| `WebGL 2: programs` | 4 | 0 | 152/152 | 100.0 | - | - | baseline | +0.0 |
| `WebGL 2: query` | 3 | 0 | 579/579 | 100.0 | - | - | baseline | +1.6 |
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

- `content-security-policy`: 81. 90% of the 3716 of 4127 subtests that need none of: `report-sha256` hash reporting (Chromium only); tentative tests; the `csp` attribute of iframes (Chromium only).
- `mixed-content`: 89. 90% of the 2272 of 2281 subtests that need none of: tentative tests.
- `x-frame-options`: 90. 90% of the 157 of 157 subtests that need none of: tentative tests.
- `html/browsers/sandboxing`: 64. 90% of the 23 of 32 subtests that need none of: tentative tests; the `csp` attribute of iframes (Chromium only).
- `html/browsers/origin`: 40. 90% of the 302 of 676 subtests that need none of: `Origin-Agent-Cluster` (an opt-in finer than the site isolation of Q2; not in Safari); tentative tests.
- `css/css-grid`: 69. 90% of the 13617 of 17578 subtests that need none of: grid-lanes (masonry; in no stable browser); tentative tests.
- `css/css-flexbox`: 87. 90% of the 5686 of 5816 subtests that need none of: tentative tests.
- `css/box-sizing`: 76. 90% of the 7272 of 8549 subtests that need none of: `margin-trim` (Safari only); tentative tests; the `stretch` sizing keyword (Chromium only).
- `css/css-fonts`: 89. 90% of the 8355 of 8390 subtests that need none of: MathML font scaling; tentative tests.
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
