#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/skills-scaling && PYTHONPATH=$S/rel403-pr2:$S/baseline-rdocx RDOCX_HOME=$S/rdocx-home $S/venv403/bin/python -m pytest tests/test_pptx_scaling.py --runxfail -q -p no:cacheprovider "$@"
