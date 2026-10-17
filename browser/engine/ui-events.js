(function () {
const G = globalThis;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });

// https://html.spec.whatwg.org/multipage/nav-history-apis.html#named-access-on-the-window-object
const NAMED_BY_NAME = 'embed[name],form[name],img[name],object[name]';
const byName = (selector, k) => Array.from(document.querySelectorAll(selector)).find(e => e.getAttribute('name') === k);
function namedElement(k) {
  if (typeof k !== 'string' || k === '' || !G.document) return null;

  const frame = byName('iframe[name]', k);

  return (frame && frame.contentWindow) || document.getElementById(k) || byName(NAMED_BY_NAME, k) || null;
}
const windowProto = Object.getPrototypeOf(G);
const windowProperties = new Proxy(Object.create(Object.getPrototypeOf(windowProto)), {
  get: (t, k, r) => (k in t ? Reflect.get(t, k, r) : namedElement(k) ?? undefined),
  has: (t, k) => k in t || namedElement(k) !== null,
});
Object.setPrototypeOf(windowProto, windowProperties);

// https://w3c.github.io/uievents/#dom-keyboardevent-getmodifierstate
const MODIFIERS = { Control: 'ctrlKey', Shift: 'shiftKey', Alt: 'altKey', Meta: 'metaKey', AltGraph: '_altGraph', CapsLock: '_capsLock' };
function getModifierState(key) {
  return Object.hasOwn(MODIFIERS, key) && !!this[MODIFIERS[key]];
}
hidden(KeyboardEvent.prototype, 'getModifierState', getModifierState);
hidden(MouseEvent.prototype, 'getModifierState', getModifierState);

// https://drafts.csswg.org/cssom-view/#extensions-to-the-mouseevent-interface
function paddingEdge(t) {
  if (!(t instanceof Element)) return { left: 0, top: 0 };

  const r = t.getBoundingClientRect();

  return { left: r.left + t.clientLeft, top: r.top + t.clientTop };
}
const MOUSE_COORDINATES = {
  pageX() { return this.clientX + G.scrollX; },
  pageY() { return this.clientY + G.scrollY; },
  x() { return this.clientX; },
  y() { return this.clientY; },
  offsetX() { return this.target ? this.clientX - paddingEdge(this.target).left : this.pageX; },
  offsetY() { return this.target ? this.clientY - paddingEdge(this.target).top : this.pageY; },
  layerX() { return this.pageX; },
  layerY() { return this.pageY; },
};
for (const k in MOUSE_COORDINATES) {
  if (!(k in MouseEvent.prototype)) Object.defineProperty(MouseEvent.prototype, k, { get: MOUSE_COORDINATES[k], enumerable: true, configurable: true });
}

// https://w3c.github.io/uievents/#idl-uievent
Object.defineProperty(UIEvent.prototype, 'pseudoTarget', { get() { return null; }, enumerable: true, configurable: true });

for (const C of [Event, CustomEvent, UIEvent, MouseEvent, KeyboardEvent, FocusEvent, InputEvent, G.CompositionEvent, G.WheelEvent, G.PointerEvent]) {
  Object.defineProperty(C, 'length', { value: 1, configurable: true });
}

function requireArgument(C, name, init) {
  hidden(C.prototype, name, function (type) {
    if (arguments.length < 1) throw new TypeError("Failed to execute '" + name + "' on '" + C.name + "': 1 argument required, but only 0 present.");

    init.apply(this, arguments);
  });
}
requireArgument(KeyboardEvent, 'initKeyboardEvent', KeyboardEvent.prototype.initKeyboardEvent);
requireArgument(G.CompositionEvent, 'initCompositionEvent', function (type, bubbles, cancelable, view, data) {
  if (this._dispatching) return;

  this.initUIEvent(type, bubbles, cancelable, view, 0);
  this.data = data === undefined ? '' : String(data);
});

// https://w3c.github.io/uievents/#legacy-interface-TextEvent
const MAKE_TEXT_EVENT = Symbol();
const TextEvent = class TextEvent extends UIEvent {
  constructor(key) {
    if (key !== MAKE_TEXT_EVENT) throw new TypeError("Failed to construct 'TextEvent': Illegal constructor");

    super('');
    this.data = '';
  }
};
hidden(G, 'TextEvent', TextEvent);
requireArgument(TextEvent, 'initTextEvent', function (type, bubbles, cancelable, view, data) {
  if (this._dispatching) return;

  this.initUIEvent(type, bubbles, cancelable, view, 0);
  this.data = String(data);
});
const createEvent = Document.prototype.createEvent;
hidden(Document.prototype, 'createEvent', function (kind) {
  if (String(kind).toLowerCase() !== 'textevent') return createEvent.call(this, kind);

  const ev = new TextEvent(MAKE_TEXT_EVENT);

  hidden(ev, '_uninit', true);

  return ev;
});
})();
