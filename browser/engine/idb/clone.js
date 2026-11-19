// spec: https://html.spec.whatwg.org/multipage/structured-data.html#structuredserializeforstorage
(function (I) {
const G = globalThis;

I.serialize = function (value) {
  const wire = G.__wire.encode(value, []);
  let out = '';

  for (const b of wire.buffers) out += I.hexOf(new Uint8Array(b)) + ',';

  return out + ';' + I.ascii(wire.json);
};
I.deserialize = function (s) {
  const at = s.indexOf(';');
  const buffers = s.slice(0, at).split(',');

  buffers.pop();

  return G.__wire.decode(s.slice(at + 1), buffers.map(h => I.unhex(h, 0, h.length).buffer));
};
})(globalThis.__idb);
