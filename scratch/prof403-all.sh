#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
for call in add_run add_shape text_frame.text cell.fill add_slide title shapes.remove replace_image; do
  for slides in 5 20; do
    PYTHONPATH=$1 $S/venv403/bin/python $S/prof403.py $slides $call
  done
done
