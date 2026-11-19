// spec: https://w3c.github.io/selection-api/
// chromium: third_party/blink/renderer/core/editing/dom_selection.cc
(function () {
const G = globalThis;
const { lengthOf, rootOf, position } = G.__ranges;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const fail = (method, why, name) => new DOMException("Failed to execute '" + method + "' on 'Selection': " + why, name);

function checked(method, node, offset) {
  const o = offset >>> 0;
  if (o > lengthOf(node)) throw fail(method, 'The offset ' + o + ' is larger than the node\'s length.', 'IndexSizeError');

  return o;
}

let noted = false;

function changed() {
  if (noted) return;

  noted = true;
  queueMicrotask(() => {
    noted = false;
    document.dispatchEvent(new Event('selectionchange'));
    if (G.__selectionShown) G.__selectionShown();
  });
}

class Selection {
  constructor() {
    hidden(this, '_range', null);
    hidden(this, '_backwards', false);
  }

  get rangeCount() { return this._range ? 1 : 0; }
  get isCollapsed() { return !this._range || this._range.collapsed; }
  get type() { return !this._range ? 'None' : this._range.collapsed ? 'Caret' : 'Range'; }
  get direction() { return !this._range || this._range.collapsed ? 'none' : this._backwards ? 'backward' : 'forward'; }
  get anchorNode() { return !this._range ? null : this._backwards ? this._range.endContainer : this._range.startContainer; }
  get anchorOffset() { return !this._range ? 0 : this._backwards ? this._range.endOffset : this._range.startOffset; }
  get focusNode() { return !this._range ? null : this._backwards ? this._range.startContainer : this._range.endContainer; }
  get focusOffset() { return !this._range ? 0 : this._backwards ? this._range.startOffset : this._range.endOffset; }

  getRangeAt(i) {
    if (i >>> 0 !== 0 || !this._range) throw fail('getRangeAt', i + ' is not a valid index.', 'IndexSizeError');

    return this._range;
  }

  addRange(range) {
    if (this._range || rootOf(range.startContainer) !== document) return;

    this._range = range;
    this._backwards = false;
    changed();
  }

  removeRange(range) {
    if (this._range !== range) throw fail('removeRange', 'The given range is not in the selection.', 'NotFoundError');

    this._range = null;
    changed();
  }

  removeAllRanges() {
    this._range = null;
    changed();
  }

  empty() { this.removeAllRanges(); }

  collapse(node, offset) {
    if (node === null || node === undefined) return this.removeAllRanges();
    if (node.nodeType === 10) throw fail('collapse', 'The node provided is a DocumentType.', 'InvalidNodeTypeError');

    const o = checked('collapse', node, offset);
    if (rootOf(node) !== document) return;

    const r = new Range();
    r.setStart(node, o);
    this._range = r;
    this._backwards = false;
    changed();
  }

  setPosition(node, offset) { this.collapse(node, offset); }

  collapseToStart() {
    if (!this._range) throw fail('collapseToStart', 'there is no selection.', 'InvalidStateError');

    this.collapse(this._range.startContainer, this._range.startOffset);
  }

  collapseToEnd() {
    if (!this._range) throw fail('collapseToEnd', 'there is no selection.', 'InvalidStateError');

    this.collapse(this._range.endContainer, this._range.endOffset);
  }

  setBaseAndExtent(anchor, anchorOffset, focus, focusOffset) {
    const a = checked('setBaseAndExtent', anchor, anchorOffset), f = checked('setBaseAndExtent', focus, focusOffset);
    if (rootOf(anchor) !== document || rootOf(focus) !== document) return;

    const r = new Range();
    this._backwards = position(anchor, a, focus, f) > 0;
    this._backwards ? (r.setStart(focus, f), r.setEnd(anchor, a)) : (r.setStart(anchor, a), r.setEnd(focus, f));
    this._range = r;
    changed();
  }

  extend(node, offset) {
    if (rootOf(node) !== document) return;
    if (!this._range) throw fail('extend', 'This Selection object doesn\'t have any Ranges.', 'InvalidStateError');

    this.setBaseAndExtent(this.anchorNode, this.anchorOffset, node, offset === undefined ? 0 : offset);
  }

  selectAllChildren(node) {
    if (node.nodeType === 10) throw fail('selectAllChildren', 'The node provided is a DocumentType.', 'InvalidNodeTypeError');
    if (rootOf(node) !== document) return;

    const r = new Range();
    r.selectNodeContents(node);
    this._range = r;
    this._backwards = false;
    changed();
  }

  deleteFromDocument() {
    if (this._range) this._range.deleteContents();

    changed();
  }

  containsNode(node, partly) {
    if (!this._range || rootOf(node) !== document) return false;

    return partly ? this._range.intersectsNode(node) : this._range._contains(node);
  }

  getComposedRanges() { return this._range ? [new StaticRange(this._range)] : []; }
  modify() {}
  toString() { return this._range ? this._range.toString() : ''; }
}

const selection = new Selection();

hidden(G, 'Selection', Selection);
hidden(G, 'getSelection', function getSelection() { return selection; });
hidden(Document.prototype, 'getSelection', function getSelection() { return selection; });
delete G.__ranges;
})();
