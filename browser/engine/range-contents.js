// spec: https://dom.spec.whatwg.org/#dom-range-deletecontents
// chromium: third_party/blink/renderer/core/dom/range.cc (ProcessContents)
(function () {
const G = globalThis;
const { lengthOf, indexOf, holds, fail } = G.__ranges;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const isData = n => n.nodeType === 3 || n.nodeType === 4 || n.nodeType === 7 || n.nodeType === 8;
const DELETE = 0, EXTRACT = 1, CLONE = 2;

function between(start, so, end, eo) {
  const r = new Range();
  [r._sc, r._so, r._ec, r._eo] = [start, so, end, eo];

  return r;
}

function sliced(node, from, to, how, fragment) {
  if (how !== DELETE) {
    const copy = node.cloneNode(false);
    copy.data = node.data.slice(from, to);
    fragment.appendChild(copy);
  }

  if (how !== CLONE) node.replaceData(from, to - from, '');
}

function partial(range, child, atStart, how, fragment) {
  if (isData(child)) return atStart ? sliced(child, range._so, lengthOf(child), how, fragment) : sliced(child, 0, range._eo, how, fragment);

  const sub = atStart ? between(range._sc, range._so, child, lengthOf(child)) : between(child, 0, range._ec, range._eo);
  const inner = processed(sub, how);
  if (how === DELETE) return;

  const copy = child.cloneNode(false);
  fragment.appendChild(copy);
  copy.appendChild(inner);
}

function afterwards(range) {
  if (holds(range._sc, range._ec)) return [range._sc, range._so];

  let ref = range._sc;
  while (ref.parentNode && !holds(ref.parentNode, range._ec)) ref = ref.parentNode;

  return [ref.parentNode, indexOf(ref) + 1];
}

function processed(range, how) {
  const fragment = document.createDocumentFragment();
  if (range.collapsed) return fragment;

  if (range._sc === range._ec && isData(range._sc)) {
    sliced(range._sc, range._so, range._eo, how, fragment);

    return fragment;
  }

  const common = range.commonAncestorContainer;
  const kids = Array.from(common.childNodes);
  const first = holds(range._sc, range._ec) ? null : kids.find(c => holds(c, range._sc)) || null;
  const last = holds(range._ec, range._sc) ? null : kids.find(c => holds(c, range._ec)) || null;
  const whole = kids.filter(c => range._contains(c));
  if (how !== DELETE && whole.some(c => c.nodeType === 10)) throw fail(how === CLONE ? 'cloneContents' : 'extractContents', 'A DocumentType is in the range.', 'HierarchyRequestError');

  const [node, offset] = afterwards(range);
  if (first) partial(range, first, true, how, fragment);

  for (const c of whole) {
    if (how === CLONE) fragment.appendChild(c.cloneNode(true));
    else if (how === EXTRACT) fragment.appendChild(c);
    else c.parentNode.removeChild(c);
  }

  if (last) partial(range, last, false, how, fragment);
  if (how !== CLONE) [range._sc, range._so, range._ec, range._eo] = [node, offset, node, offset];

  return fragment;
}

if (!Text.prototype.splitText) {
  hidden(Text.prototype, 'splitText', function splitText(offset) {
    const o = offset >>> 0;
    if (o > this.data.length) throw new DOMException("Failed to execute 'splitText' on 'Text': The offset " + o + " is larger than the Text node's length.", 'IndexSizeError');

    const rest = document.createTextNode(this.data.slice(o));
    this.deleteData(o, this.data.length - o);
    if (this.parentNode) this.parentNode.insertBefore(rest, this.nextSibling);

    return rest;
  });
}

hidden(Range.prototype, 'deleteContents', function deleteContents() { processed(this, DELETE); });
hidden(Range.prototype, 'extractContents', function extractContents() { return processed(this, EXTRACT); });
hidden(Range.prototype, 'cloneContents', function cloneContents() { return processed(this, CLONE); });

hidden(Range.prototype, 'insertNode', function insertNode(node) {
  const sc = this._sc;
  if (sc.nodeType === 7 || sc.nodeType === 8 || (sc.nodeType === 3 && !sc.parentNode) || sc === node) throw fail('insertNode', 'Nodes of this type cannot be inserted here.', 'HierarchyRequestError');

  let ref = sc.nodeType === 3 ? sc : sc.childNodes[this._so] || null;
  const parent = ref ? ref.parentNode : sc;
  if (sc.nodeType === 3) ref = sc.splitText(this._so);
  if (node === ref) ref = ref.nextSibling;
  if (node.parentNode) node.parentNode.removeChild(node);

  const offset = (ref ? indexOf(ref) : lengthOf(parent)) + (node.nodeType === 11 ? lengthOf(node) : 1);
  parent.insertBefore(node, ref);
  if (this.collapsed) [this._ec, this._eo] = [parent, offset];
});

hidden(Range.prototype, 'surroundContents', function surroundContents(parent) {
  const partly = n => n.nodeType !== 3 && holds(n, this._sc) !== holds(n, this._ec);
  for (let n = this._sc; n && n !== this.commonAncestorContainer; n = n.parentNode) if (partly(n)) throw fail('surroundContents', 'The Range has partially selected a non-Text node.', 'InvalidStateError');
  for (let n = this._ec; n && n !== this.commonAncestorContainer; n = n.parentNode) if (partly(n)) throw fail('surroundContents', 'The Range has partially selected a non-Text node.', 'InvalidStateError');
  if (parent.nodeType === 9 || parent.nodeType === 10 || parent.nodeType === 11) throw fail('surroundContents', 'The node provided cannot contain the contents.', 'InvalidNodeTypeError');

  const fragment = this.extractContents();
  while (parent.firstChild) parent.removeChild(parent.firstChild);

  this.insertNode(parent);
  parent.appendChild(fragment);
  this.selectNode(parent);
});
})();
