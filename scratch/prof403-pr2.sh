#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
for call in title shapes.remove replace_image cell.fill add_run add_slide; do
  for slides in 5 20; do
    $S/venv403/bin/python $S/prof403.py $slides $call
  done
done
