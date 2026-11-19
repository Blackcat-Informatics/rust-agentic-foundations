// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Positive and negative inputs of every built-in detector, shared by the
//! shipped self-test and the detector tests of this crate.
#![doc(hidden)]

pub const ENUM_MATCH: (&str, &[&str], &[&str]) = (
    crate::detectors::ENUM_CATCH_ALL,
    &[
        "enum State { Ready, Busy(u8), Retry { count: u8 }, Idle } fn copy(state: State) -> u8 { match state { State::Ready => 1, State::Busy(_) => 2, State::Retry { count: _ } => 3, _ => 4 } }",
        "mod model { pub enum State { Ready, Busy, Idle } } use model::State as Status; fn copy(state: Status) { let rest = (); match state { Status::Ready | Status::Busy => (), rest => () } }",
        "enum State { Ready, Busy, Idle } use State::*; fn copy(state: State) { match state { Ready => (), Busy => (), rest => () } }",
        "enum State { Ready, Busy, Idle } fn copy(state: State) { use crate::State::Ready as First; match state { First => (), State::Busy => (), _ => () } }",
        "mod types {} use types as rest; enum State { Ready, Busy } use State::*; #[cfg(any())] const Ready: u8 = 0; fn copy(state: State) { match state { Ready => (), rest => () } }",
    ],
    &[
        "fn copy(number: u32, text: &str) { match number { 0 => (), _ => () }; match text { \"ready\" => (), _ => () } }",
        "enum State { Ready, Busy(u8), Retry { count: u8 } } fn copy(state: State) { match state { State::Ready => (), State::Busy(_) => (), State::Retry { count: _ } => () } }",
        "enum State { Ready, Busy } fn copy(state: State) { match state { State::Ready => (), _ if true => (), State::Busy => () } }",
        "enum State { Ready, Busy } use State::*; fn copy(state: State) { match state { Ready => (), Busy => () } }",
        "enum State { Ready, Busy } use State::*; fn copy(a: State, b: State) { match a { Ready => match b { Busy => (), Ready => () }, Busy => () } }",
        "enum State { Ready, Busy } use State::*; const Ready: u8 = 0; fn copy(n: u8) { match n { Ready => (), _ => () } }",
        "enum State { Ready, Busy } #[cfg(any())] use State::*; fn copy(n: u8) { match n { Ready => (), _ => () } }",
        "enum State { #[cfg(any())] Ready, Busy } use State::*; fn copy(n: u8) { match n { Ready => (), _ => () } }",
        "enum State { Ready } fn copy(n: u8, state: State) { match n { #[cfg(any())] State::Ready => (), _ => () }; match state { State::Ready => (), #[cfg(any())] _ => () } }",
        "enum State { Ready, Busy } struct Scope; impl Scope { #[cfg(any())] fn absent(state: State) { match state { State::Ready => (), _ => () } } } fn copy(state: State) { #[cfg(any())] { match state { State::Ready => (), _ => () } } }",
        "enum State { Ready, Busy } struct Scope; impl Scope { #[cfg(any())] const ITEM: () = match State::Ready { State::Ready => (), _ => () }; } trait Policy { #[cfg(any())] const ITEM: () = match State::Ready { State::Ready => (), _ => () }; } fn copy(state: State) { #[cfg(any())] let _ = match state { State::Ready => (), _ => () }; }",
        "#[derive(PartialEq, Eq)] enum State { Ready, Busy } const FIRST: State = State::Ready; fn copy(state: State) { match state { FIRST => (), State::Busy => () } }",
    ],
);

