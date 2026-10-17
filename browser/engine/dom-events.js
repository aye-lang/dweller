(function () {
const G = globalThis;
const EventP = Event.prototype;
const ETP = EventTarget.prototype;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const invalidState = (what) => new DOMException("Failed to execute 'dispatchEvent' on 'EventTarget': The event " + what + '.', 'InvalidStateError');

// https://dom.spec.whatwg.org/#interface-event
['NONE', 'CAPTURING_PHASE', 'AT_TARGET', 'BUBBLING_PHASE'].forEach((k, i) => {
  Object.defineProperty(Event, k, { value: i, enumerable: true });
  Object.defineProperty(EventP, k, { value: i, enumerable: true });
});

function isTrusted() {
  return this._trusted;
}
function setTrusted(v) {
  hidden(this, '_trusted', !!v);
}
Object.defineProperty(EventP, 'isTrusted', {
  set(v) {
    Object.defineProperty(this, 'isTrusted', { get: isTrusted, set: setTrusted, enumerable: true, configurable: false });
    hidden(this, '_trusted', !!v);
  },
  configurable: true,
});

// https://dom.spec.whatwg.org/#dom-event-initevent
const inits = {};
function guardInit(C, name, required) {
  const init = C.prototype[name];

  inits[name] = init;
  hidden(C.prototype, name, function (type) {
    if (arguments.length < required) throw new TypeError("Failed to execute '" + name + "' on '" + C.name + "': " + required + ' argument required, but only 0 present.');
    if (this._dispatching) return;

    hidden(this, '_uninit', false);
    this._stop = false;
    this._stopNow = false;
    this.defaultPrevented = false;
    this.target = null;
    init.apply(this, arguments);
  });
}
guardInit(Event, 'initEvent', 1);
guardInit(CustomEvent, 'initCustomEvent', 1);
guardInit(UIEvent, 'initUIEvent', 1);
guardInit(MouseEvent, 'initMouseEvent', 1);
hidden(MouseEvent.prototype, 'initMouseEvent', (function (initMouse) {
  return function (type, bubbles, cancelable, view, detail, sx, sy) {
    initMouse.apply(this, arguments);

    if (this._dispatching) return;

    this.screenX = sx || 0;
    this.screenY = sy || 0;
  };
})(MouseEvent.prototype.initMouseEvent));
hidden(KeyboardEvent.prototype, 'initKeyboardEvent', function (type, bubbles, cancelable, view, key, location, ctrl, alt, shift, meta) {
  if (this._dispatching) return;

  this.initUIEvent(type, bubbles, cancelable, view, 0);
  this.key = key === undefined ? '' : String(key);
  this.location = location || 0;
  this.ctrlKey = !!ctrl;
  this.altKey = !!alt;
  this.shiftKey = !!shift;
  this.metaKey = !!meta;
});

// https://dom.spec.whatwg.org/#dom-document-createevent
const defineEvent = (name, Base) => hidden(G, name, ({ [name]: class extends Base {} })[name]);
defineEvent('BeforeUnloadEvent', Event);
defineEvent('TextEvent', UIEvent);
defineEvent('DeviceMotionEvent', Event);
defineEvent('DeviceOrientationEvent', Event);
const CREATE_EVENT = { beforeunloadevent: 'BeforeUnloadEvent', compositionevent: 'CompositionEvent', devicemotionevent: 'DeviceMotionEvent', deviceorientationevent: 'DeviceOrientationEvent', dragevent: 'DragEvent', hashchangeevent: 'HashChangeEvent', messageevent: 'MessageEvent', storageevent: 'StorageEvent', svgevents: 'Event', textevent: 'TextEvent', touchevent: 'TouchEvent' };
const createEvent = Document.prototype.createEvent;
hidden(Document.prototype, 'createEvent', function (kind) {
  const name = CREATE_EVENT[String(kind).toLowerCase()];
  const ev = name ? new G[name]('') : createEvent.call(this, kind);

  hidden(ev, '_uninit', true);

  return ev;
});

// https://dom.spec.whatwg.org/#concept-event-dispatch
function dispatchWith(dispatch) {
  return function (ev) {
    if (!(ev instanceof Event)) return dispatch.call(this, ev);
    if (ev._dispatching) throw invalidState('is already being dispatched');
    if (ev._uninit) throw invalidState('provided is uninitialized');

    ev.isTrusted = false;

    const checkable = ev instanceof MouseEvent && ev.type === 'click' ? checkableTarget(this, ev.bubbles) : null;
    const before = checkable && legacyPreActivation(checkable);
    const go = dispatch.call(this, ev);

    if (checkable) activateCheckable(checkable, before, go);

    return go;
  };
}

// https://dom.spec.whatwg.org/#eventtarget-activation-behavior
const ACTIVATES = /^(a|area|button|input|label|summary|select|textarea)$/;
function checkableTarget(target, bubbles) {
  for (let n = target; n instanceof Node; n = bubbles ? n.parentNode : null) {
    if (n.nodeType !== Node.ELEMENT_NODE || !ACTIVATES.test(n.localName) || ((n.localName === 'a' || n.localName === 'area') && !n.hasAttribute('href'))) continue;

    return n.localName === 'input' && (n.type === 'checkbox' || n.type === 'radio') ? n : null;
  }

  return null;
}

// https://html.spec.whatwg.org/multipage/input.html#the-input-element:legacy-pre-activation-behavior
function legacyPreActivation(input) {
  const before = { checked: input.checked, other: input.type === 'radio' ? checkedInGroup(input) : null };

  input.checked = input.type === 'radio' ? true : !input.checked;

  return before;
}
function checkedInGroup(radio) {
  const scope = radio.form || radio.getRootNode();

  if (!radio.name || !scope.querySelectorAll) return null;

  return Array.from(scope.querySelectorAll('input[type=radio]')).find(r => r !== radio && r.checked && r.name === radio.name && r.form === radio.form) || null;
}
function activateCheckable(input, before, go) {
  if (!go) {
    input.checked = before.checked;

    if (before.other) before.other.checked = true;

    return;
  }

  if (!input.isConnected || before.checked === input.checked) return;

  input.dispatchEvent(new Event('input', { bubbles: true, composed: true }));
  input.dispatchEvent(new Event('change', { bubbles: true }));
}
hidden(ETP, 'dispatchEvent', dispatchWith(ETP.dispatchEvent));

const click = HTMLElement.prototype.click;
hidden(HTMLElement.prototype, 'click', function () {
  if (this.hasAttribute('disabled') && /^(button|input|select|textarea)$/.test(this.localName)) return;
  if (!checkableTarget(this, true)) return click.call(this);

  this.dispatchEvent(new PointerEvent('click', { bubbles: true, cancelable: true, composed: true, pointerId: -1 }));
});
hidden(G, 'dispatchEvent', dispatchWith(G.dispatchEvent));

// https://dom.spec.whatwg.org/#concept-flatten-options
function flattenMore(options) {
  if (options === null || typeof options !== 'object') return { capture: !!options };

  const flat = { capture: !!options.capture, once: !!options.once };
  const passive = options.passive;
  const signal = options.signal;

  if (passive !== undefined) flat.passive = !!passive;
  if (signal !== undefined && !(signal instanceof AbortSignal)) throw new TypeError("Failed to execute 'addEventListener' on 'EventTarget': Failed to read the 'signal' property from 'AddEventListenerOptions': Failed to convert value to 'AbortSignal'.");
  if (signal !== undefined) flat.signal = signal;

  return flat;
}
function addWith(add, remove) {
  return function (type, listener, options) {
    const flat = flattenMore(options);
    const signal = flat.signal;

    if (signal && signal.aborted) return;

    add.call(this, type, listener, flat);

    if (signal && listener != null) signal.addEventListener('abort', () => remove.call(this, type, listener, flat.capture));
  };
}
hidden(ETP, 'addEventListener', addWith(ETP.addEventListener, ETP.removeEventListener));
hidden(G, 'addEventListener', addWith(G.addEventListener, G.removeEventListener));

// https://html.spec.whatwg.org/multipage/webappapis.html#window-reflecting-body-element-event-handler-set
const FORWARDED = ['blur', 'error', 'focus', 'load', 'scroll', 'resize'];
for (const C of [HTMLBodyElement, HTMLFrameSetElement]) {
  for (const type of FORWARDED) Object.defineProperty(C.prototype, 'on' + type, { get() { return G['on' + type]; }, set(v) { G['on' + type] = v; }, configurable: true, enumerable: true });
}
const setAttribute = Element.prototype.setAttribute;
hidden(Element.prototype, 'setAttribute', function (name, value) {
  setAttribute.call(this, name, value);

  const type = String(name).toLowerCase().slice(2);

  if ((this instanceof HTMLBodyElement || this instanceof HTMLFrameSetElement) && this !== document.body && FORWARDED.includes(type)) G['on' + type] = forwardedHandler(type, String(value));
});
function forwardedHandler(type, source) {
  return type === 'error' ? new Function('event', 'source', 'lineno', 'colno', 'error', source) : new Function('event', source);
}
})();
