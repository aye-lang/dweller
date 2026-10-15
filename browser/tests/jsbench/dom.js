// @dom
// DOM microbench, run inside a page by `dweller dump --js` (the runner wraps
// this file in an HTML page): create/append/remove nodes, setAttribute and
// classList, querySelectorAll, textContent, style writes then a forced
// layout read. SMOKE divides the size.
var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
var N = Math.floor(10000 / scale);
var root = document.createElement('div');
root.id = 'root';
document.body.appendChild(root);
var sum = 0;

// create / append / setAttribute / classList
for (var i = 0; i < N; i++) {
  var el = document.createElement(i % 3 === 0 ? 'span' : 'div');
  el.setAttribute('data-i', String(i));
  el.setAttribute('title', 't' + (i & 31));
  el.classList.add('c' + (i & 7));
  if (i & 1) el.classList.add('odd');
  el.textContent = 'n' + i;
  root.appendChild(el);
}
sum += root.childNodes.length;

// querySelectorAll
var Q = Math.floor(500 / scale);
for (var q = 0; q < Q; q++) {
  sum += root.querySelectorAll('.c' + (q & 7)).length > 0 ? 1 : 0;
  sum += document.querySelectorAll('[data-i="' + (q * 7 % N) + '"]').length;
}
sum += document.querySelectorAll('div.odd').length;

// textContent reads and writes
var kids = root.children;
for (var i = 0; i < N; i += 2) {
  kids[i].textContent = 'x' + i;
  sum += kids[i].textContent.length;
}

// style writes, then a forced layout read
var R = Math.floor(10 / scale);
for (var r = 0; r < R; r++) {
  for (var i = r % 50; i < N; i += 50) {
    kids[i].style.height = (10 + ((i + r) & 15)) + 'px';
    kids[i].style.width = (20 + (i & 31)) + 'px';
  }
  sum += root.offsetHeight > 0 ? 1 : 0;
}

// remove
for (var i = N - 1; i >= 0; i -= 2) root.removeChild(kids[i]);
sum += root.childNodes.length;
console.log('dom ' + sum);
