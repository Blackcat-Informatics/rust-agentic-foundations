// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0

//! Documentation presentations over the unchanged native diagram export.

use super::{DiagramBundle, DiagramError};
use purrdf_lex::{
    allocation::{Memory, OrderedTable, Resident},
    json::{self, Object, Value, record::ToJson as _},
};
use purrdf_rdf::projections::{ProjectionError, escape_xml_attribute, escape_xml_text};
use purrdf_rdf::viz::{VizRelation, VizSceneEdgeKind, VizSemanticRef, VizTermValue};
use raf_source::RelPath;
use std::fmt::Write as _;

/// Assemble a complete local documentation bundle without writing to disk.
///
/// Every overview and detail contributes its unchanged native SVG, JSON and
/// readable text, a self-contained explorer, Markdown using a sibling SVG, and
/// a packaged static include fragment. All links between these generated files
/// are relative, so the directory may be served under any URL prefix or opened
/// locally. Callers publish these files together in an owned directory; this
/// operation neither modifies source nor invokes Cargo or Rustdoc.
///
/// # Errors
/// Refuses duplicate view names, more than 4096 files or 64 MiB of output, and
/// preserves native escaping, allocation and path errors. No partial file set
/// is returned.
pub fn build(bundle: &DiagramBundle) -> Result<Vec<(RelPath, String)>, DiagramError> {
    let mut files = OrderedTable::default();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut bytes = 0usize;
    let mut pending = vec![bundle];
    while let Some(current) = pending.pop() {
        let name = current.view().name();
        let overview = (name != bundle.view().name()).then_some(bundle.view().name());
        for (suffix, content) in [
            ("svg", current.svg().to_owned()),
            ("json", current.json().to_owned()),
            ("txt", current.text().to_owned()),
            (
                "html",
                explorer_page(current, overview).map_err(DiagramError::data)?,
            ),
            (
                "md",
                markdown_fragment(current).map_err(DiagramError::data)?,
            ),
            (
                "include.md",
                static_fragment(current).map_err(DiagramError::data)?,
            ),
        ] {
            let path = RelPath::new(&format!("{name}.{suffix}")).map_err(DiagramError::data)?;
            if files.get(&path).is_some() {
                return Err(refused(
                    "view names must be unique within a documentation bundle",
                ));
            }
            bytes = bytes.saturating_add(content.len());
            if files.len() >= 4096 || bytes > 64 * 1024 * 1024 {
                return Err(refused("documentation exceeds 4096 files or 64 MiB"));
            }
            files
                .insert(path, content, &mut memory)
                .map_err(DiagramError::data)?;
        }
        pending.extend(current.details().iter().rev());
    }
    Ok(std::iter::from_fn(|| files.pop_first()).collect())
}

fn refused(reason: &str) -> DiagramError {
    DiagramError::Refused {
        reason: reason.to_owned(),
    }
}

/// Plain Markdown publication with a sibling SVG and complete readable meaning.
///
/// The referenced `<view>.svg` and explorer are emitted by [`build`]. HTML
/// attributes and text use the native escaping rules; generated URLs consist
/// only of the already checked view slug and a fixed file extension.
///
/// # Errors
/// Preserves native escaping failures before returning any fragment.
pub fn markdown_fragment(bundle: &DiagramBundle) -> Result<String, ProjectionError> {
    let name = bundle.view().name();
    let mut fragment = image_fragment(bundle, &format!("{name}.svg"))?;
    write!(fragment, "[Explore this diagram]({name}.html)\n\n")
        .expect("writing to String cannot fail");
    Ok(fragment)
}

/// Build a self-contained accessible explorer around one native SVG document.
///
/// The original SVG contains the native model, layout, semantic bindings and
/// element index. Browser controls consume those values without deriving another
/// model or changing layout. Script-disabled readers retain the SVG, caption,
/// complete readable alternative and explicit links to generated detail views.
/// Each bundle and detail is published as its own `<name>.html` document.
///
/// # Errors
/// Returns the native XML error for text that cannot be represented safely in
/// HTML character data or attributes. No partially built page is returned.
pub fn explorer_html(bundle: &DiagramBundle) -> Result<String, ProjectionError> {
    explorer_page(bundle, None)
}

