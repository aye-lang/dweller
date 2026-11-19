// https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#printing
(function () {
const ak = globalThis.__canvasHost;

Object.defineProperty(globalThis, 'print', {
  value: function print() { ak.printAsk(); },
  writable: true,
  configurable: true,
  enumerable: true,
});

for (const type of ['beforeprint', 'afterprint']) {
  let handler = null;

  Object.defineProperty(globalThis, 'on' + type, {
    get() { return handler; },
    set(f) {
      if (handler) removeEventListener(type, handler);

      handler = typeof f === 'function' ? f : null;

      if (handler) addEventListener(type, handler);
    },
    configurable: true,
    enumerable: true,
  });
}
})();
