//! Tests of the Dweller browser (`browser/`, written in Aye).
//!
//! - `browser/tests/X.html`: `dweller dump --tree` must print `X.tree` and
//!   `dweller dump --text` must print `X.txt`, with an empty stderr (the
//!   browser is built in debug mode, so a leak would show there).
//! - `aye test` on the browser's root module and on its standalone modules
//!   (`url`, `dom`, `encoding`) must pass.
//! - A few command-line cases: `data:` URLs, `--id`/`--tag`, errors.
//! - `browser/tests/js/X.js`: `dweller js` must print `X.out` (Node's
//!   output) and exit 0 with an empty stderr, or, when `X.err` exists,
//!   print it on stderr and exit 1 (an uncaught exception).
//! - `browser/tests/js/scopes/X.js` (or `.mjs`): `dweller js
//!   --print-scopes` must print `X.scopes` (the resolver's register and
//!   context allocation, P4.62).
//!
//! To accept new output after a deliberate change, run
//! `DWELLER_BLESS=1 cargo test --test browser` and review the diff.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

mod common;

#[path = "browser/agent.rs"]
mod agent;
#[path = "browser/cli.rs"]
mod cli;
#[path = "browser/csp_directives.rs"]
mod csp_directives;
#[path = "browser/dweller_command.rs"]
mod dweller_command;
#[path = "browser/link_events.rs"]
mod link_events;
#[path = "browser/fixtures.rs"]
mod fixtures;
#[path = "browser/history.rs"]
mod history;
#[path = "browser/input.rs"]
mod input;
#[path = "browser/js.rs"]
mod js;
#[path = "browser/jsdom.rs"]
mod jsdom;
#[path = "browser/media.rs"]
mod media;
#[path = "browser/net.rs"]
mod net;
#[path = "browser/paint.rs"]
mod paint;
#[path = "browser/perf.rs"]
mod perf;
#[path = "browser/screenshot.rs"]
mod screenshot;
#[path = "browser/scripts.rs"]
mod scripts;
#[path = "browser/security.rs"]
mod security;
#[path = "browser/storage.rs"]
mod storage;
#[path = "browser/webgl.rs"]
mod webgl;
#[path = "browser/window_login.rs"]
mod window_login;
#[path = "browser/window_renderer.rs"]
mod window_renderer;
#[path = "browser/window_renderer_loads.rs"]
mod window_renderer_loads;
#[path = "browser/window_renderer_oopif.rs"]
mod window_renderer_oopif;
#[path = "browser/window_renderer_oopif_input.rs"]
mod window_renderer_oopif_input;
#[path = "browser/window_renderer_scroll.rs"]
mod window_renderer_scroll;
#[path = "browser/window_spaces.rs"]
mod window_spaces;
#[path = "browser/wpt.rs"]
mod wpt;

use dweller_command::dweller_command;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn aye() -> Command {
    let mut c = Command::new(std::env::var_os("AYE").unwrap_or("aye".into()));
    c.current_dir(root());
    // libwasmtime from browser/wasm/fetch_wasmtime.sh, when fetched: its
    // pkg-config file links it with an rpath (P11.17).
    if let Some(pc) = wasmtime_pkgconfig() {
        let mut path = std::ffi::OsString::from(&pc);
        if let Some(old) = std::env::var_os("PKG_CONFIG_PATH") {
            path.push(":");
            path.push(old);
        }
        c.env("PKG_CONFIG_PATH", path);
    }
    c
}

/// target/wasmtime/<version>/lib/pkgconfig, the version of
/// browser/wasm/wasmtime.lock, when browser/wasm/fetch_wasmtime.sh made it.
fn wasmtime_pkgconfig() -> Option<PathBuf> {
    let lock = std::fs::read_to_string(root().join("browser/wasm/wasmtime.lock")).ok()?;
    let version = lock.lines().find_map(|l| l.strip_prefix("version "))?;
    let dir = root()
        .join("target/wasmtime")
        .join(version.trim())
        .join("lib/pkgconfig");
    dir.join("wasmtime.pc").exists().then_some(dir)
}

