const out = [];
for (const src of ['break;', 'continue;', 'x: { continue x; }', 'while (0) { break y; }', 'x: while (0) { (function () { break x; }); }', 'switch (1) { case 1: continue; }',
                   'x: for (;0;) { switch (1) { case 1: continue x; } }', 'a: b: while (0) { continue a; }', 'q: { break q; }', 'do { if (1) break; } while (0)']) {
  try { new Function(src); out.push('ok'); } catch (e) { out.push(e.constructor.name); }
  try { eval(src); out.push('ok'); } catch (e) { out.push(e.constructor.name); }
}
console.log(out.join(' '));
