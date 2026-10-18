function direct(s) { try { return eval(s); } catch (e) { return e.name; } }
function indirect(s) { try { return (0, eval)(s); } catch (e) { return e.name; } }
function viaWindow(s) { try { return window.eval(s); } catch (e) { return e.name; } }
function show(tag) {
  var names = ["gv", "gl", "gc", "gf()", "typeof gl", "missing"];
  var p = document.createElement("p");
  p.textContent = tag + " " + names.map(function (s) { return [direct(s), indirect(s), viaWindow(s)].join(); }).join(" ");
  document.body.appendChild(p);
}
