// A catch binding is a register unless a closure captures it.
function f() {
  try {
    g();
  } catch (e) {
    console.log(e);
  }
  try {
    g();
  } catch ({ message, code }) {
    return () => message + code;
  }
}
