import re
p='docs/capabilities/census-ledger.md'; s=open(p).read()
a=s.index('## Examples\n'); b=s.index('## Constraints\n')
s=s[:a]+'## Examples\n\n<!-- examples later -->\n\n'+s[b:]; open(p,'w').write(s)
