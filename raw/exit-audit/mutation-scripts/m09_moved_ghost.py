import re
p='work/changes/census-cli.md'; s=open(p).read()
i=s.index('claim:'); j=s.index('\n',i)
s=s[:j]+'\n    outcome: moved:change:ghost'+s[j:]; open(p,'w').write(s)
