class A {
  #p = 1;
  static s = 'st';
  static { this.blk = 'b'; }
  x = 10;
  constructor(y) { this.y = y; }
  get g() { return this.y * 2; }
  set g(v) { this.y = v; }
  m() { return 'A.m' + this.#p; }
  static sm() { return 'A.sm'; }
  ['comp' + 1]() { return 'c1'; }
  #priv() { return 'priv'; }
  callPriv() { return this.#priv(); }
  has(o) { return #p in o; }
}
class B extends A {
  z = 5;
  constructor(y) {
    let f = () => this;
    try { this.q = 1; } catch (e) { console.log('tdz', e instanceof ReferenceError); }
    super(y + 1);
    console.log(f() === this, this.x, this.z);
  }
  m() { return 'B>' + super.m(); }
  static sm() { return 'B>' + super.sm(); }
  get g() { return super.g + 1; }
}
class C extends B {}
const b = new B(1);
console.log(b.y, b.g, b.m(), B.sm(), b.comp1(), b.callPriv(), b.has(b), b.has({}), A.s, A.blk);
const c = new C(3);
console.log(c instanceof C, c instanceof A, c.y, c.z, c.x);
const E = class Named { who() { return Named.name; } };
console.log(new E().who());
class R extends Array { sum() { return this.reduce((a, b) => a + b, 0); } }
const r = new R(); r.push(1, 2, 3); console.log(r.sum(), r.length, Array.isArray(r));
class Ret extends A { constructor() { return {o: 1}; } }
console.log(JSON.stringify(new Ret()));
function mk(k) { return class { [k]() { return k; } }; }
console.log(new (mk('hi'))().hi());
class S extends A { constructor(...a) { super(...a); } }
console.log(new S(7).y);
try { A(); } catch (e) { console.log(e.message); }
class N extends null {}
console.log(typeof N, Object.getPrototypeOf(N.prototype));
let o = { __proto__: { hi() { return 'proto hi'; } }, hi() { return super.hi() + '!'; } };
console.log(o.hi());
class F {
  static #count = 0;
  static inc() { return ++F.#count; }
  arrow = () => this.tag;
  tag = 'f';
  viaSuper = super.toString === Object.prototype.toString;
  constructor() { this.nt = new.target === F; }
}
F.inc();
const f = new F();
console.log(F.inc(), f.arrow.call(null), f.viaSuper, f.nt);
{
  class Local { v() { return Local.name; } }
  const fns = [];
  for (let i = 0; i < 3; i++) {
    class K { n() { return i; } }
    fns.push(() => new K().n());
  }
  console.log(new Local().v(), fns.map((g) => g()).join(','));
}
class Base {
  constructor() { this.kind = new.target.name; }
  static create() { return new this(); }
}
class Derived extends Base {
  constructor() { super(); this.extra = super.constructor === Base; }
}
console.log(Derived.create().kind, new Derived().extra);
class Twice extends Base {
  constructor() { const s = () => super(); s(); }
}
console.log(new Twice().kind);
function hot() {
  class P { constructor(x) { this.x = x; } get dbl() { return this.x * 2; } }
  class Q extends P { constructor(x) { super(x + 1); } get dbl() { return super.dbl + 1; } }
  let s = 0;
  for (let i = 0; i < 2000; i++) {
    s += new Q(i).dbl;
  }
  return s;
}
console.log(hot());
try { new (class extends Base { constructor() { this.a = 1; } })(); } catch (e) { console.log(e.constructor.name); }
try { class X extends 3 {} } catch (e) { console.log(e.constructor.name); }
