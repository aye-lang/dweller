const out = [];
for (const src of ['for (let x = 3 in {}) {}', 'for (const x = 3 in {}) {}', 'for (var x = 3 in {}) {}', '"use strict"; for (var x = 3 in {}) {}',
                   'for (var [x] = [] in {}) {}', 'for (var x = 3 of []) {}', 'for (let x = 3 of []) {}', 'for (var x in {}) {}']) {
  try { new Function(src); out.push('ok'); } catch (e) { out.push(e.constructor.name); }
}
try { eval('for (let x = 3 in {}) {}'); } catch (e) { out.push(e.message); }
console.log(out.join(' '));
