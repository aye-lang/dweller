# Web Audio test pages

- `core.html`, `core.txt`: P11.19 (`webaudio_core`). The expected samples are
  computed in `tests/browser.rs`.
- `effects.html`, `effects.ref`, `effects.txt`: P11.20 (`webaudio_effects`).
  `effects.ref` is the page's `ref NAME v,v,..` lines (without the `ref `)
  as Chrome's `OfflineAudioContext` rendered them; the test compares
  Dweller's lines with it within 1e-3 (NaN must be NaN). `effects.txt` is the
  other lines, also from Chrome.

To make them again, open the page in Chrome and take the text of `#out`:

    agent-browser --session NAME open file://$PWD/browser/tests/audio/effects.html
    agent-browser --session NAME get text "#out" > chrome.txt
    grep '^ref ' chrome.txt | sed 's/^ref //' > browser/tests/audio/effects.ref
    grep -v '^ref ' chrome.txt > browser/tests/audio/effects.txt

Chrome differs from a plain convolution in one place: with a stereo input and
a mono impulse response, its right channel is off by up to 1.3e-3 between
frames 384 and 630 (its staggered FFT stages). The page therefore keeps only
the left channel of that case as a reference and checks the right channel
against the direct sum; Chrome prints `false` on the line `convolver
stereo.mono right is the direct convolution`, `effects.txt` has `true`.
