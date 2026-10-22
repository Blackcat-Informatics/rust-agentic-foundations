#[cfg(test)]
mod tests {
    use std::fmt::Write as _;

    use super::{Run, lex_text, place_runs};
    use crate::item::Namespace;
    use crate::reach::Node;
    use crate::work::{self, Work};
    use crate::{MemoryTree, Workspace};

    /// The runs of `text` as their text and the byte where each starts.
    fn runs(text: &str) -> Vec<(String, usize)> {
        let lexed = lex_text(text).unwrap_or_else(|failure| panic!("{}", failure.message));
        lexed
            .runs
            .into_iter()
            .map(|run| (run.text, run.start))
            .collect()
    }

    /// The texts of the runs of `text`.
    fn texts(text: &str) -> Vec<String> {
        runs(text).into_iter().map(|(text, _)| text).collect()
    }

    #[test]
    fn a_run_is_an_identifier_path_that_no_dot_precedes_with_its_first_byte() {
        assert_eq!(runs("a::b::c"), [("a::b::c".to_owned(), 0)]);
        assert_eq!(texts("x.helper(); x.0.y"), ["x", "x"]);
        assert_eq!(texts("0..LIMIT; ..=END"), ["LIMIT", "END"]);
        assert_eq!(texts("::std::fmt::Debug"), ["::std::fmt::Debug"]);
        assert_eq!(texts("Vec::<u8>::new()"), ["Vec", "u8"]);
        assert_eq!(texts("<T as Tr>::f"), ["T", "Tr"]);
        assert_eq!(texts("r#match::x"), ["match::x"]);
        assert_eq!(texts("a :: b"), ["a::b"]);
        assert_eq!(texts("f(a::b, c)"), ["f", "a::b", "c"]);
    }

    #[test]
    fn path_keywords_start_a_run_and_other_keywords_are_no_segment() {
        assert_eq!(
            texts("crate::a; self::b; super::super::c; self::super::d"),
            ["crate::a", "self::b", "super::super::c", "self::super::d"]
        );
        // A keyword ends the run, and what goes on after it is no run.
        assert_eq!(texts("a::crate; a::self::b"), ["a", "a"]);
        assert_eq!(texts("r#fn::r#match"), ["fn::match"]);
        assert_eq!(texts("Self::new(); impl Tr for S"), ["Tr", "S"]);
        assert_eq!(texts("pub use a as b;"), ["a", "b"]);
    }

    #[test]
    fn a_lifetime_a_metavariable_a_declared_name_and_a_bound_name_are_no_run() {
        assert_eq!(texts("&'a T; 'label: loop {}"), ["T"]);
        assert_eq!(texts("&'static Foo"), ["Foo"]);
        assert_eq!(texts("$x; $crate::helper()"), ["crate::helper"]);
        assert_eq!(texts("fn helper(x: u8) -> Foo {}"), ["u8", "Foo"]);
        assert_eq!(
            texts(
                "struct S { helper: u8 } enum E {} trait T {} type A = B; mod m {} const C: u8 = 1; static D: X = Y;"
            ),
            ["u8", "B", "u8", "X", "Y"]
        );
        assert_eq!(
            texts("macro_rules! m { () => { g() } }"),
            ["macro_rules", "g"]
        );
        assert_eq!(texts("S { helper: 1, other }"), ["S", "other"]);
        assert_eq!(texts("where T::Assoc: Bound"), ["T::Assoc", "Bound"]);
        assert_eq!(texts("x: ::std::Y"), ["::std::Y"]);
        assert_eq!(texts("x:&Y"), ["Y"]);
    }

    #[test]
    fn a_byte_order_mark_and_a_shebang_are_left_out_and_offsets_are_those_of_the_text() {
        let text = "\u{feff}fn f() { g() }";
        assert_eq!(runs(text), [("g".to_owned(), text.find('g').expect("g"))]);
        let text = "#!/usr/bin/env run\nfn f() { g() }";
        assert_eq!(runs(text), [("g".to_owned(), text.find("g(").expect("g"))]);
        // An inner attribute is no shebang.
        assert_eq!(texts("#![allow(unused)]\nfn f() {}"), ["allow", "unused"]);
        // A text that does not lex says where.
        let failure = lex_text("fn f() {\n\"").expect_err("an open string");
        assert_eq!(failure.line, 2);
    }

    fn run_at(start: usize) -> Run {
        Run {
            start,
            leading_colon: false,
            segs: vec!["x".to_owned()],
            text: "x".to_owned(),
        }
    }

    #[test]
    fn each_run_goes_to_the_innermost_unit_that_holds_its_start() {
        // An impl block (0..100) holding two members, beside an item after
        // it, and a run outside every unit.
        let units = vec![(120, 150), (10, 20), (0, 100), (30, 40), (0, 100)];
        let runs: Vec<Run> = [5, 10, 25, 35, 99, 100, 130, 160]
            .into_iter()
            .map(run_at)
            .collect();
        let placed = place_runs(&runs, units);
        let by: Vec<((usize, usize), Vec<usize>)> = placed.into_iter().collect();
        assert_eq!(
            by,
            [
                ((0, 100), vec![0, 2, 4]),
                ((10, 20), vec![1]),
                ((30, 40), vec![3]),
                ((120, 150), vec![6]),
            ]
        );
    }

