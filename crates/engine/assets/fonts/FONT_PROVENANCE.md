# Font Provenance

Phase 2 will vendor the default Datum text bundle here.

Each entry must record:
- family id
- upstream name
- upstream URL
- version
- SHA-256 of vendored file
- license
- local asset filename

Required default bundle target:
- `newstroke`
- `inter`
- `ibm_plex_sans_condensed`
- `inter_display`
- `jetbrains_mono`

Current vendored outline assets:

- family id: `inter`
- upstream name: `Inter Variable`
- upstream URL: `https://github.com/rsms/inter`
- version: repo snapshot cloned on 2026-04-18
- SHA-256: `4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/inter/InterVariable.ttf`

- family id: `inter_display`
- upstream name: `Inter Display`
- upstream URL: `https://github.com/rsms/inter`
- version: temporary Phase 2 shared asset using `InterVariable.ttf` until explicit `opsz`
  instance selection lands
- SHA-256: `4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/inter/InterVariable.ttf`

- family id: `ibm_plex_sans_condensed`
- upstream name: `IBM Plex Sans Condensed Regular`
- upstream URL: `https://github.com/google/fonts/tree/main/ofl/ibmplexsanscondensed`
- version: Google Fonts main snapshot downloaded on 2026-04-18
- SHA-256: `e7437c072eef2ef592ae6f2beb0000446287385907abb57ac1cf07bcbaa2aa33`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Regular.ttf`

Additional IBM Plex faces vendored 2026-07-08 for the Rendering Book typography
(`docs/gui/DATUM_RENDERING_BOOK.md` §5): weighted Sans Condensed hierarchy plus IBM
Plex Mono for aligned numeric data. Upstream `IBM/plex` tag `v5.0.0`, SIL OFL 1.1:

- family id: `ibm_plex_sans_condensed_medium`
- upstream name: `IBM Plex Sans Condensed Medium`
- upstream URL: `https://github.com/IBM/plex` (tag `v5.0.0`)
- version: v5.0.0 asset fetched on 2026-07-08
- SHA-256: `5494fa48878fbcff8b3e938efedbee075f7b4532451d3990490d47a743a9951e`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-Medium.ttf`

- family id: `ibm_plex_sans_condensed_semibold`
- upstream name: `IBM Plex Sans Condensed SemiBold`
- upstream URL: `https://github.com/IBM/plex` (tag `v5.0.0`)
- version: v5.0.0 asset fetched on 2026-07-08
- SHA-256: `ca6eeb68a1b06d0c671e4df80a923a5ab7d325c1eda12f009eba4281b30f4be9`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/ibm_plex_sans_condensed/IBMPlexSansCondensed-SemiBold.ttf`

- family id: `ibm_plex_mono`
- upstream name: `IBM Plex Mono Regular`
- upstream URL: `https://github.com/IBM/plex` (tag `v5.0.0`)
- version: v5.0.0 asset fetched on 2026-07-08
- SHA-256: `0b1292004f8bc6ff82d4490820e01e42cf839248822c0b9835aa795a8235f79c`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/ibm_plex_mono/IBMPlexMono-Regular.ttf`

- family id: `ibm_plex_mono_medium`
- upstream name: `IBM Plex Mono Medium`
- upstream URL: `https://github.com/IBM/plex` (tag `v5.0.0`)
- version: v5.0.0 asset fetched on 2026-07-08
- SHA-256: `50f39f344a345d637f34531e47454a1c2ac5f432325a861f0f485e24d74568a6`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/ibm_plex_mono/IBMPlexMono-Medium.ttf`

- family id: `jetbrains_mono`
- upstream name: `JetBrains Mono Regular`
- upstream URL: `https://github.com/JetBrains/JetBrainsMono`
- version: repo snapshot cloned on 2026-04-18
- SHA-256: `e6fd0d7e91550b3ed2b735d4312474362c4716edc4fc0577a0f61ed782d5aed1`
- license: SIL OFL 1.1
- local asset filename: `assets/fonts/jetbrains_mono/JetBrainsMono-Regular.ttf`

Temporary Phase 2D/2E harness asset:

- family id: `dev_dejavu_sans`
- upstream name: `DejaVu Sans`
- upstream URL: local system package install (`/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`)
- version: system package copy present on the development machine at vendoring time
- SHA-256: `57f73e11f51999432bf7ab22ce55b6f945d5eca1bf824404cfa9ec2e3718c84e`
- license: Bitstream Vera Fonts license with DejaVu extensions
- local asset filename: `assets/fonts/dev/DejaVuSans.ttf`

This entry is a development/test harness font only.
It does not change the researched default product bundle.
