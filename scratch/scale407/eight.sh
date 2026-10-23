#!/bin/bash
# usage: eight.sh PYTHONPATH -- the 8 #407 calls of the skills scaling test, with --runxfail
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
cd $S/skills-main
T="tests/test_docx_scaling.py::test_call_cost_does_not_grow_with_the_document"
ids=()
for n in Document.insert_paragraph Document.remove_content Paragraph.insert_paragraph_before Document.remove_content_range Document.add_paragraph Document.insert_table Document.add_table Paragraph.xml; do
  ids+=("$T[$n]")
done
PYTHONPATH=$1 $S/venv407/bin/python -m pytest -p no:cacheprovider -q --runxfail -rA "${ids[@]}"
