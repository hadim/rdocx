#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
PYTHONPATH=$S/sym403-pr2 $S/venv403/bin/python $S/prof403b.py 80 replace_image 10 --sample $S/sample403-replace80.txt
PYTHONPATH=$S/sym403-pr2 $S/venv403/bin/python $S/prof403b.py 80 slides.remove 10 --sample $S/sample403-remove80.txt
