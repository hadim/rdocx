# Noto Sans JP F-266a subset

Source: `ofl/notosansjp/NotoSansJP[wght].ttf` from the Google Fonts `main`
branch, retrieved 2026-09-19.

Source SHA-256:
`c2f3b4d463500a2ddcd3849cded1fceeb9fd6d1c32e6cbecd568453ba50fc68f`

Output SHA-256:
`03baa76ccf6c13a66f54814e912a90156f8d8ba32ef7b443f7d59031ece65be7`

The approved fixture repertoire is ASCII space, comma, and the Kana, CJK
punctuation and Kanji of `こんにちは、カタカナ世界`. Reproduce it with
FontTools:

```text
pyftsubset NotoSansJP.ttf --output-file=NotoSansJP-F266a-subset.ttf --unicodes=U+0020,U+002C,U+3001,U+3053,U+3061,U+306B,U+306F,U+3093,U+30AB,U+30BF,U+30CA,U+4E16,U+754C --glyph-names --symbol-cmap --legacy-cmap --notdef-glyph --notdef-outline --recommended-glyphs --name-IDs=* --name-legacy --name-languages=* --layout-features=* --no-hinting
```
