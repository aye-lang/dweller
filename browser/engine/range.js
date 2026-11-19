// spec: https://dom.spec.whatwg.org/#ranges
// chromium: third_party/blink/renderer/core/dom/range.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const fail = (method, why, name) => new DOMException("Failed to execute '" + method + "' on 'Range': " + why, name);

function lengthOf(n) {
  if (n.nodeType === 10) return 0;
  return n.nodeType === 3 || n.nodeType === 4 || n.nodeType === 7 || n.nodeType === 8 ? n.data.length : n.childNodes.length;
}

function indexOf(n) {
  let i = 0;
  for (let s = n.previousSibling; s; s = s.previousSibling) i++;

  return i;
}

function rootOf(n) {
  while (n.parentNode) n = n.parentNode;

  return n;
}

function holds(a, b) {
  for (let n = b; n; n = n.parentNode) if (n === a) return true;

  return false;
}

function position(na, oa, nb, ob) {
  if (na === nb) return oa === ob ? 0 : oa < ob ? -1 : 1;
  if (nb.compareDocumentPosition(na) & 4) return -position(nb, ob, na, oa) || 0;
  if (!holds(na, nb)) return -1;

  let child = nb;
  while (child.parentNode !== na) child = child.parentNode;

  return indexOf(child) < oa ? 1 : -1;
}

function offsetIn(method, node, offset) {
  if (node.nodeType === 10) throw fail(method, 'The node provided is a DocumentType.', 'InvalidNodeTypeError');

  const o = offset >>> 0;
  if (o > lengthOf(node)) throw fail(method, 'The offset ' + o + ' is larger than the node\'s length (' + lengthOf(node) + ').', 'IndexSizeError');

  return o;
}

class AbstractRange {
  constructor() {
    if (new.target === AbstractRange) throw new TypeError('Illegal constructor');
  }

  get startContainer() { return this._sc; }
  get startOffset() { return this._so; }
  get endContainer() { return this._ec; }
  get endOffset() { return this._eo; }
  get collapsed() { return this._sc === this._ec && this._so === this._eo; }
}

class Range extends AbstractRange {
  constructor() {
    super();
    hidden(this, '_sc', document);
    hidden(this, '_so', 0);
    hidden(this, '_ec', document);
    hidden(this, '_eo', 0);
  }

  get commonAncestorContainer() {
    let c = this._sc;
    while (!holds(c, this._ec)) c = c.parentNode;

    return c;
  }

  setStart(node, offset) {
    const o = offsetIn('setStart', node, offset);
    if (rootOf(node) !== rootOf(this._sc) || position(node, o, this._ec, this._eo) > 0) [this._ec, this._eo] = [node, o];

    [this._sc, this._so] = [node, o];
  }

  setEnd(node, offset) {
    const o = offsetIn('setEnd', node, offset);
    if (rootOf(node) !== rootOf(this._sc) || position(node, o, this._sc, this._so) < 0) [this._sc, this._so] = [node, o];

    [this._ec, this._eo] = [node, o];
  }

  _beside(method, node, after, start) {
    if (!node.parentNode) throw fail(method, 'the given Node has no parent.', 'InvalidNodeTypeError');

    start ? this.setStart(node.parentNode, indexOf(node) + after) : this.setEnd(node.parentNode, indexOf(node) + after);
  }

  setStartBefore(node) { this._beside('setStartBefore', node, 0, true); }
  setStartAfter(node) { this._beside('setStartAfter', node, 1, true); }
  setEndBefore(node) { this._beside('setEndBefore', node, 0, false); }
  setEndAfter(node) { this._beside('setEndAfter', node, 1, false); }

  collapse(toStart) {
    if (toStart) [this._ec, this._eo] = [this._sc, this._so];
    else [this._sc, this._so] = [this._ec, this._eo];
  }

  selectNode(node) {
    if (!node.parentNode) throw fail('selectNode', 'the given Node has no parent.', 'InvalidNodeTypeError');

    const i = indexOf(node);
    [this._sc, this._so, this._ec, this._eo] = [node.parentNode, i, node.parentNode, i + 1];
  }

  selectNodeContents(node) {
    if (node.nodeType === 10) throw fail('selectNodeContents', 'The node provided is a DocumentType.', 'InvalidNodeTypeError');

    [this._sc, this._so, this._ec, this._eo] = [node, 0, node, lengthOf(node)];
  }

