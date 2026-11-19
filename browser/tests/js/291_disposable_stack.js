const log = [];
const res = name => ({ [Symbol.dispose]() { log.push(name); } });
const stack = new DisposableStack();
stack.use(res("a"));
stack.defer(() => log.push("b"));
stack.adopt("c", v => log.push(v));
console.log(stack.use(null), stack.disposed);
const moved = stack.move();
console.log(stack.disposed, moved.disposed, moved instanceof DisposableStack);
moved.dispose();
moved.dispose();
console.log(log.join(), moved.disposed);
try { moved.use(res("x")); } catch (e) { console.log(e.name); }
try { new DisposableStack().use({}); } catch (e) { console.log(e.name); }
const failing = new DisposableStack();
failing.defer(() => { throw new Error("first"); });
failing.defer(() => { throw new Error("second"); });
try { failing.dispose(); } catch (e) { console.log(e.name, e instanceof Error, e.error.message, e.suppressed.message); }
const s = new SuppressedError(1, 2, "m");
console.log(s.message, s.error, s.suppressed, Object.prototype.toString.call(new DisposableStack()), DisposableStack.prototype[Symbol.dispose] === DisposableStack.prototype.dispose);
const order = [];
const async = new AsyncDisposableStack();
async.use({ async [Symbol.asyncDispose]() { order.push("async"); } });
async.use({ [Symbol.dispose]() { order.push("sync"); } });
async.disposeAsync().then(() => console.log(order.join(), async.disposed));
const m = new Map([[1, "one"]]);
console.log(m.getOrInsert(1, "x"), m.getOrInsert(2, "two"), m.getOrInsertComputed(3, k => "k" + k), m.size, new WeakMap().getOrInsert(stack, 5));
