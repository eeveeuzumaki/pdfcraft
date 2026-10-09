# Windows x64 portable preview (GitHub Actions)

The workflow in `.github/workflows/windows-portable-preview.yml` builds a Windows
x64 test version of the PdfCraft desktop app and command-line utility remotely.
It **does not install or run any software on the computer using the browser**.

## Build

1. On GitHub, open **Actions → Windows Portable Preview → Run workflow**.
2. Choose the `main` branch, and click **Run workflow**.
3. After the job has finished successfully, open its run page and download
   the **pdfcraft-windows-x64-portable-preview** artifact.

The workflow also runs automatically when its own configuration file is
added/changed on `main`. Other commits do not automatically rebuild it,
avoiding unnecessary GitHub Actions minutes.

## Use later on a personal Windows computer

The downloaded Actions artifact contains a ZIP. Unzip the inner portable ZIP
to a writable folder and run `pdfcraft.exe` from inside the extracted
`PdfCraft-portable-preview` folder. No MSI installation is required.

Keep `portable.txt` next to `pdfcraft.exe`: it causes settings and
crash recovery files to stay in `PdfCraftData` alongside the executable.
This is **not** a promise that the app leaves zero operating-system traces.

This is an **unsigned development preview**, not a tested or signed release.
Windows may display a SmartScreen warning. Only run software whose origin
you trust, and use disposable sample PDFs until its editing behavior has
been tested on your own system. The artifact expires after 30 days.

## Licensing and release limitations

PdfCraft's Rust code is MIT OR Apache-2.0. Its ArtCraft brand names/logos
are separately protected under `docs/brand/LICENSE-brand.txt`. A
distributed customized edition must remove/rebrand those marks as required.
This preview should be used for internal evaluation only, not shared as a
public release. A properly rebranded and tested build is a separate milestone.