fn explorer_page(
    bundle: &DiagramBundle,
    overview: Option<&str>,
) -> Result<String, ProjectionError> {
    let title = escape_xml_text(bundle.view().name())?;
    let caption = escape_xml_text(bundle.caption())?;
    let text = escape_xml_text(bundle.text())?;
    let relationships = escape_xml_text(&relationship_index(bundle)?)?;
    let full_model_hash = escape_xml_attribute(bundle.full_model_hash().unwrap_or(""))?;
    let mut html = format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width,initial-scale=1\">\
         <title>{title}</title><meta name=\"raf-full-model-hash\" content=\"{full_model_hash}\"><style>{}</style></head><body>\
         <header><h1>{title}</h1><p>{caption}</p></header><main>\
         <section id=\"controls\" aria-label=\"Diagram controls\" hidden>\
         <div class=\"tools\"><label>Search <input id=\"search\" type=\"search\"></label>\
         <button id=\"zoom-in\" type=\"button\">Zoom in</button>\
         <button id=\"zoom-out\" type=\"button\">Zoom out</button>\
         <button id=\"reset\" type=\"button\">Reset view</button></div>\
         <fieldset id=\"relationships\"><legend>Visible relationships</legend></fieldset>\
         <p id=\"navigation-help\">Drag to pan. Focus the diagram and use arrow keys to pan, \
         plus or minus to zoom, and Home to reset. Use the element list to inspect an entity.</p>\
         <div class=\"panels\"><label>Elements <select id=\"items\" size=\"8\"></select></label>\
         <section aria-label=\"Selected element\"><pre id=\"details\">Select an element to inspect its native identities and provenance.</pre></section></div>\
         </section><p id=\"status\" role=\"status\" aria-live=\"polite\">Static diagram and complete readable relationships are available below.</p>\
         {}<pre id=\"relationship-index\" hidden>{relationships}</pre>\
         <details><summary>Complete readable relationships</summary><pre>{text}</pre></details>",
        include_str!("explorer.css"),
        bundle.svg(),
    );
    if let Some(overview) = overview {
        let href = escape_xml_attribute(&format!("{overview}.html"))?;
        write!(
            html,
            "<nav aria-label=\"Overview\"><a href=\"{href}\">Overview</a></nav>"
        )
        .expect("writing to String cannot fail");
    }
    if !bundle.details().is_empty() {
        html.push_str("<nav aria-label=\"Detail views\">");
        for detail in bundle.details() {
            let name = detail.view().name();
            let href = escape_xml_attribute(&format!("{name}.html"))?;
            let label = escape_xml_text(detail.caption())?;
            write!(html, "<a href=\"{href}\">{label}</a>").expect("writing to String cannot fail");
        }
        html.push_str("</nav>");
    }
    writeln!(
        html,
        "</main><script>{}</script></body></html>",
        include_str!("explorer.js")
    )
    .expect("writing to String cannot fail");
    Ok(html)
}

/// Render a packaged static fragment for Markdown and Rustdoc `include_str!`.
///
/// The SVG is a self-contained image document, so its native metadata and
/// accessibility identifiers cannot collide with another diagram on the page.
/// The full readable alternative remains ordinary HTML even without scripts.
/// A terminating blank line keeps following Markdown outside the HTML block.
/// Generation is explicit; consuming this fragment requires no build script,
/// documentation generator, browser code or external asset download.
///
/// # Errors
/// Preserves native text/attribute escaping failures before returning a fragment.
pub fn static_fragment(bundle: &DiagramBundle) -> Result<String, ProjectionError> {
    let svg = purrdf_xsd::canonical_base64(bundle.svg().as_bytes());
    image_fragment(bundle, &format!("data:image/svg+xml;base64,{svg}"))
}

