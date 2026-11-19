// spec: https://w3c.github.io/editing/docs/execCommand/#the-undo-command
// chromium: third_party/blink/renderer/core/editing/commands/undo_stack.cc
(function () {
const G = globalThis;
const LIMIT = 100;
const stacks = new WeakMap();

function stacksOf(host) {
  if (!stacks.has(host)) stacks.set(host, { undo: [], redo: [] });

  return stacks.get(host);
}

function shot(host) {
  const sel = getSelection(), path = [];
  if (!sel.rangeCount || !host.contains(sel.focusNode)) return { html: host.innerHTML, path: null, offset: 0 };

  for (let n = sel.focusNode; n !== host; n = n.parentNode) path.unshift(Array.prototype.indexOf.call(n.parentNode.childNodes, n));

  return { html: host.innerHTML, path, offset: sel.focusOffset };
}

function restore(host, from) {
  host.innerHTML = from.html;

  let node = host;
  for (const i of from.path || []) node = node.childNodes[i] || node;

  const length = node.nodeType === 3 ? node.data.length : node.childNodes.length;
  getSelection().collapse(node, from.path ? Math.min(from.offset, length) : length);
}

Object.defineProperty(G, '__editHistory', { configurable: true, value: {
  record(host) {
    const s = stacksOf(host);
    s.undo.push(shot(host));
    if (s.undo.length > LIMIT) s.undo.shift();

    s.redo.length = 0;
  },

  has(host, redo) {
    return !!host && stacksOf(host)[redo ? 'redo' : 'undo'].length > 0;
  },

  step(host, redo) {
    const s = stacksOf(host);
    (redo ? s.undo : s.redo).push(shot(host));
    restore(host, (redo ? s.redo : s.undo).pop());
  },
} });
})();