pub const RDF: [(&str, &str, &str); 3] = [
    (
        "serde-json",
        "use serde_json as json; fn copy(){let _=json::from_str::<u8>(\"1\");}",
        "mod serde_json {pub fn local(){}} use serde_json as json; fn copy(){json::local();} mod local_trait {trait serde_json {fn local();} fn copy<T:serde_json>(){T::local();}} mod local_union {union serde_json {value:u8} impl serde_json {fn local(){}} fn copy(){serde_json::local();}} mod generic_union {union Storage<serde_json:Copy> {value:serde_json}}",
    ),
    (
        "rdf-text-writer",
        "fn copy(s:&str,p:&str,o:&str)->String {format!(\"<{s}> <{p}> <{o}> .\\n\")}",
        "fn copy(s:&str,p:&str,o:&str)->String {format!(\"subject={s}, predicate={p}, object={o}\")}",
    ),
    (
        "rdf-term-model",
        "enum Term {Iri(String), BlankNode(String), Literal(String)}",
        "enum Term {Iri(String), BlankNode(String), Literal(String), TripleTerm(Box<(Term,Term,Term)>)}",
    ),
];

pub const RDF_NESTED: [(&str, &str, &str); 5] = [
    (
        "serde-json",
        "mod outer {extern crate purrdf_lex as serde_json; mod inner {fn copy(){let _=::serde_json::from_str::<u8>(\"1\");}}}",
        "extern crate purrdf_lex as serde_json; use ::serde_json::allocation as alloc; fn copy(){let _=alloc::Resident;} mod inner {struct serde_json; fn copy(){let _=::serde_json::allocation::Resident;}} mod absolute {use ::serde_json; fn copy(){let _=serde_json::allocation::Resident;}} mod relative {use serde_json; fn copy(){let _=serde_json::allocation::Resident;}} mod absolute_head {use ::serde_json::allocation as serde_json; fn copy(){let _=serde_json::Resident;}} mod relative_head {use serde_json::allocation as serde_json; fn copy(){let _=serde_json::Resident;}} fn scopes(){use purrdf_lex as native; use native as serde_json; {use serde_json::allocation as native; let _=serde_json::allocation::Resident;}}",
    ),
    (
        "serde-json",
        "use purrdf_lex as serde_json; mod inner {fn copy(){let _=serde_json::from_str::<u8>(\"1\");}}",
        "use purrdf_lex as serde_json; mod inner {use purrdf_lex as serde_json; fn copy(){let _=serde_json::allocation::Resident;}}",
    ),
    (
        "serde-json",
        "macro_rules! parse {()=>{serde_json::from_str::<u8>(\"1\")}}",
        "extern crate purrdf_lex as serde_json; fn copy(){let mut out=String::new();::serde_json::json_escape::push_string(&mut out,\"x\",::serde_json::json_escape::JsonEscapes::Controls);}",
    ),
    (
        "rdf-text-writer",
        "macro_rules! emit {($s:expr,$p:expr,$o:expr)=>{format!(\"<urn:subject> <urn:predicate> \\\"{}\\\" .\",$o)}}",
        "macro_rules! emit {($s:expr,$p:expr,$o:expr)=>{format!(\"subject={}, predicate={}, object={}\",$s,$p,$o)}}",
    ),
    (
        "rdf-term-model",
        "macro_rules! terms {()=>{pub enum Term<T> {Iri(T), BlankNode(T), Literal(T)}}}",
        "macro_rules! terms {()=>{pub enum Term<T> {Iri(T), BlankNode(T), Literal(T), TripleTerm(Box<(Term<T>,Term<T>,Term<T>)>)}}}",
    ),
];

pub const HASHER_POSITIVE: [&str; 11] = [
    "use std::collections::HashMap as Map; fn copy(){let _=Map::<u8,u8>::new();}",
    "use std::collections::HashMap as r#Map; fn copy(){let _=r#Map::<u8,u8>::r#new();}",
    "mod std {} use ::std::collections::HashMap as Map; fn copy(){let _=Map::<u8,u8>::new();}",
    "use std::collections as c; fn copy(_:c::HashSet<u8>){}",
    "use std::collections as c; use c::HashMap as Map; fn copy(_:Map<u8,u8>){}",
    "struct Cache { map: std::collections::HashMap<u8,u8> }",
    "type Cache = std::collections::HashMap<u8,u8>;",
    "fn copy(){assert_eq!(1,std::collections::HashSet::from([1]));}",
    "#[cfg(test)] fn copy(){let _=std::collections::HashMap::<u8,u8>::default();}",
    "mod a {use foreign::HashMap;} mod b {use std::collections::HashMap; fn copy(_:HashMap<u8,u8>) {}}",
    "fn copy(){let _: std::collections::HashMap<u8,u8,Fixed> = wrap(std::collections::HashMap::default());}",
];

