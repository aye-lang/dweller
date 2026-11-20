use super::*;

/// The streams of a PDF in file order, inflated.
fn pdf_streams(pdf: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let mut at = 0;
    while let Some(start) = find(&pdf[at..], b"stream\n") {
        let from = at + start + 7;
        let len = find(&pdf[from..], b"\nendstream").unwrap();
        out.push(inflate(&pdf[from + 2..from + len]));
        at = from + len + 10;
    }
    out
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// The strings of a page's text layer, in order.
fn text_layer(content: &[u8]) -> Vec<String> {
    let s = String::from_utf8_lossy(content);
    s.lines().filter_map(|l| Some(l[l.find('(')? + 1..l.rfind(") Tj")?].to_string())).collect()
}

struct Printed {
    pdf: Vec<u8>,
    pages: Vec<Vec<String>>,
    images: Vec<Vec<u8>>,
}

fn printed(pdf: Vec<u8>) -> Printed {
    let streams = pdf_streams(&pdf);
    assert!(pdf.starts_with(b"%PDF-1.4\n") && pdf.ends_with(b"%%EOF\n") && streams.len() % 2 == 0, "not a PDF of page pairs");
    let pages = streams.iter().step_by(2).map(|c| text_layer(c)).collect();
    let images = streams.into_iter().skip(1).step_by(2).collect();
    Printed { pdf, pages, images }
}

fn print_doc(extra: &[&str], name: &str) -> Printed {
    print_file("browser/tests/print/doc.html", extra, name)
}

fn print_file(file: &str, extra: &[&str], name: &str) -> Printed {
    let out = scratch().join(name);
    let run = dweller_command()
        .current_dir(root())
        .args(["print", file, "--font", "fixed", "-o"])
        .arg(&out)
        .args(extra)
        .output()
        .unwrap();
    assert!(run.status.success(), "{}", text(&run.stderr));
    printed(std::fs::read(&out).unwrap())
}

/// The pixel of an A4 page image (1436 x 2096, 2 device px per CSS px).
fn page_pixel(image: &[u8], x: usize, y: usize) -> [u8; 3] {
    let at = (y * 1436 + x) * 3;
    [image[at], image[at + 1], image[at + 2]]
}

/// What `browser/tests/print/doc.html` must print as: five A4 pages, cut
/// above the line that would straddle the first page's end, at
/// `break-before: page` and the legacy `page-break-before: always`, with
/// the `break-inside: avoid` block whole on a page of its own start; the
/// `@media print` rules applied and the `@media screen` ones not;
/// `beforeprint` seen before the layout.
fn check_doc(p: &Printed, what: &str) {
    let pdf = String::from_utf8_lossy(&p.pdf);

    assert_eq!(p.pages.len(), 5, "{what}");
    assert!(pdf.contains("/Count 5 ") && pdf.contains("/MediaBox [0 0 595 842]") && pdf.contains("/Title (Print test)"), "{what}");
    assert_eq!(p.pages[0][..4], ["Printed", "print only", "before seen print=false screen=true", "line 1"], "{what}");
    assert_eq!(p.pages[0].len(), 52, "{what}");
    assert_eq!(p.pages[0][51], "line 49", "{what}");
    assert_eq!(p.pages[1][0], "line 50", "{what}");
    assert_eq!(p.pages[1].last().unwrap(), "line 60", "{what}");
    assert_eq!(p.pages[2][0], "chapter two", "{what}");
    assert_eq!(p.pages[2].last().unwrap(), "more 30", "{what}");
    assert_eq!(p.pages[3], ["the end"], "{what}");
    assert_eq!(p.pages[4], ["legacy break", "not yet"], "{what}");
    assert!(!p.pages.iter().flatten().any(|s| s == "screen only"), "{what}");

    for image in &p.images {
        assert_eq!(image.len(), 1436 * 2096 * 3, "{what}");
    }

    assert_eq!(page_pixel(&p.images[3], 10, 10), [0xcc, 0, 0], "{what}");
    assert_eq!(page_pixel(&p.images[3], 1400, 1190), [0xcc, 0, 0], "{what}");
    assert_eq!(page_pixel(&p.images[3], 1400, 1290), [255, 255, 255], "{what}");
    assert_eq!(page_pixel(&p.images[2], 10, 1300), [255, 255, 255], "{what}");
    assert_eq!(page_pixel(&p.images[2], 10, 2090), [255, 255, 255], "{what}");
    assert!((0..2 * 20 * 1436).any(|i| p.images[0][i * 3] < 100), "{what}: no ink in the first line");
}

/// Q5.15: `dweller print` writes a PDF of the page as paper would show
/// it; the text layer places each run on its line; `--paper letter`
/// changes the media box and the cuts; the cross-reference table points
/// at the objects.
#[test]
fn print_to_pdf() {
    let a4 = print_doc(&[], "print_a4.pdf");

    check_doc(&a4, "a4");

    let content = String::from_utf8_lossy(&pdf_streams(&a4.pdf)[0]).to_string();

    assert!(content.starts_with("q 538.5 0 0 786 28 28 cm /Im0 Do Q\nBT 3 Tr\n/F1 12 Tf "), "{content}");
    assert!(content.contains(" Tz 1 0 0 1 28 802.9 Tm (Printed) Tj\n"), "{content}");
    assert!(content.contains(" Tz 1 0 0 1 28 787.9 Tm (print only) Tj\n"), "{content}");

    let xref = find(&a4.pdf, b"\nxref\n").unwrap() + 1;
    let table = String::from_utf8_lossy(&a4.pdf[xref..]).to_string();
    let rows: Vec<&str> = table.lines().skip(3).take_while(|l| l.ends_with(" n ")).collect();

    assert_eq!(rows.len(), 4 + 5 * 3);
    for (k, row) in rows.iter().enumerate() {
        let at: usize = row[..10].parse().unwrap();
        assert!(a4.pdf[at..].starts_with(format!("{} 0 obj\n", k + 1).as_bytes()), "object {}", k + 1);
    }
    assert!(table.ends_with(&format!("startxref\n{xref}\n%%EOF\n")));

    let letter = print_doc(&["--paper", "letter"], "print_letter.pdf");
    let pdf = String::from_utf8_lossy(&letter.pdf);

    assert!(pdf.contains("/MediaBox [0 0 612 792]"));
    assert_eq!(letter.pages.len(), 5);
    assert_eq!(letter.pages[0].len(), 49);
}

/// Q5.15 (follow-up): `@page` gives the paper its size and margins: the
/// media box, the width the page is laid out at, where the page's image
/// sits; `size: landscape` turns the paper asked for; a rule for
/// `:first` and one in `@media screen` do not count.
#[test]
fn print_page_rule() {
    let small = print_file("browser/tests/print/page-rule.html", &[], "print_page_rule.pdf");
    let pdf = String::from_utf8_lossy(&small.pdf).to_string();
    let content = String::from_utf8_lossy(&pdf_streams(&small.pdf)[0]).to_string();

    assert!(pdf.contains("/Count 1 ") && pdf.contains("/MediaBox [0 0 432 288]"), "the media box of 6in by 4in");
    assert!(pdf.contains("/Width 1008 /Height 480 "), "the image of a content box 504 by 240 CSS px");
    assert!(content.starts_with("q 378 0 0 180 18 36 cm /Im0 Do Q\n"), "{content}");
    assert_eq!(small.images[0][..3], [0, 0, 255], "the bar starts at the content box's corner");
    assert_eq!(small.images[0][1007 * 3..1008 * 3], [0, 0, 255], "the bar is as wide as the content box");
    assert_eq!(small.pages[0], ["Small page"]);

    let turned = print_file("browser/tests/print/page-turn.html", &["--paper", "letter"], "print_page_turn.pdf");
    let pdf = String::from_utf8_lossy(&turned.pdf).to_string();

    assert!(pdf.contains("/MediaBox [0 0 792 612]") && pdf.contains("/Width 2112 /Height 1632 "), "letter, turned, without margins");
    assert!(String::from_utf8_lossy(&pdf_streams(&turned.pdf)[0]).starts_with("q 792 0 0 612 0 0 cm /Im0 Do Q\n"));
}

/// Q5.15 (follow-up): with a font system, text is drawn from glyph
/// outlines in Type 3 fonts: the glyphs' codes, places and color in the
/// page's content, the outline of "A", the `ToUnicode` map, and the
/// page's image without that text; a run that something covers, a
/// translucent one and one in an opacity layer stay in the image (and
/// in the invisible text layer); a clipped run keeps its clip.
#[test]
fn print_vector_text() {
    let out = scratch().join("print_vector.pdf");
    let run = dweller_command().current_dir(root()).args(["print", "browser/tests/print/vector.html", "--font", "system", "-o"]).arg(&out).output().unwrap();

    if text(&run.stderr).contains("no system font found") {
        println!("skipped: no system font");
        return;
    }
    assert!(run.status.success(), "{}", text(&run.stderr));

    let pdf = std::fs::read(&out).unwrap();
    let streams = pdf_streams(&pdf);
    let all = String::from_utf8_lossy(&pdf).to_string();
    let content = String::from_utf8_lossy(&streams[0]).to_string();
    let map = String::from_utf8_lossy(&streams[2]).to_string();
    let glyph = String::from_utf8_lossy(&streams[3]).to_string();

    assert_eq!(streams.len(), 2 + 1 + 15, "the page, the map and 15 glyphs");
    assert!(all.contains("/Font << /F1 3 0 R /T0 8 0 R >>"), "the page names the font");
    assert!(all.contains("8 0 obj\n<< /Type /Font /Subtype /Type3 /FontBBox [0 0 0 0] /FontMatrix [0.001 0 0 0.001 0 0] /Resources << >> /FirstChar 0 /LastChar 14 /ToUnicode 9 0 R /Encoding << /Type /Encoding /Differences [0 /g0 /g1 "), "the font");
    assert!(all.contains("/Widths [639.16 563.96 615.23 257.81 ") && all.contains("/CharProcs << /g0 10 0 R /g1 11 0 R "), "the widths and the glyphs");
    assert!(content.contains("q 0 0 0 rg BT 0 Tr\n/T0 15 Tf\n1 0 0 1 28 797.97 Tm <00> Tj\n1 0 0 1 37.59 797.97 Tm <01> Tj\n1 0 0 1 46.05 797.97 Tm <00> Tj\n1 0 0 1 59.53 797.97 Tm <02> Tj\n"), "{content}");
    assert!(content.contains("ET Q\nq 28 694 37.5 30 re W n 1 0 0 rg BT 0 Tr\n/T0 15 Tf\n1 0 0 1 28 707.97 Tm <07> Tj\n"), "{content}");

    let mut hidden = text_layer(&streams[0]);

    hidden.sort();
    assert_eq!(hidden, ["Covered", "Faint", "Layered"], "what stayed in the image has its invisible text");
    assert!(map.contains("<00> <0041>\n") && map.contains("<01> <00E9>\n") && map.contains("<07> <0043>\n"), "{map}");
    assert_eq!(glyph, "639.16 0 d0\n544.92 0 m\n458.98 221.19 l\n175.78 221.19 l\n90.82 0 l\n0 0 l\n278.81 716.8 l\n359.86 716.8 l\n638.18 0 l\nh\n432.13 300.78 m\n352.05 517.09 l\n317.87 624.02 l\n308.11 584.96 297.85 549.32 287.11 517.09 c\n206.05 300.78 l\nh\nf\n");

    let ink = |from: usize, to: usize| streams[1][from * 1436 * 3..to * 1436 * 3].iter().filter(|&&b| b < 128).count();

    assert_eq!(ink(0, 80), 0, "the vector line is not in the image");
    assert!(ink(80, 160) > 500 && ink(160, 240) > 100 && ink(320, 400) > 100, "the covered, translucent and layered lines are");
    assert_eq!(ink(240, 320), 0, "the clipped vector line is not");
}

/// Q5.15: an unknown paper is a usage error.
#[test]
fn print_usage() {
    let bad = dweller_command().current_dir(root()).args(["print", "browser/tests/print/doc.html", "-o", "x.pdf", "--paper", "b5"]).output().unwrap();

    assert_eq!(bad.status.code(), Some(2));
    assert!(text(&bad.stderr).contains("usage: dweller print FILE|URL -o OUT.pdf"), "{}", text(&bad.stderr));
}
