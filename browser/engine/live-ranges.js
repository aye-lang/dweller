// spec: https://dom.spec.whatwg.org/#concept-live-range
// chromium: third_party/blink/renderer/core/dom/range.cc (NodeChildrenChanged, NodeWillBeRemoved, DidModifyData)
(function () {
const G = globalThis;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const { indexOf, holds, live } = G.__ranges;
const ENDS = [['_sc', '_so'], ['_ec', '_eo']];
const isData = n => n.nodeType === 3 || n.nodeType === 4 || n.nodeType === 7 || n.nodeType === 8;

function each(f) {
  let kept = 0;
  for (const ref of live) {
    const r = ref.deref();
    if (!r) continue;

    live[kept++] = ref;
    for (const [c, o] of ENDS) f(r, c, o);
  }

  live.length = kept;
}

function removedAt(target, prev, next, gone) {
  if (prev === null) return 0;
  if (prev.parentNode === target) return indexOf(prev) + 1;
  if (gone.has(prev)) return gone.get(prev);

  return next && next.parentNode === target ? indexOf(next) : 0;
}

function removed(target, node, index) {
  each((r, c, o) => {
    if (holds(node, r[c])) [r[c], r[o]] = [target, index];
    else if (r[c] === target && r[o] > index) r[o]--;
  });
}

function added(target, node) {
  if (node.parentNode !== target) return;

  const index = indexOf(node);
  each((r, c, o) => { if (r[c] === target && r[o] > index) r[o]++; });
}

hidden(G, '__rangeMutations', function (recs) {
  if (live.length === 0) return;

  const gone = new Map();
  for (let i = 0; i < recs.length; i += 7) {
    const kind = recs[i], target = recs[i + 1], node = recs[i + 2];
    if (!target || !node) continue;

    if (kind === 3) added(target, node);
    if (kind !== 0) continue;

    const index = removedAt(target, recs[i + 3], recs[i + 4], gone);
    gone.set(node, index);
    removed(target, node, index);
  }
});

// https://dom.spec.whatwg.org/#concept-cd-replace
function replaced(node, offset, count, length) {
  each((r, c, o) => {
    if (r[c] !== node || r[o] <= offset) return;

    r[o] = r[o] <= offset + count ? offset : r[o] + length - count;
  });
}

function wrapSetter(proto, name) {
  const d = Object.getOwnPropertyDescriptor(proto, name);
  if (!d || !d.set) return;

  Object.defineProperty(proto, name, { get: d.get, configurable: true, enumerable: d.enumerable, set(v) {
    const before = isData(this) && !data.busy ? this.data.length : -1;
    d.set.call(this, v);
    if (before >= 0 && live.length > 0) replaced(this, 0, before, this.data.length);
  } });
}

const data = { busy: false, replaced };
hidden(G, '__rangeData', data);
wrapSetter(CharacterData.prototype, 'data');
wrapSetter(Node.prototype, 'nodeValue');
wrapSetter(Node.prototype, 'textContent');

// https://dom.spec.whatwg.org/#concept-text-split
hidden(G.__ranges, 'split', function (node, offset, rest) {
  const parent = node.parentNode, index = indexOf(node) + 1;
  each((r, c, o) => {
    if (r[c] === node && r[o] > offset) [r[c], r[o]] = [rest, r[o] - offset];
    else if (r[c] === parent && r[o] === index) r[o]++;
  });
});
})();
