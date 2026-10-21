const out = [];
for (const src of ['if (false) let // ASI\n{}', 'while (false) let\n{}', 'L: let\n{}', 'for (;false;) let\n{}', 'for (var x in null) let\n{}', 'do let\n{} while (false)',
                   'if (false) let {}', 'if (false) let x', 'if (false) let [a] = []', 'if (false) let\nx', '"use strict"; if (false) let\n{}', 'if (0) {} else let\n{}']) {
  try { new Function(src); out.push('ok'); } catch (e) { out.push(e.constructor.name); }
}
try { eval('if (false) let x = 1;'); } catch (e) { out.push(e.message); }
console.log(out.join(' '));
