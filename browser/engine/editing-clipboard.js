// spec: https://w3c.github.io/clipboard-apis/#clipboard-event-interfaces
// chromium: third_party/blink/renderer/core/clipboard/clipboard_commands.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const edit = G.__hostEdit;
const exec = Document.prototype.execCommand;
const PLAIN = 'text/plain';
const typeOf = t => { const s = String(t).toLowerCase(); return s === 'text' ? PLAIN : s; };

class DataTransfer {
  #data = new Map();

  constructor() {
    this.dropEffect = 'none';
    this.effectAllowed = 'none';
  }

  get types() { return [...this.#data.keys()]; }
  get files() { return []; }
  get items() { return []; }
  getData(type) { return this.#data.get(typeOf(type)) || ''; }
  setData(type, value) { this.#data.set(typeOf(type), String(value)); }

  clearData(type) {
    type === undefined ? this.#data.clear() : this.#data.delete(typeOf(type));
  }

  setDragImage() {}
}

class ClipboardEvent extends Event {
  constructor(type, init) {
    super(type, init);
    hidden(this, 'clipboardData', init && init.clipboardData !== undefined ? init.clipboardData : null);
  }
}

function fired(type, host, pasted) {
  const data = new DataTransfer();
  if (pasted !== null) data.setData(PLAIN, pasted);

  const go = host.dispatchEvent(new ClipboardEvent(type, { bubbles: true, cancelable: true, composed: true, clipboardData: data }));

  return { go, data };
}

function copied(host, cut) {
  const chosen = getSelection().toString();
  const { go, data } = fired(cut ? 'cut' : 'copy', host, null);
  if (!go) return data.types.length === 0 || ak.clipboardWrite(data.getData(PLAIN));
  if (chosen === '') return true;

  const written = ak.clipboardWrite(chosen);
  if (cut && written) exec.call(document, 'delete');

  return written;
}

function pasted(host, text) {
  if (fired('paste', host, text).go && text !== '') edit(host, 'inserttext\n' + text);
}

hidden(G, '__hostEdit', function (host, line) {
  const at = line.indexOf('\n'), command = line.slice(0, at);
  if (command === 'copy' || command === 'cut') return copied(host, command === 'cut');
  if (command === 'paste') return pasted(host, line.slice(at + 1));

  return edit(host, line);
});

hidden(Document.prototype, 'execCommand', function execCommand(name, showUI, value) {
  const command = String(name).toLowerCase();
  if (command !== 'copy' && command !== 'cut') return exec.call(this, name, showUI, value);

  const host = document.activeElement;
  if (!navigator.userActivation.isActive || !host || (command === 'cut' && !host.isContentEditable)) return false;

  return copied(host, command === 'cut');
});

if (!G.DataTransfer) hidden(G, 'DataTransfer', DataTransfer);
if (!G.ClipboardEvent) hidden(G, 'ClipboardEvent', ClipboardEvent);
})();
