# A local HTTP server for the browser's tests (tests/browser.rs). It
# serves the directory given as its argument on an ephemeral port of
# 127.0.0.1, prints the port on the first line of stdout, and adds a few
# routes that a file server cannot produce:
#
#   /redirect        302 to /basic.html
#   /redirect-style  302 to /style/link.html (relative URLs resolve
#                    against the final URL)
#   /redirect-loop   302 to itself
#   /oauth/approve?redirect_uri=U&state=S&seen=X
#                    302 to U?code=c0de&state=S&seen=X (the sign-in
#                    origin sending a popup back to its opener's
#                    origin, tests/browser.rs `window_login_popup`)
#   /agent           the request's User-Agent, as text/plain
#   /gzip            an HTML page, gzip-encoded when the client accepts it
#   /latin1          an HTML page in ISO-8859-1, declared by Content-Type
#   /echo            (GET and POST) an HTML page that shows the request:
#                    its method, path, Content-Type, the fields of the
#                    query or of the body (urlencoded, multipart or
#                    text/plain) as "name = value" items, and the raw
#                    body (CR written as \r)
#   /echo-redirect   (POST) 303 to /echo?redirected=1
#   /fetch-echo      (any method) the request as JSON: its method, path,
#                    headers (names in lower case), body (as UTF-8 and in
#                    hex) and form fields (as /echo); HEAD gets no body;
#                    answers Access-Control-Allow-Origin: * and a CORS
#                    preflight (OPTIONS with Access-Control-Request-Method)
#   /fetch-redirect/STATUS  (any method) STATUS to /fetch-echo?redirected
#   /net/start       302 to /net/page.html, setting the cookies of COOKIES
#   /headers         the request's Accept, Accept-Language, Referer and
#                    Cookie headers, as text/plain
#   /log/NAME        (GET and POST) records the request (method, path,
#                    Accept, Referer, Cookie, Content-Type) and answers
#                    with a small body of the type of NAME's extension
#                    (`Cache-Control: no-store` when NAME has "nostore")
#   /fail/NAME       records the request (as /log/NAME) and closes the
#                    connection without an answer: a load that fails
#   /log             the recorded requests, one per line, sorted (requests
#                    made concurrently arrive in any order), as text/plain
#   /big             33 MiB of text (with Content-Length; /big?stream
#                    without, the end of the body closing the connection)
#   /meta-charset    a Windows-1252 page declared only by <meta charset>
#   /sleep?ms=N      after N ms: a small HTML page ("Slept", "slept N ms")
#   /gbk             a GBK page declared only by Content-Type
#   /slow/NAME       after 0.3 s: the file NAME of the directory when
#                    there is one (the query ignored), else a small body
#                    of the type of NAME's extension (as /log/NAME)
#   /delay/MS/NAME   after MS ms: as /slow/NAME (not counted in
#                    /slow-peak); `Cache-Control: no-store` when NAME
#                    has "nostore"
#   /slow-peak       the most /slow/ requests that were in flight at
#                    once, as text/plain
#   /held/NAME       as /slow/NAME, but answered once a /held-now
#                    request counted it instead of after 0.3 s
#   /after-fail/NAME as /slow/NAME, but answered once a /fail/ request
#                    was closed without an answer (waits stop after
#                    HOLD_LIMIT s, which only a broken browser reaches)
#   /held-now        once a /held/ request is in flight, the number in
#                    flight, as text/plain; then it answers them
#                    (waits stop after HOLD_LIMIT s, which only a broken
#                    browser reaches)
#   /held-now?A,B    as /held-now, but once the /held/ requests whose
#                    queries are A and B are in flight (requests on
#                    separate connections arrive in any order)
#   /samesite/...    (GET and POST) the SameSite fixture (P10.21), on the
#                    hosts 127.0.0.1 and localhost of one port (two
#                    sites): /set sets strict, lax, dflt (no SameSite)
#                    and none (SameSite=None without Secure: rejected);
#                    /show a page "METHOD cookies: ..." of the Cookie
#                    header; /echo the Cookie header as text/plain;
#                    /omit-set the same, setting `omitted`; /script.js a
#                    script showing the cookies it was sent; /go?to=U a
#                    302 to U; /embed?host=H a page with an iframe of
#                    H's /log/samesite-frame.html, H's /script.js and fetch() of H's /echo
#                    with each credentials mode (and XHR with and without
#                    withCredentials); /nav?method=M&to=U a page whose
#                    script submits a form to U by M
import gzip
import html
import json
import http.server
import mimetypes
import os
import secrets
import socket
import sys
import threading
import time
import urllib.parse

