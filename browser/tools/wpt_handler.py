"""Runs the simple wptserve Python handlers (`def main(request, response)`
returning a body, `(headers, body)` or `(status, headers, body)`) for the
static server of wpt.py, without the rest of wptserve."""
import importlib
import runpy
import sys
import types
from urllib.parse import parse_qsl


class Query(dict):
    def first(self, key, default=None):
        values = self.get(key)
        return values[0] if values else default

    def get_list(self, key):
        return self.get(key, [])


class Headers(list):
    def set(self, k, v):
        self[:] = [(a, b) for a, b in self if a.lower() != text(k).lower()]
        self.append((text(k), text(v)))

    def append(self, k, v):
        list.append(self, (text(k), text(v)))


class Request:
    def __init__(self, query, headers):
        self.GET = Query()
        for k, v in parse_qsl(query, keep_blank_values=True):
            self.GET.setdefault(k.encode("latin-1", "replace"), []).append(v.encode("latin-1", "replace"))
        self.headers = {k.lower().encode(): v.encode() for k, v in headers.items()}


class Response:
    def __init__(self):
        self.headers = Headers()
        self.status = 200
        self.content = b""


def text(v):
    return v.decode("latin-1") if isinstance(v, bytes) else str(v)


def load_utils(path):
    package = types.ModuleType("wptserve")
    package.__path__ = [str(path.parent)]
    sys.modules["wptserve"] = package
    package.utils = importlib.import_module("wptserve.utils")


def run(root, file, query, headers):
    if "wptserve" not in sys.modules:
        load_utils(root / "tools" / "wptserve" / "wptserve" / "utils.py")
    request, response = Request(query, headers), Response()
    out = runpy.run_path(str(file))["main"](request, response)
    if isinstance(out, tuple):
        if len(out) == 3:
            response.status = out[0][0] if isinstance(out[0], tuple) else out[0]
            out = out[1:]
        for k, v in out[0]:
            response.headers.set(k, v)
        out = out[1]
    body = out if out is not None else response.content
    body = body.encode("utf-8") if isinstance(body, str) else body
    ctype = "text/html"
    extra = [(k, v) for k, v in response.headers if k.lower() != "content-type" or (ctype := v) is None]
    return int(response.status), ctype, body, extra
