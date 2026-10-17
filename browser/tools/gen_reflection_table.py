#!/usr/bin/env python3
"""Generates browser/engine/reflection-table.js, the reflected IDL
attributes of each HTML element interface, from the tables of WPT's
html/dom/elements-*.js (the HTML spec's IDL as WPT lists it).

    python3 -I browser/tools/gen_reflection_table.py [WPT_ROOT] [OUT]

WPT_ROOT defaults to target/wpt (`wpt.py fetch`), OUT to
browser/engine/reflection-table.js. Needs node. Each interface maps to
rows [idlName, contentName, type, options]; the tag names of the WPT
tables are mapped to their interface below, `undefinedelement` (the
global attributes) to HTMLElement. Attributes with a custom getter in
the spec and DOMTokenList attributes are left out: the prelude has them.
"""

import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

FILES = ["embedded", "forms", "forms-weekmonth", "grouping", "metadata", "misc",
         "obsolete", "sections", "tabular", "text"]

INTERFACES = {
    "undefinedelement": "HTMLElement", "picture": "HTMLPictureElement",
    "img": "HTMLImageElement", "iframe": "HTMLIFrameElement", "embed": "HTMLEmbedElement",
    "object": "HTMLObjectElement", "param": "HTMLParamElement", "video": "HTMLVideoElement",
    "audio": "HTMLAudioElement", "source": "HTMLSourceElement", "track": "HTMLTrackElement",
    "canvas": "HTMLCanvasElement", "map": "HTMLMapElement", "area": "HTMLAreaElement",
    "input": "HTMLInputElement", "form": "HTMLFormElement", "fieldset": "HTMLFieldSetElement",
    "legend": "HTMLLegendElement", "label": "HTMLLabelElement", "button": "HTMLButtonElement",
    "select": "HTMLSelectElement", "datalist": "HTMLDataListElement",
    "optgroup": "HTMLOptGroupElement", "option": "HTMLOptionElement",
    "textarea": "HTMLTextAreaElement", "output": "HTMLOutputElement",
    "progress": "HTMLProgressElement", "meter": "HTMLMeterElement", "p": "HTMLParagraphElement",
    "hr": "HTMLHRElement", "pre": "HTMLPreElement", "blockquote": "HTMLQuoteElement",
    "q": "HTMLQuoteElement", "ol": "HTMLOListElement", "ul": "HTMLUListElement",
    "li": "HTMLLIElement", "dl": "HTMLDListElement", "div": "HTMLDivElement",
    "head": "HTMLHeadElement", "title": "HTMLTitleElement", "base": "HTMLBaseElement",
    "link": "HTMLLinkElement", "meta": "HTMLMetaElement", "style": "HTMLStyleElement",
    "html": "HTMLHtmlElement", "script": "HTMLScriptElement", "template": "HTMLTemplateElement",
    "slot": "HTMLSlotElement", "ins": "HTMLModElement", "del": "HTMLModElement",
    "details": "HTMLDetailsElement", "menu": "HTMLMenuElement", "dialog": "HTMLDialogElement",
    "marquee": "HTMLMarqueeElement", "frameset": "HTMLFrameSetElement",
    "frame": "HTMLFrameElement", "dir": "HTMLDirectoryElement", "font": "HTMLFontElement",
    "body": "HTMLBodyElement", "table": "HTMLTableElement", "caption": "HTMLTableCaptionElement",
    "colgroup": "HTMLTableColElement", "col": "HTMLTableColElement",
    "tbody": "HTMLTableSectionElement", "thead": "HTMLTableSectionElement",
    "tfoot": "HTMLTableSectionElement", "tr": "HTMLTableRowElement",
    "td": "HTMLTableCellElement", "th": "HTMLTableCellElement", "a": "HTMLAnchorElement",
    "data": "HTMLDataElement", "h1": "HTMLHeadingElement", "h2": "HTMLHeadingElement",
    "h3": "HTMLHeadingElement", "h4": "HTMLHeadingElement", "h5": "HTMLHeadingElement",
    "h6": "HTMLHeadingElement", "time": "HTMLTimeElement", "br": "HTMLBRElement",
}

GLOBALS = [["title", "title", "string", {}], ["lang", "lang", "string", {}],
           ["dir", "dir", "enum", {"keywords": ["ltr", "rtl", "auto"]}],
           ["accessKey", "accesskey", "string", {}], ["autofocus", "autofocus", "boolean", {}],
           ["inert", "inert", "boolean", {}]]

LOAD = """
const fs = require('fs');
const elements = {};
globalThis.extraTests = [];
globalThis.mergeElements = src => {
  for (const tag in src) {
    const old = elements[tag] || {};
    for (const a in src[tag]) {
      if (old[a] && old[a].keywords) src[tag][a].keywords = [...new Set([...old[a].keywords, ...src[tag][a].keywords])];
    }
    elements[tag] = Object.assign(old, src[tag]);
  }
};
for (const f of process.argv.slice(1)) eval(fs.readFileSync(f, 'utf8'));
process.stdout.write(JSON.stringify(elements));
"""


def rows(attrs):
    out = []
    for idl, spec in attrs.items():
        if isinstance(spec, str):
            spec = {"type": spec}
        if isinstance(spec["type"], dict):
            spec = {**spec, **spec["type"]}
        if spec.get("customGetter") or "tokenlist" in spec["type"]:
            continue
        content = spec.get("domAttrName", idl).lower()
        opts = {k: v for k, v in spec.items() if k not in ("type", "domAttrName")}
        if isinstance(opts.get("defaultVal"), list):
            opts["defaultVal"] = "metadata" if "metadata" in opts["defaultVal"] else opts["defaultVal"][0]
        if idl == "formAction":
            opts["documentUrl"] = True
        out.append([idl, content, spec["type"], opts])
    return out


def main():
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else REPO / "target/wpt"
    out = Path(sys.argv[2]) if len(sys.argv) > 2 else REPO / "browser/engine/reflection-table.js"
    files = [str(root / f"html/dom/elements-{f}.js") for f in FILES]
    elements = json.loads(subprocess.run(["node", "-e", LOAD, *files], check=True,
                                         capture_output=True, text=True).stdout)
    table = {"HTMLElement": GLOBALS}
    for tag, attrs in elements.items():
        found = rows(attrs)
        if not found:
            continue
        rows_of = table.setdefault(INTERFACES[tag], [])
        names = {r[0] for r in rows_of}
        rows_of += [r for r in found if r[0] not in names]
    lines = [f"  {json.dumps(name)}: [\n" + ",\n".join("    " + json.dumps(r) for r in rs) + "\n  ]"
             for name, rs in sorted(table.items())]
    out.write_text("// generated by browser/tools/gen_reflection_table.py from WPT html/dom/elements-*.js\n"
                   "globalThis.__reflection = {\n" + ",\n".join(lines) + "\n};\n")


main()
