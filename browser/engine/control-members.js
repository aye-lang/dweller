// https://html.spec.whatwg.org/multipage/forms.html
(function () {
const define = (P, key, get, set) => Object.defineProperty(P, key, { get, set, configurable: true, enumerable: true });

class FileList {
  get length() { return 0; }

  item() { return null; }

  *[Symbol.iterator]() {}
}

Object.defineProperty(globalThis, 'FileList', { value: FileList, writable: true, configurable: true });

const fileLists = new WeakMap();

define(HTMLInputElement.prototype, 'files', function () {
  if (this.type !== 'file') return null;
  if (!fileLists.has(this)) fileLists.set(this, new FileList());

  return fileLists.get(this);
}, function (v) {
  if (this.type === 'file' && v instanceof FileList) fileLists.set(this, v);
});

for (const P of [HTMLInputElement.prototype, HTMLTextAreaElement.prototype]) {
  define(P, 'dirName', function () { return this.getAttribute('dirname') ?? ''; }, function (v) { this.setAttribute('dirname', v); });
}

define(HTMLTextAreaElement.prototype, 'type', function () { return 'textarea'; });
define(HTMLTextAreaElement.prototype, 'textLength', function () { return this.value.length; });
define(HTMLOutputElement.prototype, 'type', function () { return 'output'; });

define(HTMLOptionElement.prototype, 'index', function () {
  const select = this.closest('select');
  const i = select ? [...select.options].indexOf(this) : -1;

  return i < 0 ? 0 : i;
});
})();
