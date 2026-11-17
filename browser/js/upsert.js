(function () {
'use strict';
// spec: https://tc39.es/proposal-upsert/
const apply = Reflect.apply;
const mapHas = Map.prototype.has;
const mapGet = Map.prototype.get;
const mapSet = Map.prototype.set;
const weakHas = WeakMap.prototype.has;
const weakGet = WeakMap.prototype.get;
const weakSet = WeakMap.prototype.set;

function install(proto, has, get, set) {
  const methods = {
    getOrInsert(key, value) {
      if (apply(has, this, [key])) return apply(get, this, [key]);
      apply(set, this, [key, value]);
      return value;
    },
    getOrInsertComputed(key, callbackfn) {
      const present = apply(has, this, [key]);
      if (typeof callbackfn !== 'function') throw new TypeError('callbackfn is not a function');
      if (present) return apply(get, this, [key]);
      const value = callbackfn(key === 0 ? 0 : key);
      apply(set, this, [key, value]);
      return value;
    },
  };
  for (const name of ['getOrInsert', 'getOrInsertComputed']) {
    Object.defineProperty(proto, name, { value: methods[name], writable: true, enumerable: false, configurable: true });
  }
}
install(Map.prototype, mapHas, mapGet, mapSet);
install(WeakMap.prototype, weakHas, weakGet, weakSet);
})();
