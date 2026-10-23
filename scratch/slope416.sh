#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
for slides in 5 20 40; do PYTHONPATH=${PYPATH:-$S/rel403-pr2} $S/venv403/bin/python $S/slope416.py $slides; done
