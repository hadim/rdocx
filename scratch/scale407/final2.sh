#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
O=$S/scale407/final2; mkdir -p $O
for b in baseline release407b release407c; do
  $S/timed407.sh env PYTHONPATH=$S/$b $S/venv407/bin/python $S/scale407/open407.py $S/skills-main/tests $S/scale407/opendocs > $O/open-$b.txt 2>&1
  $S/timed407.sh env PYTHONPATH=$S/$b $S/venv407/bin/python $S/scale407/first407b.py $S/scale407/opendocs > $O/first-$b.txt 2>&1
done
cd $S/skills-main/tests
$S/timed407.sh env PYTHONPATH=$S/release407c $S/venv407/bin/python $S/scale407/costs407.py $S/skills-main/tests Document.insert_paragraph Document.remove_content Paragraph.insert_paragraph_before Document.remove_content_range Document.add_paragraph Document.add_heading Document.add_page_break Document.insert_table Document.add_table Paragraph.xml Run.xml Table.xml Cell.xml > $O/eight.txt 2>&1
cd $S/skills-main
COLUMNS=400 $S/timed407.sh env PYTHONPATH=$S/release407c $S/venv407/bin/python -m pytest -p no:cacheprovider -q --runxfail -rf --tb=no tests/test_docx_scaling.py > $O/scaling.txt 2>&1
for k in 1 2 3; do $S/timed407.sh env PYTHONPATH=$S/release407c $S/venv407/bin/python $S/repro407.py > $O/repro407-$k.txt 2>&1; echo "repro407 $k exit=$?" >> $O/summary; done
$S/timed407.sh env PYTHONPATH=$S/release407c:$S/pypath407 $S/venv407/bin/python $S/repro389_0.py > $O/repro389.txt 2>&1; echo "repro389 exit=$?" >> $O/summary
echo done >> $O/summary
