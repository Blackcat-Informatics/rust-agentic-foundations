p='docs/capabilities/census-cli.md'; s=open(p).read(); s=s.replace('## Usage\n','## Usage\n\n> [!PROPOSED change:census-cli]\n> A new flag.\n',1); open(p,'w').write(s)
