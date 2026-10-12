import os
os.makedirs('docs/other',exist_ok=True)
s=open('docs/capabilities/census-cli.md').read().replace('id: cap:census-cli','id: cap:stray')
open('docs/other/stray.md','w').write(s)
