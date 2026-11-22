// Deleting a property keeps an object's own keys, values and order right
// whether it rolls the layout back (the last key) or rebuilds it (another).
function show(name, o) { console.log(name, JSON.stringify(Object.keys(o)), JSON.stringify(o)); }

const last = { a: 1, b: 2, c: 3 };
delete last.c;
show('last', last);
last.c = 30;
last.d = 4;
show('last again', last);

const mid = { a: 1, b: 2, c: 3 };
delete mid.b;
show('middle', mid);
mid.b = 20;
show('middle again', mid);

const twins = [{ x: 1, y: 2 }, { x: 1, y: 2 }];
delete twins[0].y;
show('twin 0', twins[0]);
show('twin 1', twins[1]);
twins[0].y = 5;
show('twin 0 again', twins[0]);

function readA(o) { return o.a; }
const shapes = [{ a: 1, z: 0 }, { a: 2 }, { a: 3, q: 1 }];
delete shapes[0].z;
delete shapes[2].q;
let sum = 0;
for (let i = 0; i < 3000; i++) sum += readA(shapes[i % 3]);
console.log('reads', sum);

const proto = { m() { return 'm'; }, n() { return 'n'; } };
const child = Object.create(proto);
console.log('before', child.m(), child.n());
delete proto.n;
console.log('after', child.m(), typeof child.n);
proto.n = () => 'N';
console.log('again', child.n());

Function.prototype.zz = 1;
delete Function.prototype.zz;
function f(v) { return v; }
let total = 0;
for (let i = 0; i < 1000; i++) total += f.call(null, i);
console.log('call', total, 'zz' in Function.prototype);

const many = {};
for (let i = 0; i < 40; i++) many['k' + i] = i;
for (let i = 0; i < 40; i += 3) delete many['k' + i];
show('many', many);

const acc = { get g() { return 'g'; }, v: 1 };
delete acc.v;
console.log('accessor', acc.g, Object.getOwnPropertyDescriptor(acc, 'g').get !== undefined);
