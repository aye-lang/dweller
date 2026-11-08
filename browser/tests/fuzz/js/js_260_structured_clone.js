// structuredClone as game code and workers use it: DataCloneError, typed
// arrays and buffers (shared views stay shared), transfer, special numbers,
// maps keyed by objects, cycles through collections.
try { structuredClone(() => 1); } catch (e) { console.log(e instanceof Error, e.name); }
try { structuredClone({ a: Symbol('s') }); } catch (e) { console.log(e.name); }
try { structuredClone(); } catch (e) { console.log(e instanceof TypeError); }
const buf = new ArrayBuffer(8);
const u8 = new Uint8Array(buf, 2, 4);
u8.set([1, 2, 3, 4]);
const f64 = new Float64Array([1.5, -0, NaN]);
const c = structuredClone({ buf, u8, f64, big: 12345678901234567890n, nz: -0, nan: NaN, inf: -Infinity, bad: new Date(NaN) });
console.log(c.buf !== buf, c.buf.byteLength, c.u8.buffer === c.buf, c.u8.byteOffset, c.u8.length, [...c.u8]);
console.log(c.f64 instanceof Float64Array, Object.is(c.f64[1], -0), c.f64[2], c.big === 12345678901234567890n, Object.is(c.nz, -0), c.nan, c.inf, isNaN(c.bad.getTime()));
const moved = new ArrayBuffer(16);
new Uint8Array(moved)[0] = 7;
const t = structuredClone({ moved }, { transfer: [moved] });
console.log(moved.byteLength, moved.detached, t.moved.byteLength, new Uint8Array(t.moved)[0]);
try { structuredClone({}, { transfer: [{}] }); } catch (e) { console.log('transfer of a non-buffer', e instanceof Error); }
const key = { k: 1 };
const m = new Map([[key, new Set([key])]]);
m.set('self', m);
const mc = structuredClone(m);
const [ck] = [...mc.keys()];
console.log(mc.get('self') === mc, ck !== key, ck.k, mc.get(ck).has(ck), mc.size);
const o = structuredClone({ get g() { return 3; }, [Symbol('hidden')]: 1, d: Object.create({ inherited: 1 }) });
console.log(Object.getOwnPropertyDescriptor(o, 'g').value, Object.getOwnPropertySymbols(o).length, 'inherited' in o.d);
