const out = [];
// es.string.ends-with / starts-with: correct-is-regexp-logic
function regexpLogic(name) {
  const re = /./;
  try { '/./'[name](re); return 'no throw'; } catch (e) {
    try { re[Symbol.match] = false; return '/./'[name](re) ? 'ok' : 'false'; } catch (e2) { return 'second throws ' + e2; }
  }
}
out.push('endsWith ' + regexpLogic('endsWith'), 'startsWith ' + regexpLogic('startsWith'), 'includes ' + regexpLogic('includes'));
// es.string.trim: whitespaces and non-whitespaces, and the function name
const ws = '\u0009\u000A\u000B\u000C\u000D                　  ﻿';
const non = '​\u0085᠎';
for (const m of ['trim', 'trimStart', 'trimEnd']) {
  const bad = !!ws[m]() || non[m]() !== non || ''[m].name !== m;
  out.push(m + ' ' + (bad ? 'core-js would replace it (ws left ' + JSON.stringify(ws[m]()) + ', non ' + (non[m]() === non) + ', name ' + ''[m].name + ')' : 'ok'));
}
// classof: Object.prototype.toString with Symbol.toStringTag
out.push('classof ' + Object.prototype.toString.call(function () { return arguments; }()) + ' ' + Object.prototype.toString.call(null) + ' ' + Object.prototype.toString.call(new Map()));
console.log(out.join('\n'));
console.log(JSON.stringify(['   '.trimEnd(), ' a '.trimEnd(), ' a '.trimStart(), '　'.trim(), ''.trimEnd(), '​x​'.trim()]));
console.log(Number('　 12  '), Number('​1'), parseFloat('  3.5'), ''.trimLeft === ''.trimStart, ''.trimRight === ''.trimEnd);
const fake = { [Symbol.match]: true, toString() { return 'x'; } };
for (const m of ['startsWith', 'endsWith', 'includes']) {
  try { 'x'[m](fake); console.log(m, 'no throw'); } catch (e) { console.log(m, e.constructor.name); }
}
