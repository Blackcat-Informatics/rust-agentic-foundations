p='.github/PULL_REQUEST_TEMPLATE.md'; s=open(p).read(); s=s+'\nSee [the guide](../CONTRIBUTINGX.md).\n'; open(p,'w').write(s)
