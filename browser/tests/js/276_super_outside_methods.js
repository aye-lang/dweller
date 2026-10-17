const out = [];
class A { m() { return 'A'; } static s() { return 'sA'; } }
class B extends A {
  f = super.m();
  static g = super.s();
  static { out.push('blk ' + super.s()); }
  constructor(x = () => super.m) { super(); out.push(typeof x(), this.f, B.g); }
  m() { return [super.m(), (() => super.m())(), eval('super.m()'), super['m']()].join(); }
  [`k${'1'}`]() {}
}
out.push(new B().m());
const o = { __proto__: { v: 1 }, get w() { return super.v; } };
out.push(o.w);
for (const src of ['super.x', 'super()', 'super[0]', '() => super.x']) {
  try { (0, eval)(src); out.push('ok'); } catch (e) { out.push(e.constructor.name); }
}
console.log(out.join(' '));
