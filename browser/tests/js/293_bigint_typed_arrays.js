var b = new BigInt64Array([1n, -2n, 2n ** 63n - 1n]), u = new BigUint64Array([2n ** 64n - 1n, 5n]);
console.log(typeof b[0], b[0], b[1], b[2], u[0], u[1], b.map(x => x * 2n).join(), Array.from(u).join());
b[0] = 2n ** 64n + 3n; u[1] = -1n; console.log(b[0], u[1]);
try { b[0] = 1 } catch (e) { console.log(e.name) }
try { b.fill(1) } catch (e) { console.log(e.name) }
console.log(b.fill(7n, 1).join(), new BigInt64Array(u.buffer)[0], u.includes(5n), b.indexOf(7n), u.at(-1));
var d = new DataView(new ArrayBuffer(16)); d.setBigInt64(0, -5n); d.setBigUint64(8, 2n ** 64n - 2n, true);
console.log(d.getBigInt64(0), d.getBigUint64(8, true), d.getBigUint64(0), d.getUint8(7));
try { d.setBigInt64(0, 1) } catch (e) { console.log(e.name) }
console.log(new BigInt64Array([9007199254740993n])[0], JSON.stringify(Array.from(new Float64Array([1.5, 2]))), new Uint8Array([300, -1])[0]);