ROOT = sys.argv[1]

COOKIES = [
    "a=1; Path=/",
    "b=2",
    "c=3; Domain=example.com; Path=/",
    "s=4; Secure; Path=/",
    "p=5; Max-Age=3600; Path=/",
    "gone=6; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    "h=7; Path=/; HttpOnly",
    "__Host-x=8; Path=/",
    "d=9; Domain=127.0.0.1; Path=/; SameSite=Lax",
]

LOG = []

SLOW = {"now": 0, "peak": 0}
SLOW_LOCK = threading.Lock()

# The /held/ requests in flight (and the queries of those), and the
# number of /held-now requests answered: a /held/ request waits until
# that number changes.
HELD = {"now": 0, "counted": 0, "failed": 0, "queries": []}
HELD_CHANGED = threading.Condition()
HOLD_LIMIT = 30

# The login fixture: the accounts, the CSRF tokens handed out and the
# signed-in sessions (sid -> user name).
USERS = {"alice": ("wonderland", "Alice Liddell")}
CSRF = set()
SESSIONS = {}
LOGIN_LOCK = threading.Lock()
SESSION_AGE = 30 * 24 * 3600
BASIC = "Basic " + __import__("base64").b64encode(b"alice:wonderland").decode()

BODIES = {
    ".css": ("text/css", b"p { color: green }"),
    ".js": ("text/javascript", b"window.fromScript = 1;"),
    ".png": ("image/png", b"not really a png"),
}


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=ROOT, **kwargs)

    def log_message(self, *args):
        pass

    def reply(self, status, headers, body=b""):
        self.send_response(status)
        for name, value in headers:
            self.send_header(name, value)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def reply_file(self, path):
        """The file `path` (the query ignored) of the directory when there
        is one, else a small body of the type of its extension."""
        name = urllib.parse.unquote(path.split("?", 1)[0])
        file = os.path.realpath(os.path.join(ROOT, name))
        if file.startswith(os.path.realpath(ROOT) + os.sep) and os.path.isfile(file):
            with open(file, "rb") as f:
                data = f.read()
            ctype = mimetypes.guess_type(file)[0] or "application/octet-stream"
        else:
            ctype, data = BODIES.get(name[name.rfind("."):], ("text/plain", b"slow"))
        headers = [("Content-Type", ctype)]
        if "nostore" in name:
            headers.append(("Cache-Control", "no-store"))
        self.reply(200, headers, data)

    def do_GET(self):
        path = self.path
        if path.startswith("/fetch-"):
            self.fetch_route()
        elif path == "/redirect":
            self.reply(302, [("Location", "/basic.html")])
        elif path == "/redirect-style":
            self.reply(302, [("Location", "/style/link.html")])
        elif path == "/canvas-image-redirect":
            port = self.server.server_address[1]
            self.reply(302, [("Location", f"http://localhost:{port}/paint/img/photo.png")])
        elif path == "/redirect-loop":
            self.reply(302, [("Location", "/redirect-loop")])
        elif path.startswith("/oauth/approve?"):
            query = urllib.parse.parse_qs(urllib.parse.urlsplit(path).query)
            back = query.get("redirect_uri", ["/"])[0]
            extra = urllib.parse.urlencode({"code": "c0de", "state": query.get("state", [""])[0], "seen": query.get("seen", [""])[0]})
            self.reply(302, [("Location", back + "?" + extra)])
        elif path == "/agent":
            agent = self.headers.get("User-Agent", "")
            self.reply(200, [("Content-Type", "text/plain")], agent.encode() + b"\n")
        elif path == "/gzip":
            body = b"<p>" + b"compressed " * 50 + b"</p>"
            headers = [("Content-Type", "text/html; charset=utf-8")]
            if "gzip" in self.headers.get("Accept-Encoding", ""):
                body = gzip.compress(body)
                headers.append(("Content-Encoding", "gzip"))
            self.reply(200, headers, body)
        elif path == "/latin1":
            body = "<p>Café à la crème</p>".encode("latin-1")
            self.reply(200, [("Content-Type", "text/html; charset=ISO-8859-1")], body)
        elif path == "/echo" or path.startswith("/echo?"):
            self.echo("GET", b"")
        elif path == "/net/start":
            self.reply(302, [("Location", "/net/page.html")] + [("Set-Cookie", c) for c in COOKIES])
        elif path == "/headers":
            lines = "".join(
                "%s: %s\n" % (name, self.headers.get(name, ""))
                for name in ["Accept", "Accept-Language", "Referer", "Cookie"]
            )
            self.reply(200, [("Content-Type", "text/plain")], lines.encode())
        elif path == "/log":
            self.reply(200, [("Content-Type", "text/plain")], "".join(sorted(LOG)).encode())
        elif path.startswith("/log/"):
            self.log_request_line("GET")
        elif path.startswith("/fail/"):
            LOG.append("GET " + path + "\n")
            self.close_connection = True
            try:
                self.connection.shutdown(socket.SHUT_RDWR)
            except OSError:
                pass
            with HELD_CHANGED:
                HELD["failed"] += 1
                HELD_CHANGED.notify_all()
        elif path.startswith("/after-fail/"):
            with HELD_CHANGED:
                HELD_CHANGED.wait_for(lambda: HELD["failed"] > 0, HOLD_LIMIT)
            self.reply_file(path[len("/after-fail/"):])
        elif path.startswith("/held/"):
            query = urllib.parse.urlsplit(path).query
            with HELD_CHANGED:
                counted = HELD["counted"]
                HELD["now"] += 1
                HELD["queries"].append(query)
                HELD_CHANGED.notify_all()
                HELD_CHANGED.wait_for(lambda: HELD["counted"] > counted, HOLD_LIMIT)
                HELD["now"] -= 1
                HELD["queries"].remove(query)
            self.reply_file(path[len("/held/"):])
        elif path == "/held-now" or path.startswith("/held-now?"):
            names = [q for q in urllib.parse.urlsplit(path).query.split(",") if q]
            with HELD_CHANGED:
                HELD_CHANGED.wait_for(
                    lambda: HELD["now"] > 0 and all(q in HELD["queries"] for q in names),
                    HOLD_LIMIT,
                )
                now = HELD["now"]
                HELD["counted"] += 1
                HELD_CHANGED.notify_all()
            self.reply(200, [("Content-Type", "text/plain")], b"%d\n" % now)
        elif path.startswith("/slow/"):
            with SLOW_LOCK:
                SLOW["now"] += 1
                SLOW["peak"] = max(SLOW["peak"], SLOW["now"])
            time.sleep(0.3)
            with SLOW_LOCK:
                SLOW["now"] -= 1
            self.reply_file(path[len("/slow/"):])
        elif path.startswith("/delay/"):
            ms, _, name = path[len("/delay/"):].partition("/")
            time.sleep(int(ms) / 1000)
            self.reply_file(name)
        elif path.startswith("/sleep?ms="):
            ms = int(path[len("/sleep?ms="):])
            time.sleep(ms / 1000)
            body = b"<title>Slept</title><p>slept %d ms</p>" % ms
            self.reply(200, [("Content-Type", "text/html")], body)
        elif path == "/slow-peak":
            self.reply(200, [("Content-Type", "text/plain")], b"%d\n" % SLOW["peak"])
        elif path == "/big":
            self.reply(200, [("Content-Type", "text/plain")], b"x" * (33 << 20))
        elif path == "/big?stream":
            self.send_response(200)
            self.send_header("Content-Type", "text/plain")
            self.end_headers()
            for _ in range(33):
                self.wfile.write(b"x" * (1 << 20))
        elif path == "/gbk":
            body = "<title>\u4e2d\u6587</title><p>\u4e2d\u6587\u7f51\u9875</p>".encode("gbk")
            self.reply(200, [("Content-Type", "text/html; charset=GBK")], body)
        elif path == "/login" or path.startswith("/login?"):
            self.login_form()
        elif path == "/account":
            self.account()
        elif path == "/logout":
            self.logout()
        elif path == "/basic":
            self.basic()
        elif path.startswith("/sso/"):
            self.sso()
        elif path.startswith("/samesite/"):
            self.samesite("GET")
        elif path == "/meta-charset":
            body = "<meta charset=windows-1252><p>\u201cquoted\u201d caf\u00e9</p>".encode("cp1252")
            self.reply(200, [("Content-Type", "text/html")], body)
        else:
            super().do_GET()

    def log_request_line(self, method, body=b""):
        parts = [method + " " + self.path]
        for name in ["Accept", "Referer", "Cookie", "Content-Type"]:
            if name in self.headers:
                parts.append("%s: %s" % (name.lower(), self.headers[name]))
        if body:
            parts.append("body: " + body.decode("utf-8", "replace"))
        LOG.append(" | ".join(parts) + "\n")
        name = urllib.parse.urlsplit(self.path).path
        ctype, data = "text/html", b"<title>Logged</title><p>logged " + name.encode() + b"</p>"
        for ext, answer in BODIES.items():
            if name.endswith(ext):
                ctype, data = answer
        headers = [("Content-Type", ctype)]
        if "nostore" in name:
            headers.append(("Cache-Control", "private, no-store"))
        self.reply(200, headers, data)

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0") or "0")
        body = self.rfile.read(length)
        if self.path.startswith("/fetch-"):
            self.fetch_route(body)
        elif self.path == "/login" or self.path.startswith("/login?"):
            self.login_post(body)
        elif self.path == "/logout":
            self.logout()
        elif self.path == "/api/login":
            self.api_login(body)
        elif self.path == "/echo-redirect":
            self.reply(303, [("Location", "/echo?redirected=1")])
        elif self.path.startswith("/samesite/"):
            self.samesite("POST")
        elif self.path.startswith("/log/"):
            self.log_request_line("POST", body)
        elif self.path == "/echo" or self.path.startswith("/echo?"):
            self.echo("POST", body)
        else:
            self.reply(404, [("Content-Type", "text/plain")], b"not found\n")

    def do_OPTIONS(self):
        if not self.path.startswith("/fetch-") or "Access-Control-Request-Method" not in self.headers:
            self.fetch_route()
            return
        headers = [
            ("Access-Control-Allow-Origin", "*"),
            ("Access-Control-Allow-Methods", self.headers["Access-Control-Request-Method"]),
        ]
        if self.headers.get("Access-Control-Request-Headers"):
            headers.append(("Access-Control-Allow-Headers", self.headers["Access-Control-Request-Headers"]))
        self.reply(204, headers)

    def do_HEAD(self):
        if self.path.startswith("/fetch-"):
            self.fetch_route()
        else:
            super().do_HEAD()

    # Every other method (PUT, DELETE, custom ones) goes to fetch_route.
    def __getattr__(self, name):
        if name.startswith("do_"):
            return self.fetch_route
        raise AttributeError(name)

    def fetch_route(self, body=None):
        if body is None:
            length = int(self.headers.get("Content-Length", "0") or "0")
            body = self.rfile.read(length)
        path = urllib.parse.urlsplit(self.path).path
        if path.startswith("/fetch-redirect/"):
            self.reply(int(path[len("/fetch-redirect/"):]), [("Location", "/fetch-echo?redirected"), ("Access-Control-Allow-Origin", "*")])
            return
        if path != "/fetch-echo":
            self.reply(404, [("Content-Type", "text/plain")], b"not found\n")
            return
        echo = {
            "method": self.command,
            "path": self.path,
            "headers": {k.lower(): v for k, v in self.headers.items()},
            "body": body.decode("utf-8", "replace"),
            "hex": body.hex(),
            "fields": form_fields(self.headers.get("Content-Type", ""), body),
        }
        data = json.dumps(echo).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(data)))
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(data)

    # ---- the login fixture

    def cookies(self):
        """The request's cookies, as a dict."""
        jar = {}
        for part in self.headers.get("Cookie", "").split(";"):
            name, eq, value = part.strip().partition("=")
            if eq:
                jar[name] = value
        return jar

    def page(self, status, title, body):
        doc = "<!doctype html><title>%s</title>%s" % (title, body)
        headers = [("Content-Type", "text/html; charset=utf-8"), ("Cache-Control", "no-store")]
        self.reply(status, headers, doc.encode())

    def login_page(self, status, next_path, message):
        token = secrets.token_hex(8)
        with LOGIN_LOCK:
            CSRF.add(token)
        body = (
            "<h1>Sign in</h1><p>%s</p>"
            '<form method="post" action="/login">'
            '<input type="hidden" name="csrf" value="%s">'
            '<input type="hidden" name="next" value="%s">'
            '<p><input name="username" autocomplete="username"></p>'
            '<p><input type="password" name="password" autocomplete="current-password"></p>'
            '<p><button type="submit">Sign in</button></p></form>'
            % (html.escape(message), token, html.escape(next_path, quote=True))
        )
        self.page(status, "Sign in", body)

    def login_form(self):
        query = urllib.parse.parse_qs(urllib.parse.urlsplit(self.path).query)
        self.login_page(200, query.get("next", ["/account"])[0], "Not signed in")

    def login_post(self, body):
        fields = dict(form_fields(self.headers.get("Content-Type", ""), body))
        with LOGIN_LOCK:
            known = fields.get("csrf", "") in CSRF
            CSRF.discard(fields.get("csrf", ""))
        if not known:
            self.page(403, "Forbidden", "<h1>Forbidden</h1><p>bad CSRF token</p>")
            return
        user = USERS.get(fields.get("username", ""))
        next_path = fields.get("next", "/account")
        if not next_path.startswith("/") or next_path.startswith("//"):
            next_path = "/account"
        if user is None or user[0] != fields.get("password", ""):
            self.login_page(401, next_path, "Wrong password")
            return
        sid = secrets.token_hex(16)
        with LOGIN_LOCK:
            SESSIONS[sid] = fields["username"]
        # Secure only over https, which this server never speaks: a
        # Secure cookie would not come back over http.
        secure = "; Secure" if getattr(self.server, "tls", False) else ""
        self.reply(
            303,
            [
                ("Location", next_path),
                ("Set-Cookie", "sid=%s; Path=/; HttpOnly; SameSite=Lax; Max-Age=%d%s" % (sid, SESSION_AGE, secure)),
                ("Set-Cookie", "tab=1; Path=/; SameSite=Lax"),
            ],
        )

    def api_login(self, body):
        if "json" in self.headers.get("Content-Type", ""):
            fields = json.loads(body or b"{}")
        else:
            fields = dict(form_fields(self.headers.get("Content-Type", ""), body))
        user = USERS.get(fields.get("username", ""))
        if user is None or user[0] != fields.get("password", ""):
            self.reply(401, [("Content-Type", "application/json")], b'{"ok":false}')
            return
        sid = secrets.token_hex(16)
        with LOGIN_LOCK:
            SESSIONS[sid] = fields["username"]
        self.reply(
            200,
            [
                ("Content-Type", "application/json"),
                ("Set-Cookie", "sid=%s; Path=/; HttpOnly; SameSite=Lax; Max-Age=%d" % (sid, SESSION_AGE)),
                ("Set-Cookie", "tab=1; Path=/; SameSite=Lax"),
            ],
            b'{"ok":true}',
        )

    def account(self):
        jar = self.cookies()
        with LOGIN_LOCK:
            name = SESSIONS.get(jar.get("sid", ""))
        if name is None:
            self.reply(302, [("Location", "/login?next=/account")])
            return
        body = "<h1>Account</h1><p>Signed in as %s</p><p>tab cookie: %s</p>" % (
            html.escape(USERS[name][1]),
            "yes" if jar.get("tab") == "1" else "no",
        )
        body += '<form method="post" action="/logout"><button>Sign out</button></form>'
        self.page(200, "Account", body)

    def logout(self):
        if self.command == "POST":
            length = int(self.headers.get("Content-Length", "0") or "0")
            self.rfile.read(length)
        with LOGIN_LOCK:
            SESSIONS.pop(self.cookies().get("sid", ""), None)
        self.reply(
            303,
            [
                ("Location", "/login"),
                ("Set-Cookie", "sid=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0"),
                ("Set-Cookie", "tab=; Path=/; SameSite=Lax; Max-Age=0"),
            ],
        )

    def basic(self):
        if self.headers.get("Authorization", "") != BASIC:
            self.reply(
                401,
                [("WWW-Authenticate", 'Basic realm="r"'), ("Content-Type", "text/html")],
                b"<title>Unauthorized</title><p>401: sign in to realm r</p>",
            )
            return
        self.page(200, "Basic", "<p>Basic: signed in as alice</p>")

    def sso(self):
        port = self.server.server_address[1]
        host = self.headers.get("Host", "").split(":")[0]
        other = "localhost" if host == "127.0.0.1" else "127.0.0.1"
        path = urllib.parse.urlsplit(self.path).path
        if path == "/sso/start":
            self.reply(302, [
                ("Location", "http://%s:%d/sso/cb" % (other, port)),
                ("Set-Cookie", "sso1=%s; Path=/" % host),
            ])
        elif path == "/sso/cb":
            self.reply(302, [
                ("Location", "http://127.0.0.1:%d/sso/done" % port),
                ("Set-Cookie", "sso2=%s; Path=/" % host),
            ])
        elif path == "/sso/done":
            cookie = html.escape(self.headers.get("Cookie", ""))
            self.page(200, "SSO", "<p>SSO done on %s, cookies: %s</p>" % (host, cookie))
        else:
            self.reply(404, [("Content-Type", "text/plain")], b"not found\n")

    def samesite(self, method):
        port = self.server.server_address[1]
        parts = urllib.parse.urlsplit(self.path)
        query = dict(urllib.parse.parse_qsl(parts.query))
        cookie = self.headers.get("Cookie", "")
        text_plain = [("Content-Type", "text/plain"), ("Cache-Control", "no-store")]
        if "Origin" in self.headers:
            text_plain += [("Access-Control-Allow-Origin", self.headers["Origin"]), ("Access-Control-Allow-Credentials", "true")]
        if parts.path == "/samesite/set":
            self.reply(200, [
                ("Content-Type", "text/html"),
                ("Set-Cookie", "strict=1; Path=/; SameSite=Strict"),
                ("Set-Cookie", "lax=1; Path=/; SameSite=Lax"),
                ("Set-Cookie", "dflt=1; Path=/"),
                ("Set-Cookie", "none=1; Path=/; SameSite=None"),
            ], b"<p>set</p>")
        elif parts.path == "/samesite/show":
            self.page(200, "Show", "<p>%s cookies: %s</p>" % (method, html.escape(cookie)))
        elif parts.path == "/samesite/echo":
            self.reply(200, text_plain, cookie.encode())
        elif parts.path == "/samesite/omit-set":
            self.reply(200, text_plain + [("Set-Cookie", "omitted=1; Path=/")], cookie.encode())
        elif parts.path == "/samesite/script.js":
            body = "document.getElementById('script').textContent = 'script cookies: ' + %s;" % json.dumps(cookie)
            self.reply(200, [("Content-Type", "text/javascript"), ("Cache-Control", "no-store")], body.encode())
        elif parts.path == "/samesite/go":
            self.reply(302, [("Location", query.get("to", "/"))])
        elif parts.path == "/samesite/embed":
            other = "http://%s:%d" % (query.get("host", "localhost"), port)
            doc = (
                "<iframe src='%s/log/samesite-frame.html'></iframe>"
                "<p id=script>script: not run</p>"
                "<p id=omit></p><p id=same></p><p id=include></p><p id=xhr></p><p id=xhrcred></p><p id=after></p>"
                "<script src='%s/samesite/script.js'></script>"
                "<script>"
                "const at = '%s/samesite/';"
                "const show = (id, t) => { document.getElementById(id).textContent = id + ': ' + t; };"
                "(async () => {"
                "  for (const mode of ['omit', 'same-origin', 'include']) {"
                "    const r = await fetch(at + (mode === 'omit' ? 'omit-set' : 'echo'), { credentials: mode });"
                "    show(mode === 'same-origin' ? 'same' : mode, await r.text());"
                "  }"
                "  for (const cred of [false, true]) {"
                "    const x = new XMLHttpRequest();"
                "    x.open('GET', at + 'echo', false);"
                "    x.withCredentials = cred;"
                "    x.send();"
                "    show(cred ? 'xhrcred' : 'xhr', x.responseText);"
                "  }"
                "  show('after', await (await fetch(at + 'echo')).text());"
                "})();"
                "</script>"
            ) % (other, other, other)
            self.page(200, "Embed", doc)
        elif parts.path == "/samesite/nav":
            doc = (
                "<form method=%s action='%s'><input name=x value=1></form>"
                "<script>document.forms[0].submit()</script>"
            ) % (html.escape(query.get("method", "get")), html.escape(query.get("to", "/")))
            self.page(200, "Nav", doc)
        else:
            self.reply(404, [("Content-Type", "text/plain")], b"not found\n")

    def echo(self, method, body):
        query = urllib.parse.urlsplit(self.path).query
        ctype = self.headers.get("Content-Type", "")
        fields = urllib.parse.parse_qsl(query, keep_blank_values=True)
        if method == "POST":
            fields = form_fields(ctype, body)
        items = "".join(
            "<li>%s = %s</li>" % (html.escape(k), html.escape(v)) for k, v in fields
        )
        raw = body.decode("utf-8", "replace").replace("\r", "\\r")
        page = (
            "<!doctype html><title>Echo</title><h1>%s %s</h1>"
            "<p>Content-Type: %s</p><ul>%s</ul><pre>%s</pre>"
            % (method, html.escape(self.path), html.escape(ctype), items, html.escape(raw))
        )
        self.reply(200, [("Content-Type", "text/html; charset=utf-8")], page.encode())


def form_fields(ctype, body):
    """The (name, value) pairs of a form body."""
    kind = ctype.split(";")[0].strip().lower()
    if kind == "application/x-www-form-urlencoded":
        return urllib.parse.parse_qsl(body.decode("utf-8", "replace"), keep_blank_values=True)
    if kind == "text/plain":
        pairs = []
        for line in body.decode("utf-8", "replace").split("\r\n"):
            if line:
                name, _, value = line.partition("=")
                pairs.append((name, value))
        return pairs
    if kind == "multipart/form-data":
        boundary = ctype.split("boundary=", 1)[1].strip().encode()
        pairs = []
        for part in body.split(b"--" + boundary)[1:]:
            if part.startswith(b"--"):
                break
            head, _, data = part.partition(b"\r\n\r\n")
            disposition = head.decode("utf-8", "replace")
            name = disposition.split('name="', 1)[1].split('"', 1)[0]
            if 'filename="' in disposition:
                name += ' (file "%s")' % disposition.split('filename="', 1)[1].split('"', 1)[0]
            pairs.append((name, data[:-2].decode("utf-8", "replace")))
        return pairs
    return []


server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
print(server.server_address[1], flush=True)
server.serve_forever()
