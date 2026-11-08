import http.server
import os
import socket
import ssl
import sys
import threading

DIR = sys.argv[1]
PAGE = b"<!doctype html><title>Secure</title><p>secure page</p>"


class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = PAGE
        if self.path == "/switch":
            self.server.ctx, self.server.next_ctx = self.server.next_ctx, self.server.ctx
        if self.path.startswith("/fetch.html"):
            with open(os.path.join(DIR, "fetch.html"), "rb") as f:
                body = f.read()
        self.send_response(200)
        self.send_header("Content-Type", "text/html")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


def tls11(ctx):
    ctx.minimum_version = ssl.TLSVersion.TLSv1_1
    ctx.maximum_version = ssl.TLSVersion.TLSv1_1
    ctx.set_ciphers("DEFAULT:@SECLEVEL=0")


def serve(name, old=False):
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    if old:
        tls11(ctx)
    ctx.load_cert_chain(os.path.join(DIR, name + ".pem"))
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    srv.socket = ctx.wrap_socket(srv.socket, server_side=True)
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv.server_address[1]


class Switching(http.server.ThreadingHTTPServer):
    def get_request(self):
        sock, addr = self.socket.accept()
        return self.ctx.wrap_socket(sock, server_side=True), addr


def context(name):
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    ctx.load_cert_chain(os.path.join(DIR, name + ".pem"))
    return ctx


def switching():
    srv = Switching(("127.0.0.1", 0), Handler)
    srv.ctx = context("self-signed")
    srv.next_ctx = context("self-signed-b")
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv.server_address[1]


def offers_tls11(port):
    try:
        ctx = ssl.create_default_context(cafile=os.path.join(DIR, "ca.pem"))
        tls11(ctx)
        with socket.create_connection(("127.0.0.1", port)) as s:
            with ctx.wrap_socket(s, server_hostname="127.0.0.1"):
                return True
    except (OSError, ValueError):
        return False


def old_server():
    try:
        port = serve("valid", old=True)
    except (OSError, ValueError):
        return 0
    return port if offers_tls11(port) else 0


ports = [serve(n) for n in ("valid", "expired", "self-signed", "wrong-host")]
print(*ports, old_server(), switching(), flush=True)
threading.Event().wait()