    /// A library of `items` public functions, each calling its own private
    /// helper `references` times.
    fn fan(items: usize, references: usize) -> Workspace {
        let mut text = String::new();
        for i in 0..items {
            let calls = format!("h{i}(); ").repeat(references);
            writeln!(text, "pub fn f{i}() {{ {calls}}}\nfn h{i}() {{}}").expect("write");
        }
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        let tree = MemoryTree::from([("Cargo.toml", manifest), ("src/lib.rs", text.as_str())]);
        Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"))
    }

    /// The kinds of work the walk counts.
    const KINDS: [Work; 6] = [
        Work::ReachTokens,
        Work::ReachUnits,
        Work::ReachPaths,
        Work::ReachSteps,
        Work::SegmentReads,
        Work::Lookups,
    ];

    /// The work of reaching every public function of `fan(items,
    /// references)`, by kind, and how many nodes were reached.
    fn work_of(items: usize, references: usize) -> ([usize; 6], usize) {
        let ws = fan(items, references);
        let roots: Vec<Node> = (0..items)
            .map(|i| {
                let path = format!("demo::f{i}");
                Node::Item(ws.resolve_in(&path, Namespace::Value).expect("f").sites[0])
            })
            .collect();
        let (reached, tally) = work::measure(usize::MAX, || ws.reachable(&roots));
        let reached = reached.unwrap_or_else(|e| panic!("{e}")).reached.len();
        (KINDS.map(|kind| tally.get(kind)), reached)
    }

    #[test]
    fn the_work_of_a_walk_is_counted_exactly() {
        for (n, r) in [(1, 1), (3, 2), (5, 4)] {
            let ([tokens, units, paths, steps, ..], reached) = work_of(n, r);
            assert_eq!(reached, 2 * n, "{n} {r}");
            // Each function is `pub fn f() { h(); ... }`, five tokens and
            // three for each call, and each helper `fn h() {}`, four.
            assert_eq!(tokens, n * (9 + 3 * r), "{n} {r}");
            // One push of the first unit, then for each further function
            // the helper before it and the function opened and closed, and
            // each run placed once.
            assert_eq!(units, n * r + 4 * n - 3, "{n} {r}");
            // Each helper's name is resolved once in its module, however
            // often it is called; a declared name is no path.
            assert_eq!(paths, n, "{n} {r}");
            // The tables (each item of the module and the crate root), the
            // roots, each node taken from the queue (a root, and a helper
            // once for each call), and each run and site of a function.
            assert_eq!(steps, 4 * n + 1 + 3 * n * r, "{n} {r}");
        }
    }

    #[test]
    fn the_work_grows_with_the_items_and_the_references_and_not_their_product() {
        // Three sizes in items at a fixed number of references, then in
        // references at a fixed number of items: each step adds the same
        // work, so no kind grows with a product of the two.
        let by_items: Vec<[usize; 6]> = [10, 20, 30].map(|n| work_of(n, 3).0).to_vec();
        let by_references: Vec<[usize; 6]> = [2, 4, 6].map(|r| work_of(10, r).0).to_vec();
        for sizes in [&by_items, &by_references] {
            for (kind, name) in KINDS.iter().enumerate() {
                let first = sizes[1][kind] - sizes[0][kind];
                let second = sizes[2][kind] - sizes[1][kind];
                assert_eq!(first, second, "{name:?}: {sizes:?}");
            }
        }
        // The references add no resolution: a run is resolved once in its
        // module.
        assert_eq!(by_references[0][2], by_references[2][2]);
    }

    #[test]
    fn a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix() {
        const SEGMENTS: usize = 50_000;
        let text = format!(
            "pub mod m {{ pub use crate::m; pub struct End; }}\npub type T = {}End;\n",
            "m::".repeat(SEGMENTS)
        );
        let manifest = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
        let tree = MemoryTree::from([("Cargo.toml", manifest), ("src/lib.rs", text.as_str())]);
        let ws = Workspace::load(&tree, &["."]).unwrap_or_else(|e| panic!("{e}"));
        let alias = ws.resolve("demo::T").expect("the alias").sites[0];
        let end = ws.resolve("demo::m::End").expect("the struct").sites[0];
        // Each proper prefix is one step of one walk and the last segment is
        // read in each name space, so a segment is read a few times; a walk
        // from the start for each prefix would read it once for each prefix
        // after it, and passes the ceiling at once.
        let segments = SEGMENTS + 1;
        let (reached, tally) = work::measure_kind(Work::SegmentReads, 4 * segments, || {
            ws.reachable(&[Node::Item(alias)])
        });
        let reached = reached.unwrap_or_else(|e| panic!("{e}"));
        assert!(reached.contains(Node::Item(end)));
        assert!(tally.get(Work::SegmentReads) >= segments);
        assert!(tally.get(Work::Lookups) <= 4 * segments);
        assert_eq!(tally.get(Work::ReachPaths), 1);
    }
}
