// Arrays keep the attributes Object.defineProperty gives their elements and
// their length (ECMA-262 10.4.2: array exotic [[DefineOwnProperty]]).
function show(name, v) { console.log(name, JSON.stringify(v)); }
function threw(f) {
  try { f(); return 'no error'; } catch (e) { return e.constructor.name; }
}
function desc(o, k) { return Object.getOwnPropertyDescriptor(o, k); }

const ro = [1, 2, 3];
Object.defineProperty(ro, '0', { writable: false });
ro[0] = 9;
show('read-only element', [ro[0], desc(ro, '0')]);
show('strict write', threw(() => { 'use strict'; ro[0] = 9; }));
show('defineProperty again', threw(() => Object.defineProperty(ro, '0', { value: 5 })));

const acc = [1, 2, 3];
let sets = [];
Object.defineProperty(acc, '1', { get() { return 42; }, set(v) { sets.push(v); }, enumerable: true, configurable: true });
acc[1] = 7;
show('accessor', [acc[1], sets, acc.length, Object.keys(acc), 1 in acc]);
for (let i = 0; i < 3; i++) acc[i] = i * 10;
show('accessor in a loop', [acc[0], acc[1], acc[2], sets]);
show('spread', [...acc]);
show('join and slice', [acc.join('-'), acc.slice(1)]);
show('map', acc.map(x => x + 1));
delete acc[1];
show('deleted accessor', [acc[1], 1 in acc, Object.keys(acc)]);

const hidden = ['a', 'b', 'c'];
Object.defineProperty(hidden, '1', { enumerable: false });
show('non-enumerable', [Object.keys(hidden), Object.getOwnPropertyNames(hidden), JSON.stringify(hidden)]);
const forIn = [];
for (const k in hidden) forIn.push(k);
show('for-in', forIn);

const fixed = [1, 2, 3, 4];
Object.defineProperty(fixed, '1', { configurable: false });
show('delete non-configurable', [delete fixed[1], fixed]);
fixed.length = 0;
show('shrink stops above it', [fixed.length, fixed]);
show('strict shrink', threw(() => { 'use strict'; fixed.length = 0; }));
show('define a shorter length', threw(() => Object.defineProperty(fixed, 'length', { value: 0 })));

const len = [1, 2, 3];
Object.defineProperty(len, 'length', { writable: false });
show('read-only length', desc(len, 'length'));
len[5] = 6;
len.length = 10;
show('past a read-only length', [len.length, len[5]]);
show('push', threw(() => len.push(4)));
show('pop', threw(() => len.pop()));
show('after push and pop', len);
show('strict index past it', threw(() => { 'use strict'; len[3] = 1; }));

const grow = [1];
Object.defineProperty(grow, '3', { value: 'x', writable: true, enumerable: true, configurable: true });
show('define past the end', [grow.length, grow]);
Object.defineProperty(grow, 'length', { value: 2 });
show('define a shorter length', [grow.length, grow]);
show('bad length', threw(() => Object.defineProperty(grow, 'length', { value: -1 })));
show('length not configurable', threw(() => Object.defineProperty(grow, 'length', { configurable: true })));

const sealed = [1, 2];
Object.preventExtensions(sealed);
sealed[2] = 3;
show('not extensible', [sealed.length, threw(() => sealed.push(3)), threw(() => { 'use strict'; sealed[2] = 3; })]);

const frozen = Object.freeze([1, 2]);
show('frozen', [Object.isFrozen(frozen), threw(() => frozen.pop()), threw(() => { 'use strict'; frozen.length = 0; }), frozen]);

const all = [1, 2];
Object.defineProperty(all, '0', { writable: false, configurable: false });
Object.defineProperty(all, '1', { writable: false, configurable: false });
Object.defineProperty(all, 'length', { writable: false });
Object.preventExtensions(all);
show('frozen by hand', Object.isFrozen(all));

const order = [];
const counted = [0, 1, 2];
Object.defineProperty(counted, '0', { get() { order.push('get 0'); return 0; } });
counted.forEach((v, i) => order.push(v + '@' + i));
show('forEach order', order);
show('reverse', threw(() => counted.reverse()));
show('copy of the getter', [Array.from(counted), counted.indexOf(2), counted.includes(0)]);
