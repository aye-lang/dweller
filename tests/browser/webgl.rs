use super::*;

/// `dweller gpu-smoke` on SwiftShader (P11.2, docs/webgl.md): a 64x64
/// framebuffer cleared to red reads back `ff0000ff`. ANGLE comes from
/// `DWELLER_ANGLE` or `target/angle/<rev>/lib` (browser/gpu/fetch_angle.sh).
/// Until browser/gpu/angle.lock has hashes, CI has no ANGLE: then the test
/// checks the "WebGL unavailable" path and skips the pixels.
#[test]
fn gpu_smoke() {
    let rev = std::fs::read_to_string(root().join("browser/gpu/angle.rev")).unwrap();
    let lib = root().join("target/angle").join(rev.trim()).join("lib");
    let present = std::env::var_os("DWELLER_ANGLE").is_some()
        || ["libEGL.dylib", "libEGL.so"]
            .iter()
            .any(|f| lib.join(f).exists());
    let out = dweller_command()
        .current_dir(root())
        .arg("gpu-smoke")
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    let got = text(&out.stdout);
    assert!(
        out.status.success(),
        "{}\n{got}\n{}",
        out.status,
        text(&out.stderr)
    );
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
        assert!(got.contains("WebGL unavailable: ANGLE not found"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    assert!(got.starts_with("backend swiftshader\n"), "{got}");
    assert!(got.contains("SwiftShader"), "{got}");
    assert!(got.ends_with("ff0000ff\nok\n"), "{got}");
}

/// P11.4 (docs/webgl.md §2, §3): `dump --text --js` of
/// `browser/tests/webgl/context.html` on SwiftShader prints its `.txt`:
/// the context attributes, drawing buffer size, limits, `getError`
/// sequences, cleared pixels, and the drawing buffer cleared after a
/// frame. Without ANGLE, `getContext('webgl2')` gives null.
#[test]
fn webgl_context() {
    let page = "browser/tests/webgl/context.html";
    let out = dweller_command()
        .current_dir(root())
        .args(["dump", "--text", "--js", page])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(
            got.contains("webgl2 null\n2d after a failed webgl2 true"),
            "{got}"
        );
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/context.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.4: `dweller screenshot` of `browser/tests/webgl/clear.html` on
/// SwiftShader shows its WebGL canvases (scissored clears, premultiplied
/// alpha over the page, a frame drawn in requestAnimationFrame) exactly
/// as `clear.png`. Without ANGLE the canvases stay blank.
#[test]
fn webgl_screenshot() {
    let out_png = scratch().join("webgl_clear.png");
    let res = dweller_command()
        .current_dir(root())
        .args([
            "screenshot",
            "browser/tests/webgl/clear.html",
            "-o",
            out_png.to_str().unwrap(),
            "--font",
            "fixed",
            "--width",
            "200",
            "--height",
            "180",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}\n{}",
        res.status,
        text(&res.stderr)
    );
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let got = std::fs::read(&out_png).unwrap();
    let golden = root().join("browser/tests/webgl/clear.png");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    assert!(
        got == std::fs::read(&golden).unwrap(),
        "the image differs from {} (new one: {})",
        golden.display(),
        out_png.display()
    );
}

/// P11.5 (docs/webgl.md §3, §4): `dweller screenshot` on SwiftShader of
/// the pages in `browser/tests/webgl` that draw: a triangle with
/// per-vertex colors (`triangle.html`), an indexed quad with a uniform
/// color and matrix (`quad.html`), and the info logs of a shader that
/// does not compile and a program that does not link
/// (`shader_errors.html`); each is its `.png`. Without ANGLE the canvases
/// stay blank.
#[test]
fn webgl_triangle() {
    for page in ["triangle", "quad", "shader_errors"] {
        let out_png = scratch().join(format!("webgl_{page}.png"));
        let res = dweller_command()
            .current_dir(root())
            .args([
                "screenshot",
                &format!("browser/tests/webgl/{page}.html"),
                "-o",
                out_png.to_str().unwrap(),
                "--font",
                "fixed",
                "--width",
                "200",
                "--height",
                "160",
            ])
            .env("DWELLER_GL", "swiftshader")
            .output()
            .unwrap();
        assert!(
            res.status.success() && res.stderr.is_empty(),
            "{page}: {}\n{}",
            res.status,
            text(&res.stderr)
        );
        if !angle_present() {
            println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
            return;
        }
        let got = std::fs::read(&out_png).unwrap();
        let golden = root().join(format!("browser/tests/webgl/{page}.png"));
        if std::env::var_os("DWELLER_BLESS").is_some() {
            std::fs::write(&golden, &got).unwrap();
            continue;
        }
        assert!(
            got == std::fs::read(&golden).unwrap(),
            "the image differs from {} (new one: {})",
            golden.display(),
            out_png.display()
        );
    }
}

/// P11.5: `dump --text --js` of `browser/tests/webgl/objects.html` on
/// SwiftShader prints `objects.txt`: the error of each WebGL object rule
/// (another context, deleted objects, buffer targets, uniform locations
/// after a relink, draws), active attributes and uniforms, and the pixels
/// of a drawn triangle.
#[test]
fn webgl_objects() {
    let out = dweller_command()
        .current_dir(root())
        .args(["dump", "--text", "--js", "browser/tests/webgl/objects.html"])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/objects.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.6 (docs/webgl.md §3): `dump --text --js` of
/// `browser/tests/webgl/textures.html` on SwiftShader, served over HTTP
/// (the page loads an image from a second origin), prints
/// `textures.txt`: a textured quad from a PNG, a 2D canvas, an
/// OffscreenCanvas, an ImageData, ImageBitmaps and typed arrays read back
/// with and without UNPACK_FLIP_Y_WEBGL and UNPACK_PREMULTIPLY_ALPHA_WEBGL,
/// the object and pixel store rules, and the SecurityError of a
/// cross-origin image. It samples with NEAREST at texel centers, so the
/// pixels do not depend on the backend.
#[test]
fn webgl_textures() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = dweller_command()
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            &format!("{}/webgl/textures.html", srv.base),
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/textures.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.8 (docs/webgl.md §3): `dump --text --js` of
/// `browser/tests/webgl/webgl2_objects.html` on SwiftShader prints
/// `webgl2_objects.txt`: VAOs, instanced draws, drawRangeElements,
/// vertexAttribIPointer, a float MRT framebuffer sampled afterwards,
/// renderbuffers with a multisample blit and a depth test, uniform
/// buffers, samplers, 2D-array and 3D textures and float and integer
/// formats, each drawn and read back.
#[test]
fn webgl2_objects() {
    let out = dweller_command()
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            "browser/tests/webgl/webgl2_objects.html",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/webgl2_objects.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.12: `browser/tests/webgl/webgl2_rest.html` on SwiftShader prints
/// `webgl2_rest.txt`: occlusion and transform feedback queries and fences
/// (available only after the task), transform feedback, buffer copies and
/// readback, readPixels into a pixel-pack buffer, copyTexSubImage2D/3D,
/// clearBufferfi, invalidateFramebuffer and the indexed, internal format,
/// buffer and vertex attribute getters, with the errors of each.
#[test]
fn webgl2_rest() {
    let out = dweller_command()
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            "browser/tests/webgl/webgl2_rest.html",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/webgl2_rest.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.13: `browser/tests/webgl/extensions.html` on SwiftShader prints
/// `extensions.txt`: the supported list, the objects three.js probes,
/// rendering to an RGBA16F framebuffer, anisotropy set and read back, a
/// compressed texture uploaded and sampled, and `getExtension('NOPE')`.
#[test]
fn webgl_extensions() {
    let out = dweller_command()
        .current_dir(root())
        .args([
            "dump",
            "--text",
            "--js",
            "browser/tests/webgl/extensions.html",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    if !angle_present() {
        assert!(got.contains("webgl2 null"), "{got}");
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let golden = root().join("browser/tests/webgl/extensions.txt");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
        return;
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    assert_eq!(got, want, "differs from {}", golden.display());
}

/// P11.7 (docs/webgl.md §5): the window shows a WebGL canvas through its
/// readback. `spin.html` turns a triangle on a 1280x720 canvas in 120
/// animation frames; each frame is read back at its end and mapped into
/// the canvas in the next, so the canvas changes in 120 frames (and the
/// cleared one before them), each damaging the canvas alone (with the
/// one-pixel margin below it the damage keeps for antialiasing), and
/// `DWELLER_DAMAGE_CHECK` finds every frame drawn right. Prints the
/// `webgl_readback` p50 and p95. Without ANGLE it is skipped.
#[test]
fn window_webgl() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_DAMAGE_CHECK", "1")
        .env("DWELLER_TIMING", "1")
        .env("DWELLER_GL", "swiftshader")
        .args([
            "browser/tests/webgl/spin.html",
            "--font",
            "fixed",
            "--width",
            "1280",
            "--height",
            "800",
            "--frames",
            "1000",
            "--until-idle",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(!err.contains("DAMAGE CHECK"), "{err}");
    let record = |phase: &str| -> Vec<(u64, String)> {
        let key = format!("DWELLER_TIMING\t{phase}\t");
        err.lines()
            .filter_map(|l| l.split(key.as_str()).nth(1))
            .map(|r| {
                let mut f = r.split('\t');
                let ns = f.next().unwrap().parse().unwrap();
                (ns, f.next().unwrap_or("").to_string())
            })
            .collect()
    };
    let mut readback: Vec<u64> = record("webgl_readback")
        .into_iter()
        .map(|(ns, label)| {
            assert_eq!(label, "1280x720");
            ns
        })
        .collect();
    assert!(readback.len() >= 120, "{} readbacks:\n{err}", readback.len());
    // The page area is 1280x760 (the chrome takes 40 px).
    let canvas = 1280 * 721;
    let full = 1280 * 760;
    let px: Vec<u64> = record("ui_damage_px")
        .iter()
        .map(|(_, label)| label.split(' ').next().unwrap().parse().unwrap())
        .collect();
    assert!(
        px.iter().all(|&p| p == 0 || p == canvas || p == full),
        "damage outside the canvas: {px:?}"
    );
    let updates = px.iter().filter(|&&p| p == canvas).count();
    assert!(updates >= 120, "{updates} canvas updates: {px:?}");
    assert!(px.iter().filter(|&&p| p == full).count() <= 2, "{px:?}");
    readback.sort();
    let ms = |q: f64| readback[((readback.len() as f64 * q) as usize).min(readback.len() - 1)] as f64 / 1e6;
    println!("webgl_readback 1280x720: p50 {:.2} ms, p95 {:.2} ms", ms(0.5), ms(0.95));
}

/// P11.9: three.js r170 (`browser/tests/webgl/three/`) draws
/// `three_cube.html` on SwiftShader: a MeshStandardMaterial cube with a
/// PNG texture and one directional light, in a 60-frame
/// `setAnimationLoop`. `dump --js` logs its message and no error; `dweller screenshot`
/// (the last frame) is `three_cube.png` but for at most 1% of the pixels
/// off by more than 8 in a channel (SwiftShader on another CPU may round
/// otherwise); in the window, the 60 frames draw (60 `webgl_readback`
/// records of 240x180). Prints the JS time of the drawing frames' timers
/// (the rAF callback, three's render and the readback's start) p50/p95,
/// and the same with `#finish` (the page calls gl.finish() after each
/// render): their difference is the GPU's time. Without ANGLE it is
/// skipped.
#[test]
fn webgl_three_cube() {
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let page = "browser/tests/webgl/three_cube.html";
    let out = dweller_command()
        .current_dir(root())
        .args(["dump", "--text", "--js", page])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    // The console (on stderr) has the page's one message and no error.
    assert!(out.status.success(), "{}", out.status);
    assert_eq!(text(&out.stderr), "[log] three cube: 60 frames\n");

    let out_png = scratch().join("three_cube.png");
    let res = dweller_command()
        .current_dir(root())
        .args([
            "screenshot",
            page,
            "-o",
            out_png.to_str().unwrap(),
            "--font",
            "fixed",
            "--width",
            "240",
            "--height",
            "180",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}\n{}",
        res.status,
        text(&res.stderr)
    );
    let got = std::fs::read(&out_png).unwrap();
    let golden = root().join("browser/tests/webgl/three_cube.png");
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    }
    let (w, h, a) = png_rgba(&got);
    let (gw, gh, b) = png_rgba(&std::fs::read(&golden).unwrap());
    assert_eq!((w, h), (gw, gh), "size differs from {}", golden.display());
    let off = a
        .chunks(4)
        .zip(b.chunks(4))
        .filter(|(p, q)| p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > 8))
        .count();
    assert!(
        off * 100 <= w * h,
        "{off} pixels differ from {} (new one: {})",
        golden.display(),
        out_png.display()
    );

    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let frames = |target: &str| -> Vec<u64> {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_DAMAGE_CHECK", "1")
            .env("DWELLER_TIMING", "1")
            .env("DWELLER_GL", "swiftshader")
            .args([
                target, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "600",
                "--until-idle",
            ])
            .output()
            .unwrap();
        let err = text(&out.stderr);
        assert!(out.status.success(), "{}\n{err}", out.status);
        assert!(!err.contains("DAMAGE CHECK"), "{err}");
        // A frame that drew: its timer, then its readback.
        let recs: Vec<(&str, u64, &str)> = err
            .lines()
            .filter_map(|l| l.split("DWELLER_TIMING\t").nth(1))
            .map(|r| {
                let f: Vec<&str> = r.splitn(3, '\t').collect();
                (f[0], f[1].parse().unwrap(), f.get(2).copied().unwrap_or(""))
            })
            .filter(|r| r.0 == "timers" || r.0 == "webgl_readback")
            .collect();
        let readbacks = recs.iter().filter(|r| r.0 == "webgl_readback").count();
        assert!(readbacks >= 60, "{readbacks} readbacks:\n{err}");
        assert!(
            recs.iter()
                .all(|r| r.0 != "webgl_readback" || r.2 == "240x180"),
            "{err}"
        );
        let mut js: Vec<u64> = recs
            .windows(2)
            .filter(|p| p[0].0 == "timers" && p[1].0 == "webgl_readback")
            .map(|p| p[0].1)
            .collect();
        assert!(js.len() >= 60, "{} drawn frames:\n{err}", js.len());
        js.sort();
        js
    };
    let ms = |v: &[u64], q: f64| v[((v.len() as f64 * q) as usize).min(v.len() - 1)] as f64 / 1e6;
    let js = frames(page);
    let fin = frames(&format!("{page}#finish"));
    println!(
        "three cube frame JS: p50 {:.2} ms, p95 {:.2} ms; with gl.finish(): p50 {:.2} ms, p95 {:.2} ms",
        ms(&js, 0.5),
        ms(&js, 0.95),
        ms(&fin, 0.5),
        ms(&fin, 0.95)
    );
}