/// A child whose stdout and stderr are read on threads while it runs
/// (`SpawnDrained`), so that a window's verbose output (DWELLER_TIMING)
/// cannot fill a pipe and block it while the test waits for it to exit.
struct Drained {
    child: std::process::Child,
    out: std::thread::JoinHandle<Vec<u8>>,
    err: std::thread::JoinHandle<Vec<u8>>,
}

impl Drained {
    fn id(&self) -> u32 {
        self.child.id()
    }

    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }

    fn kill(&mut self) -> std::io::Result<()> {
        self.child.kill()
    }

    /// Waits for the child alone (what it started may keep its pipes).
    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait()
    }

    /// Waits for the child to exit and for its pipes to close.
    fn wait_with_output(mut self) -> std::io::Result<Output> {
        let status = self.child.wait()?;
        let stdout = self.out.join().unwrap_or_default();
        let stderr = self.err.join().unwrap_or_default();
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }

    /// `wait_with_output`, but the child is killed after `limit`: a run
    /// that hangs fails its test instead of holding it for hours.
    fn wait_with_output_within(mut self, limit: std::time::Duration) -> std::io::Result<Output> {
        let start = std::time::Instant::now();
        while self.try_wait()?.is_none() {
            if start.elapsed() > limit {
                let _ = self.kill();
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        self.wait_with_output()
    }
}

trait SpawnDrained {
    /// Spawns the command with its stdout and stderr piped and read on
    /// threads (`Drained`).
    fn spawn_drained(&mut self) -> Drained;
}

impl SpawnDrained for Command {
    fn spawn_drained(&mut self) -> Drained {
        use std::io::Read;
        let mut child = self
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        let out = std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = stdout.read_to_end(&mut b);
            b
        });
        let err = std::thread::spawn(move || {
            let mut b = Vec::new();
            let _ = stderr.read_to_end(&mut b);
            b
        });
        Drained { child, out, err }
    }
}

/// The browser, built once for all tests.
fn dweller() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let dir = std::env::temp_dir().join(format!("dweller-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("dweller");
        let out = aye()
            .arg("build")
            .arg("browser")
            .arg("--mode")
            .arg("safe")
            .arg("--leak-check")
            .arg("-o")
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "building the browser failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        exe
    })
}

