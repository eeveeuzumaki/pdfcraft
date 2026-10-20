//! Small, synthetic PDF fixtures for edit -> save -> reopen regressions.
//!
//! All fixture bytes are generated here: no user PDFs, network, or external files.
//! Keep these tests focused on preservation of *untouched* page content.

use std::sync::Arc;

use pdfcraft_cos::{Dict, Document, Object, SaveOptions, Stream, write_incremental};
use pdfcraft_edit::{ImageChange, change_image, page_images, replace_line, text_lines};
use pdfcraft_model::pages;

/// Two pages with separate content streams and one image XObject on page one.
/// This fixture is intentionally small and self-contained for cloud CI.
fn sample_pdf() -> Vec<u8> {
    let first = concat!(
        "q 160 0 0 80 100 100 cm /Im0 Do Q\n",
        "BT /F1 12 Tf 72 700 Td (Original heading) Tj ET\n",
        "BT /F1 12 Tf 72 640 Td (Preserve caption) Tj ET"
    );
    let second = "BT /F1 12 Tf 72 700 Td (Untouched second page) Tj ET";

    fn stream(dictionary: &str, data: &[u8]) -> Vec<u8> {
        let mut bytes = format!("<< {dictionary} /Length {} >>\nstream\n", data.len()).into_bytes();
        bytes.extend_from_slice(data);
        bytes.extend_from_slice(b"\nendstream");
        bytes
    }

    let objects: Vec<Vec<u8>> = vec![
        b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
        b"<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 /MediaBox [0 0 600 800] >>".to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /Contents 6 0 R ",
            "/Resources << /Font << /F1 5 0 R >> ",
            "/XObject << /Im0 8 0 R >> >> >>"
        )
        .as_bytes()
        .to_vec(),
        concat!(
            "<< /Type /Page /Parent 2 0 R /Contents 7 0 R ",
            "/Resources << /Font << /F1 5 0 R >> >> >>"
        )
        .as_bytes()
        .to_vec(),
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>"
            .to_vec(),
        stream("", first.as_bytes()),
        stream("", second.as_bytes()),
        stream(
            concat!(
                "/Type /XObject /Subtype /Image /Width 4 /Height 2 ",
                "/ColorSpace /DeviceGray /BitsPerComponent 8"
            ),
            &[0, 0, 0, 0, 0, 0, 0, 0],
        ),
    ];

    let mut bytes = b"%PDF-1.7\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        bytes.extend_from_slice(object);
        bytes.extend_from_slice(b"\nendobj\n");
    }
    let xref = bytes.len();
    bytes.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    bytes
}

fn open_sample() -> (Vec<u8>, Document) {
    let bytes = sample_pdf();
    let doc = Document::open(Arc::new(bytes.clone())).expect("the synthetic fixture must open");
    (bytes, doc)
}

fn save_and_reopen(doc: &Document) -> (Vec<u8>, Document) {
    let saved = write_incremental(doc, &SaveOptions::default()).expect("save must succeed");
    // Independently verify that the output is a parsable PDF, not just accepted
    // by the same document parser used to create it.
    hayro_syntax::Pdf::new(saved.clone()).expect("saved PDF must parse independently");
    let reopened = Document::open(Arc::new(saved.clone())).expect("saved PDF must reopen");
    (saved, reopened)
}

fn page_stream(doc: &Document, page_index: usize) -> Vec<u8> {
    let page = &pages(doc)[page_index];
    let contents = page.dict.get(b"Contents").expect("fixture page contents");
    match &*doc.resolve(contents) {
        Object::Stream(s) => s.decoded().expect("decoded stream"),
        other => panic!("expected one content stream, found {other:?}"),
    }
}

fn page_text(doc: &Document, page_index: usize) -> Vec<String> {
    text_lines(doc, page_index)
        .expect("text detection")
        .into_iter()
        .map(|line| line.text)
        .collect()
}

