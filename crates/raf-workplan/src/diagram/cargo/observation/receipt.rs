// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Closed native RDF transport of successful driver observations.

use super::super::{DiagramError, refused};
use crate::exec::{DEFAULT_OUTPUT_BOUND, Output};
use purrdf_lex::allocation::{Memory, Resident};
use raf_rdf::{Builder, Format, Graph, TermValue, Triple, vocab};

const NS: &str = "urn:raf:rustdoc-observation:";
pub(super) const MAX_ITEMS: usize = 4096;

#[derive(Debug, Clone)]
pub(super) struct Observation {
    pub(super) driver: String,
    pub(super) delegate: String,
    pub(super) directory: String,
    pub(super) arguments: Vec<String>,
    pub(super) query_arguments: Vec<String>,
    pub(super) cargo_environment: Vec<(String, String)>,
    pub(super) compiler_environment: Vec<(String, String)>,
    pub(super) configuration: Output,
    pub(super) html: Output,
    pub(super) assembly: Option<super::super::assembly::Preparation>,
}

fn put(
    builder: &mut Builder,
    subject: &str,
    key: &str,
    value: TermValue,
) -> Result<(), DiagramError> {
    builder
        .insert(Triple {
            subject: TermValue::iri(subject),
            predicate: TermValue::iri(format!("{NS}{key}")),
            object: value,
        })
        .map_err(DiagramError::data)
}

fn number(builder: &mut Builder, key: &str, value: usize) -> Result<(), DiagramError> {
    integer(builder, key, value as u128)
}

fn integer(builder: &mut Builder, key: &str, value: u128) -> Result<(), DiagramError> {
    put(
        builder,
        "",
        key,
        TermValue::typed_literal(
            purrdf_xsd::exact::Integer::from(value).canonical_lexical(),
            vocab::INTEGER,
        ),
    )
}

fn words(builder: &mut Builder, key: &str, values: &[String]) -> Result<(), DiagramError> {
    if values.len() > MAX_ITEMS {
        return Err(refused("Rustdoc receipt has too many arguments"));
    }
    number(builder, &format!("{key}Count"), values.len())?;
    for (index, value) in values.iter().enumerate() {
        put(
            builder,
            &format!("#{key}/{index}"),
            "value",
            TermValue::simple_literal(value),
        )?;
    }
    Ok(())
}

fn environment(
    builder: &mut Builder,
    key: &str,
    values: &[(String, String)],
) -> Result<(), DiagramError> {
    if values.len() > MAX_ITEMS {
        return Err(refused("Rustdoc receipt has too many environment entries"));
    }
    if values.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(refused(
            "Rustdoc environment names must be unique and sorted",
        ));
    }
    number(builder, &format!("{key}Count"), values.len())?;
    for (index, (name, value)) in values.iter().enumerate() {
        let subject = format!("#{key}/{index}");
        put(builder, &subject, "name", TermValue::simple_literal(name))?;
        put(builder, &subject, "value", TermValue::simple_literal(value))?;
    }
    Ok(())
}

pub(in crate::diagram::cargo) fn output(
    builder: &mut Builder,
    key: &str,
    value: &Output,
) -> Result<(), DiagramError> {
    if !value.success() {
        return Err(refused(
            "a successful Rustdoc receipt requires successful subprocess outcomes",
        ));
    }
    number(builder, &format!("{key}Code"), 0)?;
    integer(
        builder,
        &format!("{key}ElapsedSeconds"),
        u128::from(value.elapsed.as_secs()),
    )?;
    integer(
        builder,
        &format!("{key}ElapsedNanoseconds"),
        u128::from(value.elapsed.subsec_nanos()),
    )?;
    for (stream, bytes) in [("Stdout", &value.stdout), ("Stderr", &value.stderr)] {
        if bytes.len() > DEFAULT_OUTPUT_BOUND {
            return Err(refused("Rustdoc receipt stream exceeds its bound"));
        }
        let text = std::str::from_utf8(bytes).map_err(DiagramError::data)?;
        put(
            builder,
            "",
            &format!("{key}{stream}"),
            TermValue::simple_literal(text),
        )?;
    }
    Ok(())
}

impl Observation {
    pub(super) fn graph(&self) -> Result<Graph, DiagramError> {
        let mut builder = Builder::new();
        number(&mut builder, "form", 1)?;
        for (key, value) in [
            ("driver", &self.driver),
            ("delegate", &self.delegate),
            ("directory", &self.directory),
        ] {
            put(&mut builder, "", key, TermValue::simple_literal(value))?;
        }
        words(&mut builder, "argument", &self.arguments)?;
        words(&mut builder, "queryArgument", &self.query_arguments)?;
        environment(&mut builder, "cargoEnvironment", &self.cargo_environment)?;
        environment(
            &mut builder,
            "compilerEnvironment",
            &self.compiler_environment,
        )?;
        output(&mut builder, "configuration", &self.configuration)?;
        output(&mut builder, "html", &self.html)?;
        if let Some(assembly) = &self.assembly {
            put(
                &mut builder,
                "",
                "assembly",
                TermValue::simple_literal(assembly.encode()?),
            )?;
        }
        builder.finish().map_err(DiagramError::data)
    }

