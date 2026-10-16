(function () {
const define = (proto, name, get, set) => Object.defineProperty(proto, name, { get, set, configurable: true, enumerable: true });
const autocomplete = {
  get: function () { const v = this.getAttribute('autocomplete'); return v === null ? '' : v; },
  set: function (v) { this.setAttribute('autocomplete', String(v)); },
};
for (const n of ['Form', 'Input', 'Select', 'TextArea']) {
  define(globalThis['HTML' + n + 'Element'].prototype, 'autocomplete', autocomplete.get, autocomplete.set);
}
const refuse = () => Promise.reject(new DOMException('Credentials are not supported', 'NotSupportedError'));
class CredentialsContainer {
  get() { return refuse(); }
  create() { return refuse(); }
  store() { return refuse(); }
  preventSilentAccess() { return Promise.resolve(); }
}
Object.defineProperty(globalThis, 'CredentialsContainer', { value: CredentialsContainer, writable: true, configurable: true });
const container = new CredentialsContainer();
define(Navigator.prototype, 'credentials', function () { return container; });
})();
