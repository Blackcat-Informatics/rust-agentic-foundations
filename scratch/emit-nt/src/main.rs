use raf_rdf::{Format, Graph};
fn main() {
    let mut args = std::env::args().skip(1);
    let input = args.next().unwrap();
    let output = args.next().unwrap();
    let text = std::fs::read_to_string(input).unwrap();
    let graph = Graph::from_text(&text, Format::Turtle).unwrap();
    std::fs::write(output, graph.to_text(Format::NTriples).unwrap()).unwrap();
}
