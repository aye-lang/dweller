use super::*;
use std::collections::HashMap;

const PAGE: &str = "browser/tests/fonts/os-families.html";
const VERDANA: &str = "/System/Library/Fonts/Supplemental/Verdana.ttf";
const MENLO: &str = "/System/Library/Fonts/Menlo.ttc";
const DEJAVU_MONO: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf";
const DEJAVU_SERIF: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSerif.ttf";

/// Q6.21: the widths the page measures (ten `m` at 50 px per family) in
/// a headless window with the system fonts and `envs`; the harness's
/// `DWELLER_FONT_SET=fixed` is dropped unless `envs` sets it again.
fn widths(envs: &[(&str, &str)]) -> HashMap<String, f64> {
    let out = dweller_command()
        .current_dir(root())
        .env_remove("DWELLER_FONT_SET")
        .envs(envs.iter().copied())
        .args([PAGE, "--font", "system", "--width", "800", "--height", "300", "--frames", "600", "--until-idle", "--text"])
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{envs:?}: {}\n{}", out.status, text(&out.stderr));
    let got: HashMap<String, f64> = text(&out.stdout)
        .lines()
        .filter_map(|l| l.split_once(' '))
        .filter_map(|(k, v)| Some((k.to_string(), v.parse::<f64>().ok()?)))
        .collect();
    assert!(got.len() >= 7, "{envs:?}: no widths in\n{}", text(&out.stdout));
    got
}

/// The advance of `c` in em in the font file `path` (the first face of a
/// collection): its `hmtx` entry through the first format 4 `cmap` subtable.
fn advance_em(path: &str, c: u16) -> f64 {
    let d = std::fs::read(path).unwrap();
    let u16_at = |o: usize| u16::from_be_bytes([d[o], d[o + 1]]) as usize;
    let u32_at = |o: usize| u32::from_be_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]]) as usize;
    let font = if &d[0..4] == b"ttcf" { u32_at(12) } else { 0 };
    let table = |tag: &[u8]| (0..u16_at(font + 4)).map(|i| font + 12 + 16 * i).find(|&r| &d[r..r + 4] == tag).map(|r| u32_at(r + 8)).unwrap();
    let cmap = table(b"cmap");
    let sub = (0..u16_at(cmap + 2)).map(|i| cmap + u32_at(cmap + 8 + 8 * i)).find(|&s| u16_at(s) == 4).unwrap();
    let segs = u16_at(sub + 6) / 2;
    let (ends, c) = (sub + 14, c as usize);
    let (starts, deltas) = (ends + 2 * segs + 2, ends + 4 * segs + 2);
    let ranges = deltas + 2 * segs;
    let k = (0..segs).find(|&k| u16_at(ends + 2 * k) >= c).unwrap();
    let raw = match u16_at(ranges + 2 * k) {
        0 => c,
        r => u16_at(ranges + 2 * k + r + 2 * (c - u16_at(starts + 2 * k))),
    };
    let glyph = (raw + u16_at(deltas + 2 * k)) & 0xFFFF;
    let long = u16_at(table(b"hhea") + 34);
    u16_at(table(b"hmtx") + 4 * glyph.min(long - 1)) as f64 / u16_at(table(b"head") + 18) as f64
}

fn near(got: f64, want: f64) -> bool {
    (got - want).abs() < 0.01
}

/// Q6.21: a named `font-family` is the OS font of that name (Verdana,
/// Menlo after three absent names; on Linux DejaVu Serif after the
/// absent Segoe UI), with the CSS font matching, in a renderer under its
/// sandbox (the default) and in one process. Each assertion runs where
/// its font file is installed; a Linux renderer reads the font
/// directory only with Landlock (without it seccomp refuses every open,
/// docs/process-model.md §6).
#[test]
fn os_font_families_window() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let landlock = std::fs::read_to_string("/sys/kernel/security/lsm").is_ok_and(|l| l.contains("landlock"));
    for envs in [&[][..], &[IN_PROCESS][..]] {
        let readable = !cfg!(target_os = "linux") || !envs.is_empty() || landlock;
        let present = |p: &str| Path::new(p).exists() && readable;
        let w = widths(envs);
        let each = |path: &str| 500.0 * advance_em(path, 'm' as u16);
        if present(VERDANA) {
            assert!(near(w["verdana"], each(VERDANA)) && !near(w["verdana"], w["arial"]), "{envs:?}: {w:?}");
        }
        if present(MENLO) {
            assert!(near(w["menlo"], each(MENLO)) && !near(w["menlo"], w["courier"]), "{envs:?}: {w:?}");
        }
        if present(DEJAVU_MONO) {
            assert!(near(w["dejavu-mono"], each(DEJAVU_MONO)), "{envs:?}: {w:?}");
        }
        if present(DEJAVU_SERIF) {
            assert!(near(w["dejavu-serif"], each(DEJAVU_SERIF)) && !near(w["dejavu-serif"], w["sans"]), "{envs:?}: {w:?}");
        }
    }
}

/// Q6.21: the page sees the fixed font set, the same on every machine,
/// in tests (`DWELLER_FONT_SET=fixed`), while recording and replaying an
/// archive, and in `dump`: a named family is the generic face its name
/// stands for, as before.
#[test]
fn os_font_families_pinned() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let archive = scratch().join("os_fonts_archive");
    let _ = std::fs::remove_dir_all(&archive);
    std::fs::create_dir_all(&archive).unwrap();
    let at = archive.to_str().unwrap();
    for envs in [&[("DWELLER_FONT_SET", "fixed"), IN_PROCESS][..], &[("DWELLER_RECORD", at)][..], &[("DWELLER_REPLAY", at)][..]] {
        let w = widths(envs);
        assert!(w["verdana"] == w["arial"] && w["menlo"] == w["courier"] && w["dejavu-serif"] == w["sans"], "{envs:?}: {w:?}");
    }

    let out = dweller_command().current_dir(root()).env_remove("DWELLER_FONT_SET").args(["dump", "--layout", "--font", "system", PAGE]).output().unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}", text(&out.stderr));
    let boxes: Vec<String> = text(&out.stdout)
        .lines()
        .filter(|l| l.trim_start().starts_with("text ") && l.ends_with("\"mmmmmmmmmm\""))
        .map(|l| l.split(") ").nth(1).unwrap().to_string())
        .collect();
    assert_eq!(boxes.len(), 7, "{}", text(&out.stdout));
    assert!(boxes[0] == boxes[1] && boxes[2] == boxes[3] && boxes[5] == boxes[6], "{boxes:?}");
}
