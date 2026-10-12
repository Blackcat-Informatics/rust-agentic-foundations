p='docs/capabilities/census-cli.md'; s=open(p).read(); s=s.replace('## Usage\n','## Usage\n\nSee the [ledger][l].\n\n[l]: ./census-ledgerX.md\n',1); open(p,'w').write(s)
