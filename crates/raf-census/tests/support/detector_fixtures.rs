// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Shared inputs for the production Gate's built-in detector cases.

pub(crate) const RDF: [(&str, &str, &str); 3] = [
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

pub(crate) const RDF_NESTED: [(&str, &str, &str); 5] = [
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
        "macro_rules! terms {()=>{enum Term<T> {Iri(T), BlankNode(T), Literal(T)}}}",
        "macro_rules! terms {()=>{enum Term<T> {Iri(T), BlankNode(T), Literal(T), TripleTerm(Box<(Term<T>,Term<T>,Term<T>)>)}}}",
    ),
];

pub(crate) const HASHER_POSITIVE: [&str; 11] = [
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

pub(crate) const HASHER_NEGATIVE: [&str; 8] = [
    "use foreign::HashMap; fn copy(_:HashMap<u8,u8>){}",
    "use std::collections::HashMap; fn copy<HashMap>(_:HashMap){}",
    "use std::collections::HashMap; struct Cache<HashMap>{map:HashMap}",
    "fn copy(_:HashMap<u8,u8>){}",
    "use std::collections::HashMap; fn copy(){let _:HashMap<u8,u8,Fixed>=HashMap::default();}",
    "fn copy(){let _=std::collections::HashSet::<u8,Fixed>::default();}",
    "use std::collections::HashMap; mod a {struct HashMap<K,V>(K,V); fn copy(_:HashMap<u8,u8>) {}}",
    "fn copy(){let _=std::collections::HashMap::with_hasher(Fixed);}",
];

pub(crate) const LAYOUT: [(&str, &str, &str); 5] = [
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

pub(crate) const FACTS: [(&str, &str, &str, bool); 2] = [
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

pub(crate) const HEX: [(&str, &str, &str, bool); 3] = [
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

pub(crate) const CONTEXTUAL: [(&str, &str, &str, &str); 2] = [
    (
        "home-literal",
        "/// Home.\npub fn home()->&'static str {\"token\"}",
        "fn copy(){let _=\"token\";}",
        "fn copy(){let _=\"different\";}",
    ),
    (
        "vocabulary-literal",
        "/// Vocabulary.\npub mod home {pub const NS:&str={ (\"urn:demo:\") };}",
        "fn copy(){let _=\"urn:demo:term\";}",
        "fn copy(){let _=\"urn:other:term\";}",
    ),
];

pub(crate) const HASHER_ID: &str = "std-default-hasher";
pub(crate) const SOLE_NAME_ID: &str = "sole-name";
pub(crate) const SOLE_NAME_POSITIVE: [&str; 2] =
    ["fn copy(){}", "#[cfg(test)] mod oracle {fn copy(){}}"];
pub(crate) const SOLE_NAME_NEGATIVE: [(&str, &str); 1] =
    [("/// Home.\npub fn home(){} pub fn copy(){}", "demo::copy")];
