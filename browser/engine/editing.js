// spec: https://w3c.github.io/editing/docs/execCommand/
// chromium: third_party/blink/renderer/core/editing/commands/editor_command.cc
(function () {
const G = globalThis;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });
const getter = (o, k, get, set) => Object.defineProperty(o, k, { get, set, enumerable: true, configurable: true });
const INLINE = { bold: ['b', 'strong'], italic: ['i', 'em'], underline: ['u'] };
const INPUT_TYPES = { inserttext: 'insertText', delete: 'deleteContentBackward', forwarddelete: 'deleteContentForward', insertlinebreak: 'insertLineBreak', insertparagraph: 'insertParagraph', inserthtml: 'insertHTML', bold: 'formatBold', italic: 'formatItalic', underline: 'formatUnderline' };
let design = false;

function editableState(el) {
  const v = el.getAttribute('contenteditable');
  if (v === null) return 'inherit';

  const s = v.toLowerCase();

  return s === '' || s === 'true' ? 'true' : s === 'plaintext-only' || s === 'false' ? s : 'inherit';
}

function hostOf(node) {
  let host = null;
  for (let n = node.nodeType === 1 ? node : node.parentNode; n && n.nodeType === 1; n = n.parentNode) {
    const s = editableState(n);
    if (s === 'false') break;
    if (s !== 'inherit') host = n;
  }

  return host || (design ? document.body : null);
}

getter(HTMLElement.prototype, 'contentEditable', function () { return editableState(this); }, function (v) {
  const s = String(v).toLowerCase();
  if (s === 'inherit') return this.removeAttribute('contenteditable');
  if (s !== 'true' && s !== 'false' && s !== 'plaintext-only') throw new DOMException("Failed to set the 'contentEditable' property on 'HTMLElement': The value provided ('" + v + "') is not one of 'true', 'false', 'plaintext-only', or 'inherit'.", 'SyntaxError');

  this.setAttribute('contenteditable', s);
});
getter(HTMLElement.prototype, 'isContentEditable', function () { return hostOf(this) !== null; });
getter(Document.prototype, 'designMode', function () { return design ? 'on' : 'off'; }, function (v) {
  const s = String(v).toLowerCase();
  if (s === 'on' || s === 'off') design = s === 'on';
});

function target() {
  const sel = getSelection();
  if (sel.rangeCount === 0) return null;

  const range = sel.getRangeAt(0);
  const host = hostOf(range.startContainer);

  return host && host === hostOf(range.endContainer) ? { sel, range, host } : null;
}

function caretAfter(sel, node) {
  sel.collapse(node.parentNode, Array.prototype.indexOf.call(node.parentNode.childNodes, node) + 1);
}

function insertNodes(t, node) {
  t.range.deleteContents();

  const last = node.nodeType === 11 ? node.lastChild : node;
  t.range.insertNode(node);
  if (last) caretAfter(t.sel, last);
}

function insertText(t, data) {
  t.range.deleteContents();

  const c = t.range.startContainer, o = t.range.startOffset;
  if (c.nodeType === 1 && c.childNodes.length === 1 && c.firstChild.localName === 'br') {
    c.removeChild(c.firstChild);
    t.sel.collapse(c, 0);
    t.range = t.sel.getRangeAt(0);
  }

  if (c.nodeType !== 3) return insertNodes(t, document.createTextNode(data));

  c.insertData(o, data);
  t.sel.collapse(c, o + data.length);
}

function textBeside(t, forward) {
  const c = t.range.startContainer, at = c.nodeType === 1 ? c.childNodes[t.range.startOffset] : null;
  const walker = document.createTreeWalker(t.host, 4);
  let from = c;
  if (c.nodeType === 1 && !at) while (from.lastChild) from = from.lastChild;
  if (at) from = at;
  if (from.nodeType === 3 && from !== c && from.data.length > 0 && forward === (from === at)) return from;

  walker.currentNode = from;

  let n = forward ? walker.nextNode() : walker.previousNode();
  while (n && n.data.length === 0) n = forward ? walker.nextNode() : walker.previousNode();

  return n;
}

function unitAt(data, at, forward) {
  const pair = forward ? /^[\uD800-\uDBFF][\uDC00-\uDFFF]/.test(data.slice(at)) : /[\uD800-\uDBFF][\uDC00-\uDFFF]$/.test(data.slice(0, at));

  return pair ? 2 : 1;
}

function deleteOne(t, forward) {
  if (!t.range.collapsed) {
    t.range.deleteContents();

    return t.sel.collapse(t.range.startContainer, t.range.startOffset);
  }

  let c = t.range.startContainer, o = t.range.startOffset;
  if (c.nodeType !== 3 || (forward ? o === c.data.length : o === 0)) {
    c = textBeside(t, forward);
    if (!c) return;

    o = forward ? 0 : c.data.length;
  }

  const n = unitAt(c.data, o, forward), block = c.parentNode;
  c.deleteData(forward ? o : o - n, n);
  t.sel.collapse(c, forward ? o : o - n);
  if (c.data.length > 0 || block === t.host || block.childNodes.length > 1) return;

  block.replaceChild(document.createElement('br'), c);
  t.sel.collapse(block, 0);
}

