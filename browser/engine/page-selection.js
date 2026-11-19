// spec: https://w3c.github.io/selection-api/#user-interactions
// chromium: third_party/blink/renderer/core/editing/selection_controller.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
let dragging = false;

function rectsOf(range) {
  const out = [], top = range.commonAncestorContainer;
  if (range.collapsed) return out;
  if (top.nodeType === 3) return ak.textRects(top, range.startOffset, range.endOffset);

  const walker = document.createTreeWalker(top, 4);
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    if (!range.intersectsNode(n)) continue;

    out.push(...ak.textRects(n, n === range.startContainer ? range.startOffset : 0, n === range.endContainer ? range.endOffset : n.data.length));
  }

  return out;
}

function boxes(flat) {
  const out = [];
  for (let i = 0; i < flat.length; i += 4) out.push(new DOMRect(flat[i] - scrollX, flat[i + 1] - scrollY, flat[i + 2], flat[i + 3]));

  return out;
}

hidden(G, '__selectionShown', function () {
  const sel = getSelection();
  ak.pageSelection(...(sel.rangeCount > 0 ? rectsOf(sel.getRangeAt(0)) : []));
});

hidden(Range.prototype, 'getClientRects', function getClientRects() { return boxes(rectsOf(this)); });
hidden(Range.prototype, 'getBoundingClientRect', function getBoundingClientRect() {
  const all = boxes(rectsOf(this));
  if (all.length === 0) return new DOMRect(0, 0, 0, 0);

  const left = Math.min(...all.map(r => r.left)), top = Math.min(...all.map(r => r.top));

  return new DOMRect(left, top, Math.max(...all.map(r => r.right)) - left, Math.max(...all.map(r => r.bottom)) - top);
});

function selectable(target) {
  return !!target && target.nodeType === 1 && !target.closest('input,textarea,select,button');
}

function pointed(e, extend) {
  const at = ak.caretAt(e.clientX + scrollX, e.clientY + scrollY), sel = getSelection();
  if (!at) return;

  extend && sel.rangeCount > 0 ? sel.extend(at[0], at[1]) : sel.collapse(at[0], at[1]);
}

document.addEventListener('mousedown', e => {
  dragging = e.button === 0 && selectable(e.target);
  if (dragging) pointed(e, e.shiftKey);
}, true);
document.addEventListener('mousemove', e => {
  if (dragging && (e.buttons & 1)) pointed(e, true);
}, true);
document.addEventListener('mouseup', () => { dragging = false; }, true);
})();
