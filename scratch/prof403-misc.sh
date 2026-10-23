#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
for slides in 5 20; do PYTHONPATH=$S/rel403-pr2 $S/venv403/bin/python $S/prof403b.py $slides slides.remove; done
echo "PR1 tip:"
for slides in 5 20; do PYTHONPATH=$S/rel403 $S/venv403/bin/python $S/outofsync403.py $slides; done
for call in fit_text flip_h notes_text slides.remove; do for slides in 5 20; do PYTHONPATH=$S/rel403 $S/venv403/bin/python $S/prof403b.py $slides $call; done; done
