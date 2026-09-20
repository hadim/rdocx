# Noto Sans Hebrew F-266a subset

Source: `ofl/notosanshebrew/NotoSansHebrew[wdth,wght].ttf` from the Google Fonts
`main` branch, retrieved 2026-09-19.

Source SHA-256:
`7ef36a2c3593758cdb622e1bdef4f84523e92fbc3ccc667438dd80ff54c2de88`

Output SHA-256:
`b54ddced1bec92db800c88b0aac20988118f92d89fc2113f64c08fa8023ccf04`

The approved fixture repertoire is ASCII space, comma, and the Hebrew letters
of `שלום עולם`. Reproduce it with FontTools:

```text
pyftsubset NotoSansHebrew.ttf --output-file=NotoSansHebrew-F266a-subset.ttf --unicodes=U+0020,U+002C,U+05D5,U+05DC,U+05DD,U+05E2,U+05E9 --glyph-names --symbol-cmap --legacy-cmap --notdef-glyph --notdef-outline --recommended-glyphs --name-IDs=* --name-legacy --name-languages=* --layout-features=* --no-hinting
```
