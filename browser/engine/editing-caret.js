// spec: https://drafts.csswg.org/cssom-view/#dom-document-caretpositionfrompoint
// chromium: third_party/blink/renderer/core/editing/frame_caret.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const edit = G.__hostEdit;
let goal = null;

function rectOf(node, offset) {
  if (node.nodeType === 3) return ak.textCaret(node, offset);

  const before = node.childNodes[offset - 1], after = node.childNodes[offset];
  if (after && after.nodeType === 3) return ak.textCaret(after, 0);
  if (before && before.nodeType === 3) return ak.textCaret(before, before.data.length);

  const el = after && after.nodeType === 1 ? after : before && before.nodeType === 1 ? before : node;
  const r = el.getBoundingClientRect();
  const size = parseFloat(getComputedStyle(node).fontSize) || 16;

  return [(el === before && el !== after ? r.right : r.left) + scrollX, r.top + scrollY, el === node || !r.height ? size * 1.2 : r.height];
}

function showCaret() {
  const sel = getSelection(), host = document.activeElement;
  const shown = sel.rangeCount > 0 && host && host.isContentEditable && host.contains(sel.focusNode) ? rectOf(sel.focusNode, sel.focusOffset) : null;

  shown ? ak.hostCaret(shown[0], shown[1], shown[2]) : ak.hostCaret();
}

function caretAt(x, y) {
  const at = ak.caretAt(+x + scrollX, +y + scrollY);

  return at ? { offsetNode: at[0], offset: at[1] } : null;
}

function texts(host) {
  const walker = document.createTreeWalker(host, 4), out = [];
  for (let n = walker.nextNode(); n; n = walker.nextNode()) if (n.data.length > 0) out.push(n);

  return out;
}

// chromium: third_party/blink/renderer/core/editing/selection_modifier.cc (LineDirectionPointForBlockDirectionNavigation)
function lineBeside(host, node, offset, down) {
  const at = rectOf(node, offset);
  if (!at) return null;
  if (goal === null) goal = at[0];

  for (let k = 0; k < 6; k++) {
    const to = ak.caretAt(goal, down ? at[1] + at[2] * (1.25 + k / 2) : at[1] - at[2] * (0.25 + k / 2));
    const line = to && host.contains(to[0]) ? rectOf(to[0], to[1]) : null;
    if (line && line[1] !== at[1]) return to;
  }

  return null;
}

function moved(host, how) {
  const sel = getSelection(), all = texts(host);
  if (all.length === 0 || !sel.rangeCount || !host.contains(sel.focusNode)) return;

  let node = sel.focusNode, offset = sel.focusOffset;
  if (node.nodeType !== 3) [node, offset] = node.childNodes[offset] && node.childNodes[offset].nodeType === 3 ? [node.childNodes[offset], 0] : [all[0], 0];

  const i = all.indexOf(node), last = all[all.length - 1];
  let to = null;
  if (how.endsWith('up')) to = lineBeside(host, node, offset, false) || [all[0], 0];
  else if (how.endsWith('down')) to = lineBeside(host, node, offset, true) || [last, last.data.length];
  else if (how.endsWith('home')) to = [all[0], 0];
  else if (how.endsWith('end')) to = [last, last.data.length];
  else if (how.endsWith('left')) to = offset > 0 ? [node, offset - 1] : i > 0 ? [all[i - 1], all[i - 1].data.length] : null;
  else to = offset < node.data.length ? [node, offset + 1] : i + 1 < all.length ? [all[i + 1], 0] : null;

  if (to) how.startsWith('extend') ? sel.extend(to[0], to[1]) : sel.collapse(to[0], to[1]);
}

hidden(G, '__hostEdit', function (host, line) {
  const command = line.slice(0, line.indexOf('\n'));
  if (!/(up|down)$/.test(command)) goal = null;
  command.startsWith('move') || command.startsWith('extend') ? moved(host, command) : edit(host, line);
  showCaret();
});

document.addEventListener('mousedown', e => {
  const host = e.target && e.target.nodeType === 1 && e.target.isContentEditable ? e.target : null;
  const at = host ? caretAt(e.clientX, e.clientY) : null;
  if (at && host.contains(at.offsetNode)) getSelection().collapse(at.offsetNode, at.offset);
  goal = null;
}, true);
document.addEventListener('focusin', showCaret, true);
document.addEventListener('focusout', () => ak.hostCaret(), true);
document.addEventListener('mouseup', showCaret, true);

hidden(Document.prototype, 'caretPositionFromPoint', function caretPositionFromPoint(x, y) {
  const at = caretAt(x, y);

  return at && Object.assign(at, { getClientRect() { const r = rectOf(at.offsetNode, at.offset); return r && new DOMRect(r[0] - scrollX, r[1] - scrollY, 0, r[2]); } });
});
hidden(Document.prototype, 'caretRangeFromPoint', function caretRangeFromPoint(x, y) {
  const at = caretAt(x, y), range = document.createRange();
  if (!at) return null;

  range.setStart(at.offsetNode, at.offset);

  return range;
});
})();
