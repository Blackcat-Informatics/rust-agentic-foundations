import os,subprocess
old,new='workplan-prose-assessment','workplan-prose-review'
files=subprocess.run(['git','ls-files'],capture_output=True,text=True).stdout.split()
for f in files:
    if f.endswith('.md') and os.path.exists(f):
        s=open(f).read(); t=s.replace(old,new)
        if t!=s: open(f,'w').write(t)
os.rename(f'docs/capabilities/{old}.md',f'docs/capabilities/{new}.md'); os.rename(f'work/changes/{old}.md',f'work/changes/{new}.md')
