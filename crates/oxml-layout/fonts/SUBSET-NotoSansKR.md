# Noto Sans KR F-266a subset

Source: `ofl/notosanskr/NotoSansKR[wght].ttf` from the Google Fonts `main`
branch, retrieved 2026-09-19.

Source SHA-256:
`194018e6b2b293a7964f037b25c0249ce1418bc9ab3c971060a03aa57861e252`

Output SHA-256:
`2bcb03d663d97d3b7db7e0ff3c7a67773130063b13f5c68331dbee9cc8bc5a2c`

The approved fixture repertoire is ASCII space, comma, and the Hangul syllables
of `안녕하세요 세계`. Reproduce it with FontTools:

```text
pyftsubset NotoSansKR.ttf --output-file=NotoSansKR-F266a-subset.ttf --unicodes=U+0020,U+002C,U+ACC4,U+B155,U+C138,U+C548,U+C694,U+D558 --glyph-names --symbol-cmap --legacy-cmap --notdef-glyph --notdef-outline --recommended-glyphs --name-IDs=* --name-legacy --name-languages=* --layout-features=* --no-hinting
```
