globalThis.__cssomSyntax = function (ak, validKeyframesName) {
// spec: https://drafts.csswg.org/css-syntax-3/#parsing
const IDENT = /^(?:--|-?(?:[a-zA-Z_]|[^\x00-\x7f]|\\.))(?:[\w-]|[^\x00-\x7f]|\\.)*$/;
const WIDE = /^(?:initial|inherit|unset|revert|revert-layer)$/i;
const PSEUDO_ELEMENT = /::|:(?:before|after|first-line|first-letter)\b/i;
const OPENERS = '({[';
const CLOSERS = ')}]';

function skipString(text, i) {
  const quote = text[i];

  i++;
  while (i < text.length && text[i] !== quote) i += text[i] === '\\' ? 2 : 1;

  return i;
}

function closeOf(text, open) {
  let depth = 0;

  for (let i = open; i < text.length; i++) {
    const c = text[i];

    if (c === '"' || c === "'") i = skipString(text, i);
    else if (c === '\\') i++;
    else if (OPENERS.includes(c)) depth++;
    else if (CLOSERS.includes(c) && --depth === 0) return i;
  }

  return -1;
}

function topLevel(text, wanted, from) {
  for (let i = from || 0; i < text.length; i++) {
    const c = text[i];

    if (wanted.includes(c)) return i;
    if (c === '"' || c === "'") i = skipString(text, i);
    else if (c === '\\') i++;
    else if (c === '(' || c === '[') i = closeOf(text, i) < 0 ? text.length : closeOf(text, i);
  }

  return -1;
}

function splitTop(text, separator) {
  const parts = [];
  let start = 0;

  for (let i = topLevel(text, separator); i >= 0; i = topLevel(text, separator, start)) {
    parts.push(text.slice(start, i));
    start = i + 1;
  }
  parts.push(text.slice(start));

  return parts;
}

function stripComments(text) {
  return text.replace(/\/\*[^]*?\*\//g, '');
}

function ruleParts(text) {
  const at = /^@([-\w]+)/.exec(text);
  const open = topLevel(text, '{');
  const head = open < 0 ? text.replace(/;\s*$/, '') : text.slice(0, open);
  const close = open < 0 ? -1 : closeOf(text, open);
  const body = open < 0 ? null : text.slice(open + 1, close < 0 ? text.length : close);

  return { name: at ? at[1].toLowerCase() : '', prelude: stripComments(at ? head.slice(at[0].length) : head).trim(), body };
}

function isCustomDeclaration(text) {
  return /^\s*--[^\s:]*\s*:/.test(text);
}

function bodyItems(body) {
  const items = [];
  let start = 0;

  while (start < body.length) {
    const custom = isCustomDeclaration(body.slice(start));
    const stop = topLevel(body, custom ? ';' : ';{', start);
    const brace = stop >= 0 && body[stop] === '{';
    const close = brace ? closeOf(body, stop) : stop;
    const end = stop < 0 || close < 0 ? body.length : close + 1;
    const text = stripComments(body.slice(start, end)).trim();

    if (text && text !== ';') items.push({ rule: brace || text[0] === '@', text });
    start = end;
  }

  return items;
}

function relativeProbe(part) {
  const anchored = part.trim().replace(/&/g, ':scope');

  return /^[>+~]/.test(anchored) ? ':scope ' + anchored : anchored;
}

function selectorText(text, relative) {
  const source = text.trim();
  const direct = source ? ak.selectorText(source) : null;

  if (direct !== null || !relative || !source) return direct;
  if (splitTop(source, ',').some(part => ak.selectorText(relativeProbe(part)) === null)) return null;

  return splitTop(source, ',').map(part => part.trim().replace(/\s+/g, ' ')).join(', ');
}

function scopeSelector(text) {
  return PSEUDO_ELEMENT.test(text) ? null : selectorText(text, true);
}

// spec: https://drafts.csswg.org/css-cascade-6/#scope-syntax
function scopeParts(prelude) {
  const parts = { start: null, end: null };
  let rest = prelude.trim();

  if (rest[0] === '(') {
    const close = closeOf(rest, 0);

    if (close < 0) return null;
    parts.start = scopeSelector(rest.slice(1, close));
    if (parts.start === null) return null;
    rest = rest.slice(close + 1).trim();
  }

  if (/^to\s*\(/i.test(rest)) {
    const open = rest.indexOf('(');

    if (closeOf(rest, open) !== rest.length - 1) return null;
    parts.end = scopeSelector(rest.slice(open + 1, -1));
    if (parts.end === null) return null;
    rest = '';
  }

  return rest ? null : parts;
}

function validLayerName(name) {
  return name.split('.').every(part => IDENT.test(part) && !WIDE.test(part));
}

// spec: https://drafts.csswg.org/css-cascade-5/#layering
function layerNames(prelude, block) {
  const names = prelude.trim() ? splitTop(prelude, ',').map(name => name.trim()) : [];

  if (!names.every(validLayerName) || (block ? names.length > 1 : names.length === 0)) return null;

  return names;
}

// spec: https://drafts.csswg.org/css-fonts-4/#font-feature-values-syntax
function validFamilies(prelude) {
  const names = splitTop(prelude, ',').map(name => name.trim());

  return names.every(name => /^"[^"]*"$|^'[^']*'$/.test(name) || (name !== '' && name.replace(/\\[0-9a-f]+ /gi, 'x').split(/\s+/).every(word => IDENT.test(word) && !word.startsWith('--'))));
}

function validRule(text, nested) {
  const parts = ruleParts(text);

  if (!parts.name) return parts.body !== null && selectorText(parts.prelude, nested) !== null;
  if (parts.name === 'keyframes' || parts.name === '-webkit-keyframes') return validKeyframesName(parts.prelude);
  if (parts.name === 'scope') return parts.body !== null && scopeParts(parts.prelude) !== null;
  if (parts.name === 'layer') return layerNames(parts.prelude, parts.body !== null) !== null;
  if (parts.name === 'font-feature-values') return parts.body !== null && validFamilies(parts.prelude);

  return true;
}

return { ruleParts, bodyItems, selectorText, scopeParts, layerNames, validRule, splitTop, closeOf };
};
