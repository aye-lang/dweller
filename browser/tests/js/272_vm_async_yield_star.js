const log = [];
async function* inner() { log.push('i1'); const x = yield 1; log.push('got ' + x); yield Promise.resolve(2); return 'r'; }
async function* outer() { const v = yield* inner(); log.push('ret ' + v); const w = yield* [Promise.resolve(3), 4]; log.push('w ' + w); }
(async () => {
  const g = outer();
  for (let i = 0; i < 6; i++) { const r = await g.next('s' + i); log.push(JSON.stringify(r.done) + ':' + (r.value instanceof Promise ? 'promise' : String(r.value))); }
  const h = outer(); await h.next(); const t = await h.return('early'); log.push('return ' + t.value + ' ' + t.done);
  async function* thrower() { try { yield* inner(); } catch (e) { log.push('caught ' + e); } }
  const k = thrower(); await k.next(); const kk = await k.throw('boom'); log.push('throw ' + kk.done);
  console.log(log.join('\n'));
})();
