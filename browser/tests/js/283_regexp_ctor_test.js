function fails(f) { try { return !!f(); } catch (e) { return true; } }
const re1 = /a/g;
const re2 = /a/g;
const correctNew = new RegExp(re1) !== re1;
const unsupportedY = fails(function () { const t = RegExp('a', 'y'); t.lastIndex = 2; return t.exec('abcd') != null; });
const ctorBroken = fails(function () { re2[Symbol.match] = false; return RegExp(re1) != re1 || RegExp(re2) == re2 || '/a/i' != RegExp(re1, 'i'); });
let execCalled = false;
const delegates = (function () { const t = /[ac]/; t.exec = function () { execCalled = true; return /./.exec.apply(this, arguments); }; return t.test('abc') === true && execCalled; })();
const toStringOk = !fails(function () { return '/a/b' != RegExp.prototype.toString.call({ source: 'a', flags: 'b' }); }) && RegExp.prototype.toString.name === 'toString';
const flagsOk = /./g.flags === 'g';
console.log(correctNew && !unsupportedY && !ctorBroken ? 'native RegExp is fine' : 'core-js replaces RegExp');
console.log(delegates ? 'native test is fine' : 'core-js replaces test');
console.log(toStringOk && flagsOk ? 'native toString and flags are fine' : 'core-js replaces toString or flags');
const re3 = /b/;
console.log(RegExp(re3) === re3, RegExp(re3, 'g') === re3, String(RegExp(re3, 'g')), new RegExp(re3) === re3);
re3.constructor = Object;
console.log(RegExp(re3) === re3, String(RegExp(re3)));
const like = { source: 'x+', flags: 'i' };
like[Symbol.match] = true;
console.log(String(new RegExp(like)), String(RegExp(like)), String(new RegExp(like, 'g')));
like.constructor = RegExp;
console.log(RegExp(like) === like);
const falsy = /c/m;
falsy[Symbol.match] = 0;
console.log(RegExp(falsy) === falsy, String(RegExp(falsy)));
console.log(String(RegExp()), String(RegExp(undefined, 'g')), String(new RegExp('d', undefined)));
const own = /z/;
own.exec = function (s) { return s === 'hit' ? [s] : null; };
console.log(own.test('hit'), own.test('z'));
own.exec = function () { return 1; };
try { own.test('z'); } catch (e) { console.log(e.constructor.name); }
own.exec = 'not callable';
console.log(own.test('z'), own.test('y'));
const plainObj = { exec: function (s) { return { s }; } };
console.log(RegExp.prototype.test.call(plainObj, 'q'));
try { RegExp.prototype.test.call({}, 'q'); } catch (e) { console.log(e.constructor.name); }
try { RegExp.prototype.test.call(1, 'q'); } catch (e) { console.log(e.constructor.name); }
const g = /o/g;
console.log(g.test('foo'), g.lastIndex, g.test('foo'), g.lastIndex, g.test('foo'), g.lastIndex);
