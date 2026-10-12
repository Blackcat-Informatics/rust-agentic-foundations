import re
p='docs/capabilities/census-ledger.md'; s=open(p).read()
s=re.sub(r'^title: .*$', 'title: "Census ledger', s, count=1, flags=re.M); open(p,'w').write(s)
