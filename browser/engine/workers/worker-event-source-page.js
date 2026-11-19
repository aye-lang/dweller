// spec: https://html.spec.whatwg.org/multipage/server-sent-events.html
(function (G) {
  const PREFIX = '\u0001es';
  const held = new WeakMap();

  function relayed(ev) {
    if (!(ev instanceof MessageEvent)) return { type: ev.type };

    return { type: ev.type, data: ev.data, origin: ev.origin, lastEventId: ev.lastEventId };
  }

  function open(sources, id, href, credentials, send) {
    const source = new EventSource(href, { withCredentials: credentials });

    Object.defineProperty(source, '_fire', { value: ev => send(PREFIX + JSON.stringify([id, source.readyState, relayed(ev)])) });
    sources.set(id, source);
  }

  Object.defineProperty(G, '__workerEventSource', { value: function (worker, json, send) {
    const [op, id, href, credentials] = JSON.parse(json);
    if (!held.has(worker)) held.set(worker, new Map());

    const sources = held.get(worker);
    if (op === 'open') return open(sources, id, href, credentials, send);

    for (const [key, source] of sources) {
      if (op === 'close' && key !== id) continue;

      source.close();
      sources.delete(key);
    }
  } });
})(globalThis);
