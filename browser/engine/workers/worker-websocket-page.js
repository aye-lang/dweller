// spec: https://websockets.spec.whatwg.org/
(function (G) {
  const PREFIX = '\u0001ws';
  const held = new WeakMap();

  function relayed(ev) {
    if (ev.type === 'message') return { type: 'message', data: typeof ev.data === 'string' ? ev.data : '', origin: ev.origin };
    if (ev.type === 'close') return { type: 'close', code: ev.code, reason: ev.reason, wasClean: ev.wasClean };

    return { type: ev.type };
  }

  function open(sockets, id, href, protocols, send) {
    let socket;
    try {
      socket = new WebSocket(href, protocols);
    } catch (e) {
      send(PREFIX + JSON.stringify([id, 3, '', { type: 'error' }]));
      send(PREFIX + JSON.stringify([id, 3, '', { type: 'close', code: 1006, reason: '', wasClean: false }]));
      return;
    }
    for (const type of ['open', 'message', 'error', 'close']) {
      socket.addEventListener(type, ev => send(PREFIX + JSON.stringify([id, socket.readyState, socket.protocol, relayed(ev)])));
    }
    sockets.set(id, socket);
  }

  Object.defineProperty(G, '__workerWebSocket', { value: function (worker, json, send) {
    const [op, id, a, b] = JSON.parse(json);
    if (!held.has(worker)) held.set(worker, new Map());

    const sockets = held.get(worker);
    if (op === 'open') return open(sockets, id, a, b, send);

    const socket = sockets.get(id);
    if (op === 'send' && socket) return socket.send(a);
    if (op === 'close' && socket) return a === null ? socket.close() : socket.close(a, b);

    if (op === 'end') {
      for (const s of sockets.values()) s.close();
      sockets.clear();
    }
  } });
})(globalThis);
