const out = [];
let finished = false;
const ticking = setInterval(() => { if (finished) clearInterval(ticking); }, 10);
function log(s) {
  out.push(s);
  document.getElementById('log').textContent = out.join(' | ');
}
function done(s) {
  if (s) log(s);
  finished = true;
  document.title += ' done';
}
function wait(req) {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = e => { e.preventDefault(); reject(req.error); };
  });
}
function ended(tx) {
  return new Promise(resolve => {
    tx.oncomplete = () => resolve('complete');
    tx.onabort = () => resolve('abort ' + (tx.error ? tx.error.name : 'null'));
  });
}
function fresh(name, upgrade, version) {
  return wait(indexedDB.deleteDatabase(name)).then(() => {
    const req = indexedDB.open(name, version || 1);
    req.onupgradeneeded = () => upgrade(req.result, req.transaction);
    return wait(req);
  });
}
window.onerror = () => true;
