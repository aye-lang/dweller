// three.js r170 (vendor/three/three.cjs, MIT): a scene of 2000 Object3Ds
// updated per frame, and Vector3/Matrix4/Quaternion math loops.
// The runner (browser/tools/jsbench.py) replaces the next line with the
// contents of the file, wrapped as a CommonJS module bound to `THREE`.
// @include THREE vendor/three/three.cjs
var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
var FRAMES = Math.floor(100 / scale);
var MATH = Math.floor(100000 / scale);

// A deterministic generator (the same in Node).
var seed = 12345;
function rnd() {
  seed = (seed * 1103515245 + 12345) & 0x7fffffff;
  return seed / 0x7fffffff;
}

var scene = new THREE.Scene();
var objects = [];
for (var p = 0; p < 100; p++) {
  var parent = new THREE.Object3D();
  parent.position.set(p % 10, Math.floor(p / 10), 0);
  scene.add(parent);
  for (var c = 0; c < 19; c++) {
    var child = new THREE.Object3D();
    child.position.set(c, c * 0.5, -c);
    parent.add(child);
    objects.push(child);
  }
  objects.push(parent);
}
// 100 parents + 1900 children = 2000 objects.
var axis = new THREE.Vector3(0.3, 0.8, 0.5).normalize();
var q = new THREE.Quaternion();
for (var f = 0; f < FRAMES; f++) {
  var t = f * 0.05;
  for (var i = 0; i < objects.length; i++) {
    var o = objects[i];
    o.position.x = Math.sin(t + i) * 5;
    o.position.y = Math.cos(t + i * 0.5) * 5;
    o.rotation.z = t + i * 0.01;
    q.setFromAxisAngle(axis, t * 0.5 + i * 0.001);
    o.quaternion.copy(q);
    var s = 1 + 0.1 * Math.sin(t + i);
    o.scale.set(s, s, s);
  }
  scene.updateMatrixWorld();
}
var sum = 0;
for (var i = 0; i < objects.length; i++) {
  var e = objects[i].matrixWorld.elements;
  for (var k = 0; k < 16; k++) sum += e[k];
}
console.log('scene ' + Math.round(sum * 1000));

var v = new THREE.Vector3();
var w = new THREE.Vector3(1, 2, 3);
var m1 = new THREE.Matrix4().makeRotationFromQuaternion(q.setFromAxisAngle(axis, 0.7));
var m2 = new THREE.Matrix4().makeTranslation(1, 2, 3);
var m3 = new THREE.Matrix4();
var qa = new THREE.Quaternion(0, 0, 0, 1);
var qb = new THREE.Quaternion().setFromAxisAngle(axis, 0.01);
var acc = 0;
for (var n = 0; n < MATH; n++) {
  v.set(n & 15, (n >> 4) & 15, n & 7);
  v.applyMatrix4(m1);
  w.add(v).normalize().multiplyScalar(3);
  acc += v.dot(w) + v.clone().cross(w).length();
  m3.multiplyMatrices(m1, m2);
  acc += m3.elements[12];
  qa.multiply(qb).normalize();
  acc += qa.w;
}
console.log('math ' + Math.round(acc));
