// Runs 1.5 s after the page was shown (early_paint.html).
document.getElementById('log').textContent =
  'script ran; checked before it: ' + document.getElementById('c').checked +
  ', scrolled before it: ' + (scrollY > 0) +
  ', color: ' + getComputedStyle(document.getElementById('t')).color;
