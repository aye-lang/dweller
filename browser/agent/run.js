(function (G, request) {
  const log = [];
  let calls = 0;

  function command(cmd, args) {
    calls++;
    const r = JSON.parse(request(cmd, JSON.stringify(Object.assign({}, args, { cmd }))));
    if (!r.ok) {
      const e = new Error(r.error);
      e.call = cmd;
      e.index = calls;
      throw e;
    }
    delete r.id;
    delete r.ok;
    return r;
  }

  function text(v) {
    return typeof v === 'string' ? v : JSON.stringify(v);
  }

  function report(fields) {
    return JSON.stringify(Object.assign(fields, { log, calls }));
  }

  const type = (ref, text, opts) => command('type', Object.assign({ ref, text }, opts));
  const fns = {
    navigate: url => command('navigate', { url }),
    back: () => command('back'),
    forward: () => command('forward'),
    reload: () => command('reload'),
    snapshot: opts => command('snapshot', opts),
    read: opts => command('read', opts),
    find: text => command('find', { text }),
    click: ref => command('click', { ref }),
    type,
    fill: type,
    select: (ref, option) => command('select', { ref, option }),
    press: key => command('press', { key }),
    scroll: v => command('scroll', typeof v === 'number' ? { dy: v } : { ref: v }),
    wait: opts => command('wait', typeof opts === 'number' ? { ms: opts } : opts),
    evaluate: expression => command('eval', { expression }).result,
    screenshot: opts => command('screenshot', opts),
    tabs: () => command('tabs'),
    tabNew: url => command('tab_new', url === undefined ? {} : { url }),
    tabSelect: id => command('tab_select', { id }),
    tabClose: id => command('tab_close', id === undefined ? {} : { id }),
    log: (...values) => { log.push(values.map(text).join(' ')); },
  };
  Object.assign(G, fns);

  G.__finish = function (code) {
    try {
      const v = Function(code)();
      return report({ ok: true, value: v === undefined ? null : JSON.parse(JSON.stringify(v) ?? 'null') });
    } catch (e) {
      if (e && e.call) {
        return report({ ok: false, error: `${e.call} (call ${e.index}): ${e.message}` });
      }
      return report({ ok: false, error: 'Uncaught ' + (e instanceof Error ? `${e.name}: ${e.message}` : text(e)) });
    }
  };
  G.__failed = error => report({ ok: false, error });
  delete G.__request;
})(globalThis, globalThis.__request);
