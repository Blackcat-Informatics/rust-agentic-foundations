p='docs/capabilities/census-cli.md'; s=open(p).read(); s=s.replace('  code: []','  code: [raf_census::Gate]',1); open(p,'w').write(s)
