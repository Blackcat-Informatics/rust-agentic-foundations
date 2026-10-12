import os,re,subprocess
slug='census-cli'
for p in [f'docs/capabilities/{slug}.md',f'work/changes/{slug}.md']: os.remove(p)
files=subprocess.run(['git','ls-files','*.md'],capture_output=True,text=True).stdout.split()
pat=re.compile(r'\[([^\]]*)\]\([^)]*'+re.escape(slug)+r'\.md[^)]*\)')
for f in files:
    if not os.path.exists(f): continue
    s=open(f).read(); t=pat.sub(r'\1',s)
    t=re.sub(r'\n  - cap:'+slug+r'\b[^\n]*','',t); t=t.replace(f'cap:{slug}, ','').replace(f', cap:{slug}','').replace(f'[cap:{slug}]','[]')
    if t!=s: open(f,'w').write(t)
