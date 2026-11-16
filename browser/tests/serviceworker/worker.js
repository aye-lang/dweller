const seen = [];
self.addEventListener('install', e => { seen.push('install'); e.waitUntil(new Promise(r => setTimeout(r, 5))); });
self.addEventListener('activate', e => { seen.push('activate'); e.waitUntil(clients.claim()); });
self.addEventListener('fetch', e => {
  const url = new URL(e.request.url);
  if (url.pathname.endsWith('/made.txt')) e.respondWith(new Response('made by the worker for ' + e.request.method + ' ' + e.request.destination, { status: 201, headers: { 'Content-Type': 'text/plain', 'X-From': 'worker' } }));
  else if (url.pathname.endsWith('/proxied.txt')) e.respondWith(fetch(new URL('plain.txt', url)).then(r => r.text()).then(t => new Response('proxied: ' + t.trim())));
  else if (url.pathname.endsWith('/broken.txt')) e.respondWith(Promise.reject(new Error('no')));
  else if (url.pathname.endsWith('/page.html') && url.search === '?made') e.respondWith(new Response('<!doctype html><title>made done</title><pre id=log>a page made by the worker</pre><script>document.getElementById("log").textContent += ", controlled " + (navigator.serviceWorker.controller !== null) + ", " + ' + JSON.stringify(e.request.mode + ' ' + e.request.destination) + ';fetch("made.txt").then(r => r.text()).then(t => { document.getElementById("log").textContent += ", " + t; });</scr' + 'ipt>', { headers: { 'Content-Type': 'text/html' } }));
});
self.addEventListener('message', e => {
  if (e.data.ask === 'seen') e.source.postMessage({ seen: seen.join(','), scope: new URL(registration.scope).pathname, isWorker: self instanceof ServiceWorkerGlobalScope });
  if (e.data.ask === 'clients') clients.matchAll().then(list => e.source.postMessage({ clients: list.map(c => new URL(c.url).pathname.split('/').pop()).sort().join(',') }));
});
