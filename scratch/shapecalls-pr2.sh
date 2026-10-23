#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S && PYTHONPATH=$S/rel403-pr2 $S/venv403/bin/python $S/shape_calls.py "$@"
