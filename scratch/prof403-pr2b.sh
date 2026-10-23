#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
for call in title shapes.remove replace_image cell.fill fit_text slides.remove notes_text flip_h add_run add_slide; do
  for slides in 5 20; do
    PYTHONPATH=$S/rel403-pr2 $S/venv403/bin/python $S/prof403b.py $slides $call
  done
done
for slides in 5 20; do PYTHONPATH=$S/rel403-pr2 $S/venv403/bin/python $S/outofsync403.py $slides; done
