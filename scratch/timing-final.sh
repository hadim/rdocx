#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/skills-scaling && PYTHONPATH=$S/rel403:$S/baseline-rdocx RDOCX_HOME=$S/rdocx-home $S/venv403/bin/python -m pytest tests/test_pptx_scaling.py --runxfail -q -p no:cacheprovider -rA > $S/scaling-final.txt 2>&1
$S/prof403-all.sh $S/rel403 > $S/prof403-final.txt 2>&1
cd $S && PYTHONPATH=$S/rel403 $S/venv403/bin/python $S/shape_calls.py 10 40 > $S/shape-calls-final.txt 2>&1
