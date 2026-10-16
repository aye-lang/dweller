// Closures and contexts in compiled code (P4.66).
function counter(a, b) {
  let x = a;
  let y = b + 1;
  return function inc() {
    x += y;
    return x;
  };
}
const c1 = counter(1, 2);
console.log(c1(), c1(), c1());

function makeAdders(n) {
  const fns = new Array();
  for (let i = 0; i < n; i++) {
    fns.push(() => i * 10);
  }
  return fns;
}
const adders = makeAdders(4);
console.log(adders.at(0)(), adders.at(1)(), adders.at(2)(), adders.at(3)());

function loopBreak() {
  const fns = new Array();
  for (let i = 0; i < 10; i++) {
    let j = i * 2;
    fns.push(function () { return j + i; });
    if (i == 3) {
      break;
    }
    if (i == 1) {
      continue;
    }
  }
  let s = 0;
  for (let k = 0; k < fns.length; k++) {
    s = s * 100 + fns.at(k)();
  }
  return s;
}
console.log(loopBreak());

function blocks(v) {
  let out = 0;
  {
    let a = v + 1;
    const get = () => a;
    a += 1;
    out += get();
  }
  outer: for (let i = 0; i < 3; i++) {
    {
      let b = i;
      const g = () => b;
      if (i == 2) {
        break outer;
      }
      out = out * 10 + g();
    }
  }
  return out;
}
console.log(blocks(5));

function hoisted(n) {
  return twice(n) + helper();
  function twice(k) { return k * 2; }
  function helper() { return n; }
}
console.log(hoisted(7));

function inBlock(n) {
  let r = 0;
  if (n > 0) {
    function sq(k) { return k * k; }
    r = sq(n);
  }
  return r;
}
console.log(inBlock(9));

const fact = function f(n) { return n <= 1 ? 1 : n * f(n - 1); };
console.log(fact(10));
const named = function g() { return typeof g; };
console.log(named());

function tdzBlock() {
  {
    const early = () => z;
    early();
    let z = 1;
  }
}
function tdz() {
  const read = () => late;
  let late = 3;
  return read();
}
try { tdzBlock(); } catch (e) { console.log(e.name, e.message); }
console.log(tdz());
function tdzWrite() {
  {
    const early = () => { w = 2; };
    early();
    let w = 1;
  }
}
try { tdzWrite(); } catch (e) { console.log(e.name, e.message); }

function Outer() {
  this.n = 4;
  const reads = () => this.n + arguments.length;
  const nested = () => () => this.n;
  return reads() * 10 + nested()();
}
console.log(new Outer(1, 2).constructor === Outer, Outer.call({}, 1));

function sloppy(a, b) {
  return arguments.length + a + b;
}
console.log(sloppy(1, 2, 3));

function shared() {
  let count = 0;
  const inc = () => count++;
  const dec = () => --count;
  inc(); inc(); inc(); dec();
  return count;
}
console.log(shared());

function deep(a) {
  return function (b) {
    return function (c) {
      a += 1;
      return a * 100 + b * 10 + c;
    };
  };
}
const d1 = deep(1)(2);
console.log(d1(3), d1(4));

function captureParam(p) {
  const get = () => p;
  p = p * 3;
  return get();
}
console.log(captureParam(5));

function constWrite() {
  const k = 1;
  const f = () => { k = 2; };
  f();
}
try { constWrite(); } catch (e) { console.log(e.name); }

let total = 0;
for (let i = 0; i < 2000; i++) {
  total += counter(i, 1)();
}
console.log(total);
