// blink: core/frame/remote_dom_window.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;
const ports = new Map();
const isPort = v => typeof G.MessagePort === 'function' && v instanceof G.MessagePort;
const stage = { stage: b => ak.stageRemote(b) };

// blink: core/messaging/message_port.cc (Entangle: a port whose other end is in another process)
const bind = (port, tab, key) => {
  port._remote = { tab, key };
  ports.set(tab + ':' + key, port);
  if (tab === 0) ak.portOpen(tab, key);
};

const movable = p => {
  if (p._remote || p._closed || !p._other) throw new G.DOMException('A MessagePort could not be transferred to another process.', 'DataCloneError');
};

ak.remotePost = function (w, message, origin, transfer) {
  const wire = G.__wire;
  const list = Array.from(wire.transferOf(transfer));
  const moved = list.filter(isPort);

  moved.forEach(movable);

  const keys = moved.map(() => ak.newPortKey());
  const json = wire.send({ stage: stage.stage, port: p => moved.indexOf(p) }, message, list);

  ak.postRemote(w, json, origin, keys.join(','));
  moved.forEach((p, i) => {
    const kept = p._other;

    p._other = null;
    p._closed = true;
    kept._other = null;
    bind(kept, 0, keys[i]);
  });
};

ak.portSend = function (port, data) {
  ak.postPort(port._remote.tab, port._remote.key, G.__wire.send(stage, data, []));
};

ak.remoteFocus = function (w) {
  for (const f of document.querySelectorAll('iframe, frame')) {
    if (f.contentWindow === w) return f.focus();
  }
};

const bufferCount = id => {
  let n = 0;
  while (ak.remoteBuffer(id, n) >= 0) n++;

  return n;
};

const received = (json, id, made) => {
  try {
    return [G.__wire.receive({ size: i => ak.remoteBuffer(id, i), fill: (i, b) => ak.remoteBuffer(id, i, b) }, json, bufferCount(id), made)];
  } catch (e) {
    return null;
  } finally {
    ak.dropRemoteBuffers(id);
  }
};

const toPort = (json, id, tab, key) => {
  const port = ports.get(tab + ':' + key);
  const got = received(json, id, []);

  if (port && got && !port._closed) port._deliver(got[0]);
};

ak.setRemoteMessageHandler(function remoteMessage(json, origin, source, id, tab, keys, key) {
  if (key) return toPort(json, id, tab, key);

  const made = keys ? keys.split(',').map(k => {
    const p = new G.MessagePort();

    bind(p, tab, Number(k));

    return p;
  }) : [];
  const got = received(json, id, made);

  if (!got) return G.dispatchEvent(new MessageEvent('messageerror', { data: null, origin, source }));

  ak.messageEvent(got[0], origin, source, made);
});
})();
