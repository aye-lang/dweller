const spreadOff = [];
spreadOff[Symbol.isConcatSpreadable] = false;
const spreadableOk = spreadOff.concat()[0] === spreadOff;
const withSpecies = [];
(withSpecies.constructor = {})[Symbol.species] = function () { return { foo: 1 }; };
const speciesOk = withSpecies.concat(Boolean).foo === 1;
console.log(spreadableOk && speciesOk ? 'native is fine' : 'core-js polyfills concat');
const like = { length: 2, 0: 'a', 1: 'b', [Symbol.isConcatSpreadable]: true };
console.log([1].concat(like, [2, , 3], 'x').length, [1].concat(like)[2]);
class Sub extends Array {}
const sub = new Sub();
sub.push(1, 2);
const s = sub.concat([3]);
console.log(s instanceof Sub, s.length, s[2]);
console.log(Array.prototype.concat.call('ab', [1]).length, typeof Array.prototype.concat.call(1)[0]);
Array.prototype[1] = 'proto';
console.log([0, , 2].concat()[1]);
delete Array.prototype[1];
const nonCtor = [];
nonCtor.constructor = { [Symbol.species]: 1 };
try { nonCtor.concat(); } catch (e) { console.log(e instanceof TypeError); }
