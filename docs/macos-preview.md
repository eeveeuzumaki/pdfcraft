# macOS test builds without installing anything on the work computer

The `.github/workflows/macos-preview.yml` GitHub Actions workflow builds
PdfCraft remotely on **standard GitHub-hosted macOS runners**. It does not
install any software on the computer that opens GitHub in a browser.

## Preview architectures

| Mac at home | GitHub Actions job | Download artifact |
| --- | --- | --- |
| Apple Silicon (M1, M2, M3, M4, etc.) | macOS Apple Silicon preview | `pdfcraft-macos-aarch64-preview` |
| Intel (older Mac models) | macOS Intel preview | `pdfcraft-macos-x86_64-preview` |

This uses the project's existing `packaging/macos/package.sh` to build a
`PdfCraft.app` inside a `.dmg`, and a separate `pdfcraft-cli` ZIP. Both
architectures are built separately rather than via a large/expensive runner.
PdfCraft's packaging declares macOS **11.0 or newer** as the minimum.

## Where to check/download (on a personal Mac, not at work)

1. Visit this repository's **Actions** tab.
2. Select **macOS Preview** on the left and open a successful run.
3. In **Artifacts**, download the matching architecture's preview bundle.
   For new builds, use **Run workflow** on `main`; manually triggered builds
   and builds caused by changes to this workflow use the same setup.
4. The downloaded GitHub artifact ZIP contains the DMG, a CLI ZIP, and
   `SHA256SUMS.txt`. On a personal Mac, mount the DMG to inspect the app.

The preview artifacts are retained for **30 days**. This preview is **not a
published or notarized Mac release**. The script uses ad-hoc signing without
an Apple Developer ID certificate, so macOS Gatekeeper **may block it**.
Do not disable Gatekeeper system-wide. A future public distribution build
should use appropriate Developer ID signing and Apple notarization. Only
run untrusted development software on devices you control, and test using
disposable documents before editing important PDFs.

## CI verification and limitations

The existing CI already tests Rust code on macOS. This workflow additionally
creates a packaged Mac app with the repository's packaging script and checks
the CLI version, DMG/ZIP presence and checksums. Passing does **not** mean
the graphical interface has been manually tested on either type of Mac.

The app remains PdfCraft-based. The source is licensed under MIT OR Apache-2.0
but ArtCraft branding has separate terms; see `docs/brand/LICENSE-brand.txt`.
Do not redistribute a customized build without fully addressing those terms
and retaining third-party license notices. Rebranding and public release
are separate tasks.
