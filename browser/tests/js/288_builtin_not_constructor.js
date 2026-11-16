function isCtor(f) { try { Reflect.construct(function () {}, [], f); return true; } catch (e) { return false; } }
const fns = { "Math.abs": Math.abs, "Array.prototype.map": Array.prototype.map, "JSON.parse": JSON.parse, parseInt, escape, "Date.prototype.setFullYear": Date.prototype.setFullYear, "DataView.prototype.getInt16": DataView.prototype.getInt16, "Object.keys": Object.keys, "Function.prototype.call": Function.prototype.call, Array, Object, Map, Proxy, Date, Promise, Function, RegExp, Error, "bound Array": Array.bind(null), "bound abs": Math.abs.bind(null), arrow: () => 1, method: { m() {} }.m, plain: function () {}, cls: class {} };
console.log(Object.keys(fns).filter(k => isCtor(fns[k])).join(" "));
for (const f of [Math.abs, [].map, parseInt]) { try { new f(); console.log("constructed"); } catch (e) { console.log(e.constructor.name); } }
try { Reflect.construct(Math.abs, []); } catch (e) { console.log(e.constructor.name); }
console.log(new Proxy({}, {}) instanceof Object, Reflect.construct(Date, [0]) instanceof Date, typeof new Array(2), new (Array.bind(null, 3))().length);
