#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
PYTHONPATH=$S/sym403-pr2 $S/venv403/bin/python $S/slope416.py 40 SlideTransition.duration --sample $S/sample416-duration.txt 6
PYTHONPATH=$S/sym403-pr2 $S/venv403/bin/python $S/slope416.py 40 HeaderFooter.footer --sample $S/sample416-footer.txt 6
