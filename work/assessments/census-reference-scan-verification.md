---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-reference-scan-verification
subject: cap:census-reference-scan#verification
inputs:
  - kind: section
    name: cap:census-reference-scan#verification
    digest: e99bb04b2d15c613318d468ea11ab3dfa5d75a729760ae6eb742d71a9208f24c
  - kind: symbol
    name: raf_census::Gate::check_tree_with_references
    digest: c5087ae773dfc4b1246c1a9498aa45fbe698686a15afef9ecc6786beb2ff8ec8
  - kind: symbol
    name: raf_census::Gate::reference_roots
    digest: fb7552389a703dcbb4da65f26224530d5b67448fa60350c87a0f9f48e66a93b5
  - kind: symbol
    name: raf_census::ReferenceSource
    digest: 92b6b841671cc5df99751e19ba571a19300e7a0e51e29e51e3b54a078cb20d0f
  - kind: symbol
    name: raf_census::ReferenceWitness
    digest: b2c06ca3ba1b22ec1504d2ff6838f1b7052f833014f4fb8932dd2d1068bc8e10
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_reference_tree
    digest: a51a69b5cd64c86485c4805976f6227a456d43770b1e5d0933a8da21d178fbec
  - kind: symbol
    name: raf_census::gate::reference
    digest: db050836a9253a37c97ae7c23c3ac3dc4be615bf92daf12060a962e2026593f8
  - kind: symbol
    name: raf_census::report::reference
    digest: 7c7f1698a910ad8f061dc1511b9328b2811c884186cde6b1c4e0fcfad7a6860f
  - kind: symbol
    name: raf_source::ReferenceObservation
    digest: de8fa7c6e008ee0dbfe4be3e089b77fc3351f0411c6552fd89aeb60a0edacb6d
  - kind: symbol
    name: raf_source::ReferenceRequest
    digest: 866ef48e0fd38b0cf973154ac083374c203f96c6a26983e604aae4eaeb3f15c1
  - kind: symbol
    name: raf_source::workspace::locked::reference
    digest: 051ec14e99f33939d929703ad16b2f1e238a0a7510dd1ab2bf4a762c3e81e376
verdict: accepted
by: agent
---
