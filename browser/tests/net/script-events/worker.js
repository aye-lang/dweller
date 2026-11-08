self.hits = [];
self.onmessage = e => import(e.data + '/cookie-module?acao&acac&worker').then(() => postMessage(self.lastCookie), err => postMessage(err.name));
