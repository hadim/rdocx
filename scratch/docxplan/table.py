import re, sys
S="/private/tmp/claude-501/-Users-hadim-Code-libs-rdocx/08057ee4-afc5-4751-aca1-2668752380de/scratchpad"
scal={}
for line in open(f"{S}/scale407/final/scaling.txt"):
    m=re.search(r"AssertionError: (\S+): ([\d.]+) ms at 100 paragraphs, ([\d.]+) ms at 25",line)
    if m: scal[m.group(1)]=(float(m.group(3)),float(m.group(2)))
best={}
for line in open(f"{S}/docxplan/profile-all.log"):
    m=re.match(r"(\S+) @ 100: best ([\d.]+) ms",line)
    if m: best[m.group(1)]=float(m.group(2))
print(f"{'call':42s} {'N':>6s} {'4N':>7s} {'prof':>7s} | copy  inv  save stage reopen valid addr  rust  edit")
for line in open(sys.argv[1]):
    _,name,total,*v=line.split()
    n,n4=scal.get(name,(0,0))
    print(f"{name:42s} {n:6.1f} {n4:7.1f} {best.get(name,0):7.1f} | "+" ".join(f"{float(x):4.0f}" for x in v))
