p='README.md'; s=open(p).read(); s=s.replace('src="docs/raf-logo.svg"','src="docs/raf-logo-missing.svg"',1); open(p,'w').write(s)
