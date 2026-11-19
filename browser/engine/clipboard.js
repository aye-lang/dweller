// spec: https://w3c.github.io/clipboard-apis/
// chromium: third_party/blink/renderer/modules/clipboard/clipboard.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const PLAIN = 'text/plain';
let making = false;

function refused(method, why) {
  return new DOMException("Failed to execute '" + method + "' on 'Clipboard': " + why, 'NotAllowedError');
}

function allowed() {
  return navigator.userActivation.isActive && document.hasFocus();
}

class ClipboardItem {
  constructor(items, options) {
    if (arguments.length < 1) throw new TypeError("Failed to construct 'ClipboardItem': 1 argument required, but only 0 present.");
    if (items === null || typeof items !== 'object' || Object.keys(items).length === 0) throw new TypeError("Failed to construct 'ClipboardItem': Empty dictionary argument");

    hidden(this, '_items', Object.assign({}, items));
    hidden(this, '_style', options && options.presentationStyle !== undefined ? String(options.presentationStyle) : 'unspecified');
  }

  get types() { return Object.keys(this._items); }
  get presentationStyle() { return this._style; }

  getType(type) {
    const key = String(type);
    if (!(key in this._items)) return Promise.reject(new DOMException("Failed to execute 'getType' on 'ClipboardItem': The type was not found", 'NotFoundError'));

    return Promise.resolve(this._items[key]).then(v => v instanceof Blob ? v : new Blob([String(v)], { type: key }));
  }

  static supports(type) { return String(type) === PLAIN; }
}

class Clipboard extends EventTarget {
  constructor() {
    if (!making) throw new TypeError('Illegal constructor');
    super();
  }

  writeText(data) {
    if (arguments.length < 1) return Promise.reject(new TypeError("Failed to execute 'writeText' on 'Clipboard': 1 argument required, but only 0 present."));
    if (!allowed()) return Promise.reject(refused('writeText', 'Write permission denied.'));
    if (!ak.clipboardWrite(String(data))) return Promise.reject(refused('writeText', 'The clipboard is not available.'));

    return Promise.resolve();
  }

  readText() {
    const pasted = allowed() ? ak.clipboardRead() : null;
    if (pasted === null) return Promise.reject(refused('readText', 'Read permission denied.'));

    return Promise.resolve(pasted);
  }

  write(items) {
    const item = Array.from(items || [])[0];
    if (!(item instanceof ClipboardItem) || !item.types.includes(PLAIN)) return Promise.reject(refused('write', 'Only text/plain is supported.'));
    if (!allowed()) return Promise.reject(refused('write', 'Write permission denied.'));

    return item.getType(PLAIN).then(blob => blob.text()).then(data => {
      if (!ak.clipboardWrite(data)) throw refused('write', 'The clipboard is not available.');
    });
  }

  read() {
    return this.readText().then(pasted => [new ClipboardItem({ [PLAIN]: new Blob([pasted], { type: PLAIN }) })], () => { throw refused('read', 'Read permission denied.'); });
  }
}

making = true;
const clipboard = new Clipboard();
making = false;

hidden(G, 'Clipboard', Clipboard);
hidden(G, 'ClipboardItem', ClipboardItem);
Object.defineProperty(Navigator.prototype, 'clipboard', { get() { return clipboard; }, enumerable: true, configurable: true });
})();