pub const HASHER_NEGATIVE: [&str; 8] = [
    "use foreign::HashMap; fn copy(_:HashMap<u8,u8>){}",
    "use std::collections::HashMap; fn copy<HashMap>(_:HashMap){}",
    "use std::collections::HashMap; struct Cache<HashMap>{map:HashMap}",
    "fn copy(_:HashMap<u8,u8>){}",
    "use std::collections::HashMap; fn copy(){let _:HashMap<u8,u8,Fixed>=HashMap::default();}",
    "fn copy(){let _=std::collections::HashSet::<u8,Fixed>::default();}",
    "use std::collections::HashMap; mod a {struct HashMap<K,V>(K,V); fn copy(_:HashMap<u8,u8>) {}}",
    "fn copy(){let _=std::collections::HashMap::with_hasher(Fixed);}",
];

pub const LAYOUT: [(&str, &str, &str); 5] = [
    (
        "length-prefix-frame",
        "fn copy(out:&mut Vec<u8>,b:&[u8]) {out.extend_from_slice(&(b.len() as u64).to_le_bytes());out.extend_from_slice(b);}",
        "fn copy(out:&mut Vec<u8>,b:&[u8]) {write_varint(out,b.len() as u64);out.extend_from_slice(b);}",
    ),
    (
        "le-slice-int",
        "fn copy(b:&[u8])->u32 {u32::from_le_bytes(b[0..4].try_into().unwrap())}",
        "fn copy(b:&[u8])->u32 {u32::from_le_bytes(*b.first_chunk().unwrap())}",
    ),
    (
        "align-up-mask",
        "fn copy(n:usize)->usize {(n+7)&!7}",
        "fn copy(n:usize)->usize {n.next_multiple_of(8)}",
    ),
    (
        "div-ceil-by-hand",
        "fn copy(n:usize,d:usize)->usize {(n+d-1)/d}",
        "fn copy(n:usize,d:usize)->usize {n.div_ceil(d)}",
    ),
    (
        "xor-first-mismatch",
        "fn copy(a:&[u8;8],b:&[u8;8])->u32 {(u64::from_le_bytes(*a)^u64::from_le_bytes(*b)).trailing_zeros()}",
        "fn copy(a:&[u8;8],b:&[u8;8])->u64 {u64::from_le_bytes(*a)^u64::from_le_bytes(*b)}",
    ),
];

pub const FACTS: [(&str, &str, &str, bool); 2] = [
    (
        "hex-table",
        "fn copy(){let _=b\"0123456789abcdef\";}",
        "fn copy(){let _=b\"0123456789abcdeg\";}",
        true,
    ),
    (
        "json-pointer-escape",
        "fn copy(){let _=\"~0\";}",
        "fn copy(){let _=b\"~0\";}",
        false,
    ),
];

pub const HEX: [(&str, &str, &str, bool); 3] = [
    (
        "hex-pair-radix",
        "fn copy(s:&str){let _=<u8>::from_str_radix(s,0x10);}",
        "fn copy(s:&str){let _=u8::from_str_radix(s,10);}",
        true,
    ),
    (
        "hex-digit-radix",
        "fn copy(c:char){let _=c.to_digit(0x10);}",
        "fn copy(c:char){let _=c.to_digit(10);}",
        false,
    ),
    (
        "hex-format-loop",
        "fn copy(b:u8){let _=||format!(\"{b:02x}\");}",
        "fn copy(b:u8){let _=format!(\"{b:02x}\");}",
        true,
    ),
];

