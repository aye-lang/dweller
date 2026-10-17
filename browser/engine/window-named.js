// https://html.spec.whatwg.org/multipage/nav-history-apis.html#named-access-on-the-window-object
(function () {
const ak = globalThis.__canvasHost;
const ETP = EventTarget.prototype;
const BY_NAME = /^(embed|form|img|object)$/;

const frameNamed = name => {
  for (const f of document.querySelectorAll('iframe[name], frame[name]')) {
    if (f.getAttribute('name') === name && f.contentWindow) return f.contentWindow;
  }

  return undefined;
};

const elementsNamed = name => [...document.querySelectorAll('[id], [name]')].filter(el =>
  el.id === name || (BY_NAME.test(el.localName) && el.getAttribute('name') === name));

const namedValue = name => {
  const frame = frameNamed(name);
  if (frame) return frame;

  const found = elementsNamed(name);
  if (found.length < 2) return found[0];

  return Object.setPrototypeOf(found, HTMLCollection.prototype);
};

const handler = {
  get(target, key, receiver) {
    if (typeof key !== 'string' || key === '' || key in ETP) return Reflect.get(target, key, receiver);

    const v = namedValue(key);

    return v === undefined ? Reflect.get(target, key, receiver) : v;
  },
};

Object.setPrototypeOf(Window.prototype, new Proxy(ak.windowNamed(ETP), handler));
})();
