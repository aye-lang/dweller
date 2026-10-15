// net/hover_scripts.html: each step replaces the link, then makes enough
// garbage for a collection, which frees the old link (its wrapper and
// mutation record die), and takes longer than the window's budget.
function step(n) {
  document.getElementById('d').innerHTML = '<a href="link' + n + '.html">Link ' + n + '</a>';
  let junk = null;
  for (let i = 0; i < 60000; i++) junk = { i: i, next: junk && i % 64 ? junk : null };
  const t = Date.now();
  while (Date.now() - t < 60) {}
}
