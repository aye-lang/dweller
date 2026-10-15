// A small ray tracer: spheres, a plane, point lights, shadows, one
// reflection bounce. Written for this suite (small objects, method calls,
// float math). The image is 64x64; the checksum folds the pixel bytes.
// SMOKE (a global number, set by the smoke test) divides the frame count.
function Vec(x, y, z) {
  this.x = x;
  this.y = y;
  this.z = z;
}
Vec.prototype.add = function(o) { return new Vec(this.x + o.x, this.y + o.y, this.z + o.z); };
Vec.prototype.sub = function(o) { return new Vec(this.x - o.x, this.y - o.y, this.z - o.z); };
Vec.prototype.scale = function(k) { return new Vec(this.x * k, this.y * k, this.z * k); };
Vec.prototype.dot = function(o) { return this.x * o.x + this.y * o.y + this.z * o.z; };
Vec.prototype.norm = function() {
  var l = Math.sqrt(this.dot(this));
  return new Vec(this.x / l, this.y / l, this.z / l);
};

function Sphere(c, r, color, refl) {
  this.c = c;
  this.r = r;
  this.color = color;
  this.refl = refl;
}
Sphere.prototype.hit = function(o, d) {
  var oc = o.sub(this.c);
  var b = oc.dot(d);
  var disc = b * b - (oc.dot(oc) - this.r * this.r);
  if (disc < 0) return -1;
  var t = -b - Math.sqrt(disc);
  return t > 1e-4 ? t : -1;
};
Sphere.prototype.normal = function(p) { return p.sub(this.c).norm(); };
Sphere.prototype.colorAt = function(p) { return this.color; };

function Plane(y, a, b, refl) {
  this.y = y;
  this.a = a;
  this.b = b;
  this.refl = refl;
  this.n = new Vec(0, 1, 0);
}
Plane.prototype.hit = function(o, d) {
  if (d.y >= -1e-6) return -1;
  var t = (this.y - o.y) / d.y;
  return t > 1e-4 ? t : -1;
};
Plane.prototype.normal = function(p) { return this.n; };
Plane.prototype.colorAt = function(p) {
  return ((Math.floor(p.x) + Math.floor(p.z)) & 1) ? this.a : this.b;
};

function Scene(frame) {
  var a = frame * 0.2;
  this.objects = [
    new Plane(-1, new Vec(0.9, 0.9, 0.9), new Vec(0.2, 0.2, 0.2), 0.2),
    new Sphere(new Vec(Math.sin(a) * 1.5, 0, 5 + Math.cos(a)), 1, new Vec(0.9, 0.2, 0.2), 0.4),
    new Sphere(new Vec(-2, 0.3, 6), 1.3, new Vec(0.2, 0.9, 0.3), 0.3),
    new Sphere(new Vec(2.2, -0.3, 7), 0.7, new Vec(0.2, 0.3, 0.9), 0.5)
  ];
  this.lights = [new Vec(-5, 6, 0), new Vec(5, 8, 2)];
}
Scene.prototype.closest = function(o, d) {
  var best = null, bt = 1e30;
  for (var i = 0; i < this.objects.length; i++) {
    var t = this.objects[i].hit(o, d);
    if (t > 0 && t < bt) {
      bt = t;
      best = this.objects[i];
    }
  }
  return best === null ? null : { obj: best, t: bt };
};
Scene.prototype.trace = function(o, d, depth) {
  var h = this.closest(o, d);
  if (h === null) return new Vec(0.1, 0.15, 0.3 + 0.2 * d.y);
  var p = o.add(d.scale(h.t));
  var n = h.obj.normal(p);
  var base = h.obj.colorAt(p);
  var lum = 0.1;
  for (var i = 0; i < this.lights.length; i++) {
    var ld = this.lights[i].sub(p);
    var dist = Math.sqrt(ld.dot(ld));
    ld = ld.scale(1 / dist);
    var sh = this.closest(p, ld);
    if (sh !== null && sh.t < dist) continue;
    var k = n.dot(ld);
    if (k > 0) lum += k * 0.8;
  }
  var c = base.scale(lum);
  if (depth > 0 && h.obj.refl > 0) {
    var r = d.sub(n.scale(2 * d.dot(n)));
    var rc = this.trace(p, r, depth - 1);
    c = c.scale(1 - h.obj.refl).add(rc.scale(h.obj.refl));
  }
  return c;
};

function byte(v) {
  v = Math.floor(v * 255);
  return v < 0 ? 0 : v > 255 ? 255 : v;
}

function render(frame, size, state) {
  var scene = new Scene(frame);
  var eye = new Vec(0, 0.5, -3);
  for (var y = 0; y < size; y++) {
    for (var x = 0; x < size; x++) {
      var d = new Vec((x - size / 2) / size, (size / 2 - y) / size, 1).norm();
      var c = scene.trace(eye, d, 2);
      state.sum = (state.sum * 31 + byte(c.x)) % 1000000007;
      state.sum = (state.sum * 31 + byte(c.y)) % 1000000007;
      state.sum = (state.sum * 31 + byte(c.z)) % 1000000007;
    }
  }
}

var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
var frames = Math.max(1, Math.floor(6 / scale));
var state = { sum: 0 };
for (var f = 0; f < frames; f++) render(f, 64, state);
console.log(state.sum);
