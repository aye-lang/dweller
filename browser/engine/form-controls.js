// https://html.spec.whatwg.org/multipage/common-dom-interfaces.html#htmlformcontrolscollection
(function () {
const FormP = HTMLFormElement.prototype;
const HTMLP = HTMLElement.prototype;
const isIndex = key => /^(0|[1-9]\d*)$/.test(key);

Object.defineProperty(HTMLFormControlsCollection.prototype, 'namedItem', {
  value: function namedItem(name) {
    const v = this[String(name)];

    return v instanceof Element || v instanceof RadioNodeList ? v : null;
  },
  writable: true, configurable: true,
});

const radios = list => [...list].filter(el => el instanceof HTMLInputElement && el.type === 'radio');

Object.defineProperty(RadioNodeList.prototype, 'value', {
  get() {
    const checked = radios(this).find(el => el.checked);

    return checked ? checked.value : '';
  },
  set(v) {
    const radio = radios(this).find(el => el.value === String(v));
    if (radio) radio.checked = true;
  },
  configurable: true,
});

Object.defineProperty(FormP, 'length', { get() { return this.elements.length; }, configurable: true });

const named = (form, key) => isIndex(key) ? form.elements[key] : form.elements.namedItem(key) ?? undefined;

const handler = {
  get(target, key, receiver) {
    if (typeof key !== 'string' || key in target || !(receiver instanceof HTMLFormElement)) return Reflect.get(target, key, receiver);

    return named(receiver, key);
  },
};

Object.setPrototypeOf(FormP, new Proxy(HTMLP, handler));
})();
