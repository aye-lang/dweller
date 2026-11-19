// https://html.spec.whatwg.org/multipage/forms.html
(function () {
const define = (P, key, get, set) => Object.defineProperty(P, key, { get, set, configurable: true, enumerable: true });

// https://w3c.github.io/FileAPI/#filelist-section
class FileList {
  get length() { return (this._files || []).length; }

  item(i) { return (this._files || [])[i >>> 0] ?? null; }

  *[Symbol.iterator]() { yield* (this._files || []); }
}

Object.defineProperty(globalThis, 'FileList', { value: FileList, writable: true, configurable: true });

function listOf(files) {
  const list = new FileList();

  Object.defineProperty(list, '_files', { value: files });
  files.forEach((f, i) => Object.defineProperty(list, i, { value: f, enumerable: true }));

  return list;
}

Object.defineProperty(HTMLInputElement.prototype, '_file', {
  value(i) {
    const info = this._fileInfo(i);
    const file = new File([], info[0], { type: info[1], lastModified: info[2] });

    file._bytes = this._fileBytes(i);

    return file;
  },
});

const fileLists = new WeakMap();

define(HTMLInputElement.prototype, 'files', function () {
  if (this.type !== 'file') return null;

  const kept = fileLists.get(this);

  if (kept && kept.stamp === this._fileStamp) return kept.list;

  const files = [];

  for (let i = 0; i < this._fileCount; i++) files.push(this._file(i));

  fileLists.set(this, { stamp: this._fileStamp, list: listOf(files) });

  return fileLists.get(this).list;
}, function (v) {
  if (this.type === 'file' && v instanceof FileList) fileLists.set(this, { stamp: this._fileStamp, list: v });
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
