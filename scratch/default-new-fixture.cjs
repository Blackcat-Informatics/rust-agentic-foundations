const fs=require('fs');
const p='crates/cargo-workplan/tests/new.rs';
let s=fs.readFileSync(p,'utf8');
s=s.replace('  cap:beta  implementing  no change of state is needed\n";', '  cap:beta  implementing  no change of state is needed\n\nThe record makes generated blocks stale. Stage the record first, since a link to a file git does not track is a finding of `check`, then bring the blocks up to date:\n  git add work/changes/fresh-change.md\n  docs sync\n";');
s=s.replace('assert!(!change.boolean("syncNeeded"));','assert!(change.boolean("syncNeeded"));');
const add=`    repo.edit(
        "docs/capabilities/beta.md",
        "Text for Verification of beta.\\n",
        &format!("Text for Verification of beta.\\n{}", support::BLOCKS),
    );
    repo.sync();
`;
if(!s.includes(add))throw Error('block add');
s=s.replace(add,'');
const stale=`        "Text for Verification of beta.\\n",
        &format!("Text for Verification of beta.\\n{}", support::BLOCKS),`;
s=s.replace(stale,`        "<!-- raf:generated:relationships -->\\n",
        "<!-- raf:generated:relationships -->\\nstale\\n",`);
const make=`        repo.edit(
            "docs/capabilities/beta.md",
            "Text for Verification of beta.\\n",
            &format!("Text for Verification of beta.\\n{}", support::BLOCKS),
        );
        repo.sync();
`;
if(!s.includes(make))throw Error('make blocks');
s=s.replace(make,'');
s=s.replace('let anchor = format!("Text for Contract of {capability}.\\n");','let anchor = "\\n## Usage\\n";');
s=s.replace('&format!("{anchor}\\n{}", printed_template(&run)),','&format!("\\n{}\\n{anchor}", printed_template(&run)),');
s=s.replace('"Text for Contract of alpha.\\n",\n        &format!("Text for Contract of alpha.\\n\\n{}", printed_template(&run)),','"\\n## Usage\\n",\n        &format!("\\n{}\\n## Usage\\n", printed_template(&run)),');
fs.writeFileSync(p,s);
