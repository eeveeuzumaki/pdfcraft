# PdfCraft desktop GUI smoke test — Windows and macOS

This checklist is for **hands-on testing on a personal computer**. GitHub CI
checks code, but it cannot certify that PdfCraft opens, displays or edits
documents correctly on a physical desktop.

Use **synthetic/disposable documents only**. Never test sensitive or unique
personal PDFs until editing and save/reopen preservation have been verified.
Preserve all inputs; use **Save As** to a new destination for every change.

## Getting a build

- Windows: follow [Windows portable preview](windows-portable-preview.md).
- macOS Intel or Apple Silicon: follow [macOS preview](macos-preview.md).
- Note the GitHub Actions **run URL**, source commit SHA, platform, OS version,
  architecture, artifact name and result. Builds are unsigned development
  previews and are not notarized release applications.
- Do not disable OS security protections globally. Test only on a device you
  control. Artifacts expire; download while they are still available.

## Test inputs

Create or export original, non-confidential PDFs:

1. **Simple.pdf** — two pages; short heading, paragraph, text in two distinct
   fonts and a visible image; at least one bookmark.
2. **MixedText.pdf** — ordinary, bold and italic text on a line, plus a
   paragraph with accented characters and a longer second line.
3. **Scanned.pdf** — one page of a clearly legible image-only document.
4. **Forms.pdf** — two standard AcroForm fields (text field, checkbox).
5. **Confidential-FAKE.pdf** — synthetic value such as
   `SECRET-TEST-9038` in visible text **and** the document metadata.
   Never use real financial, medical or identifying content.

Keep read-only originals and work on copies. Some PDFs exported from word
processors flatten content or substitute fonts; record which tool created
each sample rather than assuming the fixture's internals.

## Manual smoke passes

Fill **Pass / Fail / Blocked / Not available** for each row. A failed feature
must have reproducible steps and a screenshot or sanitized output when safe.

| ID | Action | Expected observable result | Result |
| --- | --- | --- | --- |
| VIEW-01 | Launch the packaged app | Main window appears without immediate crash or unexpected OS error | |
| VIEW-02 | Open Simple.pdf | Both pages render at 100%, page count and navigation are correct | |
| VIEW-03 | Zoom, scroll and switch pages | No stale painting, clipped text or unexpected page reorder | |
| VIEW-04 | Search a known phrase | Correct page/location is highlighted and remains selectable | |
| EDIT-01 | Change one word in existing PDF text | Edited word appears; neighboring text, image, font and page positioning stay unchanged | |
| EDIT-02 | Change an existing word to a shorter one | Adjacent words do not unexpectedly move or overlap | |
| EDIT-03 | Save As, close, reopen | Intended changes survive; untouched pages and bookmarks are intact | |
| EDIT-04 | Edit a mixed-font phrase | Original bold, italic and embedded fonts are maintained where supported; any substitution is clearly indicated | |
| IMAGE-01 | Replace or move an image on a copy | Image placement and surrounding text remain predictable | |
| PAGE-01 | Rotate, reorder and extract pages | Order, orientation and contents survive save/reopen | |
| NOTE-01 | Add a highlight or note | Annotation appears, remains attached to the intended area and survives reopen | |
| FORM-01 | Fill fields in Forms.pdf | Entered values and checkbox state survive save/reopen | |
| OCR-01 | Run OCR on Scanned.pdf where available | Recognized text is searchable; scanned pixels remain unchanged in searchable-image mode | |
| REDACT-01 | Redact `SECRET-TEST-9038` in a synthetic document | Removed visible text is not extractable via an independent PDF reader; metadata also checked separately | |
| CRASH-01 | Cancel a Save As dialog and close/reopen | No unintended overwrite or unexpected loss of the original file | |

Not all rows are currently guaranteed to be implemented in the preview GUI.
Record **Not available** honestly; do not present feature presence as proof
of correct behavior. **REDACT-01 is only a basic smoke test**, *not* a
certification that redaction/sanitization safely removes all sensitive data.

## Independent verification

- Open every changed output in a **second, independent PDF viewer**.
- For redaction testing, inspect both extracted text and document metadata,
  and recognize that PDFs may retain incremental revisions or hidden streams.
  Do not treat a screenshot or lack of selectable text as sufficient proof of
  irreversible data removal.
- Compare Simple.pdf before and after: unchanged page text, images, bookmarks,
  and form fields should not be affected.
- If the app crashes, collect the application-visible error message and a
  sanitized reproduction using the disposable sample (no personal data).
- Check PDF save/reopen on Windows and macOS separately when those machines
  are available. Passing automated platform CI does not replace these tests.

## Reporting bugs

Use this template in PR notes, a Markdown file, or a GitHub issue **if issues
are enabled** (the fork currently has GitHub Issues disabled):

```text
Test ID:
Platform / OS / architecture:
PdfCraft commit / GitHub artifact:
Fixture and how it was made:
Expected:
Observed:
Minimal steps:
Does output reopen in another viewer?:
Screenshot / sanitized output (if available):
```

**Gate to public distribution:** independent rendering/editing testing;
confirmed save/reopen reliability; robust redaction checks; rebranding of
protected marks; correct license notices; platform signing/notarization and
malware/security checks. An unsigned CI preview is not a release.
