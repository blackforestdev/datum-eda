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
