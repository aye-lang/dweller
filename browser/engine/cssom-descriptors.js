globalThis.__cssomDescriptors = function (ak, syntax) {
// spec: https://drafts.csswg.org/css-fonts-4/#font-face-rule
function kebab(key) {
  return String(key).replace(/[A-Z]/g, c => '-' + c.toLowerCase()).toLowerCase().replace(/^font-width$/, 'font-stretch');
}

function readBlock(kind, body) {
  const map = new Map();

  for (const part of syntax.splitTop(body, ';')) {
    const colon = part.indexOf(':');
    const name = kebab(part.slice(0, Math.max(colon, 0)).trim().toLowerCase());
    const value = colon < 0 ? null : ak.descriptor(kind, name, part.slice(colon + 1).trim());

    if (value !== null) {
      map.delete(name);
      map.set(name, value);
    }
  }

  return map;
}

function descriptorBlock(kind, body) {
  const map = readBlock(kind, body);

  function put(name, text) {
    const value = String(text).trim();
    const canon = value ? ak.descriptor(kind, name, value) : null;

    if (!value) map.delete(name);
    else if (canon !== null) map.set(name, canon);
  }

  const api = {
    getPropertyValue: name => map.get(kebab(name)) || '',
    getPropertyPriority: () => '',
    setProperty: (name, value) => put(kebab(name), value),
    removeProperty: name => put(kebab(name), ''),
    item: i => [...map.keys()][i] || '',
  };

  return new Proxy(api, {
    get(target, key) {
      if (key === 'cssText') return [...map].map(([n, v]) => n + ': ' + v + ';').join(' ');
      if (key === 'length') return map.size;
      if (typeof key === 'symbol' || key in target) return target[key];

      return map.get(kebab(key)) || '';
    },
    set(target, key, value) {
      put(kebab(key), value);

      return true;
    },
  });
}

return { descriptorBlock };
};
