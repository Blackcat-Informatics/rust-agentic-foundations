use purrdf_core::SerializeGraph;
use purrdf_rdf::{parse_dataset, serialize_dataset};

fn main() {
    let mut code = 0;
    for path in std::env::args().skip(1) {
        let bytes = std::fs::read(&path).expect("read");
        let base = format!("file:///probe/{}", path.rsplit('/').next().unwrap());
        let base_opt = if std::env::var_os("NOBASE").is_some() { None } else { Some(base.as_str()) };
        match parse_dataset(&bytes, "text/turtle", base_opt) {
            Ok(ds) => {
                let nt = serialize_dataset(&*ds, "application/n-triples", SerializeGraph::Dataset)
                    .expect("serialize");
                let text = String::from_utf8(nt).unwrap();
                println!("== {path}: {} lines of N-Triples", text.lines().count());
                print!("{text}");
            }
            Err(e) => { println!("== {path}: REFUSED {e:?}"); code = 1; }
        }
    }
    std::process::exit(code);
}
