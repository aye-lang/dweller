function fails(f) { try { return !!f(); } catch (e) { return true; } }
const named = !fails(function () { const e = /./; e.exec = function () { const r = []; r.groups = { a: '7' }; return r; }; return '7' !== ''.replace(e, '$<a>'); });
const keeps0 = '$0' === 'a'.replace(/./, '$0');
const undefCapture = '' === /./[Symbol.replace]('a', '$0');
const splitExec = !fails(function () { const e = /(?:)/, t = e.exec; e.exec = function () { return t.apply(this, arguments); }; const n = 'ab'.split(e); return n.length !== 2 || n[0] !== 'a' || n[1] !== 'b'; });
let delegates = true;
for (const name of ['match', 'replace', 'search', 'split']) {
  const sym = Symbol[name];
  const custom = {};
  custom[sym] = function () { return 7; };
  let called = false;
  let re = /a/;
  if (name === 'split') {
    re = { constructor: { [Symbol.species]: function () { return re; } }, flags: '' };
    re[sym] = /./[sym];
  }
  re.exec = function () { called = true; return null; };
  re[sym]('');
  if (''[name](custom) !== 7 || !called) delegates = false;
}
console.log(named && keeps0 && !undefCapture && splitExec && delegates ? 'native is fine' : 'core-js polyfills replace');
console.log('2020-11'.replace(/(?<y>\d+)-(?<m>\d+)/, '$<m>/$<y>'), 'aXbX'.replaceAll(/x/gi, '-'));
const own = /b/g;
own.exec = function () { return null; };
console.log('abc'.replace(own, 'B'), 'abc'.match(own), 'abc'.search(own));
const saved = RegExp.prototype.exec;
RegExp.prototype.exec = function (s) { return saved.call(this, s.toUpperCase()); };
console.log('abc'.replace(/B/, 'x'));
RegExp.prototype.exec = saved;
console.log('abc'.replace(/b/, 'x'), 'a1b2'.split(/\d/).join(','));
try { 'a'.replaceAll(/a/, 'b'); } catch (e) { console.log(e instanceof TypeError); }
console.log('x'.replace({ [Symbol.replace]: (s, r) => s + r }, '!'), 'x'.replace({ toString: () => 'x' }, 'y'));
