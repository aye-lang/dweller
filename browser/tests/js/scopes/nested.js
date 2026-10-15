// Three levels: the innermost function reads a binding of the outermost
// across the middle one; the middle one's own local stays a register.
function outer(p) {
  var shared = p * 2;
  var mine = 0;
  function middle(q) {
    var local = q;
    function inner() {
      return shared + local;
    }
    return inner;
  }
  return middle(mine);
}