  compareBoundaryPoints(how, source) {
    const h = how & 0xffff;
    if (h > 3) throw fail('compareBoundaryPoints', 'The comparison method provided must be one of START_TO_START, START_TO_END, END_TO_END or END_TO_START.', 'NotSupportedError');
    if (rootOf(this._sc) !== rootOf(source._sc)) throw fail('compareBoundaryPoints', 'The source range is in a different document than this range.', 'WrongDocumentError');

    const mine = h === 0 || h === 3 ? [this._sc, this._so] : [this._ec, this._eo];
    const theirs = h === 0 || h === 1 ? [source._sc, source._so] : [source._ec, source._eo];

    return position(mine[0], mine[1], theirs[0], theirs[1]);
  }

  comparePoint(node, offset) {
    if (rootOf(node) !== rootOf(this._sc)) throw fail('comparePoint', 'The node provided is not in this Range\'s Document.', 'WrongDocumentError');

    const o = offsetIn('comparePoint', node, offset);
    if (position(node, o, this._sc, this._so) < 0) return -1;

    return position(node, o, this._ec, this._eo) > 0 ? 1 : 0;
  }

  isPointInRange(node, offset) {
    if (rootOf(node) !== rootOf(this._sc)) return false;

    const o = offsetIn('isPointInRange', node, offset);

    return position(node, o, this._sc, this._so) >= 0 && position(node, o, this._ec, this._eo) <= 0;
  }

  intersectsNode(node) {
    if (rootOf(node) !== rootOf(this._sc)) return false;
    if (!node.parentNode) return true;

    const i = indexOf(node);

    return position(node.parentNode, i, this._ec, this._eo) < 0 && position(node.parentNode, i + 1, this._sc, this._so) > 0;
  }

  cloneRange() {
    const r = new Range();
    [r._sc, r._so, r._ec, r._eo] = [this._sc, this._so, this._ec, this._eo];

    return r;
  }

  detach() {}

  _contains(node) {
    return position(node, 0, this._sc, this._so) > 0 && position(node, lengthOf(node), this._ec, this._eo) < 0;
  }

  toString() {
    const isText = n => n.nodeType === 3 || n.nodeType === 4;
    if (this._sc === this._ec && isText(this._sc)) return this._sc.data.slice(this._so, this._eo);

    let s = isText(this._sc) ? this._sc.data.slice(this._so) : '';
    const walker = document.createTreeWalker(this.commonAncestorContainer, 4);
    for (let n = walker.nextNode(); n; n = walker.nextNode()) if (this._contains(n)) s += n.data;

    return isText(this._ec) ? s + this._ec.data.slice(0, this._eo) : s;
  }

  createContextualFragment(html) {
    const ctx = this._sc.nodeType === 1 ? this._sc.localName : 'body';
    const box = document.createElement(ctx === 'html' ? 'body' : ctx);
    box.innerHTML = html;

    const f = document.createDocumentFragment();
    while (box.firstChild) f.appendChild(box.firstChild);

    ak.unstartScripts(f);

    return f;
  }

  getBoundingClientRect() { return new DOMRect(0, 0, 0, 0); }
  getClientRects() { return []; }
}

class StaticRange extends AbstractRange {
  constructor(init) {
    super();
    if (init === null || typeof init !== 'object') throw new TypeError("Failed to construct 'StaticRange': 1 argument required.");

    for (const k of ['startContainer', 'startOffset', 'endContainer', 'endOffset']) if (init[k] === undefined) throw new TypeError("Failed to construct 'StaticRange': Failed to read the '" + k + "' property from 'StaticRangeInit': Required member is undefined.");
    for (const n of [init.startContainer, init.endContainer]) if (n.nodeType === 10 || n.nodeType === 2) throw new DOMException("Failed to construct 'StaticRange': Neither startContainer nor endContainer can be a DocumentType or Attribute node.", 'InvalidNodeTypeError');

    hidden(this, '_sc', init.startContainer);
    hidden(this, '_so', init.startOffset >>> 0);
    hidden(this, '_ec', init.endContainer);
    hidden(this, '_eo', init.endOffset >>> 0);
  }
}

['START_TO_START', 'START_TO_END', 'END_TO_END', 'END_TO_START'].forEach((name, value) => {
  Object.defineProperty(Range, name, { value, enumerable: true });
  Object.defineProperty(Range.prototype, name, { value, enumerable: true });
});

hidden(G, 'AbstractRange', AbstractRange);
hidden(G, 'Range', Range);
hidden(G, 'StaticRange', StaticRange);
hidden(G, '__ranges', { lengthOf, indexOf, rootOf, holds, position, fail });
hidden(Document.prototype, 'createRange', function () { return new Range(); });
})();
