#!/bin/bash
# Timed runs of the final build: repro407 x5, repro389, the 8 calls and reads, the full skills scaling test.
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
O=$S/scale407/final
mkdir -p $O
R=$S/release407b
for k in 1 2 3 4 5; do $S/timed407.sh env PYTHONPATH=$R $S/venv407/bin/python $S/repro407.py > $O/repro407-$k.txt 2>&1; echo "repro407 $k exit=$?" >> $O/summary; done
$S/timed407.sh env PYTHONPATH=$R $S/venv407/bin/python $S/repro389_0.py > $O/repro389.txt 2>&1; echo "repro389 exit=$?" >> $O/summary
cd $S/skills-main/tests
$S/timed407.sh env PYTHONPATH=$R $S/venv407/bin/python $S/scale407/costs407.py $S/skills-main/tests Document.insert_paragraph Document.remove_content Paragraph.insert_paragraph_before Document.remove_content_range Document.add_paragraph Document.insert_table Document.add_table Paragraph.xml Run.xml Table.xml Cell.xml > $O/eight.txt 2>&1
echo "eight exit=$?" >> $O/summary
cd $S/skills-main
COLUMNS=400 $S/timed407.sh env PYTHONPATH=$R $S/venv407/bin/python -m pytest -p no:cacheprovider -q --runxfail -rf --tb=no tests/test_docx_scaling.py > $O/scaling.txt 2>&1
echo "scaling exit=$?" >> $O/summary
echo done >> $O/summary
