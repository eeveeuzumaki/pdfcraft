# PdfCraft / Acrobat Pro feature-gap audit — 2026-10-19

This is a **planning audit, not a product certification**. It compares the current fork's
self-reported `parity/acrobat-features.toml` against Acrobat Pro's documented **desktop/offline**
capabilities. The feature statuses are not independent hands-on tests. The Windows x64,
macOS ARM64 and macOS Intel preview packages were built successfully in GitHub Actions,
but the GUI has not yet been interactively validated by the owner.

## Evidence and counting rule

Primary sources:
- `parity/acrobat-features.toml`, current `main` as of 2026-10-19 (individual feature statuses,
  gaps, test names). Its `shipped` definition means implementation with one or more cited tests.
- `ROADMAP.md` (architectural risks and milestone breakdown; the top-level counts were written
  on 2026-10-07 and lag the live checklist).
- `crates/ui-egui/src/lib.rs` (concrete desktop menus/panels/tools).
- Adobe official documentation, e.g.
  [editing text](https://helpx.adobe.com/acrobat/using/edit-text-pdfs1.html),
  [redaction](https://helpx.adobe.com/acrobat/desktop/protect-documents/redact-pdfs/redact.html),
  [comparing PDFs](https://helpx.adobe.com/acrobat/using/compare-documents.html),
  [forms](https://helpx.adobe.com/acrobat/using/pdf-forms.html),
  [accessibility reading order](https://helpx.adobe.com/acrobat/using/touch-reading-order-tool-pdfs.html),
  and [preflight](https://helpx.adobe.com/acrobat/using/analyzing-documents-preflight-tool-acrobat.html).

Counts from **829** `[[feature]]` records:
- **806** in scope: **416 shipped**, **74 partial**, **316 planned**.
- **23** marked `na`, excluded as Adobe-cloud-only capabilities.
- In scope: 51.6% shipped, 9.2% partial, 39.2% planned.
- Prior `ROADMAP.md` totals (411 shipped, 70 partial) refer to an earlier snapshot.
- **All 416 shipped entries list at least one test in the checklist.** This does *not*
  establish that those tests are sufficient, or that behavior matches Acrobat.

Do **not** equate shipped-feature percentage to overall product completeness.
Renderer, preservation of existing PDF content, import/export fidelity, and compliance
work have disproportionate complexity and risk.

## Feature comparison

Legend: **Present** = meaningful capability described as shipped; **Partial** = useful
functionality but known important gaps; **Missing** = key capabilities still planned.

| Acrobat Pro workflow | Current PdfCraft | Important limitations / follow-up |
| --- | --- | --- |
| Open, view, search, save, bookmarks and tabs | **Present / Partial** | Basic reader functionality, repair and saves exist; rendering still relies on vendored `hayro`, without independently established visual fidelity. Very large PDFs are buffered wholly; lazy loading is planned. |
| Existing PDF text editing and formatting | **Partial** | Existing paragraphs can be retyped, repositioned, resized and formatted, with rewrapping; per-character formatting and complex original-font fidelity are not complete. Do not claim text editing is absent. |
| Add/edit images, text, links, marks | **Present / Partial** | Added objects and replacement of existing images have implementation/tests; inspect real-world complex images, fonts, masks, transformations and save/reopen preservation. |
| Organize, insert, split, extract, replace and combine pages | **Present** | Core workflow exists; some advanced transitions, combine settings and metadata edge cases remain. |
| Highlight, markup, sticky notes, shapes, stamps, replies, XFDF | **Present / Partial** | Main annotation features exist; rich text formatting in comments and some appearance subtleties remain. |
| Fill and prepare forms; scripts | **Present / Partial** | AcroForm authoring/filling, actions, and subset JavaScript exist; broader Acrobat JavaScript APIs and complete XFA compatibility are unfinished. |
| Redact and sanitize sensitive information | **Present, high assurance not established** | Mark/apply/search patterns and sanitization exist; validate permanent removal against content streams, raster data, metadata, annotations, revisions, and independent PDF readers before trusting with sensitive data. |
| Digital certificate signatures | **Present / Partial** | PAdES B-B signing/verification exists; timestamped signatures, embedded long-term evidence and TSA workflow are partial; smart cards / other trust workflows incomplete. |
| OCR of scanned pages | **Partial** | Searchable-image OCR exists; current models chiefly support English Latin text without accents; no deskew, suspect-word correction or editable-text output. |
| Export to Word, Excel, PowerPoint; create PDF from Office | **Partial / Missing** | Word .docx export exists but is not layout-fidelity complete. Excel/PowerPoint export and Office-to-PDF import remain planned. PNG/JPEG/TIFF/text export exists. |
| Compare two document versions | **Present / Partial** | Text and image-difference detection plus PDF summary report exist; synchronized side-by-side reading, range filtering and scanned-document comparison remain planned. |
| Compress/optimize PDF | **Present / Partial** | Reduction and image/object optimizer exist; specialized optimizer panels and audit/compliance needs remain. |
| Preflight and PDF/A, PDF/X, PDF/UA | **Mostly missing** | PDF/A-2b/3b structural checking and some conversion/fixups are partial; generic preflight profiles, extensive fixups, PDF/X and PDF/UA workflows are not production-complete. |
| Accessibility check and remediation | **Partial** | 32-rule checker, HTML report and an alternate-text workflow exist. Autotagging, reading-order editing, structure/tags panel and full keyboard/screen-reader support are missing or incomplete. |
| Print, imposition, booklet, n-up | **Partial** | Layout/preview and CUPS printing exist; native Windows printer-spooler integration and print-production inspection/separations are incomplete. |
| Acrobat cloud sharing, Acrobat Sign, AI assistant | **Out of scope** | Deliberately not targeted in the original offline parity checklist. Consider optional integrations only after core reliability is demonstrated. |

The partial status is intentionally conservative: many individual features are marked
`shipped`, but practical usability on external documents is not yet known.

## Prioritized roadmap for this fork

### Gate 0 — Establish a reproducible quality baseline **before** expanding editing

1. Create a small **original/synthetic, distributable PDF fixture suite** covering:
   standard-font text, embedded/subset fonts, accented/CJK text, images with transparency,
   two-column layouts, rotated pages, forms, and encrypted PDFs.
2. Automated round-trip checks: open -> render/extract -> edit -> save -> reopen,
   with tests for unchanged content outside edited areas and expected annotations.
3. Preserve a **known-good original** for each test; no overwrite of user data.
4. Add at least one independent oracle for sensitive outputs where practical:
   e.g. qpdf/pdfinfo/pdftotext, a second renderer, or structure inspection.
5. Continue regression tests for malformed inputs and redaction; establish a manual
   Windows/macOS GUI smoke checklist. **No Codespace required** for GitHub code/CI work.

**Acceptance:** fixture suite is runnable in CI with specific assertions, not merely
'PDF opened without error'; a human has tested the native preview GUI before a release.

### Priority 1 — Finish Acrobat-style editing of *existing* PDF content

Build on `edit.text-edit`, `edit.text-reflow`, `edit.text-move-resize` and
`edit.image-replace`, rather than rebuilding them from scratch.

- Make mixed-format text runs, original font reuse/fallback and paragraph geometry
  predictable; don't silently change fonts or unrelated text.
- Test edit/save/reopen and cross-renderer appearances, including non-ASCII and rotated
  text, links and overlapping annotations.
- Provide clear unsupported-feature warnings instead of destructive or misleading edits.

**Acceptance:** controlled fixtures can change existing text and images in place,
without damaging untouched text, objects, searchability or navigation.

### Priority 2 — Useful scanned document and OCR workflow

- Add rotation detection/deskew and a review/correction path for uncertain OCR.
- Improve supported languages in a license-compliant way; expose limitations per language.
- Verify hidden searchable-text alignment and untouched original scanned pixels in
  'exact' mode.

**Acceptance:** searchable scans with selectable, well-aligned text and a repeatable
accuracy/false-positive test set.

### Priority 3 — Conversion and document interchange

- Improve .docx export fidelity first, using test documents with tables and columns.
- Then select a maintainable Office input strategy (currently planned as a LibreOffice
  sidecar; no silent remote upload) and consider .xlsx/.pptx export.
- Require loss warnings for anything that cannot round-trip.

**Acceptance:** representative output files open in common office software and their
key structures/content survive conversion.

### Priority 4 — Accessibility and professional compliance

- Tags/reading order editor, automatic tagging with auditable results and keyboard-only
  navigation.
- Preflight profiles, standardized PDF/A/PDF/X/PDF/UA checks with independent test files.
- Strengthen print production features only after content preservation is reliable.

**Acceptance:** results match independent validators and manual assistive-tech tests,
not merely the application's own checker.

### Priority 5 — Advanced signatures and ecosystem

- Timestamp authority configuration, certificate trust/revocation, LTV workflows,
  and platform smart-card integration.
- Public releases require signing/notarization, replacement of trademark-restricted
  ArtCraft marks where required, and the appropriate copyright/asset notices.
- AI/cloud features remain optional; never upload user PDFs without clear permission.

**Acceptance:** signed documents validate independently across real certificate cases,
and release artifacts meet their platform's security/distribution requirements.

## Suggested next three small pull requests

1. **`test/pdf-roundtrip-fixtures`**: a minimal set of synthetic PDFs with automated
   original-content preservation and edit/save/reopen assertions (begin with ordinary
   font text and image replacement). Tests must run under existing GitHub Actions CI.
2. **`fix/existing-text-regressions`**: select one reproduced failing fixture and fix
   its text-editing/font-handling behavior with an explicit regression test.
3. **`test/gui-smoke-checklist`**: document small manual smoke scenarios for Windows
   and both Mac architectures, including saving a copy, OCR, and redaction verification.

Only begin #2 once #1 gives a reproducible failure. Keep the change size narrow
so GitHub Actions build minutes and review burden are reasonable.

## Known limitations of this audit

- Review of code, feature tracker, and existing CI; **not an independent execution of Acrobat**.
- No hands-on UI assessment on the user's Windows or Mac computers yet.
- Acrobat functionality depends on plan, platform and integrations. This audit targets
  core desktop Acrobat Pro workflows, not exact feature-by-feature visual imitation.
- Must not copy Adobe code, assets, logos or proprietary UI resources.
- GitHub Actions can compile/test without Codespaces, but Actions usage and artifacts
  have their own quotas/retention limits.

Update this audit when the fixture suite and independent product tests change the
confidence of the statuses.
