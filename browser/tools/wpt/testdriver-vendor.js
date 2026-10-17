// blink: third_party/blink/web_tests/resources/testdriver-vendor.js
(function () {
  const pending = [];

  window.__dwellerEventsDone = function () {
    const resolve = pending.shift();
    if (resolve) resolve();
  };

  function send(words) {
    return new Promise(function (resolve) {
      pending.push(resolve);
      console.log('DWELLER_EVENTS ' + words.join(' '));
    });
  }

  function point(x, y) {
    return Math.round(x) + ',' + Math.round(y);
  }

  function centre(element) {
    const r = element.getClientRects()[0] || element.getBoundingClientRect();
    return [r.left + r.width / 2, r.top + r.height / 2];
  }

  const NAMED = {
    '': 'backspace', '': 'tab', '': 'return', '': 'return',
    '': 'shift', '': 'control', '': 'alt', '': 'meta',
    '': 'escape', '': 'space', '': 'end', '': 'home',
    '': 'left', '': 'up', '': 'right', '': 'down', '': 'delete',
  };

  function keyWords(key, down) {
    const named = NAMED[key] || (key === ' ' ? 'space' : null);
    const name = named || key;
    if (!down) return ['keyup:' + name];
    if (named && named !== 'space') return ['key:' + name];
    return ['key:' + name, 'text:' + (key === ' ' ? '_' : key)];
  }

  const MODIFIER = /^(shift|control|alt|meta)$/;

  function typed(keys) {
    let words = [];
    const held = [];
    for (const key of keys) {
      if (MODIFIER.test(NAMED[key])) held.unshift(key);
      words = words.concat(keyWords(key, true), MODIFIER.test(NAMED[key]) ? [] : keyWords(key, false));
    }
    for (const key of held) words = words.concat(keyWords(key, false));
    return words;
  }

  function pointerWords(source, state) {
    const words = [];
    for (const a of source.actions) {
      if (a.type === 'pointerMove') {
        let x = a.x || 0, y = a.y || 0;
        if (a.origin instanceof Element) {
          const c = centre(a.origin);
          x += c[0];
          y += c[1];
        } else if (a.origin === 'pointer') {
          x += state.x;
          y += state.y;
        }
        state.x = x;
        state.y = y;
        words.push('move:' + point(x, y));
      } else if (a.type === 'pointerDown') {
        words.push('click:' + point(state.x, state.y));
      } else if (a.type === 'pointerUp') {
        words.push('release:' + point(state.x, state.y));
      }
    }
    return words;
  }

  function sequenceWords(actions) {
    const state = {x: 0, y: 0};
    let words = [];
    for (const source of actions) {
      if (source.type === 'pointer') words = words.concat(pointerWords(source, state));
      if (source.type === 'key') {
        for (const a of source.actions) {
          if (a.type === 'keyDown') words = words.concat(keyWords(a.value, true));
          if (a.type === 'keyUp') words = words.concat(keyWords(a.value, false));
        }
      }
    }
    return words;
  }

  window.test_driver_internal.in_automation = true;

  window.test_driver_internal.click = function (element, coords) {
    const at = point(coords.x, coords.y);
    return send(['click:' + at, 'release:' + at]);
  };

  window.test_driver_internal.send_keys = function (element, keys) {
    if (document.activeElement !== element) element.focus();
    return send(typed(Array.from(keys)));
  };

  window.test_driver_internal.action_sequence = function (actions) {
    return send(sequenceWords(actions));
  };
})();
