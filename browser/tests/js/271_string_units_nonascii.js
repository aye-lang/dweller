var s = "aé€😀z😁\u0000b";
var n = s.length;
var out = [];

for (var i = -1; i <= n; i++) out.push(s.charCodeAt(i));
for (var i = n; i >= -1; i--) out.push(s.charCodeAt(i));
for (var i = -1; i <= n; i++) out.push(s.codePointAt(i));
for (var i = -1; i <= n; i++) out.push(s.charAt(i).length + ":" + s.charAt(i).charCodeAt(0));
for (var i = 0; i < n; i++) out.push(s[i].charCodeAt(0));
out.push(s[n], s[-1]);
console.log(out.join(","));

var order = [5, 0, 9, 3, 4, 3, 12, 1, 11, 2, 10, 6, 7, 8, 0, 12, 4];
var jumps = [];
for (var k = 0; k < order.length; k++) jumps.push(s.charCodeAt(order[k]), s.codePointAt(order[k]));
console.log(jumps.join(","));

var lone = String.fromCharCode(0xd83d) + "x" + String.fromCharCode(0xde00) + "é";
var loneUnits = [];
for (var i = 0; i < lone.length; i++) loneUnits.push(lone.charCodeAt(i), lone.codePointAt(i));
console.log(lone.length, loneUnits.join(","));

var pair = String.fromCharCode(0xd83d) + String.fromCharCode(0xde00) + "é";
console.log(pair.length, pair.codePointAt(0), pair.codePointAt(1), pair.charCodeAt(1), pair.charCodeAt(2));

var parts = [];
for (var i = 0; i <= n; i++) {
  for (var j = i; j <= n; j++) {
    var t = s.slice(i, j);
    parts.push(t.length + ":" + (t.length ? t.charCodeAt(0) + "." + t.charCodeAt(t.length - 1) : ""));
  }
}
console.log(parts.join(" "));
console.log(s.substring(3, 1).length, s.substring(2, 20).length, s.slice(-4, -1).length, s.substr(2, 4).length);

console.log(s.indexOf("z"), s.indexOf("z", 5), s.indexOf("€", 2), s.indexOf("€", 3), s.indexOf("😁"), s.indexOf("😀", 3), s.indexOf("b", 100));
console.log(s.lastIndexOf("a"), s.lastIndexOf("😀"), s.lastIndexOf("😁", 5), s.lastIndexOf("é", 1), s.lastIndexOf("é", 0));
console.log(s.includes("z", 5), s.startsWith("€", 2), s.startsWith("€", 3), s.endsWith("€", 3), s.endsWith("z", 6));

var re = /€|z|😁/g;
var m, hits = [];
while ((m = re.exec(s))) hits.push(m.index + "/" + re.lastIndex);
console.log(hits.join(" "), s.search(/z/), s.match(/é/).index);
var sticky = /😁/y;
sticky.lastIndex = 6;
console.log(sticky.test(s), sticky.lastIndex);
console.log(s.replace(/z/, "-").length, s.split("z").map(function (x) { return x.length; }).join(","));

var grown = s + s;
console.log(grown.length, grown.charCodeAt(n + 3), grown.codePointAt(n + 3), grown.slice(n - 1, n + 2).length);
console.log(Array.from(s).length, [].concat(Array.from(s)).map(function (c) { return c.codePointAt(0); }).join(","));

var big = "aé€😀".repeat(5000);
var sum = 0;
for (var i = 0; i < big.length; i++) sum += big.charCodeAt(i);
for (var i = big.length - 1; i >= 0; i--) sum += big.codePointAt(i) & 255;
for (var i = 0; i < big.length; i += 3) sum += big.charAt(i).length + big[i].length;
console.log(big.length, sum, big.slice(7777, 7782).length, big.indexOf("€", 9999), big.lastIndexOf("€", 9999));

var keep = [];
for (var r = 0; r < 200; r++) {
  var u = ("é😀" + r).repeat(20);
  keep.push(u);
  sum += u.charCodeAt(u.length - 1) + u.charCodeAt(0) + u.codePointAt(u.length >> 1);
}
for (var r = 0; r < keep.length; r += 17) sum += keep[r].charCodeAt(keep[r].length - 2);
console.log(sum);
