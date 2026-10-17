(function () {
'use strict';
for (const C of [Array, Map, Set, RegExp, ArrayBuffer, Object.getPrototypeOf(Int8Array)]) {
  if (Object.getOwnPropertyDescriptor(C, Symbol.species)) continue;
  const get = Object.getOwnPropertyDescriptor({ get [Symbol.species]() { return this; } }, Symbol.species).get;
  Object.defineProperty(C, Symbol.species, { get, configurable: true });
}
})();
