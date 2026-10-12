import re
p='work/changes/workplan-entity-history.md'; s=open(p).read(); t=re.sub(r'\n  - cap:workplan-entity-history#constraints','',s); assert t!=s; open(p,'w').write(t)
