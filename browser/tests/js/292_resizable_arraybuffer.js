const rab = new ArrayBuffer(8, { maxByteLength: 16 });
const tracking = new Uint8Array(rab), fixed = new Uint8Array(rab, 0, 4), offset = new Uint16Array(rab, 4);
tracking[7] = 9;
console.log(rab.resizable, rab.maxByteLength, tracking.length, fixed.length, offset.length);
rab.resize(16); console.log(rab.byteLength, tracking.length, fixed.length, offset.length, tracking[7], tracking[15]);
rab.resize(2); console.log(tracking.length, fixed.length, fixed.byteOffset, offset.length, offset.byteOffset, tracking[7]);
rab.resize(12); console.log(tracking.length, fixed.length, offset.length, tracking[7]);
try { rab.resize(17) } catch (e) { console.log(e.name) }
try { new ArrayBuffer(8).resize(4) } catch (e) { console.log(e.name) }
try { new ArrayBuffer(8, { maxByteLength: 4 }) } catch (e) { console.log(e.name) }
const t = rab.transfer(); console.log(t.resizable, t.maxByteLength, rab.detached, t.transferToFixedLength().resizable, new ArrayBuffer(4).resizable, new ArrayBuffer(4).maxByteLength);
var d = new DataView(new ArrayBuffer(8, { maxByteLength: 16 }), 2); d.setInt16(0, -2);
console.log(d.getInt16(0), d.byteLength, d.byteOffset, Object.prototype.toString.call(d), Object.keys(d).length);
d.buffer.resize(12); console.log(d.byteLength); d.buffer.resize(1);
try { d.byteLength } catch (e) { console.log(e.name) }
try { DataView.prototype.getInt8.call(new Int8Array(2), 0) } catch (e) { console.log(e.name) }
try { new DataView(new ArrayBuffer(2)).getInt8(Infinity) } catch (e) { console.log(e.name) }
const fixedView = new DataView(new ArrayBuffer(8), 2, 4); fixedView.setFloat64; console.log(fixedView.byteLength, fixedView.getUint8(3), DataView.prototype.getInt16.length, DataView.prototype.setInt16.length);
