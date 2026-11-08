const out = document.getElementById('out');
const log = s => { out.textContent += s + '\n'; };
const at = 'http://site.test:' + location.port;
(async () => {
  await new Promise(done => {
    const s = document.createElement('script');
    s.onload = () => { log('script: ' + ran); done(); };
    s.onerror = () => { log('script: error'); done(); };
    s.src = at + '/x.js';
    document.head.appendChild(s);
  });
  await fetch(at + '/data').then(r => r.text()).then(t => log('fetch: ' + t), e => log('fetch: ' + e.name));
  log('done');
})();
