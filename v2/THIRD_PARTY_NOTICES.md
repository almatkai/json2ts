# Credits and license inventory

Original Json2TS extension: gbarba; contributor yusifaliyevpro. Own Tezbar repository: almatkai. Both declare MIT. Original pin: `6734d85992290066fd31a7f5bee3cca3d3545a1a`; own pin: `8186f2229ec907bb3f705d1a641fd7159f0339f9`. The original icon is retained unchanged. The own root LICENSE contains literal escaped newlines; its fixture is byte-exact. `v2/LICENSE` presents those same MIT terms with readable line breaks for new Rust code.

The distribution aggregate SPDX expression is **MIT AND ISC AND Apache-2.0 AND LicenseRef-json-ts-Unspecified**. This deliberately records the unresolved json-ts license; it is not an assertion that all supplied code is MIT or that unspecified code has a redistribution grant.

| Material | Resolved version / pin | Actual license evidence |
|---|---|---|
| Own extension and original Raycast extension/icon | pins above | package manifests declare MIT; own LICENSE and Raycast root LICENSE retained |
| json-ts parser/transformer/printer and fixtures | 1.6.4; npm gitHead eb6d3acdae92f03a31f9f0494ac6fa464bdb1891 | **LicenseRef-json-ts-Unspecified**: published package has no license field or license file; that git tree has no license file. No MIT grant is inferred from its dependency or the extension's MIT declaration. Resolve this provenance gap before redistribution. |
| Embedded Lodash custom startCase/toLower build inside json-ts/_.js | 4.17.4 | embedded MIT notice; JS Foundation and contributors; based on Underscore 1.8.3, Jeremy Ashkenas, DocumentCloud and Investigative Reporters & Editors |
| needsquotes | 1.0.0 | package.json declares ISC; no separate license text or author supplied in that package; retained metadata records the evidence without inventing a holder |
| Immutable | 3.8.3 | MIT; retained LICENSE, copyright Facebook, Inc. |
| TypeScript used by json-ts and the TSX oracle | 2.9.2 | Apache-2.0; retained LICENSE.txt and ThirdPartyNoticeText.txt, including Microsoft's treatment of incorporated third party material |
| Beta common/SDK/NativeSessions/catalog/platform/renderer/schema fixtures | fd300e276356587726566026f072563eb9c34cb1 | Apache-2.0; retained beta LICENSE |
| Rust dependencies | Cargo.lock | their upstream SPDX alternatives apply; MIT chosen for MIT/Apache dual grants and Apache-2.0 for ryu-js's Apache/BSL alternatives |

`get-stdin`, `minimist` and `transducers-js` are locked json-ts package dependencies but are not imported by the command's converter entry point. The CLI/browser bundles that use them are not vendored. No JS package participates in product execution. Validation tool dependencies are pinned in `renderer/package-lock.json` and `scripts/schema-requirements.txt`; their installed copies live only in ignored validation directories.

`PROVENANCE.json` hashes every retained fixture, generated Unicode table, and exact icon. The missing json-ts license remains an explicit publication gap for parent review.
