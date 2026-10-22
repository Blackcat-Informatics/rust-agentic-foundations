#[cfg(test)]
mod tests {
    use std::fmt::Write as _;
    use std::mem::size_of;

    use super::{DECLARES, Run, RunKind, lex_text, place_runs};
    use crate::budget::Budget;
    use crate::error::LoadErrorKind;
    use crate::item::Namespace;
    use crate::reach::{Node, ReachError};
    use crate::work::{self, Work};
    use crate::{LoadOptions, MemoryTree, Workspace};

    /// The runs of `text` in `edition` as their text and the byte where each
    /// starts: a path as written without spaces, a run that `Self` starts as
    /// `Self::...`, and a method called on `self` as `.name`.
    fn runs_in(edition: u16, text: &str) -> Vec<(String, usize)> {
        let lexed = lex_text(text, edition).unwrap_or_else(|failure| panic!("{}", failure.message));
        lexed
            .runs
            .iter()
            .map(|run| {
                let words: Vec<&str> = lexed.segs[run.first..run.first + run.len]
                    .iter()
                    .map(|(start, end)| &text[*start..*end])
                    .collect();
                let shown = match run.kind {
                    RunKind::Method => format!(".{}", words.join("::")),
                    RunKind::Path | RunKind::SelfPath => format!(
                        "{}{}",
                        if run.leading_colon { "::" } else { "" },
                        words.join("::")
                    ),
                };
                (shown, run.start)
            })
            .collect()
    }

    fn runs(text: &str) -> Vec<(String, usize)> {
        runs_in(2021, text)
    }

    /// The texts of the runs of `text` in `edition`.
    fn texts_in(edition: u16, text: &str) -> Vec<String> {
        runs_in(edition, text)
            .into_iter()
            .map(|(text, _)| text)
            .collect()
    }

    /// The texts of the runs of `text` in edition 2021.
    fn texts(text: &str) -> Vec<String> {
        texts_in(2021, text)
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
        // A raw segment is read without its prefix, where it starts.
        assert_eq!(runs("r#gen::x"), [("gen::x".to_owned(), 0)]);
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
        assert_eq!(texts("pub use a as b;"), ["a", "b"]);
        // `Self` starts a run of its own and is no later segment.
        assert_eq!(
            texts("Self::new(); impl Tr for S; a::Self::b; ::Self::c"),
            ["Self::new", "Tr", "S", "a"]
        );
        assert_eq!(texts("Self { a: 1 }; Self(2)"), ["Self", "Self"]);
    }

