var smokeScale = (typeof SMOKE === 'number') ? SMOKE : 1;
function unitsOfNonAscii() {
  var s = "a\u00e9\u20ac\ud83d\ude00".repeat(Math.floor(40000 / smokeScale));
  var len = s.length;
  var sum = 0;
  for (var i = 0; i < len; i++) sum += s.charCodeAt(i);
  for (var i = len - 1; i >= 0; i--) sum += s.charCodeAt(i) & 255;
  for (var i = 0; i < len; i++) sum += s.codePointAt(i) & 255;
  for (var i = 0; i < len; i++) sum += s[i].length;
  for (var i = 0; i < len; i += 7) sum += s.slice(i, i + 3).length;
  for (var i = 0; i < len; i += 10) sum += s.indexOf("\ud83d\ude00", i);
  for (var i = len - 1; i >= 0; i -= 7) sum += s.lastIndexOf("€", i);
  return sum;
}
console.log(unitsOfNonAscii());
