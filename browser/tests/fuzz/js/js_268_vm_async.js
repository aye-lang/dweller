async function add(x) {
  const a = await x;
  const b = await Promise.resolve(a + 1);
  return a + b;
}

async function guarded() {
  try {
    await Promise.reject(new Error("boom"));
  } catch (e) {
    console.log("caught", e.message);
  } finally {
    console.log("finally");
  }
  return 7;
}

const badDefault = async (a = (() => { throw new TypeError("param"); })()) => a;
const o = { async m() { return this === o; } };

class C {
  async sum(n) {
    let s = 0;
    for (let i = 0; i < n; i++) s += await i;
    return s;
  }
  static async *range(n) {
    for (let i = 0; i < n; i++) yield await i;
  }
}

async function* gen() {
  yield 1;
  const x = await 2;
  try {
    yield x;
  } finally {
    console.log("gen finally");
  }
}

const iterable = {
  [Symbol.asyncIterator]() {
    let i = 0;
    return {
      next() { i++; return Promise.resolve({ value: i, done: i > 3 }); },
      return() { console.log("return called"); return {}; },
    };
  },
};

async function main() {
  console.log("add", await add(1));
  console.log("guarded", await guarded());
  try {
    await badDefault();
  } catch (e) {
    console.log("badDefault", e.constructor.name, e.message);
  }
  console.log("method", await o.m());
  console.log("sum", await new C().sum(5));
  for await (const v of C.range(3)) console.log("range", v);
  for await (const v of gen()) {
    console.log("gen", v);
    if (v === 2) break;
  }
  for await (const v of [Promise.resolve(10), 20]) console.log("sync", v);
  for await (const v of iterable) {
    if (v === 2) break;
    console.log("async", v);
  }
  try {
    for await (const v of iterable) throw new Error("body " + v);
  } catch (e) {
    console.log("thrown", e.message);
  }
  let n = 0;
  outer: for (const q of [1, 2]) {
    for await (const v of iterable) {
      n += q * v;
      continue outer;
    }
  }
  console.log("labels", n);
  const many = [];
  for (let i = 0; i < 300; i++) many.push(add(i));
  const all = await Promise.all(many);
  console.log("many", all.length, all[299]);
}

main().then(() => console.log("done"));
console.log("sync end");
