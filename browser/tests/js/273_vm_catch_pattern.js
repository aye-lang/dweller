const out = [];
try { throw { a: 1, b: [2, 3] }; } catch ({ a, b: [x, y = 9], c = 'd' }) { out.push(a, x, y, c); }
try { throw [1, [2]]; } catch ([p, [q], ...rest]) { out.push(p, q, rest.length); }
function f() { try { throw { v: 5 }; } catch ({ v }) { return () => v; } }
out.push(f()());
try { try { throw null; } catch ({ z }) { out.push('no'); } } catch (e) { out.push(e instanceof TypeError); }
console.log(out.join(' '));
