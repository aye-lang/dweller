// ArrayBuffer on one native byte store (P11.3): element conversions,
// two views aliasing one buffer, DataView endianness, set/subarray with
// overlap, fill/copyWithin/slice, transfer and detach errors, and
// structuredClone keeping views on one cloned buffer.
function show(a) { return a.constructor.name + "[" + Array.from(a).join(",") + "]"; }
function err(f) { try { f(); return "no error"; } catch (e) { return e.constructor.name; } }

// conversions
var vals = [0, -1, 1.5, 2.5, -2.5, 255.5, 256, 300, -129, 65536 + 7, 2147483648, NaN, Infinity, -Infinity, 1e20];
["Int8", "Uint8", "Uint8Clamped", "Int16", "Uint16", "Int32", "Uint32"].forEach(function (n) {
  var a = new globalThis[n + "Array"](vals.length);
  for (var i = 0; i < vals.length; i++) a[i] = vals[i];
  console.log(show(a));
});
var f = new Float32Array([0.1, 1 / 3, 16777217, 3.4028235e38, 1e39, -0, NaN]);
console.log(show(f), Object.is(f[5], -0));
var g = new Float64Array([0.1, -0, 1e308]);
console.log(show(g));

// aliasing: two views over one buffer
var buf = new ArrayBuffer(8);
var u8 = new Uint8Array(buf), i32 = new Int32Array(buf, 4, 1);
i32[0] = -2;
console.log(show(u8));
u8[4] = 1;
console.log(i32[0], new Uint16Array(buf)[2]);

// DataView endianness
var dv = new DataView(new ArrayBuffer(12));
dv.setUint16(0, 0x1234); dv.setUint16(2, 0x1234, true);
dv.setFloat32(4, 1.5); dv.setFloat64(4, -2, true);
console.log(show(new Uint8Array(dv.buffer)));
console.log(dv.getUint16(0), dv.getUint16(0, true), dv.getInt8(0), dv.getFloat64(4, true));
console.log(err(function () { dv.getUint32(10); }), err(function () { dv.setInt8(-1, 0); }));

// set and subarray, overlapping
var a = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
a.set(a.subarray(0, 4), 2);
console.log(show(a));
a.set(a.subarray(4), 0);
console.log(show(a));
var w = new Uint16Array([0x0102, 0x0304, 0x0506, 0x0708]);
var over = new Uint8Array(w.buffer, 1, 4);
w.set(over, 0);
console.log(show(w));
var fl = new Float32Array(4);
fl.set(new Int16Array([-1, 2, -3]), 1);
fl.set([9, "8"]);
console.log(show(fl), err(function () { fl.set([1, 2], 3); }), err(function () { fl.set(new Uint8Array(5)); }));
var sub = a.subarray(2, 5);
sub[0] = 99;
console.log(a[2], sub.byteOffset, sub.length, sub.buffer === a.buffer);

// fill, copyWithin, slice
var c = new Int16Array(6).fill(-3, 1, 4);
console.log(show(c), show(new Float64Array(3).fill(0.25)), show(new Uint8ClampedArray(3).fill(300)));
console.log(show(new Uint8Array([1, 2, 3, 4, 5]).copyWithin(1, 0, 3)), show(new Uint8Array([1, 2, 3, 4, 5]).copyWithin(0, 2)));
var s = a.slice(1, 4);
s[0] = 0;
console.log(show(s), a[1], s.buffer !== a.buffer, show(new Int8Array([1, -2, 3]).slice(-2)));
console.log(show(new Int8Array(new Uint8Array([200, 100]))), show(new Uint8Array(new Float32Array([1.7, -1]))));
console.log(new ArrayBuffer(10).slice(2, -2).byteLength, err(function () { new ArrayBuffer(-1); }));

// transfer and detach
var t = new ArrayBuffer(4);
var tv = new Uint8Array(t);
tv[0] = 7;
var moved = t.transfer(6);
console.log(t.detached, t.byteLength, tv.length, tv[0], tv.byteOffset, moved.byteLength, new Uint8Array(moved)[0]);
console.log(err(function () { new Uint8Array(t); }), err(function () { t.slice(0); }), err(function () { new DataView(moved).getInt8(9); }));
var dt = new DataView(t);
console.log(err(function () { dt.getInt8(0); }), err(function () { tv.fill(1); }), err(function () { tv.set([1]); }));

// structuredClone: views keep sharing one (new) buffer
var src = new ArrayBuffer(4);
var pair = [new Uint8Array(src), new Uint16Array(src, 2, 1)];
pair[0][2] = 1;
var cl = structuredClone(pair);
cl[1][0] = 0xffff;
console.log(cl[0].buffer === cl[1].buffer, cl[0].buffer !== src, show(cl[0]), show(pair[0]));

// TextEncoder/TextDecoder over the bytes
var enc = new TextEncoder().encode("aé€😀\ud800z");
console.log(show(enc));
var dec = new TextDecoder();
console.log(dec.decode(enc) === "aé€😀�z", dec.decode(new Uint8Array([0xEF, 0xBB, 0xBF, 0x68, 0x69])));
var bad = dec.decode(new Uint8Array([0x61, 0xC0, 0x80, 0xE2, 0x82, 0x41, 0xF0, 0x9F, 0x98, 0xED, 0xA0, 0x80, 0xF4, 0x90, 0x80, 0x80, 0x62]));
console.log(Array.from(bad, function (ch) { return ch.codePointAt(0).toString(16); }).join(" "));
console.log(dec.decode(new Uint16Array([0x6968]).buffer), dec.decode(new DataView(new Uint8Array([0x78, 0x79, 0x7a]).buffer, 1)));
var into = new Uint8Array(4);
console.log(JSON.stringify(new TextEncoder().encodeInto("a€b", into)), show(into));
