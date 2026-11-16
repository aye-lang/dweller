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
    let out = scratch().join(name);
    let run = dweller_command()
        .current_dir(root())
        .args(["print", "browser/tests/print/doc.html", "--font", "fixed", "-o"])
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

/// Q5.15: an unknown paper is a usage error.
#[test]
fn print_usage() {
    let bad = dweller_command().current_dir(root()).args(["print", "browser/tests/print/doc.html", "-o", "x.pdf", "--paper", "b5"]).output().unwrap();

    assert_eq!(bad.status.code(), Some(2));
    assert!(text(&bad.stderr).contains("usage: dweller print FILE|URL -o OUT.pdf"), "{}", text(&bad.stderr));
}
