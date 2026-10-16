self.addEventListener('message', e => {
  const d = e.data;
  if (d.kind === 'echo') postMessage({ kind: 'echo', value: d.value });
  if (d.kind === 'timer') setTimeout(() => postMessage({ kind: 'timer', value: 'fired' }), 5);
  if (d.kind === 'import') {
    importScripts('helper.js');
    postMessage({ kind: 'import', value: helper(3) });
  }
  if (d.kind === 'fetch') fetch('data.json').then(r => r.json()).then(j => postMessage({ kind: 'fetch', value: j.answer }));
  if (d.kind === 'perf') postMessage({ kind: 'perf', value: typeof performance.now() + ' ' + (self === globalThis) + ' ' + typeof TextDecoder });
  if (d.kind === 'ignored') postMessage({ kind: 'should not arrive' });
});
