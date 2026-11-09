// blink: core/frame/remote_dom_window.cc
(function () {
const G = globalThis;
const ak = G.__canvasHost;

ak.remotePost = function (w, message, origin, transfer) {
  const wire = G.__wire;
  const json = wire.send({ stage: b => ak.stageRemote(b) }, message, wire.transferOf(transfer));

  ak.postRemote(w, json, origin);
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

ak.setRemoteMessageHandler(function remoteMessage(json, origin, source, id) {
  let data;

  try {
    data = G.__wire.receive({ size: i => ak.remoteBuffer(id, i), fill: (i, b) => ak.remoteBuffer(id, i, b) }, json, bufferCount(id));
  } catch (e) {
    G.dispatchEvent(new MessageEvent('messageerror', { data: null, origin, source }));

    return;
  } finally {
    ak.dropRemoteBuffers(id);
  }

  ak.messageEvent(data, origin, source);
});
})();
