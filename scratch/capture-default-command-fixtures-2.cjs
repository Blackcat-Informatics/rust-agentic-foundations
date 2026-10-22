const fs=require('fs');
const path='crates/cargo-workplan/tests/registry.rs';
const current=fs.readFileSync(path,'utf8');
const saved='.stage/raf-workplan-strict-mode-force-coverage/raw/queue-default-command-registry-assertions-2.rs';
if(fs.existsSync(saved))throw Error('preserved assertion source already exists');
fs.writeFileSync(saved,current);
const a=current.indexOf('fn observe('),end=current.indexOf('\nfn roadmap(',a);
if(a<0||end<0)throw Error('producer bounds');
const capture=`fn observe(name: &str, run: Run, found: &mut BTreeSet<String>) {
    let graph = Graph::from_text(&run.stdout, Format::Turtle).unwrap();
    terms(&graph, found);
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/reports");
    std::fs::write(root.join(format!("{name}.ttl")), run.stdout).unwrap();
    std::fs::write(root.join(format!("{name}.nt")), graph.to_text(Format::NTriples).unwrap()).unwrap();
}
`;
const producer=current.slice(0,a)+capture+current.slice(end);
fs.writeFileSync(path,producer);
fs.writeFileSync('.stage/raf-workplan-strict-mode-force-coverage/raw/queue-default-command-registry-producer-2.rs',producer);