/// Job-scoped detectors: id, home source, positive and negative consumers.
///
/// The `vocabulary-literal` sides cover its embedded-document exemption: a
/// keyword without a line break, a line break without a keyword and keywords
/// only inside longer words stay hits; a multi-line literal with a whole-word
/// keyword in any letter case is clean.
pub const CONTEXTUAL: [(&str, &str, &[&str], &[&str]); 3] = [
    (
        "home-literal",
        "/// Home.\npub fn home()->&'static str {\"token\"}",
        &["fn copy(){let _=\"token\";}"],
        &["fn copy(){let _=\"different\";}"],
    ),
    (
        "vocabulary-literal",
        "/// Vocabulary.\npub mod home {pub const NS:&str={ (\"urn:demo:\") };}",
        &[
            "fn copy(){let _=\"urn:demo:term\";}",
            "fn copy(){let _=\"urn:demo:term WHERE\";}",
            "fn copy(){let _=\"urn:demo:term\\nurn:demo:other\";}",
            "fn copy(){let _=\"urn:demo:term\\nPREFIXED reselect select_all ask1\";}",
        ],
        &[
            "fn copy(){let _=\"urn:other:term\";}",
            "fn copy(){let _=\"urn:demo:\\nselect ?s where {?s ?p ?o}\";}",
        ],
    ),
    (
        "foreign-host-import",
        "/// Home.\npub mod home {#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(js_name = now)] pub fn read() -> f64;}}",
        &[
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {fn now() -> f64;}",
            "#[wasm_bindgen::prelude::wasm_bindgen(module = \"clock\")] extern \"C\" {fn now() -> f64;}",
            "#[cfg_attr(target_arch = \"wasm32\", wasm_bindgen(module = \"clock\"))] extern \"C\" {#[wasm_bindgen(js_name = \"now\")] fn tick() -> f64;}",
        ],
        &[
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {fn later() -> f64;}",
            "#[wasm_bindgen(module = \"other\")] extern \"C\" {fn now() -> f64;}",
            "extern \"C\" {fn now() -> f64;}",
            "#[wasm_bindgen(js_namespace = clock)] extern \"C\" {fn now() -> f64;}",
            "#[shim::cfg_attr(target_arch = \"wasm32\", wasm_bindgen(module = \"clock\"))] extern \"C\" {fn now() -> f64;}",
            "#[wasm_bindgen(raw_module = \"clock\")] extern \"C\" {fn now() -> f64;}",
            "#[wasm_bindgen(module = \"clock\")] extern \"C\" {#[wasm_bindgen(getter)] fn now(this: &Clock) -> f64;}",
        ],
    ),
];

/// An entry-point detector: id, home source, entry points, positive and negative consumers.
pub type StructureFixture = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    &'static [&'static str],
);

/// An `expect` argument constructs a distinct row on line four.
pub const EXPECT_ARGUMENT_ROW: &str = "fn copy(d:&mut Dataset,graph:u8){
    d.insert(
        demo::triple(1,rdf::REIFIES,2).expect(note(
            demo::triple(3,rdf::REIFIES,4)
        )),graph);
}";

