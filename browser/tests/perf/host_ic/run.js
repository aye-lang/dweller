function build() {
  const root = document.createElement('section');

  for (let i = 0; i < 400; i++) {
    const a = document.createElement('div');
    a.className = i % 2 ? 'a' : 'b';

    for (let j = 0; j < 4; j++) {
      const s = document.createElement(j == 3 ? 'p' : 'span');
      if (j % 2) s.setAttribute('data-x', String(i * 4 + j));
      s.textContent = 'x';
      a.appendChild(s);
    }

    root.appendChild(a);
  }

  document.body.appendChild(root);
}

function selectors() {
  let n = 0;

  for (let k = 0; k < 4; k++) {
    const spans = $('div.a > span[data-x]');
    n += spans.length;
    n += $('div.a > span[data-x]:odd').length;
    n += $('section').find('span[data-x]:not(:first-child), p:contains(x)').length;
    spans.each(function () {
      n += Number($(this).attr('data-x')) & 1;
      n += $(this).closest('div.a').length;
      n += $(this).is('span:eq(0)') ? 1 : 0;
    });
  }

  return n;
}

function matches(el) {
  let n = 0;

  for (let e = el; e && e.nodeType === 1; e = e.parentNode) {
    if (e.ownerDocument !== document) return -1;
    if (e.nodeName.toLowerCase() === 'div' && (' ' + e.className + ' ').indexOf(' a ') >= 0) n++;
    if (e.getAttribute('data-x') !== null) n++;
  }

  return n;
}

function walk() {
  const all = document.getElementsByTagName('*');
  const list = [];
  let n = 0;

  for (let i = 0; i < all.length; i++) list.push(all[i]);
  for (let k = 0; k < 20; k++) for (const el of list) n += matches(el);

  return n;
}

function hash(s) {
  let h = 0;

  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;

  return h;
}

function strings() {
  let h = 0;
  const words = [];

  for (let i = 0; i < 2000; i++) words.push('word-' + i + '-' + (i * 7919).toString(36));
  for (let k = 0; k < 40; k++) for (const w of words) h = (h + hash(w)) | 0;

  return h;
}

build();
const t0 = performance.now();
const n = selectors();
const t1 = performance.now();
const w = walk();
const t2 = performance.now();
const h = strings();
const t3 = performance.now();
console.log('host_ic selectors ' + n + ' ' + (t1 - t0).toFixed(1) + ' ms; walk ' + w + ' ' + (t2 - t1).toFixed(1) + ' ms; strings ' + h + ' ' + (t3 - t2).toFixed(1) + ' ms');
