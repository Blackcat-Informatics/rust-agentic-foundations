// SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
// SPDX-License-Identifier: MIT OR Apache-2.0 OR MulanPSL-2.0
"use strict";
(() => {
  const svg = document.querySelector("main svg");
  const status = document.querySelector("#status");
  const controls = document.querySelector("#controls");
  try {
    if (!svg) throw new Error("The native diagram is missing.");
    const metadata = svg.querySelector("metadata#purrdf-viz-export");
    if (!metadata) throw new Error("The native diagram export is missing.");
    const data = JSON.parse(metadata.textContent);
    if (!data.scene || !Array.isArray(data.element_index)) {
      throw new Error("The native diagram export has no scene or element index.");
    }
    const search = document.querySelector("#search");
    const items = document.querySelector("#items");
    const details = document.querySelector("#details");
    const kinds = document.querySelector("#relationships");
    const scene = new Map([...data.scene.nodes, ...data.scene.edges].map(row => [row.id, row]));
    const edgeById = new Map(data.scene.edges.map(edge => [edge.id, edge]));
    for (const edge of data.scene.edges) {
      if (edge.anchor) {
        scene.set(edge.anchor.id, edge.anchor);
        edgeById.set(edge.anchor.id, edge);
      }
    }
    const relationships = new Map(JSON.parse(document.querySelector("#relationship-index").textContent)
      .map(row => [row.id, row.relations]));
    for (const edge of data.scene.edges) {
      if (!relationships.get(edge.id)?.length) throw new Error(`Missing native relationship index: ${edge.id}`);
    }
    const entries = new Map();
    for (const entry of data.element_index) {
      const element = svg.querySelector(`#${CSS.escape(entry.element_id)}`);
      if (!element) throw new Error(`Missing native SVG element: ${entry.element_id}`);
      const group = element.closest("[data-scene-id]");
      if (group && !entries.has(entry.scene_id)) entries.set(entry.scene_id, { entry, group });
    }
    const initial = Array.from(svg.viewBox.baseVal ? [svg.viewBox.baseVal.x,
      svg.viewBox.baseVal.y, svg.viewBox.baseVal.width, svg.viewBox.baseVal.height] : []);
    if (initial.length !== 4 || initial.some(value => !Number.isFinite(value)) ||
        initial[2] <= 0 || initial[3] <= 0) throw new Error("Invalid native viewport.");
    const box = () => Array.from([svg.viewBox.baseVal.x, svg.viewBox.baseVal.y,
      svg.viewBox.baseVal.width, svg.viewBox.baseVal.height]);
    const setBox = values => svg.setAttribute("viewBox", values.join(" "));
    const zoom = factor => {
      const current = box();
      const width = Math.max(initial[2] / 20, Math.min(initial[2] * 10, current[2] / factor));
      const height = width * initial[3] / initial[2];
      setBox([current[0] + (current[2] - width) / 2,
        current[1] + (current[3] - height) / 2, width, height]);
    };
    document.querySelector("#zoom-in").onclick = () => zoom(1.25);
    document.querySelector("#zoom-out").onclick = () => zoom(0.8);
    document.querySelector("#reset").onclick = () => setBox(initial);
    svg.setAttribute("tabindex", "0");
    svg.setAttribute("aria-describedby", "navigation-help purrdf-desc");
    svg.addEventListener("keydown", event => {
      if (event.target !== svg) return;
      const current = box();
      const direction = { ArrowLeft: [-1, 0], ArrowRight: [1, 0],
        ArrowUp: [0, -1], ArrowDown: [0, 1] }[event.key];
      if (direction) {
        setBox([current[0] + direction[0] * current[2] / 10,
          current[1] + direction[1] * current[3] / 10, current[2], current[3]]);
      } else if (event.key === "+" || event.key === "=") zoom(1.25);
      else if (event.key === "-") zoom(0.8);
      else if (event.key === "Home") setBox(initial);
      else return;
      event.preventDefault();
    });
    let drag = null;
    svg.addEventListener("pointerdown", event => {
      if (event.button !== 0) return;
      drag = { x: event.clientX, y: event.clientY, box: box(),
        inverse: svg.getScreenCTM().inverse(), moved: false,
        selected: event.target.closest("[data-scene-id]")?.getAttribute("data-scene-id") };
      svg.setPointerCapture(event.pointerId);
    });
    svg.addEventListener("pointermove", event => {
      if (!drag) return;
      const origin = new DOMPoint(drag.x, drag.y).matrixTransform(drag.inverse);
      const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(drag.inverse);
      drag.moved ||= Math.abs(event.clientX - drag.x) + Math.abs(event.clientY - drag.y) > 3;
      if (drag.moved) setBox([drag.box[0] + origin.x - point.x,
        drag.box[1] + origin.y - point.y, drag.box[2], drag.box[3]]);
    });
    svg.addEventListener("pointerup", event => {
      const moved = drag?.moved;
      const selected = drag?.selected;
      drag = null;
      if (svg.hasPointerCapture(event.pointerId)) svg.releasePointerCapture(event.pointerId);
      if (!moved && selected) select(selected);
    });
    svg.addEventListener("pointercancel", () => { drag = null; });
    const hiddenKinds = new Set();
    for (const kind of [...new Set(Array.from(relationships.values()).flat())].sort()) {
      const label = document.createElement("label");
      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.checked = true;
      checkbox.onchange = () => {
        if (checkbox.checked) hiddenKinds.delete(kind); else hiddenKinds.add(kind);
        filter();
      };
      label.append(checkbox, document.createTextNode(kind));
      kinds.append(label);
    }
    const title = id => {
      const row = scene.get(id);
      return row?.label?.full_text || row?.label?.text || row?.accessibility?.title || id;
    };
    function select(id) {
      if (!entries.has(id)) return;
      for (const [key, { group }] of entries) group.classList.toggle("is-selected", key === id);
      items.value = id;
      const entry = entries.get(id).entry;
      const values = [];
      for (const binding of entry.bindings) {
        const collection = { term: "terms", statement: "statements", assertion: "assertions",
          relation: "relations", graph: "graphs", reference: "references" }[binding.kind];
        const value = collection && data.model[collection].find(row => row.id === binding.id);
        if (value) values.push(value);
      }
      details.textContent = JSON.stringify({ scene: scene.get(id), bindings: entry.bindings,
        values, provenance: { spec: data.spec_hash, model: data.model_hash, scene: data.scene_hash } }, null, 2);
      status.textContent = `Selected ${title(id)}`;
    }
    function filter() {
      const query = search.value.toLocaleLowerCase();
      const previous = items.value;
      items.replaceChildren();
      let count = 0;
      for (const [id, { group }] of entries) {
        const edge = edgeById.get(id);
        const hidden = Boolean(edge && relationships.get(edge.id).every(kind => hiddenKinds.has(kind)));
        group.classList.toggle("is-filtered", hidden);
        const match = title(id).toLocaleLowerCase().includes(query);
        group.classList.toggle("is-dimmed", !match);
        if (!hidden && match) {
          const option = document.createElement("option");
          option.value = id;
          option.textContent = title(id);
          option.title = title(id);
          items.append(option);
          count += 1;
        }
      }
      if (items.options.length) {
        items.value = Array.from(items.options).some(option => option.value === previous)
          ? previous : items.options[0].value;
        select(items.value);
      }
      else {
        for (const { group } of entries.values()) group.classList.remove("is-selected");
        details.textContent = "No element matches the current filters.";
      }
      status.textContent = `${count} matching elements. Diagram positions are unchanged.`;
    }
    search.oninput = filter;
    items.onchange = () => select(items.value);
    controls.hidden = false;
    filter();
  } catch (error) {
    status.textContent = `Interactive controls unavailable: ${error.message}`;
    status.setAttribute("role", "alert");
  }
})();
