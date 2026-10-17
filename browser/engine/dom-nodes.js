(function () {
const G = globalThis;
const hidden = (o, k, v) => Object.defineProperty(o, k, { value: v, writable: true, configurable: true });

// https://webidl.spec.whatwg.org/#dfn-error-names-table
const CODES = ['IndexSizeError', 'DOMStringSizeError', 'HierarchyRequestError', 'WrongDocumentError', 'InvalidCharacterError', 'NoDataAllowedError', 'NoModificationAllowedError', 'NotFoundError', 'NotSupportedError', 'InUseAttributeError', 'InvalidStateError', 'SyntaxError', 'InvalidModificationError', 'NamespaceError', 'InvalidAccessError', 'ValidationError', 'TypeMismatchError', 'SecurityError', 'NetworkError', 'AbortError', 'URLMismatchError', 'QuotaExceededError', 'TimeoutError', 'InvalidNodeTypeError', 'DataCloneError'];
const CONSTANTS = 'INDEX_SIZE_ERR DOMSTRING_SIZE_ERR HIERARCHY_REQUEST_ERR WRONG_DOCUMENT_ERR INVALID_CHARACTER_ERR NO_DATA_ALLOWED_ERR NO_MODIFICATION_ALLOWED_ERR NOT_FOUND_ERR NOT_SUPPORTED_ERR INUSE_ATTRIBUTE_ERR INVALID_STATE_ERR SYNTAX_ERR INVALID_MODIFICATION_ERR NAMESPACE_ERR INVALID_ACCESS_ERR VALIDATION_ERR TYPE_MISMATCH_ERR SECURITY_ERR NETWORK_ERR ABORT_ERR URL_MISMATCH_ERR QUOTA_EXCEEDED_ERR TIMEOUT_ERR INVALID_NODE_TYPE_ERR DATA_CLONE_ERR'.split(' ');
const DOMExceptionP = DOMException.prototype;
Object.defineProperty(DOMExceptionP, 'code', { get() { return CODES.indexOf(this.name) + 1; }, configurable: true, enumerable: true });
CONSTANTS.forEach((k, i) => {
  Object.defineProperty(DOMException, k, { value: i + 1, enumerable: true });
  Object.defineProperty(DOMExceptionP, k, { value: i + 1, enumerable: true });
});

hidden(Element.prototype, 'webkitMatchesSelector', Element.prototype.matches);

// https://dom.spec.whatwg.org/#namespaces
const XML_NS = 'http://www.w3.org/XML/1998/namespace';
const XMLNS_NS = 'http://www.w3.org/2000/xmlns/';
const HTML_NS = 'http://www.w3.org/1999/xhtml';
const PREFIX = /^[^\0\t\n\f\r />]+$/;
const LOCAL_NAME = /^(?:[A-Za-z][^\0\t\n\f\r />]*|[:_\u0080-￿][A-Za-z0-9\-.:_\u0080-￿]*)$/;
const names = new WeakMap();

function validateAndExtract(namespace, qualifiedName) {
  const ns = namespace === undefined || namespace === null || namespace === '' ? null : String(namespace);
  const qname = String(qualifiedName);
  const colon = qname.indexOf(':');
  const prefix = colon < 0 ? null : qname.slice(0, colon);
  const local = colon < 0 ? qname : qname.slice(colon + 1);

  if (prefix !== null && !PREFIX.test(prefix)) throw new DOMException("The qualified name provided ('" + qname + "') contains the invalid prefix '" + prefix + "'.", 'InvalidCharacterError');
  if (!LOCAL_NAME.test(local)) throw new DOMException("The qualified name provided ('" + qname + "') contains the invalid name-start character '" + local[0] + "'.", 'InvalidCharacterError');
  if (prefix !== null && ns === null) throw new DOMException("The namespace URI provided ('') is not valid for the qualified name provided ('" + qname + "').", 'NamespaceError');
  if (prefix === 'xml' && ns !== XML_NS) throw new DOMException("The namespace URI provided ('" + ns + "') is not valid for the qualified name provided ('" + qname + "').", 'NamespaceError');
  if ((qname === 'xmlns' || prefix === 'xmlns') !== (ns === XMLNS_NS)) throw new DOMException("The namespace URI provided ('" + ns + "') is not valid for the qualified name provided ('" + qname + "').", 'NamespaceError');

  return prefix;
}

const DocP = Document.prototype;
const NodeP = Node.prototype;
const createElement = DocP.createElement;
hidden(DocP, 'createElement', function (localName, options) {
  if (!LOCAL_NAME.test(String(localName))) throw new DOMException("Failed to execute 'createElement' on 'Document': The tag name provided ('" + localName + "') is not a valid name.", 'InvalidCharacterError');

  return createElement.call(this, localName, options);
});
const createElementNS = DocP.createElementNS;
hidden(DocP, 'createElementNS', function (namespace, qualifiedName, options) {
  const prefix = validateAndExtract(namespace, qualifiedName);
  const el = createElementNS.call(this, namespace, qualifiedName, options);
  const qname = String(qualifiedName);

  names.set(el, { namespaceURI: namespace === undefined || namespace === null || namespace === '' ? null : String(namespace), prefix, localName: prefix === null ? qname : qname.slice(prefix.length + 1), qualifiedName: qname });

  return el;
});
const cloneNode = NodeP.cloneNode;
hidden(NodeP, 'cloneNode', function (deep) {
  const copy = cloneNode.call(this, deep);

  for (let from = this, to = copy; from && to; from = next(from, this), to = next(to, copy)) {
    if (names.has(from)) names.set(to, names.get(from));
  }

  return copy;
});
function next(node, root) {
  if (node.firstChild) return node.firstChild;

  for (let n = node; n && n !== root; n = n.parentNode) {
    if (n.nextSibling) return n.nextSibling;
  }

  return null;
}
function nativeGetter(P, k) {
  for (let o = P; o; o = Object.getPrototypeOf(o)) {
    const d = Object.getOwnPropertyDescriptor(o, k);

    if (d) return d.get;
  }
}
function nameGetter(k, make) {
  const native = nativeGetter(Element.prototype, k);

  Object.defineProperty(Element.prototype, k, {
    get() {
      const n = names.get(this);

      return n ? make(n, this) : native ? native.call(this) : null;
    },
    configurable: true,
    enumerable: true,
  });
}
// https://dom.spec.whatwg.org/#element-html-uppercased-qualified-name
const htmlUppercased = n => n.namespaceURI === HTML_NS ? n.qualifiedName.replace(/[a-z]+/g, c => c.toUpperCase()) : n.qualifiedName;
nameGetter('prefix', n => n.prefix);
nameGetter('namespaceURI', n => n.namespaceURI);
nameGetter('localName', n => n.localName);
nameGetter('tagName', htmlUppercased);
nameGetter('nodeName', htmlUppercased);

function elementFor(node) {
  if (node.nodeType === Node.ELEMENT_NODE) return node;
  if (node.nodeType === Node.DOCUMENT_NODE) return node.documentElement;
  if (node.nodeType === Node.DOCUMENT_TYPE_NODE || node.nodeType === Node.DOCUMENT_FRAGMENT_NODE) return null;
  if (node.nodeType === Node.ATTRIBUTE_NODE) return node.ownerElement;

  return node.parentElement;
}

// https://dom.spec.whatwg.org/#locate-a-namespace
function locateNamespace(node, prefix) {
  if (node.nodeType !== Node.ELEMENT_NODE) {
    const el = elementFor(node);

    return el ? locateNamespace(el, prefix) : null;
  }

  if (prefix === 'xml') return XML_NS;
  if (prefix === 'xmlns') return XMLNS_NS;
  if (node.namespaceURI !== null && node.prefix === prefix) return node.namespaceURI;

  const declared = node.getAttribute(prefix === null ? 'xmlns' : 'xmlns:' + prefix);

  if (declared !== null) return declared || null;

  return node.parentElement ? locateNamespace(node.parentElement, prefix) : null;
}

// https://dom.spec.whatwg.org/#locate-a-namespace-prefix
function locatePrefix(el, namespace) {
  if (el.namespaceURI === namespace && el.prefix !== null) return el.prefix;

  for (const a of el.attributes) {
    if (a.name.startsWith('xmlns:') && a.value === namespace) return a.name.slice(6);
  }

  return el.parentElement ? locatePrefix(el.parentElement, namespace) : null;
}

hidden(NodeP, 'lookupNamespaceURI', function (prefix) {
  return locateNamespace(this, prefix === undefined || prefix === null || prefix === '' ? null : String(prefix));
});
hidden(NodeP, 'lookupPrefix', function (namespace) {
  const el = elementFor(this);

  return namespace === undefined || namespace === null || namespace === '' || !el ? null : locatePrefix(el, String(namespace));
});
hidden(NodeP, 'isDefaultNamespace', function (namespace) {
  return locateNamespace(this, null) === (namespace === undefined || namespace === null || namespace === '' ? null : String(namespace));
});

function byTagNameNS(namespace, localName) {
  const ns = namespace === null || namespace === '' ? null : String(namespace);
  const local = String(localName);
  const list = Array.prototype.filter.call(this.getElementsByTagName('*'), el => (ns === '*' || el.namespaceURI === ns) && (local === '*' || el.localName === local));

  Object.setPrototypeOf(list, HTMLCollection.prototype);
  hidden(list, 'item', i => list[i >>> 0] || null);

  return list;
}
hidden(Element.prototype, 'getElementsByTagNameNS', byTagNameNS);
hidden(DocP, 'getElementsByTagNameNS', byTagNameNS);

// https://dom.spec.whatwg.org/#dom-domimplementation-createdocumenttype
const implementation = document.implementation;
function doctype(owner, args) {
  if (args.length < 3) throw new TypeError("Failed to execute 'createDocumentType' on 'DOMImplementation': 3 arguments required, but only " + args.length + " present.");

  const name = String(args[0]);

  if (/[\0\t\n\f\r >]/.test(name)) throw new DOMException("Failed to execute 'createDocumentType' on 'DOMImplementation': The qualified name provided ('" + name + "') is not a valid doctype name.", 'InvalidCharacterError');

  const values = { name, nodeName: name, publicId: String(args[1]), systemId: String(args[2]), ownerDocument: owner, nodeValue: null, nodeType: Node.DOCUMENT_TYPE_NODE, parentNode: null };
  const d = Object.create(DocumentType.prototype);

  for (const k in values) Object.defineProperty(d, k, { value: values[k], enumerable: true, configurable: true });

  return d;
}
function implementationOf(owner) {
  const impl = Object.create(implementation);

  hidden(impl, 'createDocumentType', function () { return doctype(owner, arguments); });

  return impl;
}
const createHTMLDocument = implementation.createHTMLDocument;
const createDocument = implementation.createDocument;
hidden(implementation, 'createDocumentType', function () { return doctype(document, arguments); });
hidden(implementation, 'createHTMLDocument', function (title) {
  const d = createHTMLDocument.call(this, title);

  d.implementation = implementationOf(d);

  return d;
});
// https://dom.spec.whatwg.org/#dom-domimplementation-createdocument
hidden(implementation, 'createDocument', function (namespace, qualifiedName, doctype) {
  if (arguments.length < 2) throw new TypeError("Failed to execute 'createDocument' on 'DOMImplementation': 2 arguments required, but only " + arguments.length + " present.");

  const qname = qualifiedName === null ? '' : String(qualifiedName);

  if (qname !== '') validateAndExtract(namespace, qname);

  const d = createDocument.call(this);

  d.implementation = implementationOf(d);

  return d;
});

for (const P of [Element.prototype, DocP, DocumentFragment.prototype]) {
  for (const k of ['querySelector', 'querySelectorAll', 'matches', 'webkitMatchesSelector', 'closest']) {
    const native = P[k];

    if (!Object.prototype.hasOwnProperty.call(P, k)) continue;

    hidden(P, k, function (selectors) {
      if (arguments.length < 1) throw new TypeError("Failed to execute '" + k + "': 1 argument required, but only 0 present.");

      return native.apply(this, arguments);
    });
  }
}

const classList = Object.getOwnPropertyDescriptor(Element.prototype, 'classList');
const INDEX = /^(?:0|[1-9][0-9]*)$/;
const tokenLists = new WeakMap();
Object.defineProperty(Element.prototype, 'classList', {
  get() {
    const list = classList.get.call(this);

    if (!tokenLists.has(list)) tokenLists.set(list, new Proxy(list, { get: (t, k) => typeof k === 'string' && INDEX.test(k) ? t.item(+k) ?? undefined : Reflect.get(t, k, t), has: (t, k) => typeof k === 'string' && INDEX.test(k) ? +k < t.length : k in t }));

    return tokenLists.get(list);
  },
  set: classList.set,
  configurable: true,
});

// https://dom.spec.whatwg.org/#concept-cd-replace
const CharacterDataP = CharacterData.prototype;
function replaceData(node, offset, count, data) {
  const text = node.data;

  offset >>>= 0;
  count >>>= 0;

  if (offset > text.length) throw new DOMException("The offset " + offset + " is greater than the node's length (" + text.length + ").", 'IndexSizeError');

  node.data = text.slice(0, offset) + data + text.slice(offset + count);
}
function required(args, n, name) {
  if (args.length < n) throw new TypeError("Failed to execute '" + name + "' on 'CharacterData': " + n + " arguments required, but only " + args.length + " present.");
}
hidden(CharacterDataP, 'substringData', function (offset, count) {
  required(arguments, 2, 'substringData');
  offset >>>= 0;
  count >>>= 0;

  if (offset > this.length) throw new DOMException("The offset " + offset + " is greater than the node's length (" + this.length + ").", 'IndexSizeError');

  return this.data.substring(offset, offset + count);
});
hidden(CharacterDataP, 'appendData', function (data) {
  required(arguments, 1, 'appendData');
  this.data += String(data);
});
hidden(CharacterDataP, 'insertData', function (offset, data) {
  required(arguments, 2, 'insertData');
  replaceData(this, offset, 0, String(data));
});
hidden(CharacterDataP, 'deleteData', function (offset, count) {
  required(arguments, 2, 'deleteData');
  replaceData(this, offset, count, '');
});
hidden(CharacterDataP, 'replaceData', function (offset, count, data) {
  required(arguments, 3, 'replaceData');
  replaceData(this, offset, count, String(data));
});
})();