fn image_fragment(bundle: &DiagramBundle, image: &str) -> Result<String, ProjectionError> {
    let image = escape_xml_attribute(image)?;
    let caption = escape_xml_text(bundle.caption())?;
    let alternative = escape_xml_attribute(bundle.caption())?;
    let text = escape_xml_text(bundle.text())?;
    Ok(format!(
        "<figure><img src=\"{image}\" alt=\"{alternative}\">\
         <figcaption>{caption}</figcaption></figure>\n\n\
         <details><summary>Complete readable relationships</summary><pre>{text}</pre></details>\n\n"
    ))
}

/// Resolve filter identities once in Rust, from the native statement layer.
fn relationship_index(bundle: &DiagramBundle) -> Result<String, ProjectionError> {
    let export = bundle.export();
    let mut resident = Resident;
    let mut memory = Memory::new(&mut resident);
    let mut terms = OrderedTable::default();
    let mut statements = OrderedTable::default();
    let mut relations = OrderedTable::default();
    for term in &export.model.terms {
        terms.insert(term.id.0.as_str(), term, &mut memory)?;
    }
    for statement in &export.model.statements {
        statements.insert(statement.id.0.as_str(), statement, &mut memory)?;
    }
    for relation in &export.model.relations {
        let (VizRelation::Reifies { id, .. } | VizRelation::Annotation { id, .. }) = relation;
        relations.insert(id.0.as_str(), relation, &mut memory)?;
    }
    let predicate = |id: &purrdf_rdf::viz::VizTermId| -> Result<String, ProjectionError> {
        let term = terms.get(&id.0.as_str()).ok_or_else(|| {
            ProjectionError::integrity(format!("missing native predicate {}", id.0))
        })?;
        match &term.value {
            VizTermValue::Iri { value } => Ok(value.clone()),
            _ => Err(ProjectionError::integrity("native predicate is not an IRI")),
        }
    };
    let mut rows = Vec::new();
    for edge in &export.scene.edges {
        let mut keys = Vec::new();
        for binding in &edge.bindings {
            let key = match binding {
                VizSemanticRef::Statement(id) => {
                    let statement = statements.get(&id.0.as_str()).ok_or_else(|| {
                        ProjectionError::integrity(format!("missing native statement {}", id.0))
                    })?;
                    Some(predicate(&statement.predicate)?)
                }
                VizSemanticRef::Relation(id) => {
                    let relation = relations.get(&id.0.as_str()).ok_or_else(|| {
                        ProjectionError::integrity(format!("missing native relation {}", id.0))
                    })?;
                    Some(match relation {
                        VizRelation::Reifies { .. } => purrdf_core::vocab::rdf::REIFIES.to_owned(),
                        VizRelation::Annotation { predicate: id, .. } => predicate(id)?,
                    })
                }
                _ => None,
            };
            if let Some(key) = key
                && !keys.contains(&key)
            {
                keys.push(key);
            }
        }
        if keys.is_empty() {
            if matches!(
                edge.kind,
                VizSceneEdgeKind::Assertion
                    | VizSceneEdgeKind::Annotation
                    | VizSceneEdgeKind::Reifies
            ) {
                return Err(ProjectionError::integrity(format!(
                    "native edge {} has no predicate binding",
                    edge.id
                )));
            }
            // Incidence/reference grammar is explicitly distinct from RDF predicates.
            keys.push(format!(
                "Native structural role: {}",
                json::write_compact(&edge.kind.to_json())
            ));
        }
        rows.push(Value::from(
            Object::new()
                .with("id", edge.id.as_str())
                .with("relations", keys),
        ));
    }
    if let Some(presented) = bundle.presented_export() {
        for interaction in &presented.geometry.interactions {
            let VizSemanticRef::Statement(id) = &interaction.binding else {
                return Err(ProjectionError::integrity(
                    "authored interaction needs its native structural statement",
                ));
            };
            let statement = statements.get(&id.0.as_str()).ok_or_else(|| {
                ProjectionError::integrity(format!("missing native interaction statement {}", id.0))
            })?;
            rows.push(Value::from(
                Object::new()
                    .with("id", interaction.id.as_str())
                    .with("relations", vec![predicate(&statement.predicate)?]),
            ));
        }
    }
    Ok(json::write_compact(&Value::from(rows)))
}
