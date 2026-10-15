// What scripts that pages insert needed (P4.45): a regex after a
// function declaration or a block, async methods with computed names,
// LengthOfArrayLike, push on an array-like object, Promise species.
function f() { return 1 }/ab/.test('xaby') && console.log('regex after a declaration');
if (true) { }
/c/.test('c') && console.log('regex after a block');
try { } catch (e) { }
/d/.test('d') && console.log('regex after catch');
var q = function () { return 9 }() / 3; console.log('division after an expression', q);
var o = { v: 10 }.v / 5; console.log('division after an object', o);
const k = 'm';
class A { static async[k](x) { return x + 1 } async ['n']() { return 2 } }
const lit = { async [k]() { return 3 }, async: 4 };
A.m(1).then(v => console.log('async computed static', v));
lit.m().then(v => console.log('async computed method', v, lit.async));
console.log('negative length', [].every.call({ length: -1 }, () => false), Math.max.apply(null, { length: -5 }));
const like = { length: 2, 0: 'a', 1: 'b' };
console.log('push on array-like', [].push.call(like, 'c', 'd'), like.length, like[3], [].push.call({ length: -3 }, 'x'));
class G extends Promise {}
console.log('species', G.resolve(1).then(v => v) instanceof G, G.race([1]) instanceof G, Promise.resolve(1).then(v => v) instanceof G);
const p = Promise.resolve(1);
const FP = function (exec) { exec(function () {}, function () {}); };
p.constructor = {};
p.constructor[Symbol.species] = FP;
console.log('core-js subclassing check', p.then(function () {}) instanceof FP);
