---
# SPDX-FileCopyrightText: 2026 Blackcat Informatics® Inc. <paudley@blackcatinformatics.ca>
# SPDX-License-Identifier: CC-BY-4.0
type: Assessment
id: assessment:census-reference-scan-verification
subject: cap:census-reference-scan#verification
inputs:
  - kind: section
    name: cap:census-reference-scan#verification
    digest: b44921c5b79050f622003e4f9a78a6c32b1c1cce15c8c140991e92122b5568de
  - kind: symbol
    name: raf_census::Gate::check_tree_with_references
    digest: c5087ae773dfc4b1246c1a9498aa45fbe698686a15afef9ecc6786beb2ff8ec8
  - kind: symbol
    name: raf_census::Gate::reference_roots
    digest: fb7552389a703dcbb4da65f26224530d5b67448fa60350c87a0f9f48e66a93b5
  - kind: symbol
    name: raf_census::ReferenceConstantRow
    digest: 553be507c6f6454a9bd7b7746d27c6f9f79e9461e1c2db7fc0c4c57582e02767
  - kind: symbol
    name: raf_census::ReferenceSource
    digest: 92b6b841671cc5df99751e19ba571a19300e7a0e51e29e51e3b54a078cb20d0f
  - kind: symbol
    name: raf_census::ReferenceWitness
    digest: b2c06ca3ba1b22ec1504d2ff6838f1b7052f833014f4fb8932dd2d1068bc8e10
  - kind: symbol
    name: raf_census::gate::Gate::evaluate_reference_tree
    digest: 6ea0f3f5f3f2160b3bd5123ca04307832186cf8357075a9bc1f4e95bcbd377bf
  - kind: symbol
    name: raf_census::gate::reference
    digest: 7a4054f078acd2b6d1c8767d280e381b906005e5145f6ab8118498c8deb32d6e
  - kind: symbol
    name: raf_census::report::reference
    digest: 66c9c9a3bc32f077b87ad07d491aa110ff3fa6d6c4cffb5ee613b1b163d3ba60
  - kind: symbol
    name: raf_source::ReferenceObservation
    digest: de8fa7c6e008ee0dbfe4be3e089b77fc3351f0411c6552fd89aeb60a0edacb6d
  - kind: symbol
    name: raf_source::ReferenceRequest
    digest: 866ef48e0fd38b0cf973154ac083374c203f96c6a26983e604aae4eaeb3f15c1
  - kind: symbol
    name: raf_source::workspace::locked::reference
    digest: 3b0aeb89d8dbfa2309a08ad29247b4b1f90cb1a740ef8b37de9a23b420ede5f9
verdict: accepted
by: agent
---
