// The ogles harness's readPixels comparison (WebGL CTS conformance/ogles/ogles-utils.js compareImages).
var TOLERANCE = 24;
function compareImages(refData, tstData, width, height, diff) {
  function isPixelSame(offset) {
    return Math.abs(refData[offset] - tstData[offset]) <= TOLERANCE &&
        Math.abs(refData[offset + 1] - tstData[offset + 1]) <= TOLERANCE &&
        Math.abs(refData[offset + 2] - tstData[offset + 2]) <= TOLERANCE;
  }
  var bad = 0;
  for (var yy = 0; yy < height; ++yy) {
    for (var xx = 0; xx < width; ++xx) {
      var offset = (yy * width + xx) * 4;
      var diffOffset = ((height - yy - 1) * width + xx) * 4;
      diff[diffOffset] = 0;
      diff[diffOffset + 1] = 0;
      diff[diffOffset + 2] = 0;
      diff[diffOffset + 3] = 255;
      if (!isPixelSame(offset)) {
        diff[diffOffset] = 255;
        bad++;
      }
    }
  }
  return bad;
}
var scale = (typeof SMOKE === 'number') ? SMOKE : 1;
var w = 500, h = 500;
var ref = new Uint8Array(w * h * 4), tst = new Uint8Array(w * h * 4), diff = new Uint8ClampedArray(w * h * 4);
for (var i = 0; i < ref.length; i++) { ref[i] = i & 255; tst[i] = (i * 7 + (i >> 9)) & 255; }
var sum = 0;
for (var k = 0; k < Math.max(1, Math.floor(8 / scale)); k++) sum += compareImages(ref, tst, w, h, diff) + diff[k * 4];
console.log(sum);
