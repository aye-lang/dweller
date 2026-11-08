const out = [];
for (const src of ['super.x', 'super()', 'super[0]', 'return () => super.x', 'return { m() { return super.x; } }.m()']) {
  try { out.push(String(new Function(src)())); } catch (e) { out.push(e.constructor.name); }
}
for (const src of ['function f() { super.x }', '({ m() { return function () { super.x }; } })', 'class C { m() { function h() { super.x } } }']) {
  try { (0, eval)(src); out.push('ok'); } catch (e) { out.push(e.constructor.name); }
}
console.log(out.join(' '));