fn run(args: &[&str]) -> Output {
    dweller_command()
        .current_dir(root())
        .args(args)
        .output()
        .unwrap()
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

/// Whether ANGLE is there for the WebGL tests: `DWELLER_ANGLE` or
/// `target/angle/<rev>/lib`. Without it, the WebGL tests check the
/// unavailable path, and only while browser/gpu/angle.lock has no hashes
/// (CI fetches ANGLE once it has them).
fn angle_present() -> bool {
    let rev = std::fs::read_to_string(root().join("browser/gpu/angle.rev")).unwrap();
    let lib = root().join("target/angle").join(rev.trim()).join("lib");
    let present = std::env::var_os("DWELLER_ANGLE").is_some()
        || ["libEGL.dylib", "libEGL.so"]
            .iter()
            .any(|f| lib.join(f).exists());
    if !present {
        let lock = std::fs::read_to_string(root().join("browser/gpu/angle.lock")).unwrap();
        let pinned = lock
            .lines()
            .any(|l| !l.starts_with('#') && !l.trim().is_empty());
        assert!(
            !pinned,
            "ANGLE not found in {}: run browser/gpu/fetch_angle.sh",
            lib.display()
        );
    }
    present
}

/// `dweller args...` must exit with `status` and print exactly `stdout`
/// (and `stderr`).
fn expect(args: &[&str], status: i32, stdout: &str, stderr: &str) {
    let out = run(args);
    assert_eq!(text(&out.stdout), stdout, "stdout of dweller {args:?}");
    assert_eq!(text(&out.stderr), stderr, "stderr of dweller {args:?}");
    assert_eq!(
        out.status.code(),
        Some(status),
        "status of dweller {args:?}"
    );
}

/// The local test server (`browser/tests/server.py`) serving
/// `browser/tests`; killed when dropped. `None` without python3.
struct Server {
    child: std::process::Child,
    base: String,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn server() -> Option<Server> {
    server_on("127.0.0.1")
}

fn server_on(host: &str) -> Option<Server> {
    use std::io::BufRead;
    let tests = root().join("browser").join("tests");
    let mut child = Command::new("python3")
        .arg("-I")
        .arg(tests.join("server.py"))
        .arg(&tests)
        .arg(host)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    let mut line = String::new();
    std::io::BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .ok()?;
    let port: u16 = line.trim().parse().ok()?;
    let shown = if host.contains(':') {
        format!("[{host}]")
    } else {
        host.to_string()
    };
    Some(Server {
        child,
        base: format!("http://{shown}:{port}"),
    })
}

// ---- B4: paint and window

/// A scratch directory for the files the browser writes.
fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dweller-out-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                0xEDB8_8320 ^ (crc >> 1)
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

/// Checks a PNG: signature, chunk CRCs, IHDR (8-bit RGBA), and, when
/// python3 is there, that zlib inflates the IDAT data to the image's
/// size. Returns (width, height).
fn check_png(path: &Path) -> (u32, u32) {
    let d = std::fs::read(path).unwrap();
    assert_eq!(&d[..8], b"\x89PNG\r\n\x1a\n", "{}", path.display());
    let be = |b: &[u8]| u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
    let (mut i, mut size, mut kinds) = (8, (0, 0), Vec::new());
    while i < d.len() {
        let n = be(&d[i..]) as usize;
        let kind = &d[i + 4..i + 8];
        assert_eq!(
            crc32(&d[i + 4..i + 8 + n]),
            be(&d[i + 8 + n..]),
            "CRC of {} in {}",
            String::from_utf8_lossy(kind),
            path.display()
        );
        if kind == b"IHDR" {
            size = (be(&d[i + 8..]), be(&d[i + 12..]));
            assert_eq!(
                &d[i + 16..i + 21],
                &[8, 6, 0, 0, 0],
                "bit depth, color type and methods"
            );
        }
        kinds.push(String::from_utf8_lossy(kind).into_owned());
        i += 12 + n;
    }
    assert_eq!(kinds, ["IHDR", "IDAT", "IEND"], "{}", path.display());
    let script = "import sys, zlib, struct\nd = open(sys.argv[1], 'rb').read()\nn = struct.unpack('>I', d[33:37])[0]\nprint(len(zlib.decompress(d[41:41 + n])))";
    if let Ok(out) = Command::new("python3")
        .args(["-I", "-c", script])
        .arg(path)
        .output()
    {
        if out.status.success() {
            let len: u64 = text(&out.stdout).trim().parse().unwrap();
            assert_eq!(
                len,
                size.1 as u64 * (size.0 as u64 * 4 + 1),
                "inflated size of {}",
                path.display()
            );
        }
    }
    size
}

const WINDOW_USAGE: &str = "usage: dweller [open] [FILE|URL] [--width W] [--height H] [--font fixed|system] [--no-js] [--agent-socket PATH [--exit-when-idle]] [--frames N [--until-idle] [--events SCRIPT] [--state] [--text] [--save OUT.png]]";

fn has_sdl() -> bool {
    Command::new("pkg-config")
        .args(["--exists", "sdl2"])
        .status()
        .is_ok_and(|s| s.success())
}

/// `dweller args...` with SDL's dummy video driver (no display).
fn run_window(args: &[&str]) -> Output {
    dweller_command()
        .current_dir(root())
        .args(args)
        .output()
        .unwrap()
}

// ---- the agent interface (browser/agent, docs/agent-protocol.md)

/// `s` with the repository's path written as `$ROOT`, so that goldens
/// with absolute `file:` URLs do not depend on the checkout.
fn rooted(s: &str) -> String {
    s.replace(&root().display().to_string(), "$ROOT")
}

/// Compares `got` with the golden file (or writes it when blessing).
fn check_golden(golden: &Path, got: &str, what: &str, failures: &mut Vec<String>) {
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(golden, got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(golden).unwrap_or_default();
    if got != want {
        failures.push(format!(
            "{what}: output differs from {}\n--- expected\n{want}--- got\n{got}",
            golden.display()
        ));
    }
}

// A headless window listening at `sock` for the Space tests, its output
// read as it comes (`Drained`).
fn spaces_window(sock: &str, extra: &[&str]) -> Drained {
    spaces_window_env(sock, extra, &[])
}

fn spaces_window_env(sock: &str, extra: &[&str], env: &[(&str, &str)]) -> Drained {
    use std::time::{Duration, Instant};
    let _ = std::fs::remove_file(root().join(sock));
    let mut window = dweller_command()
        .current_dir(root())
        .envs(env.iter().copied())
        .args(["--agent-socket", sock, "--exit-when-idle", "--font", "fixed"])
        .args(["--width", "800", "--height", "600"])
        .args(extra)
        .spawn_drained();
    let start = Instant::now();
    while !root().join(sock).exists() {
        if start.elapsed() > Duration::from_secs(20) || window.try_wait().unwrap().is_some() {
            let _ = window.kill();
            let out = window.wait_with_output().unwrap();
            panic!("the window does not listen: {}", text(&out.stderr));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    window
}

// The first line of a connection to the window's socket `sock`: the
// token the window wrote next to it (Q3.12).
fn token_line(sock: &str) -> String {
    let token = std::fs::read_to_string(root().join(format!("{sock}.token"))).unwrap();
    format!("{{\"token\":\"{token}\"}}\n")
}

// A connection to the window's socket (relative to the root: short).
struct AgentConn {
    stream: std::os::unix::net::UnixStream,
    read: std::io::BufReader<std::os::unix::net::UnixStream>,
}

impl AgentConn {
    fn open(sock: &str) -> AgentConn {
        let path = root().join(sock);
        let rel = path.strip_prefix(std::env::current_dir().unwrap()).unwrap_or(&path);
        let stream = std::os::unix::net::UnixStream::connect(rel).unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(30)))
            .unwrap();
        let read = std::io::BufReader::new(stream.try_clone().unwrap());
        let mut conn = AgentConn { stream, read };
        conn.send(token_line(sock).trim_end());
        conn
    }

    fn send(&mut self, line: &str) {
        use std::io::Write;
        self.stream.write_all(line.as_bytes()).unwrap();
        self.stream.write_all(b"\n").unwrap();
    }

    // The next line ("" at the end).
    fn line(&mut self) -> String {
        use std::io::BufRead;
        let mut s = String::new();
        self.read.read_line(&mut s).unwrap();
        s
    }

    // A request and its answer, and how long it took in ms.
    fn ask(&mut self, line: &str) -> (String, u128) {
        let start = std::time::Instant::now();
        self.send(line);
        let got = self.line();
        (got, start.elapsed().as_millis())
    }
}

// `s` as a JSON string (quotes and backslashes escaped).
fn serde_like(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The pixels of a PNG as `dweller screenshot` writes it
/// (`browser/paint/png.aye`: 8-bit RGBA, filter 0 on every row, a zlib
/// stream of stored or fixed-Huffman blocks): width, height, RGBA.
fn png_rgba(png: &[u8]) -> (usize, usize, Vec<u8>) {
    let be = |i: usize| u32::from_be_bytes(png[i..i + 4].try_into().unwrap()) as usize;
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&png[24..29], b"\x08\x06\x00\x00\x00", "not 8-bit RGBA");
    let (w, h) = (be(16), be(20));
    let mut z = Vec::new();
    let mut i = 8;
    while i + 8 <= png.len() {
        let len = be(i);
        if &png[i + 4..i + 8] == b"IDAT" {
            z.extend_from_slice(&png[i + 8..i + 8 + len]);
        }
        i += 12 + len;
    }
    let raw = inflate(&z[2..]);
    assert_eq!(raw.len(), h * (w * 4 + 1));
    let mut px = Vec::with_capacity(w * h * 4);
    for row in raw.chunks(w * 4 + 1) {
        assert_eq!(row[0], 0, "a row filter other than none");
        px.extend_from_slice(&row[1..]);
    }
    (w, h, px)
}

/// Raw deflate (RFC 1951) of stored and fixed-Huffman blocks only.
fn inflate(d: &[u8]) -> Vec<u8> {
    const LEN: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LEN_EXTRA: [usize; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    let mut pos = 0usize;
    // `n` bits, least significant first.
    let take = |n: usize, pos: &mut usize| -> usize {
        let mut v = 0;
        for k in 0..n {
            v |= ((d[*pos / 8] >> (*pos % 8)) as usize & 1) << k;
            *pos += 1;
        }
        v
    };
    let mut out = Vec::new();
    loop {
        let last = take(1, &mut pos);
        let kind = take(2, &mut pos);
        if kind == 0 {
            pos = pos.div_ceil(8) * 8;
            let n = take(16, &mut pos);
            take(16, &mut pos);
            out.extend_from_slice(&d[pos / 8..pos / 8 + n]);
            pos += n * 8;
        } else {
            assert_eq!(kind, 1, "a dynamic-Huffman block");
            loop {
                // Huffman codes come most significant bit first.
                let mut code = 0;
                for _ in 0..7 {
                    code = code << 1 | take(1, &mut pos);
                }
                let sym = if code <= 0x17 {
                    256 + code
                } else {
                    code = code << 1 | take(1, &mut pos);
                    if (0x30..=0xbf).contains(&code) {
                        code - 0x30
                    } else if (0xc0..=0xc7).contains(&code) {
                        280 + code - 0xc0
                    } else {
                        (code << 1 | take(1, &mut pos)) - 0x190 + 144
                    }
                };
                if sym < 256 {
                    out.push(sym as u8);
                    continue;
                }
                if sym == 256 {
                    break;
                }
                let s = sym - 257;
                let len = LEN[s] + take(LEN_EXTRA[s], &mut pos);
                let mut dc = 0;
                for _ in 0..5 {
                    dc = dc << 1 | take(1, &mut pos);
                }
                let dist = DIST[dc] + take(dc.saturating_sub(2) / 2, &mut pos);
                for _ in 0..len {
                    out.push(out[out.len() - dist]);
                }
            }
        }
        if last == 1 {
            return out;
        }
    }
}

// ---- the window's loop iterations, from its DWELLER_TIMING records

/// A phase that ended in a loop iteration of the window: a DWELLER_TIMING
/// record (`info: DWELLER_TIMING\tPHASE\tNS\tLABEL`), in ms.
#[derive(Debug)]
struct Phase {
    name: String,
    ms: u64,
    label: String,
}

/// A loop iteration of the window: its `ui_loop` time (ms, without its
/// wait for events) and the phases that ended in it, in order.
#[derive(Debug, Default)]
struct Iteration {
    ms: u64,
    phases: Vec<Phase>,
}

/// The phases in which the window's thread waits for the network: a
/// request made at once (one in the cache returns at once), a batch of
/// prefetches, a task's wait for its background load.
const NET_WAITS: [&str; 3] = ["requests", "prefetch", "image_wait"];

/// The longest a loop iteration may wait for the network (`NET_WAITS`,
/// summed). What arrived in the background is read from the cache in
/// well under 1 ms; the slow parts of the pages take 200 ms or more.
const NET_WAIT_MS: u64 = 100;

/// The longest a loop iteration may take besides running a page's
/// scripts (making its realm and running them is CPU work that a loaded
/// machine makes slow). Its other work takes a few ms, tens on a loaded
/// machine; a wait for the slow parts of the pages takes 1 s or more.
const LOOP_MS: u64 = 1000;

impl Iteration {
    /// The ms of its phases named in `names`.
    fn ms_in(&self, names: &[&str]) -> u64 {
        self.phases
            .iter()
            .filter(|p| names.contains(&p.name.as_str()))
            .map(|p| p.ms)
            .sum()
    }

    /// Its waits for the network (`NET_WAITS`), but for the request of
    /// the document `page` itself, which the navigation makes at once.
    fn net_wait_ms(&self, page: &str) -> u64 {
        self.phases
            .iter()
            .filter(|p| NET_WAITS.contains(&p.name.as_str()))
            .filter(|p| !(p.name == "requests" && p.label == page))
            .map(|p| p.ms)
            .sum()
    }

    /// Does a phase `name` whose label contains `label` end in it?
    fn has(&self, name: &str, label: &str) -> bool {
        self.phases
            .iter()
            .any(|p| p.name == name && p.label.contains(label))
    }
}

/// The window's loop iterations from `err`, its stderr, which must hold
/// only DWELLER_TIMING records. A record is written when its phase ends,
/// so before the `ui_loop` record of its iteration; the records before
/// the first `ui_loop` (opening the page) belong to the first iteration,
/// those after the last (the iteration that drew the last frame and
/// ended the window) make an iteration of 0 ms.
fn iterations(err: &str) -> Vec<Iteration> {
    let mut v = vec![Iteration::default()];
    for line in err.lines() {
        let fields: Vec<&str> = line.trim_start_matches("info: ").split('\t').collect();
        assert!(
            fields.len() == 4 && fields[0] == "DWELLER_TIMING",
            "stderr has more than timing records:\n{err}"
        );
        let ms = fields[2].parse::<u64>().unwrap() / 1_000_000;
        if fields[1] == "ui_loop" {
            v.last_mut().unwrap().ms = ms;
            v.push(Iteration::default());
        } else {
            v.last_mut().unwrap().phases.push(Phase {
                name: fields[1].to_string(),
                ms,
                label: fields[3].to_string(),
            });
        }
    }
    v
}

/// The window never waits for the network in a loop iteration: in each,
/// the waits are short (`NET_WAIT_MS`), and what is not a page's scripts
/// takes less than `LOOP_MS`. Panics with the iteration otherwise.
fn assert_no_net_wait(loops: &[Iteration], page: &str) {
    for (i, it) in loops.iter().enumerate() {
        let waited = it.net_wait_ms(page);
        let other = it.ms.saturating_sub(it.ms_in(&["scripts"]));
        assert!(
            waited < NET_WAIT_MS && other < LOOP_MS,
            "iteration {i} of {} waited {waited} ms for the network, {other} ms besides scripts: {it:#?}",
            loops.len()
        );
    }
}

// ---- P10.20: logged-in browsing (the login fixture of server.py)

/// The `--events` words that type `s` (at most 31 bytes a word).
fn typed(s: &str) -> String {
    s.as_bytes()
        .chunks(31)
        .map(|c| format!("text:{}", std::str::from_utf8(c).unwrap()))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The `--events` words that go to `url` through the address bar.
fn go_to(url: &str) -> String {
    format!("key:cmd+l {} key:return frame frame", typed(url))
}

/// The window on `url` with `events`, headless, with the profile
/// directory `profile` when given (`DWELLER_PROFILE`, P4.50); its
/// `--state` and `--text`.
fn login_window(url: &str, events: &str, profile: Option<&Path>) -> String {
    login_window_with(url, events, profile, &[])
}

/// With `envs` (`DWELLER_PROCESS=tab`) the page loads in a renderer, so
/// the script first waits for it, and after each Enter for the input to
/// be handled and the navigation it started to load.
fn login_window_with(url: &str, events: &str, profile: Option<&Path>, envs: &[(&str, &str)]) -> String {
    let waited = format!(
        "renderer:loaded {}",
        events.replace("key:return", "key:return renderer:input renderer:loaded")
    );
    let events = if envs.is_empty() { events } else { &waited };
    let mut c = dweller_command();
    c.current_dir(root()).envs(envs.iter().copied());
    if let Some(p) = profile {
        c.env("DWELLER_PROFILE", p);
    }
    let out = c
        .args([
            url, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400",
            "--until-idle", "--state", "--text", "--events", events,
        ])
        .output()
        .unwrap();
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{url} --events {events:?}: {}\n{}\n{got}",
        out.status,
        text(&out.stderr)
    );
    got
}

/// A fresh profile directory for one test.
fn profile(name: &str) -> PathBuf {
    let dir = scratch().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// The ref of the element named `name` in a snapshot answer (`e7`).
fn ref_named(snap: &str, name: &str) -> String {
    let at = snap.find(&format!("{name}\\\" [e")).unwrap_or_else(|| panic!("no {name} in {snap}"));
    let rest = &snap[at + name.len() + 4..];
    rest[..rest.find(']').unwrap()].to_string()
}
