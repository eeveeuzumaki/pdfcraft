# PdfCraft signing workflows: local-first personal/family edition

Decision: **support both** visible handwritten/typed/image signatures and
cryptographic certificate-based signatures. This fork is primarily for
personal use, with possible sharing to family or friends, **not a public
hosted service**. No account or cloud document storage is required.

## These are different operations

| | Fill & Sign (visible mark) | Digital ID (certificate-based) |
| --- | --- | --- |
| Input | Typed name, drawn strokes, locally imported PNG/JPEG | Local PKCS #12 identity, macOS Keychain, or Windows certificate store |
| Output | A visible annotation/image of a signature or initials | Cryptographic PDF signature that covers the signed revision |
| Identity verified by the PDF | **No** | A certificate chain can be evaluated against trusted roots |
| Detect subsequent changes | **No** | Yes, if the reader validates the signature/byte range |
| Works offline | Yes | Local signing and offline signature checks work; up-to-date revocation evidence and trusted timestamps may require a network |
| Typical use | Informal document acknowledgment, adding a visual autograph | Contracts and documents where signature integrity/identity should be checked |

**Never** label a Fill & Sign mark as a verified digital signature.
An image signature can still be an electronic signature depending on
context and local law, but it is *not* a cryptographic PDF signature.

## Recommended user-facing flow

1. **Fill & Sign:** Type, draw, or import a signature image; preview and
   place it. Label it a *visible signature* and do not show any
   cryptographic-validity status for the mark.
2. **Use a Certificate / Digital ID:** Choose an existing local digital
   ID or create/import one; optionally include a visible signature
   appearance. Save the cryptographically signed document as a new PDF.
   Never imply that a self-signed ID is automatically trusted by others.
3. **Verify signatures:** Report separately:
   - Whether the signed PDF bytes remain intact.
   - Whether the certificate chain is trusted.
   - Whether permitted document changes occurred after signing.
   - Whether time and revocation evidence were available and validated.
   Missing online evidence must not be reported as proof of trust.

## Connectivity policy

- **Offline by default:** opening, editing, organizing, visual signing,
  signing with an available private key, and basic validation happen locally.
- **Online only by explicit user choice:** trusted RFC 3161 timestamps,
  OCSP/CRL revocation refresh and any other optional validation service.
  Present the service URL, what certificate or cryptographic data is
  transmitted, and expected result before the request.
- **Never silently upload the whole PDF.** Signing with a local identity
  does not require uploading the document. Some certificate and timestamp
  requests transmit public certificate identifiers or signed digests;
  these still disclose metadata to the remote service.
- Keep private keys in the OS keystore or encrypted local files, never
  transmit them. Do not cache an unlocked PFX password indefinitely.
- Trusted timestamps and long-term validation currently remain
  development areas. Existing low-level RFC 3161 and revocation parsing
  code is **not** evidence that every end-to-end UI workflow is ready.

## Acceptance checks

- Type, draw and import a visual signature. Confirm placement/editing,
  save/reopen, and no entry shown as cryptographically verified.
- Digitally sign using an available local test ID; save/reopen and verify
  integrity with PdfCraft and an independent verifier such as `pdfsig`.
- Change content *after* signing and confirm the validator reports the
  changed/invalid or permitted-change state accurately.
- Check that a self-signed certificate is not shown as trusted unless
  explicitly added to the user's trust store.
- Disconnect networking entirely and confirm local signing/verification
  still work; network-dependent evidence must be distinctly labeled.
- Test any future online timestamp and revocation flow separately,
  explicitly confirming consent and that the raw document/private key
  never leaves the device.
- Use **disposable PDFs and test identities**, not real sensitive material,
  while features are in active development.

## Distribution considerations

PdfCraft uses MIT OR Apache-2.0 code, but ArtCraft branding has separate
terms. Even sharing a modified executable with relatives may require
removing protected ArtCraft marks. See [the brand license](brand/LICENSE-brand.txt).
Unsigned developer builds are intended for testing on trusted devices, not
general distribution.