#[test]
fn existing_text_edit_saves_and_keeps_other_content_exactly() {
    let (original, mut doc) = open_sample();
    let second_page_before = page_stream(&doc, 1);
    let caption_before = text_lines(&doc, 0).expect("original page text")[1].clone();
    let replacement = "Updated heading";

    replace_line(&mut doc, 0, 0, replacement).expect("edit existing text");

    let (saved, reopened) = save_and_reopen(&doc);
    assert!(
        saved.starts_with(&original),
        "incremental save must retain the original PDF bytes"
    );
    assert_eq!(pages(&reopened).len(), 2, "no pages should disappear");
    assert_eq!(page_text(&reopened, 0), ["Updated heading", "Preserve caption"]);
    assert_eq!(page_text(&reopened, 1), ["Untouched second page"]);
    assert_eq!(
        page_stream(&reopened, 1),
        second_page_before,
        "untouched page content must remain byte-for-byte identical"
    );
    let caption_after = &text_lines(&reopened, 0).expect("edited page text")[1];
    assert_eq!(
        caption_after.rect, caption_before.rect,
        "editing the heading must not reposition the caption"
    );
}

#[test]
fn existing_image_replacement_keeps_geometry_caption_and_other_page() {
    let (_, mut doc) = open_sample();
    let other_page_before = page_stream(&doc, 1);
    let before = page_images(&doc, 0).expect("fixture image");
    assert_eq!(before.len(), 1);
    assert_eq!((before[0].width, before[0].height), (4, 2));
    let rect_before = before[0].rect;

    let mut dict = Dict::new();
    dict.set(b"Type".to_vec(), Object::name("XObject"));
    dict.set(b"Subtype".to_vec(), Object::name("Image"));
    dict.set(b"Width".to_vec(), Object::Int(1));
    dict.set(b"Height".to_vec(), Object::Int(1));
    dict.set(b"ColorSpace".to_vec(), Object::name("DeviceGray"));
    dict.set(b"BitsPerComponent".to_vec(), Object::Int(8));
    let replacement = doc.add(Object::Stream(Stream::from_raw(dict, vec![220])));
    change_image(&mut doc, 0, 0, &ImageChange::Replace(replacement))
        .expect("replace original image");

    let (_, reopened) = save_and_reopen(&doc);
    let after = page_images(&reopened, 0).expect("saved images");
    assert_eq!(after.len(), 1);
    assert_eq!((after[0].width, after[0].height), (1, 1));
    assert_eq!(after[0].rect, rect_before, "replacement must retain original placement");
    assert_eq!(page_text(&reopened, 0), ["Original heading", "Preserve caption"]);
    assert_eq!(page_text(&reopened, 1), ["Untouched second page"]);
    assert_eq!(
        page_stream(&reopened, 1),
        other_page_before,
        "image edit changed the unrelated page"
    );
}

#[test]
fn sequential_text_and_image_edits_survive_two_saves() {
    let (_, mut doc) = open_sample();
    let other_page_before = page_stream(&doc, 1);

    replace_line(&mut doc, 0, 0, "First revision").expect("first edit");
    let (first_saved, mut doc) = save_and_reopen(&doc);

    let image = page_images(&doc, 0).expect("original image");
    assert_eq!(image.len(), 1);
    let transformed = pdfcraft_edit::rect_to_rect(image[0].rect, [60.0, 100.0, 220.0, 180.0]);
    change_image(&mut doc, 0, 0, &ImageChange::Transform(transformed)).expect("second edit");
    let (second_saved, reopened) = save_and_reopen(&doc);

    assert!(
        second_saved.starts_with(&first_saved),
        "the next incremental revision must keep the first saved bytes"
    );
    assert_eq!(page_text(&reopened, 0), ["First revision", "Preserve caption"]);
    assert_eq!(page_text(&reopened, 1), ["Untouched second page"]);
    assert_eq!(page_stream(&reopened, 1), other_page_before);
    let moved = &page_images(&reopened, 0).expect("transformed image")[0];
    let expected = [60.0, 100.0, 220.0, 180.0];
    assert!(
        moved
            .rect
            .iter()
            .zip(expected)
            .all(|(actual, expected)| (actual - expected).abs() < 0.001),
        "image position after second save: {:?}",
        moved.rect,
    );
}
