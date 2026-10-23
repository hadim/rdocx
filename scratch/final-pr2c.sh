#!/bin/bash
S=/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad
uptime > $S/final-uptime2.txt
$S/scaling-pr2.sh > $S/scaling-pr2-fixpass.txt 2>&1
$S/prof403-pr2b.sh > $S/prof403-pr2-fixpass.txt 2>&1
uptime >> $S/final-uptime2.txt
