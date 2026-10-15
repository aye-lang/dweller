// A closure captures x; y and the parameter b stay registers.
function counter(a, b) {
  let x = a;
  let y = b + 1;
  return function inc() {
    x += y;
    return x;
  };
}
var c = counter(1, 2);
