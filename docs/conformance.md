# Conformance

Measured 2026-11-09 at Dweller 4f8516d. WPT `e58225abb9`, test262 `d53ebc8b19`, WebGL `193020cd37`.
Score is passed over total subtests (WPT, WebGL) or tests (test262), skipped files excluded.
Change is in points since the previous table. Regenerate with `conformance.py run`, then `table`.

| Area | Files | Skipped | Passed | Score % | Threshold % | Status | Change |
|---|---:|---:|---:|---:|---:|---|---:|
| `dom/nodes` | 411 | 6 | 7252/9403 | 77.1 | 75 | done | +0.0 |
| `dom/events` | 180 | 8 | 669/877 | 76.3 | 70 | done | +0.0 |
| `uievents` | 69 | 6 | 153/201 | 76.1 | 70 | done | +0.0 |
| `html/semantics/forms` | 685 | 173 | 2932/5398 | 54.3 | 60 | below | +0.0 |
| `css/cssom` | 292 | 11 | 2134/3818 | 55.9 | - | baseline | +0.0 |
| `fetch/api` | 191 | 0 | 1836/2483 | 73.9 | 50 | done | +0.0 |
| `xhr` | 329 | 3 | 687/1233 | 55.7 | 50 | done | +0.0 |
| `url` | 34 | 0 | 6672/9998 | 66.7 | 80 | below | +0.0 |
| `encoding` | 157 | 5 | 1138861/1139871 | 99.9 | 80 | done | +0.0 |
| `webaudio` | 247 | 63 | 1514/2385 | 63.5 | - | baseline | +0.0 |
| `css/css-flexbox` | 1422 | 469 | 3653/5816 | 62.8 | 70 | below | +0.0 |
| `css/css-transforms` | 711 | 156 | 2840/5428 | 52.3 | 50 | done | +2.2 |
| `css/css-transitions` | 143 | 59 | 1473/3348 | 44.0 | - | baseline | +0.0 |
| `css/css-animations` | 206 | 91 | 712/1398 | 50.9 | 60 | below | +0.0 |
| `html/syntax` | 98 | 0 | 2560/2732 | 93.7 | 90 | done | +0.0 |
| `dom/api` | 467 | 109 | 61380/64996 | 94.4 | 75 | done | +0.0 |
| `html/webappapis/timers` | 16 | 0 | 20/25 | 80.0 | 70 | done | +0.0 |
| `css/selectors` | 540 | 136 | 5104/6283 | 81.2 | 80 | done | +0.0 |
| `css/cascade-variables` | 398 | 31 | 805/1436 | 56.1 | 75 | below | +0.0 |
| `css/values-color` | 865 | 237 | 13543/23406 | 57.9 | 70 | below | +0.0 |
| `css/box-sizing` | 1591 | 259 | 4806/8549 | 56.2 | 70 | below | +4.8 |
| `css/floats-tables` | 709 | 251 | 888/1601 | 55.5 | 65 | below | +0.0 |
| `css/inline-text` | 2343 | 407 | 4897/8083 | 60.6 | 60 | done | +0.0 |
| `css/overflow` | 939 | 189 | 1044/1921 | 54.3 | 60 | below | +0.0 |
| `css/css-grid` | 2508 | 1085 | 7585/17578 | 43.2 | 60 | below | +0.0 |
| `css/positioning` | 915 | 239 | 1425/2310 | 61.7 | 70 | below | +0.0 |
| `css/backgrounds-borders` | 1041 | 295 | 5518/8473 | 65.1 | 65 | done | +35.0 |
| `css/css-fonts` | 560 | 325 | 2736/8356 | 32.7 | 50 | below | +0.0 |
| `images` | 715 | 268 | 4621/5963 | 77.5 | 60 | done | +0.8 |
| `intersection-observer` | 144 | 10 | 178/355 | 50.1 | - | baseline | +0.0 |
| `mixed-content` | 388 | 0 | 1216/2281 | 53.3 | 50 | done | +0.0 |
| `upgrade-insecure-requests` | 197 | 0 | 281/1000 | 28.1 | 25 | done | +0.0 |
| `html/browsers/sandboxing` | 21 | 1 | 11/32 | 34.4 | 30 | done | +0.0 |
| `fetch/metadata` | 87 | 0 | 541/1828 | 29.6 | 25 | done | +0.0 |
| `referrer-policy` | 1392 | 0 | 4166/8465 | 49.2 | 45 | done | +0.0 |
| `content-security-policy/script-src` | 97 | 0 | 127/217 | 58.5 | 55 | done | +0.0 |
| `content-security-policy/frame-ancestors` | 34 | 0 | 32/36 | 88.9 | 85 | done | +0.0 |
| `x-frame-options` | 6 | 0 | 61/157 | 38.9 | 35 | done | +0.0 |
| `fetch/nosniff` | 6 | 0 | 65/65 | 100.0 | 95 | done | +0.0 |
| `html/cross-origin-opener-policy` | 116 | 0 | 250/615 | 40.7 | 40 | done | +0.0 |
| `content-security-policy/style-src` | 42 | 0 | 80/87 | 92.0 | 80 | done | +0.0 |
| `content-security-policy/img-src` | 15 | 0 | 15/20 | 75.0 | 70 | done | +0.0 |
| `content-security-policy/connect-src` | 30 | 0 | 14/38 | 36.8 | 35 | done | +0.0 |
| `content-security-policy/securitypolicyviolation` | 38 | 0 | 30/76 | 39.5 | 35 | done | +0.0 |
| `content-security-policy` | 894 | 1 | 2323/4076 | 57.0 | 55 | done | +0.0 |
| `test262/built-ins` | 18373 | 5450 | 13352/18373 | 72.7 | 80 | below | +0.0 |
| `test262/language` | 23210 | 516 | 17032/23210 | 73.4 | 80 | below | +0.0 |
| `test262/annexB` | 1044 | 42 | 431/1044 | 41.3 | 80 | below | +0.0 |
| `WebGL 1: attribs` | 15 | 1 | 5694/5701 | 99.9 | - | baseline | +0.0 |
| `WebGL 1: buffers` | 13 | 0 | 283/285 | 99.3 | - | baseline | +0.0 |
| `WebGL 1: canvas` | 18 | 0 | 842/844 | 99.8 | - | baseline | +0.0 |
| `WebGL 1: context` | 18 | 3 | 1006/1028 | 97.9 | - | baseline | +0.0 |
| `WebGL 1: extensions` | 24 | 28 | 2710/2722 | 99.6 | - | baseline | +0.0 |
| `WebGL 1: glsl` | 310 | 6 | 7720/7724 | 99.9 | - | baseline | +0.0 |
| `WebGL 1: limits` | 5 | 0 | 31/31 | 100.0 | - | baseline | +0.0 |
| `WebGL 1: misc` | 16 | 1 | 2436/2515 | 96.9 | - | baseline | +0.0 |
| `WebGL 1: offscreencanvas` | 12 | 0 | 18/25 | 72.0 | - | baseline | +0.0 |
| `WebGL 1: ogles` | 141 | 0 | 1900/1900 | 100.0 | - | baseline | +0.0 |
| `WebGL 1: programs` | 12 | 0 | 270/270 | 100.0 | - | baseline | +0.0 |
| `WebGL 1: reading` | 3 | 0 | 368/368 | 100.0 | - | baseline | +0.0 |
| `WebGL 1: renderbuffers` | 5 | 2 | 125/129 | 96.9 | - | baseline | +0.0 |
| `WebGL 1: rendering` | 43 | 1 | 1063/1066 | 99.7 | - | baseline | +0.0 |
| `WebGL 1: state` | 6 | 3 | 688/688 | 100.0 | - | baseline | +0.0 |
| `WebGL 1: textures` | 157 | 7 | 42279/43803 | 96.5 | - | baseline | +0.0 |
| `WebGL 1: typedarrays` | 7 | 0 | 1394/1414 | 98.6 | - | baseline | +0.0 |
| `WebGL 1: uniforms` | 30 | 0 | 944/960 | 98.3 | - | baseline | +0.0 |
| `WebGL 2: attribs` | 8 | 0 | 3104/3618 | 85.8 | - | baseline | +0.0 |
| `WebGL 2: buffers` | 13 | 0 | 1857/1868 | 99.4 | - | baseline | +0.0 |
| `WebGL 2: canvas` | 2 | 0 | 7/8 | 87.5 | - | baseline | +0.0 |
| `WebGL 2: context` | 9 | 0 | 146/163 | 89.6 | - | baseline | +0.0 |
| `WebGL 2: extensions` | 30 | 0 | 747/756 | 98.8 | - | baseline | +0.0 |
| `WebGL 2: glsl3` | 60 | 0 | 961/963 | 99.8 | - | baseline | +0.0 |
| `WebGL 2: misc` | 8 | 0 | 825/847 | 97.4 | - | baseline | +0.0 |
| `WebGL 2: offscreencanvas` | 8 | 0 | 14/15 | 93.3 | - | baseline | +0.0 |
| `WebGL 2: programs` | 4 | 0 | 152/152 | 100.0 | - | baseline | +0.0 |
| `WebGL 2: query` | 3 | 0 | 60/61 | 98.4 | - | baseline | +0.0 |
| `WebGL 2: reading` | 5 | 0 | 519/588 | 88.3 | - | baseline | +0.0 |
| `WebGL 2: renderbuffers` | 10 | 0 | 633/683 | 92.7 | - | baseline | +0.0 |
| `WebGL 2: rendering` | 50 | 0 | 4676/4700 | 99.5 | - | baseline | +0.0 |
| `WebGL 2: samplers` | 3 | 0 | 169/169 | 100.0 | - | baseline | +0.0 |
| `WebGL 2: state` | 4 | 0 | 521/531 | 98.1 | - | baseline | +0.0 |
| `WebGL 2: sync` | 1 | 0 | 13/14 | 92.9 | - | baseline | +0.0 |
| `WebGL 2: textures` | 926 | 0 | 243203/243835 | 99.7 | - | baseline | +0.0 |
| `WebGL 2: transform_feedback` | 10 | 0 | 476/481 | 99.0 | - | baseline | +0.0 |
| `WebGL 2: uniforms` | 9 | 0 | 7138/7138 | 100.0 | - | baseline | +0.0 |
| `WebGL 2: vertex_arrays` | 2 | 0 | 41/49 | 83.7 | - | baseline | +0.0 |
| `WebGL 2: wasm` | 19 | 0 | 52/52 | 100.0 | - | baseline | +0.0 |
