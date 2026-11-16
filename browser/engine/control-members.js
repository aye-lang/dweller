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

// https://html.spec.whatwg.org/multipage/input.html#input-type-change
const VALUE_TYPES = /^(?:text|search|tel|url|email|password|date|month|week|time|datetime-local|number|range|color)$/;
const SELECTABLE = /^(?:text|search|tel|url|password)$/;
const inputType = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'type');

define(HTMLInputElement.prototype, 'type', inputType.get, function (v) {
  const before = this.type;
  const value = this.value;
  const selection = SELECTABLE.test(before) ? [this.selectionStart, this.selectionEnd, this.selectionDirection] : null;

  inputType.set.call(this, v);

  const now = this.type;

  if (now === before || !VALUE_TYPES.test(now)) {
    if (now !== before && now !== 'file' && VALUE_TYPES.test(before) && value !== '') this.setAttribute('value', value);

    return;
  }

  this.value = VALUE_TYPES.test(before) ? value : this.getAttribute('value') ?? '';
  if (!SELECTABLE.test(now)) return;
  if (selection && this.value === value) this.setSelectionRange(selection[0], selection[1], selection[2]);
  else this.setSelectionRange(0, 0, 'none');
});

define(HTMLTextAreaElement.prototype, 'type', function () { return 'textarea'; });
define(HTMLTextAreaElement.prototype, 'textLength', function () { return this.value.length; });
define(HTMLOutputElement.prototype, 'type', function () { return 'output'; });

define(HTMLOptionElement.prototype, 'index', function () {
  const select = this.closest('select');
  const i = select ? [...select.options].indexOf(this) : -1;

  return i < 0 ? 0 : i;
});
})();
