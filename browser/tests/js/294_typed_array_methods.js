const err = f => { try { f(); return "no error"; } catch (e) { return e.constructor.name; } };
const TA = Object.getPrototypeOf(Int8Array);
const a = new Int16Array([3, 1, 2]);
console.log(Object.getOwnPropertyNames(a).join(), Object.keys(Object.getOwnPropertyDescriptors(TA.prototype)).sort().join());
console.log(err(() => TA.prototype.at.call([1], 0)), err(() => TA.prototype.length), err(() => new TA()), err(() => TA()), TA.prototype[Symbol.toStringTag], a[Symbol.toStringTag]);
console.log(a.at(-1), a.toSorted().join(), a.toReversed().join(), a.with(0, 9).join(), a.join("-"), a.includes(2), a.indexOf(1), a.map(x => x * 2).join(), a.filter(x => x > 1).join());
class My extends Float32Array { static get [Symbol.species]() { return Uint8Array; } }
const m = new My([1.5, 300, -1]);
console.log(m.map(x => x).constructor.name, m.slice(1).join(), m.subarray(1).constructor.name, m.filter(() => true).join());
const detached = new Uint8Array(4); detached.buffer.transfer();
console.log(err(() => detached.fill(1)), err(() => detached.at(0)), err(() => detached.values()), detached.length, detached.byteLength, detached.byteOffset);
console.log(Uint8Array.from("123").join(), Uint8Array.of(1, 2).join(), Int8Array.from({ length: 2, 0: 5 }, x => (x || 0) + 1).join(), err(() => Uint8Array.from.call({}, [])), Float64Array.BYTES_PER_ELEMENT, Object.getOwnPropertyDescriptor(Float64Array, "BYTES_PER_ELEMENT").writable);
console.log(new Float64Array([NaN, 1, -0, 0, -Infinity]).sort().join(), new Uint8Array([5, 1, 4]).sort((x, y) => y - x).join(), [...new Uint8Array([7, 8]).entries()].join("|"));
const big = new Uint8Array(8); big.set([1, 2, 3]); big.copyWithin(4, 0, 3); console.log(big.join(), big.subarray(2, 6).join(), big.reverse().join(), big.reduce((s, x) => s + x, 0), big.lastIndexOf(3));
console.log(err(() => new Uint8Array(new ArrayBuffer(3), 1, 4)), err(() => new Int16Array(new ArrayBuffer(3))), err(() => new BigInt64Array(new Int8Array(1))), err(() => a.set([1, 2, 3, 4])), new Uint8Array(new Uint16Array([257, 2])).join());
