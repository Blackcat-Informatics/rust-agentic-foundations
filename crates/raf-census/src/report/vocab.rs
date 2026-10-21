// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! The closed RDF 1.2 vocabulary of census reports.

/// Layout version of the vocabulary, shape and canonical spelling.
pub const REPORT_FORM: u32 = 1;
/// Census class and property namespace.
pub const CENSUS: &str = "https://blackcatinformatics.ca/raf/census#";
/// Named hexadecimal table namespace.
pub const TABLE: &str = "https://blackcatinformatics.ca/raf/census/table/";
/// Snapshot base namespace, followed by a BLAKE3-256 hexadecimal digest.
pub const BASE: &str = "https://blackcatinformatics.ca/.well-known/genid/raf-census/";
/// RDF structural vocabulary.
pub const RDF: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#";
pub(crate) const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// The version-qualified namespace of a body or shim fingerprint.
#[must_use]
pub fn fingerprint_namespace(normal_form: u32, kind: crate::GroupKind) -> String {
    let kind = match kind {
        crate::GroupKind::Body => "body",
        crate::GroupKind::Shim => "shim",
    };
    format!("https://blackcatinformatics.ca/raf/census/nf/{normal_form}/{kind}/")
}

/// One property in a report class's closed shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    /// Report kind: baseline, index, both or debt.
    pub report: &'static str,
    /// Class local name.
    pub class: &'static str,
    /// Property local name (reifies is in the RDF namespace).
    pub property: &'static str,
    /// Minimum occurrences.
    pub min: usize,
    /// Maximum occurrences; `usize::MAX` means unbounded.
    pub max: usize,
    /// Object type and range.
    pub object: &'static str,
}
macro_rules! shapes {
    ($(($r:literal,$c:literal,$p:literal,$min:expr,$max:expr,$o:literal)),* $(,)?) => {
        /// All permitted report properties; rdf:type is required once per node.
        pub const SHAPES: &[Shape] = &[$(Shape {report:$r,class:$c,property:$p,min:$min,max:$max,object:$o}),*];
    };
}
shapes! {
    ("baseline","Baseline","minTokens",1,1,"positive integer"),
    ("baseline","Baseline","normalForm",1,1,"u32"),
    ("baseline","Baseline","reportForm",1,1,"report version"),
    ("baseline","Baseline","unitCount",1,1,"integer"),
    ("index","Index","normalForm",1,1,"u32"),
    ("index","Index","reportForm",1,1,"report version"),
    ("both","BodyGroup","fingerprint",1,1,"body fingerprint"),
    ("both","BodyGroup","member",2,usize::MAX,"unit"),
    ("both","BodyGroup","tokens",1,1,"integer"),
    ("both","ShimGroup","fingerprint",1,1,"shim fingerprint"),
    ("both","ShimGroup","member",2,usize::MAX,"unit"),
    ("both","ShimGroup","tokens",1,1,"integer"),
    ("baseline","SharedConstant","value",1,1,"large integer"),
    ("baseline","SharedConstant","packageCount",1,1,"package count"),
    ("baseline","SharedConstant","member",2,usize::MAX,"unit"),
    ("both","Unit","symbol",1,1,"string"),
    ("both","Unit","file",1,1,"string"),
    ("both","Unit","line",1,1,"positive integer"),
    ("both","Unit","startByte",1,1,"integer"),
    ("baseline","Unit","holds",0,2,"table"),
    ("index","Unit","package",1,1,"string"),
    ("index","Unit","shipping",1,1,"boolean"),
    ("index","Unit","tokens",1,1,"integer"),
    ("index","Unit","body",0,1,"body fingerprint"),
    ("index","Unit","shim",0,1,"shim fingerprint"),
    ("index","Job","jobId",1,1,"string"),
    ("index","Job","homeSymbol",1,1,"string"),
    ("index","Job","copies",1,1,"integer"),
    ("index","Job","match",0,usize::MAX,"unit"),
    ("index","Match","reifies",1,1,"triple term"),
    ("index","Match","matchClass",1,1,"match class"),
    ("debt","DebtRecord","normalForm",1,1,"u32"),
    ("debt","DebtRecord","reportForm",1,1,"report version"),
    ("debt","BodyDebt","memberSymbol",2,usize::MAX,"string"),
    ("debt","ShimDebt","memberSymbol",2,usize::MAX,"string"),
    ("debt","JobDebt","jobId",1,1,"string"),
    ("debt","JobDebt","copies",1,1,"positive integer"),
}
