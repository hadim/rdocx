#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
$S/scaling-pr2.sh > $S/scaling-pr2-final.txt 2>&1
$S/prof403-pr2b.sh > $S/prof403-pr2-final2.txt 2>&1
$S/pkgcalls-pr2.sh > $S/package-calls-pr2-final2.txt 2>&1