    pub(super) fn read(graph: &Graph) -> Result<Self, DiagramError> {
        if count(graph, "form", 1)? != 1 {
            return Err(refused("unsupported Rustdoc observation form"));
        }
        let observed = Self {
            driver: text(graph, "", "driver")?,
            delegate: text(graph, "", "delegate")?,
            directory: text(graph, "", "directory")?,
            arguments: read_words(graph, "argument")?,
            query_arguments: read_words(graph, "queryArgument")?,
            cargo_environment: read_environment(graph, "cargoEnvironment")?,
            compiler_environment: read_environment(graph, "compilerEnvironment")?,
            configuration: read_output(graph, "configuration")?,
            html: read_output(graph, "html")?,
            assembly: if graph
                .get("")
                .is_some_and(|root| root.values(&format!("{NS}assembly")).next().is_some())
            {
                Some(super::super::assembly::Preparation::read(&text(
                    graph, "", "assembly",
                )?)?)
            } else {
                None
            },
        };
        if observed.graph()?.snapshot().map_err(DiagramError::data)?
            != graph.snapshot().map_err(DiagramError::data)?
        {
            return Err(refused(
                "Rustdoc receipt contains unknown, orphaned or inconsistent assertions",
            ));
        }
        Ok(observed)
    }
}

fn value<'a>(
    graph: &'a Graph,
    subject: &str,
    key: &str,
) -> Result<raf_rdf::Value<'a>, DiagramError> {
    let node = graph
        .get(subject)
        .ok_or_else(|| refused("required Rustdoc receipt subject is absent"))?;
    let mut values = node.values(&format!("{NS}{key}"));
    let value = values
        .next()
        .ok_or_else(|| refused(&format!("missing Rustdoc receipt field {subject}:{key}")))?;
    if values.next().is_some() {
        return Err(refused("Rustdoc receipt scalar field is repeated"));
    }
    Ok(value)
}

fn text(graph: &Graph, subject: &str, key: &str) -> Result<String, DiagramError> {
    let (text, datatype) = value(graph, subject, key)?
        .literal()
        .ok_or_else(|| refused("Rustdoc receipt field must be a string"))?;
    if datatype != vocab::STRING || text.len() > DEFAULT_OUTPUT_BOUND {
        return Err(refused(
            "Rustdoc receipt string has an invalid datatype or exceeds its bound",
        ));
    }
    let mut resident = Resident;
    Memory::new(&mut resident)
        .string(text)
        .map_err(DiagramError::data)
}

fn unsigned(graph: &Graph, key: &str) -> Result<u128, DiagramError> {
    let (lexical, datatype) = value(graph, "", key)?
        .literal()
        .ok_or_else(|| refused("Rustdoc receipt count must be an integer"))?;
    if datatype != vocab::INTEGER || lexical.len() > 39 {
        return Err(refused("invalid Rustdoc receipt count"));
    }
    let integer = purrdf_xsd::numeric::parse_integer(lexical).map_err(DiagramError::data)?;
    if integer.is_negative() || integer.canonical_lexical() != lexical {
        return Err(refused(
            "Rustdoc receipt count must be canonical and nonnegative",
        ));
    }
    integer
        .into_bigint()
        .unsigned_abs_u128()
        .ok_or_else(|| refused("Rustdoc receipt integer exceeds its bound"))
}

fn count(graph: &Graph, key: &str, maximum: usize) -> Result<usize, DiagramError> {
    let count = usize::try_from(unsigned(graph, key)?)
        .ok()
        .filter(|value| *value <= maximum);
    count.ok_or_else(|| refused("Rustdoc receipt count exceeds its bound"))
}

fn read_words(graph: &Graph, key: &str) -> Result<Vec<String>, DiagramError> {
    let mut values = Vec::new();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for index in 0..count(graph, &format!("{key}Count"), MAX_ITEMS)? {
        memory
            .push(
                &mut values,
                text(graph, &format!("#{key}/{index}"), "value")?,
            )
            .map_err(DiagramError::data)?;
    }
    Ok(values)
}

fn read_environment(graph: &Graph, key: &str) -> Result<Vec<(String, String)>, DiagramError> {
    let mut values = Vec::new();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    for index in 0..count(graph, &format!("{key}Count"), MAX_ITEMS)? {
        let subject = format!("#{key}/{index}");
        memory
            .push(
                &mut values,
                (
                    text(graph, &subject, "name")?,
                    text(graph, &subject, "value")?,
                ),
            )
            .map_err(DiagramError::data)?;
    }
    Ok(values)
}

pub(in crate::diagram::cargo) fn read_output(
    graph: &Graph,
    key: &str,
) -> Result<Output, DiagramError> {
    count(graph, &format!("{key}Code"), 0)?;
    let stdout = text(graph, "", &format!("{key}Stdout"))?;
    let stderr = text(graph, "", &format!("{key}Stderr"))?;
    let seconds = u64::try_from(unsigned(graph, &format!("{key}ElapsedSeconds"))?)
        .map_err(DiagramError::data)?;
    let nanos = u32::try_from(unsigned(graph, &format!("{key}ElapsedNanoseconds"))?)
        .map_err(DiagramError::data)?;
    if nanos >= 1_000_000_000 {
        return Err(refused(
            "Rustdoc receipt nanosecond component exceeds its bound",
        ));
    }
    let mut output = Output::new(Some(0), stdout.into_bytes(), stderr.into_bytes());
    output.elapsed = std::time::Duration::new(seconds, nanos);
    Ok(output)
}

pub(super) fn encode(observation: &Observation) -> Result<String, DiagramError> {
    observation
        .graph()?
        .to_text(Format::NTriples)
        .map_err(DiagramError::data)
}
