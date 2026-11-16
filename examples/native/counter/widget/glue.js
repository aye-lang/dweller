// The DOM functions a component imports (docs/plans/cards/V3.md), over the
// page's DOM: mountComponent(root, wasmBytes) resolves once `mount` ran.
function mountComponent(root, bytes) {
  var decoder = new TextDecoder();
  var encoder = new TextEncoder();
  var memory = null;
  var wasm = null;
  var elements = [null, root];

  function string(ptr, len) {
    return decoder.decode(new Uint8Array(memory.buffer, ptr, len));
  }
  function keep(el) {
    if (!el) return 0;
    elements.push(el);
    return elements.length - 1;
  }

  var env = {
    aye_write: function (fd, ptr, len) {
      (fd === 2 ? console.error : console.log)(string(ptr, len));
    },
    aye_abort: function () {
      throw new Error('the component aborted');
    },
    dom_create: function (tag, len) {
      return keep(document.createElement(string(tag, len)));
    },
    dom_append: function (parent, child) {
      elements[parent].appendChild(elements[child]);
    },
    dom_set_attr: function (el, name, nameLen, value, valueLen) {
      elements[el].setAttribute(string(name, nameLen), string(value, valueLen));
    },
    dom_set_text: function (el, s, len) {
      elements[el].textContent = string(s, len);
    },
    dom_on: function (el, name, len, id) {
      elements[el].addEventListener(string(name, len), function () {
        wasm.event(id);
      });
    },
    dom_value: function (el, out, capacity) {
      var value = encoder.encode(elements[el].value).subarray(0, capacity);
      new Uint8Array(memory.buffer, out, value.length).set(value);
      return value.length;
    },
    dom_set_value: function (el, s, len) {
      elements[el].value = string(s, len);
    },
    dom_query: function (from, selector, len) {
      return keep(elements[from].querySelector(string(selector, len)));
    }
  };

  return WebAssembly.instantiate(bytes, { env: env }).then(function (result) {
    wasm = result.instance.exports;
    memory = wasm.memory;
    wasm.mount(1, 0, 0);
  });
}
