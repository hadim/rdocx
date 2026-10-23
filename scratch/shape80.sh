#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S && PYTHONPATH=$S/rel403 $S/venv403/bin/python $S/shape_calls.py 80 > $S/shape-calls-80.txt 2>&1