/// Entry-point detectors: id, home source, the home's entry points, positive
/// and negative consumers. The home declares the approved constructor and
/// traversal; a consumer is judged by what it calls.
///
/// `reifier-default-graph` sides cover the known constant (written in full,
/// imported, or under `vocab`), the literal IRI and the triple-term object, a
/// call through an import, a triple row and a method row in dataset code, and
/// leave clean an explicit graph on a quad, a stored triple or a selected graph,
/// as well as unrelated row terms and constructors. A graph supplied elsewhere
/// and a shadowed, expired or reassigned handle leave a default row visible.
/// `rdf-list-walk` sides cover a loop, the literal
/// IRIs and recursion, delegation to the declared traversal, one term alone
/// and two terms without iteration.
pub const STRUCTURE: [StructureFixture; 2] = [
    (
        "reifier-default-graph",
        "/// Home.\npub fn home() {}\n/// Row.\npub fn quad(s:u8,p:u8,o:u8,g:Option<u8>) {}\n/// Triple.\npub fn triple(s:u8,p:u8,o:u8) -> u8 {s}\n/// Insert.\npub fn insert(s:u8,p:u8,o:u8) {}\n/// A dataset.\npub struct Dataset;\nimpl Dataset {\n/// Insert a row.\npub fn insert(&mut self,s:u8,p:u8,o:u8) {}\n/// Construct a row.\npub fn construct(s:u8,p:u8,o:u8) {}\n/// Build a row.\npub fn build(s:u8,p:u8,o:u8) {}\n}",
        &[
            "demo::quad",
            "demo::triple",
            "demo::insert",
            "demo::Dataset::insert",
            "demo::Dataset::construct",
        ],
        &[
            "fn copy(d:&Dataset){demo::quad(1,rdf::REIFIES,2,None);}",
            "fn copy(d:&Dataset){demo::quad(1,\"http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies\",2,None);}",
            "fn copy(d:&Dataset){demo::quad(1,2,Term::Triple(3),None);}",
            "fn copy(d:&Dataset,b:&mut Builder){demo::quad(1,rdf::TYPE,b.intern_triple(1,2,3)?,None);}",
            "use demo::quad as build; fn copy(d:&Dataset){build(1,RDF_REIFIES,2,None);}",
            "fn copy(d:&Dataset){use demo::quad as build;build(1,Term::iri(RDF_REIFIES),2,None);}",
            "struct Dataset; impl Dataset {fn copy(&mut self){demo::quad(1,rdf::REIFIES,2,None);}}",
            "fn copy(g:u8,d:&Dataset){demo::quad(1,rdf::REIFIES,2,None);}",
            "fn copy(graph_name:u8){demo::triple(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset){d.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&demo::Dataset){demo::Dataset::construct(1,rdf::REIFIES,2);}",
            "use demo::Dataset as Target; mod other {pub struct Dataset; impl Dataset {pub fn insert(&mut self,s:u8,p:u8,o:u8){}}} fn copy(d:&mut Target,named_graph:u8){{use crate::other::Dataset as Target;d.insert(1,rdf::REIFIES,2);}}",
            "fn copy(store:&mut Store,d:&Dataset){store.insert(demo::triple(1,rdf::REIFIES,2),None);}",
            "fn copy(d:&mut demo::Dataset,name:u8){let g:&mut demo::Dataset=d.other(name);g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut Dataset,count:u8){d.insert(demo::triple(1,rdf::REIFIES,2),count);}",
            "fn copy(d:&Dataset,graph:u8){inspect(demo::triple(1,rdf::REIFIES,2),Some(graph));}",
            "fn copy(d:&mut Dataset,graph:u8){d.insert(copy_of(demo::triple(1,rdf::REIFIES,2)),Some(graph));}",
            "fn copy(d:&mut demo::Dataset,name:u8){let g:&mut demo::Dataset=d.graph_mut(name);{let g:&mut demo::Dataset=d.other();g.insert(1,rdf::REIFIES,2);}g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,name:u8,g:&mut demo::Dataset){{let g:&mut demo::Dataset=d.graph_mut(name);g.insert(1,rdf::REIFIES,2);}g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);g=d.other(name);g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);(g,_)=(fallback,());g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);for _ in 0..2 {g.insert(1,rdf::REIFIES,2);g=&mut *fallback;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,mut left:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);while left>0 {g.insert(1,rdf::REIFIES,2);g=&mut *fallback;left-=1;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,mut left:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);loop {g.insert(1,rdf::REIFIES,2);g=&mut *fallback;if left==0{break;}left-=1;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,mut again:bool){let mut g:&mut demo::Dataset=d.graph_mut(name);loop {g.insert(1,rdf::REIFIES,2);g=&mut *fallback;if again{again=false;continue;}break;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,mut again:bool){let mut g:&mut demo::Dataset=d.graph_mut(name);'entries: while {g.insert(1,rdf::REIFIES,2);if again{g=&mut *fallback;again=false;continue 'entries;}true} {break;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);#[cfg(feature=\"optional\")] {g=fallback;}g.insert(1,rdf::REIFIES,2);}",
            EXPECT_ARGUMENT_ROW,
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,stop:bool){let mut g:&mut demo::Dataset=d.graph_mut(name);loop {g.insert(1,rdf::REIFIES,2);if stop{g=fallback;break;}}g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);for _ in 0..2 {g.insert(1,rdf::REIFIES,2);loop {g=&mut *fallback;break;}}}",
            "async fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);async {g=fallback;}.await;g.insert(1,rdf::REIFIES,2);}",
            "use purrdf_iri::vocab::rdf::REIFIES; fn copy(d:&Dataset){demo::quad(1,REIFIES,2,None);}",
            "use raf_rdf::vocab; fn copy(d:&Dataset){demo::quad(1,vocab::REIFIES,2,None);}",
        ],
        &[
            "fn copy(d:&Dataset,graph:u8){demo::quad(1,rdf::REIFIES,2,Some(graph));}",
            "fn copy(d:&Dataset){demo::quad(1,rdf::TYPE,2,None);}",
            "fn copy(d:&Dataset){demo::quad(rdf::REIFIES,rdf::TYPE,2,None);demo::quad(Term::Triple(3),rdf::TYPE,2,None);}",
            "fn copy(d:&Dataset,b:&mut Builder){demo::quad(1,rdf::TYPE,b.intern_pair(1,2,3)?,None);demo::quad(1,rdf::REIFIES.replace(\"reifies\",\"type\"),2,None);}",
            "struct Other; impl Other {fn insert(&mut self,s:u8,p:u8,o:u8){}} fn copy(d:&Dataset,other:&mut Other){other.insert(1,rdf::REIFIES,2);}",
            "trait Build {fn build(&mut self,s:u8,p:u8,o:u8);} impl Build for demo::Dataset {fn build(&mut self,s:u8,p:u8,o:u8){}} fn copy(d:&mut demo::Dataset){d.build(1,rdf::REIFIES,2);}",
            "fn copy(d:&Dataset){other::quad(1,rdf::REIFIES,2,None);}",
            "fn copy(){demo::quad(1,rdf::REIFIES,2,None);}",
            "fn copy(#[cfg(any())] d:&Dataset){demo::quad(1,rdf::REIFIES,2,None);}",
            "fn copy(store:&mut Store,d:&Dataset,graph:u8){store.insert(demo::triple(1,rdf::REIFIES,2),Some(graph));}",
            "fn copy(d:&mut Dataset,name:u8){d.graph_mut(name).insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut Dataset,graph:u8){d.insert(demo::triple(1,rdf::REIFIES,2),graph);}",
            "fn copy(d:&mut Dataset,name:u8){d.insert(demo::triple(1,rdf::REIFIES,2),GraphName::Named(name));}",
            "fn copy(d:&mut Dataset,name:u8){d.insert(demo::triple(1,rdf::REIFIES,2),&name);}",
            "fn copy(d:&mut demo::Dataset,name:u8){let g:&mut demo::Dataset=d.graph_mut(name);g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut Dataset,destination:&GraphName){d.insert(destination,demo::triple(1,rdf::REIFIES,2)?);}",
            "fn copy(d:&mut Dataset,name:u8){d.graph_mut(name)?.insert(demo::triple(1,rdf::REIFIES,2));}",
            "fn copy(d:&mut Dataset,graph:u8){d.insert(demo::triple(1,rdf::REIFIES,2).unwrap(),graph);d.insert(demo::triple(1,rdf::REIFIES,2).expect(\"row\"),graph);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);#[cfg(any())] {g=fallback;}g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);match name {#[cfg(any())] 0=>{g=fallback;},_=>{}}g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);loop {g.insert(1,rdf::REIFIES,2);g=fallback;#[cfg(any())] {continue;}#[cfg(any())] absent!();break;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);for _ in 0..2 {g.insert(1,rdf::REIFIES,2);g=fallback;break;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);while name>0 {g.insert(1,rdf::REIFIES,2);g=fallback;break;}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,stop:bool){let mut g:&mut demo::Dataset=d.graph_mut(name);loop {g.insert(1,rdf::REIFIES,2);if stop{g=fallback;break;}}}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8){let mut g:&mut demo::Dataset=d.graph_mut(name);{let _unused=async {g=fallback;};}g.insert(1,rdf::REIFIES,2);}",
            "fn copy(d:&mut demo::Dataset,fallback:&mut demo::Dataset,name:u8,values:&mut [u8]){let g:&mut demo::Dataset=d.graph_mut(name);for _ in 0..2 {{let mut g=&mut *fallback;g=&mut *fallback;}values[g.len()]=0;g.insert(1,rdf::REIFIES,2);}}",
            "mod other {pub const REIFIES:u8=0;} use crate::other as terms; #[cfg(any())] use raf_rdf::vocab as terms; fn copy(d:&Dataset){demo::quad(1,terms::REIFIES,2,None);}",
            "fn copy(graph:&mut Graph){graph.insert(1,rdf::REIFIES,2);}",
            "fn copy(paragraph:u8,grapheme:u8,graphql:u8){demo::quad(1,rdf::REIFIES,2,None);}",
            "fn copy(d:&Dataset){demo::quad(1,REIFIES,2,None);}",
            "use other::REIFIES; fn copy(d:&Dataset){demo::quad(1,REIFIES,2,None);}",
        ],
    ),
    (
        "rdf-list-walk",
        "/// Home.\npub fn home() {}\n/// Walk.\npub fn walk_list(list:u8) -> u8 {list}\n/// A dataset.\npub struct Dataset;\nimpl Dataset {\n/// Walk a collection.\npub fn walk_list(&mut self) {}\n}",
        &["demo::walk_list", "demo"],
        &[
            "fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;n-=1;}}",
            "fn copy(l:&[u8]){for _ in l {let _=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#first\";let _=\"http://www.w3.org/1999/02/22-rdf-syntax-ns#rest\";}}",
            "fn copy(n:u8){let _=RDF_FIRST;let _=RDF_REST;copy(n);}",
            "struct Cursor; impl Cursor {fn next(&self){let _=rdf::FIRST;let _=rdf::REST;self.next();}}",
            "struct Other; impl Other {fn walk_list(&self){}} fn copy(mut n:u8,other:&Other){while n>0 {let _=rdf::FIRST;let _=rdf::REST;other.walk_list();n-=1;}}",
            "trait Walk {fn walk_list(&self);} impl Walk for demo::Dataset {fn walk_list(&self){}} fn copy(mut n:u8,mut d:demo::Dataset){while n>0 {let _=rdf::FIRST;let _=rdf::REST;d.walk_list();n-=1;}}",
            "struct Other; impl Other {fn walk_list(&mut self){}} fn copy(mut n:u8,d:&mut Other){#[cfg(any())] let d:&mut demo::Dataset=unreachable!();while n>0 {let _=rdf::FIRST;let _=rdf::REST;d.walk_list();n-=1;}}",
            "fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;#[cfg(any())] demo::walk_list(n);n-=1;}}",
            "fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;#[cfg(any())] {demo::walk_list(n);}n-=1;}}",
            "fn walk(n:u8)->u8{n} fn copy(mut n:u8){#[cfg(any())] use demo::walk_list as walk;while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=walk(n);}}",
            "fn copy(mut n:u8){let unused=||demo::walk_list(n);while n>0 {let _=rdf::FIRST;let _=rdf::REST;n-=1;}}",
            "fn copy(mut n:u8){let unused=async {demo::walk_list(n);};while n>0 {let _=rdf::FIRST;let _=rdf::REST;n-=1;}}",
            "trait Walk {fn walk_list();} impl Walk for &demo::Dataset {fn walk_list(){}} fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;<&demo::Dataset>::walk_list();n-=1;}}",
            "use demo::walk_list; fn copy(mut n:u8,walk_list:fn(u8)->u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=walk_list(n);}}",
            "use purrdf_iri::vocab::rdf::{FIRST, REST}; fn copy(mut n:u8){while n>0 {let _=FIRST;let _=REST;n-=1;}}",
            "mod other {pub const FIRST:u8=0;pub const REST:u8=0;} #[cfg(any())] use crate::other as terms; use purrdf_iri::vocab::rdf as terms; fn copy(mut n:u8){while n>0 {let _=terms::FIRST;let _=terms::REST;n-=1;}}",
            "use purrdf_iri::vocab::rdf as terms; #[cfg(any())] mod terms {} fn copy(mut n:u8){while n>0 {let _=terms::FIRST;let _=terms::REST;n-=1;}}",
            "use ::demo::walk_list as walk; #[cfg(feature=\"optional\")] use ::demo::home as walk; fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=walk(n);}}",
        ],
        &[
            "fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=demo::walk_list(n);}}",
            "use ::demo::walk_list as walk; #[cfg(any())] use ::demo::home as walk; fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=walk(n);}}",
            "fn copy(mut n:u8){use ::demo::walk_list as walk; #[cfg(any())] use ::demo::home as walk; #[cfg(any())] use ::demo::*; #[cfg(any())] fn walk(n:u8)->u8{n} while n>0 {let _=rdf::FIRST;let _=rdf::REST;n=walk(n);}}",
            "trait Walk {fn walk_list(&self);} impl Walk for demo::Dataset {fn walk_list(&self){}} fn copy(mut n:u8,d:&mut demo::Dataset){while n>0 {let _=rdf::FIRST;let _=rdf::REST;d.walk_list();n-=1;}}",
            "fn copy(mut n:u8,mut d:demo::Dataset){while n>0 {let _=rdf::FIRST;let _=rdf::REST;demo::Dataset::walk_list(&mut d);n-=1;}}",
            "fn copy(mut n:u8,mut d:demo::Dataset){while n>0 {let _=rdf::FIRST;let _=rdf::REST;<demo::Dataset>::walk_list(&mut d);n-=1;}}",
            "struct State<const N:usize>; impl<const N:usize> State<N>{fn read()->usize{N}} type A=State<{1+2}>; fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;let _=rdf::REST;let _=A::read();n=demo::walk_list(n);}}",
            "fn copy(mut n:u8){while n>0 {let _=rdf::FIRST;n-=1;}}",
            "fn copy(){let _=rdf::FIRST;let _=rdf::REST;}",
            "fn copy(){let _=rdf::FIRST;let _=rdf::REST;#[cfg(any())] loop {break;}}",
            "struct Other; impl Other {fn next(&self){}} struct Cursor; impl Cursor {fn next(&self,other:&Other){let _=rdf::FIRST;let _=rdf::REST;other.next();}}",
            "use other::{FIRST, REST}; fn copy(mut n:u8){while n>0 {let _=FIRST;let _=REST;n-=1;}}",
            "mod other {pub const FIRST:u8=0;pub const REST:u8=0;} use crate::other as terms; #[cfg(any())] use purrdf_iri::vocab::rdf as terms; fn copy(mut n:u8){while n>0 {let _=terms::FIRST;let _=terms::REST;n-=1;}}",
            "mod other {pub const FIRST:u8=0;pub const REST:u8=0;} #[cfg(feature=\"optional\")] use crate::other as terms; use purrdf_iri::vocab::rdf as terms; fn copy(mut n:u8){while n>0 {let _=terms::FIRST;let _=terms::REST;n-=1;}}",
        ],
    ),
];

pub const HASHER_ID: &str = "std-default-hasher";
pub const SOLE_NAME_ID: &str = "sole-name";
pub const SOLE_NAME_POSITIVE: [&str; 2] = ["fn copy(){}", "#[cfg(test)] mod oracle {fn copy(){}}"];
pub const SOLE_NAME_NEGATIVE: [(&str, &str); 1] =
    [("/// Home.\npub fn home(){} pub fn copy(){}", "demo::copy")];
