globalThis.__cssomRules = function (ak, hidden, syntax) {
// spec: https://drafts.csswg.org/cssom/#css-rules
const TYPES = { media: 4, import: 3, 'font-face': 5, page: 6, keyframes: 7, '-webkit-keyframes': 7, namespace: 10, supports: 12, 'font-feature-values': 14 };
const CONSTANTS = { STYLE_RULE: 1, CHARSET_RULE: 2, IMPORT_RULE: 3, MEDIA_RULE: 4, FONT_FACE_RULE: 5, PAGE_RULE: 6, KEYFRAMES_RULE: 7, KEYFRAME_RULE: 8, MARGIN_RULE: 9, NAMESPACE_RULE: 10, COUNTER_STYLE_RULE: 11, SUPPORTS_RULE: 12, FONT_FEATURE_VALUES_RULE: 14 };

const descriptors = globalThis.__cssomDescriptors(ak, syntax);

delete globalThis.__cssomDescriptors;

function illegal() {
  throw new TypeError('Illegal constructor');
}

function ruleList(items) {
  hidden(items, 'item', i => items[i] || null);

  return items;
}

function indented(text) {
  return '  ' + text.replace(/\n/g, '\n  ');
}

function mediaList(text) {
  const queries = text.trim() ? syntax.splitTop(text, ',').map(q => q.trim()) : [];

  return { mediaText: queries.join(', '), length: queries.length, item: i => queries[i] || null };
}

function styleHolder(rule, text) {
  const el = document.createElement('div');

  el.setAttribute('style', text);

  return new Proxy(el.style, {
    get(target, key) {
      const value = target[key];

      if (typeof value !== 'function') return value;
      if (key !== 'setProperty' && key !== 'removeProperty') return value.bind(target);

      return (...args) => {
        const out = value.apply(target, args);

        rule._edited(target.cssText);

        return out;
      };
    },
    set(target, key, value) {
      target[key] = value;
      rule._edited(target.cssText);

      return true;
    },
  });
}

class CSSRule {
  constructor() { illegal(); }
  get cssText() { return this._raw; }
  set cssText(v) {}
  get type() { return TYPES[this._name] || 0; }
  get parentRule() { return this._parent; }
  get parentStyleSheet() { return this._sheet; }
  _source() { return this._raw; }
  _changed() {
    if (this._parent) this._parent._changed();
    else this._sheet._ruleChanged(this);
  }
}
Object.assign(CSSRule, CONSTANTS);
Object.assign(CSSRule.prototype, CONSTANTS);

class CSSGroupingRule extends CSSRule {
  get _items() {
    if (!this._parsed) hidden(this, '_parsed', childRules(this));

    return this._parsed;
  }
  get cssRules() {
    if (!this._list) hidden(this, '_list', ruleList([]));
    this._list.splice(0, this._list.length, ...this._items);

    return this._list;
  }
  get _head() { return '@' + this._name + (this._prelude ? ' ' + this._prelude : ''); }
  get cssText() { return blockText(this._head, '', this._items); }
  _source() {
    const own = this._declarations();

    return this._head + ' { ' + (own ? own + '; ' : '') + this._items.map(r => r._source()).join(' ') + ' }';
  }
  _declarations() { return ''; }
  insertRule(text, index) {
    const items = this._items;

    index = index === undefined ? 0 : index >>> 0;
    if (index > items.length) throw new DOMException('The index provided (' + index + ') is larger than the maximum index (' + items.length + ').', 'IndexSizeError');
    text = String(text).trim();
    if (!syntax.validRule(text, true)) throw new DOMException("Failed to parse the rule '" + text + "'.", 'SyntaxError');
    items.splice(index, 0, ruleObject(this._sheet, text, this));
    this._changed();

    return index;
  }
  deleteRule(index) {
    index = index >>> 0;
    if (index >= this._items.length) throw new DOMException('The index provided (' + index + ') is larger than the maximum index.', 'IndexSizeError');
    this._items.splice(index, 1);
    this._changed();
  }
}

class CSSConditionRule extends CSSGroupingRule {
  get conditionText() { return this._prelude; }
}

class CSSMediaRule extends CSSConditionRule {
  get media() { return mediaList(this._prelude); }
}

class CSSSupportsRule extends CSSConditionRule {}
class CSSContainerRule extends CSSConditionRule {}
class CSSStartingStyleRule extends CSSGroupingRule {}

class CSSScopeRule extends CSSGroupingRule {
  get start() { return syntax.scopeParts(this._prelude).start; }
  get end() { return syntax.scopeParts(this._prelude).end; }
  get _head() {
    const { start, end } = syntax.scopeParts(this._prelude);

    return '@scope' + (start === null ? '' : ' (' + start + ')') + (end === null ? '' : ' to (' + end + ')');
  }
}

class CSSLayerBlockRule extends CSSGroupingRule {
  get name() { return this._prelude; }
  get type() { return 0; }
}

class CSSLayerStatementRule extends CSSRule {
  get nameList() { return syntax.layerNames(this._prelude, false); }
  get cssText() { return '@layer ' + this.nameList.join(', ') + ';'; }
  get type() { return 0; }
}

// spec: https://drafts.csswg.org/css-cascade-5/#at-import
class CSSImportRule extends CSSRule {
  get _parts() {
    const url = /^(?:url\(\s*(?:"([^"]*)"|'([^']*)'|([^)]*?))\s*\)|"([^"]*)"|'([^']*)')\s*/i.exec(this._prelude) || [''];
    const layer = /^layer(?:\(\s*([^()\s]+)\s*\))?(?![-\w(])\s*/i.exec(this._prelude.slice(url[0].length));
    const named = layer && (layer[1] === undefined || syntax.layerNames(layer[1], true) !== null) ? layer : null;
    const rest = this._prelude.slice(url[0].length + (named ? named[0].length : 0));
    const supports = /^supports\(/i.test(rest) ? rest.slice(0, syntax.closeOf(rest, rest.indexOf('(')) + 1) : '';

    return { href: url.slice(1).find(v => v !== undefined) || '', layerName: named ? named[1] || '' : null, supports, media: rest.slice(supports.length).trim() };
  }
  get href() { return this._parts.href; }
  get layerName() { return this._parts.layerName; }
  get supportsText() { return this._parts.supports ? this._parts.supports.slice(9, -1).trim() : null; }
  get media() { return mediaList(this._parts.media); }
  get styleSheet() { return null; }
  get cssText() {
    const { href, layerName, supports, media } = this._parts;
    const layer = layerName === null ? '' : layerName ? ' layer(' + layerName + ')' : ' layer';

    return '@import url("' + href.replace(/["\\]/g, '\\$&') + '")' + layer + (supports ? ' ' + supports : '') + (media ? ' ' + media : '') + ';';
  }
}

class CSSKeyframesRule extends CSSRule {
  get name() { return this._prelude; }
}

class CSSFontFaceRule extends CSSRule {
  get style() {
    if (!this._style) hidden(this, '_style', descriptors.descriptorBlock('font-face', this._body || ''));

    return this._style;
  }
  get cssText() { return '@font-face { ' + this.style.cssText + (this.style.length ? ' ' : '') + '}'; }
}
class CSSFontFeatureValuesRule extends CSSRule {
  get fontFamily() { return this._prelude; }
  get cssText() { return '@font-feature-values ' + this._prelude + ' { ' + descriptors.featureValuesText(this._body || '') + ' }'; }
}
class CSSFontPaletteValuesRule extends CSSRule {
  get _block() {
    if (!this._style) hidden(this, '_style', descriptors.descriptorBlock('font-palette-values', this._body || ''));

    return this._style;
  }
  get name() { return this._prelude; }
  get fontFamily() { return this._block.fontFamily; }
  get basePalette() { return this._block.basePalette; }
  get overrideColors() { return this._block.overrideColors; }
  get cssText() { return '@font-palette-values ' + this._prelude + ' { ' + this._block.cssText + (this._block.length ? ' ' : '') + '}'; }
}
class CSSPropertyRule extends CSSRule {
  get name() { return this._prelude; }
}
class CSSNamespaceRule extends CSSRule {}
class CSSPageRule extends CSSRule {}

class CSSNestedDeclarations extends CSSRule {
  get style() {
    if (!this._style) hidden(this, '_style', styleHolder(this, this._raw));

    return this._style;
  }
  get cssText() { return ak.declarationsText(this._raw); }
  _source() { return this._raw + ';'; }
  _edited(text) {
    this._raw = text;
    this._changed();
  }
}

// spec: https://drafts.csswg.org/css-nesting-1/#cssom-style
class CSSStyleRule extends CSSGroupingRule {
  get type() { return 1; }
  get selectorText() { return syntax.selectorText(this._prelude, true) ?? this._prelude; }
  set selectorText(v) {
    if (syntax.selectorText(String(v), true) === null) return;
    this._prelude = String(v).trim();
    this._changed();
  }
  get style() {
    if (!this._style) hidden(this, '_style', styleHolder(this, this._declarations()));

    return this._style;
  }
  get _head() { return this._prelude; }
  get cssText() { return blockText(this.selectorText, ak.declarationsText(this._declarations()), this._items); }
  _declarations() {
    if (this._own === undefined) this._items;

    return this._own;
  }
  _edited(text) {
    this._own = text;
    this._changed();
  }
}

const BLOCKS = { media: CSSMediaRule, supports: CSSSupportsRule, container: CSSContainerRule, 'starting-style': CSSStartingStyleRule, scope: CSSScopeRule, layer: CSSLayerBlockRule };
const OTHERS = { layer: CSSLayerStatementRule, import: CSSImportRule, keyframes: CSSKeyframesRule, '-webkit-keyframes': CSSKeyframesRule, 'font-face': CSSFontFaceRule, 'font-feature-values': CSSFontFeatureValuesRule, 'font-palette-values': CSSFontPaletteValuesRule, property: CSSPropertyRule, namespace: CSSNamespaceRule, page: CSSPageRule };

function blockText(head, declarations, rules) {
  if (rules.length === 0 && head[0] !== '@') return head + (declarations ? ' { ' + declarations + ' }' : ' { }');

  const lines = (declarations ? [declarations] : []).concat(rules.map(r => r.cssText));

  return head + ' {\n' + lines.map(line => indented(line) + '\n').join('') + '}';
}

function holdsDeclarations(rule) {
  for (let r = rule; r; r = r._parent) if (r instanceof CSSStyleRule || r instanceof CSSScopeRule) return true;

  return false;
}

function newRule(Class, sheet, parent, raw, parts) {
  const rule = Object.create(Class.prototype);

  hidden(rule, '_sheet', sheet);
  hidden(rule, '_parent', parent || null);
  hidden(rule, '_raw', raw);
  hidden(rule, '_name', parts.name);
  hidden(rule, '_prelude', parts.prelude);
  hidden(rule, '_body', parts.body);
  hidden(rule, '_own', undefined);

  return rule;
}

function childRules(rule) {
  const rules = [];
  const keeps = holdsDeclarations(rule);
  let pending = [];

  function flush() {
    if (pending.length && keeps && (rules.length || !(rule instanceof CSSStyleRule))) rules.push(newRule(CSSNestedDeclarations, rule._sheet, rule, pending.join(' '), { name: '', prelude: '', body: null }));
    else if (pending.length) rule._own = pending.join(' ');
    pending = [];
  }

  rule._own = '';
  for (const item of syntax.bodyItems(rule._body || '')) {
    if (!item.rule) pending.push(item.text);
    else if (syntax.validRule(item.text, true)) {
      flush();
      rules.push(ruleObject(rule._sheet, item.text, rule));
    }
  }
  flush();

  return rules;
}

function ruleObject(sheet, text, parent) {
  const parts = syntax.ruleParts(text);
  const Class = !parts.name ? CSSStyleRule : (parts.body !== null && BLOCKS[parts.name]) || OTHERS[parts.name] || CSSRule;

  return newRule(Class, sheet, parent, text, parts);
}

const classes = { CSSRule, CSSGroupingRule, CSSConditionRule, CSSMediaRule, CSSSupportsRule, CSSContainerRule, CSSStartingStyleRule, CSSScopeRule, CSSLayerBlockRule, CSSLayerStatementRule, CSSImportRule, CSSKeyframesRule, CSSFontFaceRule, CSSFontFeatureValuesRule, CSSFontPaletteValuesRule, CSSPropertyRule, CSSNamespaceRule, CSSPageRule, CSSNestedDeclarations, CSSStyleRule };

for (const name in classes) hidden(globalThis, name, classes[name]);

return { ruleObject, ruleList, validRule: text => syntax.validRule(text, false) };
};
