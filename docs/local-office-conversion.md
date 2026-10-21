# Offline Office-to-PDF conversion (desktop preview)

PdfCraft can use an installed copy of **LibreOffice** to convert the following
documents into PDF without sending the file to a web service:

| Word processor | Spreadsheets | Presentations |
| --- | --- | --- |
| DOCX, ODT | XLSX, ODS | PPTX, ODP |

This is an initial local conversion path, **not** full Acrobat-grade import or
perfect formatting parity. The source document is copied to a temporary local
folder, processed by the installed LibreOffice executable, and removed with
the temporary files once conversion completes.

## Requirements and privacy

- Install LibreOffice on the **computer running PdfCraft**.
  On Linux make sure `soffice` or `libreoffice` is on PATH; common macOS app
  and Windows Program Files installations are also detected.
- No account, subscription or cloud upload is required.
- PdfCraft creates a **private LibreOffice user profile** for each conversion
  so it does not interfere with your everyday LibreOffice session.
- Macro-enabled Office extensions (DOCM, XLSM, PPTM) are deliberately not
  accepted. Documents can still contain untrusted content or external links,
  so handle unfamiliar attachments cautiously.
- The web/WASM edition does not offer Office conversion. No network fallback
  is enabled.
- Conversion is currently synchronous; large or complicated Office documents
  may temporarily freeze the PdfCraft interface. A background worker/progress
  interface is still needed.
- Input limited to 128 MiB, output to 256 MiB, and the local conversion
  process is terminated after 90 seconds.

## How to try it

1. Start a desktop preview of PdfCraft and verify LibreOffice is installed.
2. Use **Create PDF from multiple files** or **Insert pages from file**.
3. Choose a DOCX, XLSX, PPTX, ODT, ODS or ODP document. PdfCraft converts it
   locally and uses the resulting PDF pages in the normal page organizer.
4. **Save As** a new PDF. Do not overwrite a valuable original.
5. Open the result in a second PDF viewer and compare page count, page size,
   fonts, images, tables, charts and slide layouts.

If LibreOffice is missing or conversion fails, the source file is unchanged
and PdfCraft displays a conversion error instead of silently uploading it.
The resulting PDF should be independently inspected before signing or sharing.

## Regression and limitations

Automated tests cover extension/magic detection, rejection of macro-enabled
formats, and malformed files. Full visual parity requires hands-on documents
from Office/LibreOffice and a working LibreOffice installation in CI.

Next steps: worker-thread conversion and progress/cancellation; richer source
validation; DOC/XLS/PPT legacy support; configurable page ranges; automated
layout comparison for a synthetic Office fixture corpus; PDF-to-Excel and
PDF-to-PowerPoint export.

See [GUI smoke testing](gui-smoke-checklist.md) for general PDF save/reopen
checks. Signature workflows remain entirely separate from conversion.
