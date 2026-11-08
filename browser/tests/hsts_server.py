import http.server
import os
import ssl
import sys
import threading

DIR = sys.argv[1]
GIF = bytes.fromhex(
    "47494638396101000100800000000000ffffff21f90401000000002c00000000010001000002024401003b"
)
HEADERS = {
    "/hsts": "max-age=3600; includeSubDomains",
    "/hsts-clear": "max-age=0",
}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        secure = isinstance(self.connection, ssl.SSLSocket)
        scheme = "https" if secure else "http"
        path = self.path.split("?")[0]
        kind = "text/html"
        body = f"<!doctype html><title>{scheme}</title><p>{scheme} page".encode()
        if path == "/x.js":
            kind = "text/javascript"
            body = f"var ran = '{scheme}';".encode()
        elif path.endswith((".html", ".js")):
            if path.endswith(".js"):
                kind = "text/javascript"
            with open(os.path.join(DIR, path[1:]), "rb") as f:
                body = f.read()
        elif path == "/img.gif":
            kind = "image/gif"
            body = GIF
        elif path == "/data":
            kind = "text/plain"
            body = f"{scheme} data".encode()
        self.send_response(200)
        self.send_header("Content-Type", kind)
        self.send_header("Access-Control-Allow-Origin", "*")
        if path in HEADERS:
            self.send_header("Strict-Transport-Security", HEADERS[path])
        if path == "/uir-header.html":
            self.send_header("Content-Security-Policy", "upgrade-insecure-requests")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


def serve(secure):
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    if secure:
        ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        ctx.load_cert_chain(os.path.join(DIR, "site.pem"))
        srv.socket = ctx.wrap_socket(srv.socket, server_side=True)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv.server_address[1]


print(serve(True), serve(False), flush=True)
threading.Event().wait()