    #[test]
    fn the_keywords_are_those_of_the_edition() {
        let text = "async(); await(); dyn(); try(); gen(); r#gen(); r#try()";
        assert_eq!(
            texts_in(2015, text),
            ["async", "await", "dyn", "try", "gen", "gen", "try"]
        );
        for edition in [2018, 2021] {
            assert_eq!(texts_in(edition, text), ["gen", "gen", "try"], "{edition}");
        }
        assert_eq!(texts_in(2024, text), ["gen", "try"]);
        // Every edition's keywords end a run.
        assert_eq!(texts_in(2015, "a::fn::b; a::dyn::b"), ["a", "a::dyn::b"]);
        assert_eq!(texts_in(2024, "a::gen::b; a::r#gen::b"), ["a", "a::gen::b"]);
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
    fn const_declares_nothing_so_a_pointer_and_a_raw_borrow_name_their_path() {
        assert!(!DECLARES.contains(&"const"));
        assert_eq!(
            texts("fn f(p: *const Helper, q: *mut Other) {}"),
            ["Helper", "Other"]
        );
        assert_eq!(
            texts("&raw const VALUE; &raw mut OTHER"),
            ["raw", "VALUE", "raw", "OTHER"]
        );
        // Where `const` declares, what it declares is still no run: a
        // constant, a const fn, a const block and a const parameter.
        assert_eq!(texts("const C: u8 = 1; pub const D: E = F;"), ["u8", "E", "F"]);
        assert_eq!(texts("const fn helper() {} const unsafe fn other() {}"), Vec::<String>::new());
        assert_eq!(texts("const { g() }"), ["g"]);
        assert_eq!(texts("fn f<const N: usize>() {}"), ["usize"]);
        assert_eq!(texts("impl<const N: usize> S<N> {}"), ["usize", "S", "N"]);
    }

    #[test]
    fn a_where_clause_keeps_the_type_its_bound_is_put_on() {
        assert_eq!(
            texts("fn f<T>() where Helper: Clone, T: Into<Other> {}"),
            ["Helper", "Clone", "T", "Into", "Other"]
        );
        // The clause ends at the body or at `;`, and a bound name after it is
        // no run again.
        assert_eq!(
            texts("fn f(helper: u8) where T: X { S { field: 1 } }"),
            ["u8", "T", "X", "S"]
        );
        assert_eq!(
            texts("fn g() where Self: Sized; fn h(x: u8) {}"),
            ["Self", "u8"]
        );
        assert_eq!(
            texts("struct S<T> where T: X { field: u8 } struct P(u8) where Q: Y; static Z: u8 = 0;"),
            ["T", "X", "u8", "u8", "Q", "Y", "u8"]
        );
        // A generic parameter's own bound is a name being bound.
        assert_eq!(texts("fn f<T: Clone>() {}"), ["Clone"]);
    }

    #[test]
    fn a_name_called_on_self_is_a_method_run_and_a_field_of_self_is_not() {
        assert_eq!(
            texts("self.helper(); self.field; self.turbo::<u8>(); other.method(); self.0.x()"),
            ["self", ".helper", "self", "self", ".turbo", "u8", "other", "self"]
        );
        let text = "fn f(&self) { self.r#match() }";
        assert_eq!(
            runs(text),
            [
                ("self".to_owned(), text.find("self.").expect("self")),
                (".match".to_owned(), text.find("r#match").expect("the method")),
            ]
        );
        // `self` as a path, and a name after a `.` of anything else, are not.
        assert_eq!(texts("self::helper(); s.helper(); Self.x()"), ["self::helper", "s", "Self"]);
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
        let failure = lex_text("fn f() {\n\"", 2021).expect_err("an open string");
        assert_eq!(failure.line, 2);
    }

    fn run_at(start: usize) -> Run {
        Run {
            start,
            first: 0,
            len: 1,
            leading_colon: false,
            kind: RunKind::Path,
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

    const MANIFEST: &str = "[package]\nname = \"demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

    fn load_with(text: &str, options: &LoadOptions) -> Workspace {
        let tree = MemoryTree::from([("Cargo.toml", MANIFEST), ("src/lib.rs", text)]);
        Workspace::load_with(&tree, &["."], options).unwrap_or_else(|e| panic!("{e}"))
    }

    fn load(text: &str) -> Workspace {
        load_with(text, &LoadOptions::default())
    }

    /// A library of `items` public functions, each calling its own private
    /// helper `references` times.
    fn fan(items: usize, references: usize) -> Workspace {
        let mut text = String::new();
        for i in 0..items {
            let calls = format!("h{i}(); ").repeat(references);
            writeln!(text, "pub fn f{i}() {{ {calls}}}\nfn h{i}() {{}}").expect("write");
        }
        load(&text)
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

    /// The work of reaching `roots` in `ws`, by kind, and how many nodes
    /// were reached.
    fn work_of(ws: &Workspace, roots: &[Node]) -> ([usize; 6], usize) {
        let (reached, tally) = work::measure(usize::MAX, || ws.reachable(roots));
        let reached = reached.unwrap_or_else(|e| panic!("{e}")).reached.len();
        (KINDS.map(|kind| tally.get(kind)), reached)
    }

    /// The work of reaching every public function of `fan(items,
    /// references)`.
    fn fan_work(items: usize, references: usize) -> ([usize; 6], usize) {
        let ws = fan(items, references);
        let roots: Vec<Node> = (0..items)
            .map(|i| {
                let path = format!("demo::f{i}");
                Node::Item(ws.resolve_in(&path, Namespace::Value).expect("f").sites[0])
            })
            .collect();
        work_of(&ws, &roots)
    }

    #[test]
    fn the_work_of_a_walk_is_counted_exactly() {
        for (n, r) in [(1, 1), (3, 2), (5, 4)] {
            let ([tokens, units, paths, steps, ..], reached) = fan_work(n, r);
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
            // roots, each node taken from the queue (a root and its helper),
            // each run of a function, and the one site its first run names:
            // a run read again in its module gives nothing.
            assert_eq!(steps, 5 * n + 1 + n * r, "{n} {r}");
        }
    }

    #[test]
    fn the_work_grows_with_the_items_and_the_references_and_not_their_product() {
        // Three sizes in items at a fixed number of references, then in
        // references at a fixed number of items: each step adds the same
        // work, so no kind grows with a product of the two.
        let by_items: Vec<[usize; 6]> = [10, 20, 30].map(|n| fan_work(n, 3).0).to_vec();
        let by_references: Vec<[usize; 6]> = [2, 4, 6].map(|r| fan_work(10, r).0).to_vec();
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

    /// A library of a trait of `members` methods and `types` modules below
    /// one module `top`, each holding a struct with an inherent block of
    /// `members` methods and a block of the trait, an enum with a block,
    /// a type alias, a constant, a static and a function. Each method calls
    /// its next sibling through `Self::`, through `self.` and names the
    /// constant and the static; each trait method calls its inherent
    /// namesake on `self`.
    fn kinds(types: usize, members: usize) -> String {
        let mut text = String::from("pub trait Tr {\n");
        for j in 0..members {
            writeln!(text, "    fn t{j}(&self) -> u8;").expect("write");
        }
        text.push_str("}\npub mod top {\n");
        for i in 0..types {
            writeln!(
                text,
                "pub mod m{i} {{\n\
                 pub struct T{i} {{ f: u8 }}\n\
                 pub enum E{i} {{ A, B }}\n\
                 pub type A{i} = T{i};\n\
                 const K{i}: u8 = 1;\n\
                 static S{i}: u8 = 2;\n\
                 pub fn helper{i}() -> E{i} {{ E{i}::A }}\n\
                 impl E{i} {{ pub fn is_a(&self) -> bool {{ matches!(self, Self::A) }} }}"
            )
            .expect("write");
            text.push_str(&format!("impl T{i} {{\n"));
            for j in 0..members {
                let next = (j + 1) % members;
                writeln!(
                    text,
                    "    pub fn a{j}(&self) -> u8 {{ Self::a{next}(self) + self.a{next}() + K{i} + S{i} }}"
                )
                .expect("write");
            }
            text.push_str(&format!("}}\nimpl crate::Tr for T{i} {{\n"));
            for j in 0..members {
                writeln!(text, "    fn t{j}(&self) -> u8 {{ self.a{j}() }}").expect("write");
            }
            text.push_str("}\n}\n");
        }
        text.push_str("}\n");
        text
    }

    /// The work of reaching the module `top` of `kinds(types, members)`.
    fn kinds_work(types: usize, members: usize) -> ([usize; 6], usize) {
        let ws = load(&kinds(types, members));
        let top = ws.resolve_in("demo::top", Namespace::Type).expect("top").sites[0];
        work_of(&ws, &[Node::Item(top)])
    }

    #[test]
    fn the_work_over_types_impl_blocks_traits_and_members_grows_with_each_and_not_their_product() {
        // Every item and block below `top` is reached: the module, and for
        // each type its module, the struct, the enum, the alias, the
        // constant, the static, the function, the method of the enum, the
        // three blocks and the 2k methods.
        for (n, k) in [(1, 1), (2, 3), (4, 2)] {
            let (_, reached) = kinds_work(n, k);
            assert_eq!(reached, 1 + n * (11 + 2 * k), "{n} {k}");
        }
        // Equal steps in the types at a fixed number of members, and in the
        // members at a fixed number of types, in every counted kind: no work
        // grows with the types times the blocks, nor with the members times
        // the paths.
        let by_types: Vec<[usize; 6]> = [4, 8, 12].map(|n| kinds_work(n, 3).0).to_vec();
        let by_members: Vec<[usize; 6]> = [2, 4, 6].map(|k| kinds_work(4, k).0).to_vec();
        for sizes in [&by_types, &by_members] {
            for (kind, name) in KINDS.iter().enumerate() {
                let first = sizes[1][kind] - sizes[0][kind];
                let second = sizes[2][kind] - sizes[1][kind];
                assert_eq!(first, second, "{name:?}: {sizes:?}");
            }
        }
        // The exact counts, by kind, for n types of k members.
        for (n, k) in [(1, 1), (2, 3), (4, 2), (3, 5)] {
            let ([tokens, units, paths, steps, segments, lookups], _) = kinds_work(n, k);
            assert_eq!(tokens, KINDS_TOKENS.0 + n * (KINDS_TOKENS.1 + KINDS_TOKENS.2 * k) + KINDS_TOKENS.3 * k, "{n} {k}");
            let _ = (units, paths, steps, segments, lookups);
        }
    }

    const KINDS_TOKENS: (usize, usize, usize, usize) = (0, 0, 0, 0);

    #[test]
    fn a_path_of_fifty_thousand_segments_is_walked_once_and_not_once_for_each_prefix() {
        const SEGMENTS: usize = 50_000;
        let text = format!(
            "pub mod m {{ pub use crate::m; pub struct End; }}\npub type T = {}End;\n",
            "m::".repeat(SEGMENTS)
        );
        let ws = load(&text);
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

    /// A library whose public function calls `a` `runs` times.
    fn dense(runs: usize) -> String {
        format!("pub fn f() {{ {}}}\nfn a() {{}}\n", "a; ".repeat(runs))
    }

    /// The least item-bytes limit under which the walk from `demo::f` of
    /// `text` is given, less what the load holds: what the walk charges.
    fn walk_charge(text: &str) -> usize {
        let ws = load(text);
        let held = ws.usage().item_bytes;
        let walk = |limit: usize| {
            let ws = load_with(
                text,
                &LoadOptions {
                    max_item_bytes: limit,
                    ..LoadOptions::default()
                },
            );
            let f = ws.resolve_in("demo::f", Namespace::Value).expect("f").sites[0];
            ws.reachable(&[Node::Item(f)])
        };
        let (mut low, mut high) = (held, held + (1 << 24));
        assert!(walk(high).is_ok());
        while high - low > 1 {
            let mid = low + (high - low) / 2;
            if walk(mid).is_ok() {
                high = mid;
            } else {
                low = mid;
            }
        }
        let error = walk(low).expect_err("one byte less is refused");
        match error {
            ReachError::Budget(error) => {
                assert!(
                    matches!(
                        error.kind,
                        LoadErrorKind::Budget {
                            budget: Budget::ItemBytes,
                            limit
                        } if limit == low
                    ),
                    "{error}"
                );
                assert_eq!(error.file.as_str(), "src/lib.rs");
            }
            other => panic!("{other}"),
        }
        high - held
    }

    #[test]
    fn what_the_walk_holds_is_charged_to_the_item_bytes_budget_on_from_the_load() {
        // Each run holds its record, its segment and its place in its unit,
        // and nothing else: the text it resolves to is held once.
        let per_run = size_of::<Run>() + size_of::<(usize, usize)>() + size_of::<usize>();
        let charges = [1000, 2000, 3000].map(|runs| walk_charge(&dense(runs)));
        assert_eq!(charges[1] - charges[0], 1000 * per_run);
        assert_eq!(charges[2] - charges[1], 1000 * per_run);
        // `a; ` is three bytes of text for each run.
        assert!(per_run <= 20 * 3, "{per_run}");
    }
}