function insertParagraph(t) {
  t.range.deleteContents();

  let block = t.range.startContainer;
  while (block !== t.host && !(block.nodeType === 1 && /^(p|div|li|h[1-6]|blockquote|pre)$/.test(block.localName))) block = block.parentNode;

  const rest = document.createRange();
  rest.setStart(t.range.startContainer, t.range.startOffset);
  rest.setEnd(block, block.childNodes.length);

  const next = block === t.host ? document.createElement('div') : block.cloneNode(false);
  next.appendChild(rest.extractContents());
  if (!next.firstChild) next.appendChild(document.createElement('br'));

  block === t.host ? t.host.appendChild(next) : block.parentNode.insertBefore(next, block.nextSibling);
  t.sel.collapse(next, 0);
}

function styledBy(t, names) {
  for (let n = t.range.startContainer; n && n !== t.host; n = n.parentNode) if (n.nodeType === 1 && names.includes(n.localName)) return n;

  return null;
}

function pruned(host) {
  const walker = document.createTreeWalker(host, 4), empty = [];
  for (let n = walker.nextNode(); n; n = walker.nextNode()) if (n.data.length === 0) empty.push(n);

  for (const n of empty) n.parentNode.removeChild(n);
}

function toggleInline(t, names) {
  const styled = styledBy(t, names);
  if (t.range.collapsed) return;

  if (styled) {
    const first = styled.firstChild, last = styled.lastChild;
    while (styled.firstChild) styled.parentNode.insertBefore(styled.firstChild, styled);

    styled.parentNode.removeChild(styled);
    pruned(t.host);
    if (first) t.sel.setBaseAndExtent(first, 0, last, last.nodeType === 3 ? last.data.length : last.childNodes.length);

    return;
  }

  const el = document.createElement(names[0]);
  el.appendChild(t.range.extractContents());
  t.range.insertNode(el);
  pruned(t.host);
  t.sel.selectAllChildren(el);
}

const COMMANDS = {
  inserttext: (t, value) => insertText(t, value),
  delete: t => deleteOne(t, false),
  forwarddelete: t => deleteOne(t, true),
  insertlinebreak: t => insertNodes(t, document.createElement('br')),
  insertparagraph: t => insertParagraph(t),
  inserthtml: (t, value) => insertNodes(t, t.range.createContextualFragment(value)),
  bold: t => toggleInline(t, INLINE.bold),
  italic: t => toggleInline(t, INLINE.italic),
  underline: t => toggleInline(t, INLINE.underline),
};

function editEvent(type, host, command, value, cancelable) {
  const data = command === 'inserttext' ? value : null;

  return host.dispatchEvent(new InputEvent(type, { bubbles: true, cancelable, inputType: INPUT_TYPES[command], data }));
}

hidden(Document.prototype, 'execCommand', function execCommand(name, showUI, value) {
  const command = String(name).toLowerCase();
  const text = value === undefined || value === null ? '' : String(value);
  const t = target();

  if (command === 'selectall') {
    getSelection().selectAllChildren(t ? t.host : document.body);

    return true;
  }

  if (!COMMANDS[command] || !t) return false;
  if (!editEvent('beforeinput', t.host, command, text, true)) return true;

  COMMANDS[command](t, text);
  editEvent('input', t.host, command, text, false);

  return true;
});
hidden(G, '__hostEdit', function (host, line) {
  const at = line.indexOf('\n'), t = target();
  if (!t || t.host !== host) {
    getSelection().selectAllChildren(host);
    getSelection().collapseToEnd();
  }

  document.execCommand(line.slice(0, at), false, line.slice(at + 1));
});
hidden(Document.prototype, 'queryCommandSupported', function queryCommandSupported(name) {
  const command = String(name).toLowerCase();

  return command === 'selectall' || command in COMMANDS;
});
hidden(Document.prototype, 'queryCommandEnabled', function queryCommandEnabled(name) {
  const command = String(name).toLowerCase();

  return command === 'selectall' || (command in COMMANDS && target() !== null);
});
hidden(Document.prototype, 'queryCommandState', function queryCommandState(name) {
  const names = INLINE[String(name).toLowerCase()], t = target();

  return !!(names && t && styledBy(t, names));
});
hidden(Document.prototype, 'queryCommandIndeterm', function queryCommandIndeterm() { return false; });
hidden(Document.prototype, 'queryCommandValue', function queryCommandValue() { return ''; });
})();
