# rpptx-py

The Python `rpptx` package provides a focused native binding for creating,
opening, editing, and saving PowerPoint-compatible presentations.

## Capabilities

- Create a deck or open a PPTX path, then save or serialize it.
- Enumerate layouts and add slides.
- Read and edit shapes, text frames, paragraphs, runs, and fonts.
- Add text boxes, preset shapes, pictures, and tables.
- Edit table text and column widths through Python collections.
- Render deterministic slide and speaker-note PDF and PNG output.
- Read speaker-note text and inspect or mutate modern comment threads.

## Use it when

Choose this binding when Python code needs focused presentation editing.

## Relationship

It wraps the real `rpptx::Presentation` and shares paths, revisions, units, and
errors through `oxml-py-support`. Rendering uses the native deterministic
facade and releases the GIL. Collaboration snapshots are frozen values, and a
successful collaboration mutation advances the presentation revision. This
Cargo package is unpublished and is not the user-facing installation target.

## Example

```python
from rpptx import Presentation

presentation = Presentation("deck.pptx")
print(len(presentation.slides))
pdf = presentation.to_pdf()
first_slide = presentation.render_slide_to_png(0)
```
