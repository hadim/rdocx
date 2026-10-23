#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
$S/pkgcalls-pr2.sh > $S/package-calls-pr2-final.txt 2>&1
$S/shapecalls-pr2.sh --media 10 40 80 > $S/shape-calls-pr2-media-final.txt 2>&1
