feat(workplan): let a capability require measured evidence with units and conditions

An evidence record may state optional units (one unit for each measured value) and
conditions (how and where the values were taken). A record that states either key must
give every value a unit, and must state conditions when it holds values and units.

A capability may state measured_evidence_required: true. The check then refuses the
capability, and so does change complete, unless some evidence record for it carries
values, a unit for each value, and conditions. The finding names the capability.

A capability that does not state the key is judged exactly as before, and a values-only
record with no units and no conditions stays valid.

All new keys are optional. Existing documents are unaffected. An older binary refuses a
document that states the new keys, because the key sets are closed.

Defect-Class: none
