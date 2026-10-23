#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
PYTHONPATH=$S/rel403-pr2 $S/venv403/bin/python $S/prof403b.py 5 fit_text
PYTHONPATH=$S/rel403-pr2 $S/venv403/bin/python $S/prof403b.py 20 fit_text 8 --sample $S/sample403-fit.txt
