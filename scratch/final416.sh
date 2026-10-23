#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
uptime > $S/final416-uptime.txt
PYPATH=$S/rel416 $S/slope416.sh > $S/slope416-after.txt 2>&1
cd $S/skills-main && PYTHONPATH=$S/rel416:$S/baseline-rdocx RDOCX_HOME=$S/rdocx-home $S/venv403/bin/python -m pytest tests/test_pptx_scaling.py --runxfail -q -p no:cacheprovider > $S/scaling416-main.txt 2>&1
cd $S/skills-scaling && PYTHONPATH=$S/rel416:$S/baseline-rdocx RDOCX_HOME=$S/rdocx-home $S/venv403/bin/python -m pytest tests/test_pptx_scaling.py --runxfail -q -p no:cacheprovider > $S/scaling416-old.txt 2>&1
uptime >> $S/final416-uptime.txt
