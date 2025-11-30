# 📜 Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [1.1.0] - 2025-11-30
### 📝 Other Changes
- Updated version to v1.1.0 [bed9345](https://github.com/emirror-de/axum-gate/commit/bed9345b9fee70227ccc53efe2c854b64c1e468f)

- Moved features for sea-orm to dev-dependencies [4a7082f](https://github.com/emirror-de/axum-gate/commit/4a7082fa45dc3da459f47f342d338bdf81cb5381)

- Added hints for storage-seaorm-v2 feature [cf9fe01](https://github.com/emirror-de/axum-gate/commit/cf9fe0131d322aa87ff1813741958d4a00c31d5f)

- Removed test for aws_lc_rs because it requires a storage to be enabled [201032f](https://github.com/emirror-de/axum-gate/commit/201032facb002b7d5c67985b6cbf2fe77f368551)

- Removed --workspace for CI calls with feature storage-seaorm-v2 as well [8a333b1](https://github.com/emirror-de/axum-gate/commit/8a333b1e2f54c8da3342299283a3addd5b5b3f5f)

- Removed --workspace from CI calls for storage-seaorm-v2 because examples are using storage-seaorm [c779de5](https://github.com/emirror-de/axum-gate/commit/c779de565007f7a0abb969e2202bf0a969619ac1)

- Updated CI to integrate storage-seaorm-v2 feature [cc8bf36](https://github.com/emirror-de/axum-gate/commit/cc8bf368f282e57129406007cbba4b0a8fcfd3ae)

- Re-arranged use statements in tests/timing_repositories.rs [58b6637](https://github.com/emirror-de/axum-gate/commit/58b66371c7e6e2fb03ba14ea2d8443bda3dccc07)

- Add SeaORM v2 and sort dependencies (#10) [a83ba6f](https://github.com/emirror-de/axum-gate/commit/a83ba6f82ae811e5c00a6d992b3bae1d81293f01)



## [1.0.2] - 2025-11-24
### 📝 Other Changes
- Version bump to v1.0.2 [8250dd7](https://github.com/emirror-de/axum-gate/commit/8250dd7022c1ba7b06e445a249aae175c7c2075b)

- Added missing feature to CI [f2c6d87](https://github.com/emirror-de/axum-gate/commit/f2c6d8725b017ff0c24a99425da3862478764241)

- Added release checks for CI [25bd543](https://github.com/emirror-de/axum-gate/commit/25bd543724645dadf39010f1d976ae9d2702c3c8)

- Fixed release compilation by adding missing attributes [bb5572c](https://github.com/emirror-de/axum-gate/commit/bb5572c5f7510ea56878c796e4298a0ab97f89d7)

- Credentials are now available when default-features = false [0087a6f](https://github.com/emirror-de/axum-gate/commit/0087a6f902aed731c4390ad87b7eb327a7f357b8)

- Removed warning of unused function when default-features = false [fe46d24](https://github.com/emirror-de/axum-gate/commit/fe46d241f4043bea75b746656bcc89b49d9a1164)

- Added rt-multi-thread feature to tokio on server feature [111c683](https://github.com/emirror-de/axum-gate/commit/111c68346162e93379dfb0e77f6b67b800d7fbac)

- Added serde_json to server feature [709d85d](https://github.com/emirror-de/axum-gate/commit/709d85d6ca116babe2e554b6ecc21fd47f5ef3af)

- Added short paragraph about WASM/client usage to crate documentation [1a735b4](https://github.com/emirror-de/axum-gate/commit/1a735b4aba0933b82b9af4f6d87b304426d7e1d9)

- Added missing server feature to CI [8b44ea1](https://github.com/emirror-de/axum-gate/commit/8b44ea18c93e8ecae9be2fa0680a1ee6f3534f93)

- Enabled usage in wasm32-unknown-unknown target for client side usage [d5cccc1](https://github.com/emirror-de/axum-gate/commit/d5cccc1eb40d8f316c84e644de666b983ff0b2d6)

- Now using workspace dependencies in axum-gate [6eb1a0a](https://github.com/emirror-de/axum-gate/commit/6eb1a0a06e01c37200774a074aec5aab367587cf)

- Futures from repository traits are now Send + Sync as well [b291c22](https://github.com/emirror-de/axum-gate/commit/b291c22f86ec02b4ebcfcd26d4ee17678cfaa61d)

- AccountRepository and SecretRepository now has Self: Send + Sync bounds [d4bfb01](https://github.com/emirror-de/axum-gate/commit/d4bfb013467294919c654c9afc9c6535d1cde433)

- Hardcoded changelog commit id links [e5bfb09](https://github.com/emirror-de/axum-gate/commit/e5bfb092569ef27f2a6f6427a80acf4d7ab22c9c)

- Updated cliff.toml and CHANGELOG.md [dc80252](https://github.com/emirror-de/axum-gate/commit/dc80252a5e975a1c8be68e36eb1c6a6c52dc464d)

- Removed duplicated square brackets in changelog generation [25f3deb](https://github.com/emirror-de/axum-gate/commit/25f3deb9f866b5b58d4fd54d9cd459f32e769cac)



## [1.0.1] - 2025-11-10
### 🐞 Bug Fixes
- Update_account and update_secret panic in seaorm (#8) * fix: update_account and update_secret panic in seaorm * fix: add unique constraints to account_id in Account and Credentials models * fix: qurey the primary key from database,then update secret or account --------- Co-authored-by: Yale <yale.yu@kuka.com> [dc39e3d](https://github.com/emirror-de/axum-gate/commit/dc39e3dcde18acc49fcd0aee4ad3adc9983ad9b9)



### 📝 Other Changes
- Updated CHANGELOG.md [a9e0bde](https://github.com/emirror-de/axum-gate/commit/a9e0bde4f2eddc260b0c83b7a622acd4e97a1ea5)

- Updated to v1.0.1 [172d82c](https://github.com/emirror-de/axum-gate/commit/172d82c5e2b5240105ab6c9457e5893263c23137)

- Docs.rs now excluding aws_lc_rs feature during build [5dda698](https://github.com/emirror-de/axum-gate/commit/5dda6983df18a1522399b05bc07b39c787166506)

- Updated Cargo.lock [72fbecc](https://github.com/emirror-de/axum-gate/commit/72fbecc0c96e90c921f983cb8b83acbaecbb6c18)

- Added basic update secret and account to sea-orm example [3aafc61](https://github.com/emirror-de/axum-gate/commit/3aafc613b5b510204560fe71ce18766bb5e978e4)

- Update feature arguments in CI workflow [e5dae94](https://github.com/emirror-de/axum-gate/commit/e5dae942b11738e9ee705f7eb47692987dd4b1ff)

- Updated CI semver checks to pass with selected features [7a27c62](https://github.com/emirror-de/axum-gate/commit/7a27c62aa612014390454728e33cd19250e0bf4c)

- Removed unnecessary comment form Cargo.toml [21576f0](https://github.com/emirror-de/axum-gate/commit/21576f091ae2189867411ea76338c1195a925e43)

- Trying to fix CI [2c32e12](https://github.com/emirror-de/axum-gate/commit/2c32e1218819a47294ffc8f20ec3daf9f9bc6acd)

- Tests in CI [13bfa2c](https://github.com/emirror-de/axum-gate/commit/13bfa2c7fa84edf294cd175029b043ac96145348)

- Added missing features for clippy to pass integration tests [ca16a08](https://github.com/emirror-de/axum-gate/commit/ca16a086654d52a71412caf78dc63348faedb8a6)

- Clippy call in CI should not check the whole workspace when using aws_lc_rs feature [93edee0](https://github.com/emirror-de/axum-gate/commit/93edee0f6da93611213351133d88a548eb3bc45b)

- Updated pipeline to run for default and aws_lc_rs encryption [0fe3b4d](https://github.com/emirror-de/axum-gate/commit/0fe3b4d4894336d67b0327a4f13175c84c1f9319)

- Merge commit [48884c8](https://github.com/emirror-de/axum-gate/commit/48884c822fb5081dec92a75db06c148e036a672a)

- OAuth2 github example now uses axum from workspace [67c4097](https://github.com/emirror-de/axum-gate/commit/67c4097f9c55f9f98aa181f1e9b100d5badf3f93)

- Support rust_crypto encryption algorithms (#5) [ee5bc7c](https://github.com/emirror-de/axum-gate/commit/ee5bc7c8997bddc1e020adb83ba8a56fc5bda935)

- OAuth2 github example now uses axum from workspace [58f51da](https://github.com/emirror-de/axum-gate/commit/58f51da827c74ba6b878e596e7a55f5ad4bcbb7b)

- Updated MSRV to 1.88 [3d7111c](https://github.com/emirror-de/axum-gate/commit/3d7111c1bac7a90f64eccb0d27ad73f483fdf065)

- Updated misleading documentation about key rotation [6327909](https://github.com/emirror-de/axum-gate/commit/6327909f70774c9f02b0bbdb489790bd3d916cda)

- Updated MSRV CI workflows [ebbb895](https://github.com/emirror-de/axum-gate/commit/ebbb8955575a9af62083b7b1b5551ccd4c7ea798)



## [1.0.0] - 2025-10-31
### 📝 Other Changes
- Updated CHANGELOG.md [b7ff32a](https://github.com/emirror-de/axum-gate/commit/b7ff32a604b37a24b1fc8902265d6985add3a3d4)

- Git cliff no longer ignores pre-release tags [531304b](https://github.com/emirror-de/axum-gate/commit/531304be9d61302bfa46b89daf60fdc7e5c23bc5)

- Updated Cargo.lock [373c604](https://github.com/emirror-de/axum-gate/commit/373c60413e9454d2763102adbde66409eba1728b)

- Bumped to v1.0.0 [51fada8](https://github.com/emirror-de/axum-gate/commit/51fada8d0fb13fb505f2642b33e7a06d7c4a7360)

- Increased MSRV to 1.88 to be able to upgrade home dependency [ab60a36](https://github.com/emirror-de/axum-gate/commit/ab60a36bd6cec0234c8da424f5aa39646ec1acee)

- Update axum-extra to 0.12 [8a496ca](https://github.com/emirror-de/axum-gate/commit/8a496caa071de0d0a0700cc8ff85a06866a2d27f)

- Updated Cargo.lock [1c14a51](https://github.com/emirror-de/axum-gate/commit/1c14a514c29733705074f3054954ec67ecde17da)

- Revert "feat: Added semver checks to CI" This reverts commit 1a7e94f1272abe6f45f58fd9b185d139f01bc246. [9c495d2](https://github.com/emirror-de/axum-gate/commit/9c495d2d725ad006d91c15d169ba4eace96a8808)

- Added semver checks to CI [1a7e94f](https://github.com/emirror-de/axum-gate/commit/1a7e94f1272abe6f45f58fd9b185d139f01bc246)

- Removed .rules and added to .gitignore as this file does not add value to the crate [149e9a3](https://github.com/emirror-de/axum-gate/commit/149e9a32798566c5eaada2f467dcbbf35cd4c7a0)

- Crate level documentation improvements [7a6a525](https://github.com/emirror-de/axum-gate/commit/7a6a5258a65238faf95bf47c5e675c3e27bb3a80)

- Now using subtle >= 2.6.1 as 2.6.0 is yanked [d1d41d5](https://github.com/emirror-de/axum-gate/commit/d1d41d57ad9d421ab06c24f1dc7602e3a0a81ec9)

- OAuth2Gate no longer uses anyhow::Result [3d1e7b2](https://github.com/emirror-de/axum-gate/commit/3d1e7b2a0d482d92ea4b8db4d9f790605f561f1f)

- Merged nightly branch [e949763](https://github.com/emirror-de/axum-gate/commit/e9497634dbcd4bf2764da7209ad18a0fa889f0be)

- Merged nightly branch [03d8c1e](https://github.com/emirror-de/axum-gate/commit/03d8c1ea5f675f3021f1196ab4c81eae04731a25)

- Update GitHub Sponsors username in FUNDING.yml [ef68d49](https://github.com/emirror-de/axum-gate/commit/ef68d49152df14e7c315224d917822b9d6bc74fb)

- Added license notice for surrealdb [7e6c733](https://github.com/emirror-de/axum-gate/commit/7e6c7336586bb669aac0b15ed9d67f0aa3c4cd13)

- Added "Apache-2.0 WITH LLVM-exception" to deny.toml [9d8a930](https://github.com/emirror-de/axum-gate/commit/9d8a9301301d38f5b7b7254176790b366ea97782)

- GitHub Bug issue template [69610bd](https://github.com/emirror-de/axum-gate/commit/69610bd4370785ba51417c3d861ac6664905a5fb)

- Trying to fix those github issue templates [7ea12c5](https://github.com/emirror-de/axum-gate/commit/7ea12c5b3e6d15e4456b0e346e8fb8f587c39f09)

- Bug issue template [7a0885f](https://github.com/emirror-de/axum-gate/commit/7a0885f28ad029ab15723b31d1f86cee392e939c)

- Updated issue templates [bc61d87](https://github.com/emirror-de/axum-gate/commit/bc61d8726d1c29ffa1e16eeb15566754b21b18e1)

- Added missing jwt_codec variable in OAuth2 doc-test [8111879](https://github.com/emirror-de/axum-gate/commit/81118798ed1e0cc0017059e4ad30233baa044198)

- Applied clippy hints [9f2f625](https://github.com/emirror-de/axum-gate/commit/9f2f625355456cb59cd64371072843f335f4b559)

- Merge branch 'feat/oauth2' into 'nightly' Feat/oauth2 See merge request lprobst/axum-gate!6 [b4da1c8](https://github.com/emirror-de/axum-gate/commit/b4da1c8ad68bb88579f992ff1f1af87d18420e01)

- Feat/oauth2 [a38df7c](https://github.com/emirror-de/axum-gate/commit/a38df7c7626fb6cb25e10d171cd110d34e189993)

- Added issue template config [930749e](https://github.com/emirror-de/axum-gate/commit/930749e4cad76a2417fe700a918b7013d1b6a93f)

- Added github issue templates [dac7e73](https://github.com/emirror-de/axum-gate/commit/dac7e73a02536f7ca732e61fe9443c75cceec785)

- Aligned BearerGate implementation to only insert Option<_> extensions [39d00e3](https://github.com/emirror-de/axum-gate/commit/39d00e3fd373d1dcb346b71d9873d88dc8337c95)

- Added Eq trait bound to Gate::cookie for clarity during build time [9ee26ec](https://github.com/emirror-de/axum-gate/commit/9ee26ec9e088bf312df541728a9971abd43d424c)

- Updated cookie_template documentation [5786ca0](https://github.com/emirror-de/axum-gate/commit/5786ca0f09650454b8adc2eaf62bf5d651c0c052)

- :allow_anonymous_with_optional_user now only inserts Option<_> extensions [c14ca3d](https://github.com/emirror-de/axum-gate/commit/c14ca3db9536e6d67e2199315f3adde7b8d99381)

- Updated bearer gate implementation [abf03fd](https://github.com/emirror-de/axum-gate/commit/abf03fd01a6562cab3c2a1c0ca6495b13827b30a)

- Added all-features = true to docs.rs build [f45bd84](https://github.com/emirror-de/axum-gate/commit/f45bd849f97c21195e9e1bbb0abfe1aa7917c0c4)

- Updated .rules [33e4f66](https://github.com/emirror-de/axum-gate/commit/33e4f66f7d3818f60f96af0dcb0e5c24c9bb68f5)

- Updated .rules [dafda2d](https://github.com/emirror-de/axum-gate/commit/dafda2d2e6cae23f1b8cd4a0d764d29aaadeabb1)

- BearerGate doc tests [0266b23](https://github.com/emirror-de/axum-gate/commit/0266b2346d38e75457ab5c42f251f9eed6afc76b)

- Merge branch 'docs/cookie-gate' into 'nightly' docs: Updated documentation for CookieGate See merge request lprobst/axum-gate!7 [0f8cce2](https://github.com/emirror-de/axum-gate/commit/0f8cce28e2b587767425bb43b5914685d400f520)

- Updated documentation for CookieGate [6a14edd](https://github.com/emirror-de/axum-gate/commit/6a14edd1028423bcb8bb26fba33e33277111d251)

- Removed unused dependencies [809fa37](https://github.com/emirror-de/axum-gate/commit/809fa3779de622e7fb90d3c28e365983f6a3fe16)

- Added cargo-machete to flake.nix [40dd708](https://github.com/emirror-de/axum-gate/commit/40dd7083ab7b115fb1f30caacc12554e74b86765)

- Added Troubleshooting section to CookieGate [d52168e](https://github.com/emirror-de/axum-gate/commit/d52168e5408bbf205fcfd769484cf4b638b698d2)

- Disabled coverage report in gitlab ci [2050a82](https://github.com/emirror-de/axum-gate/commit/2050a827c2f90e8ac6a21af2ff1a20b116dd098e)



## [1.0.0-rc.0] - 2025-10-25
### 🏗 Refactoring
- **💥 BREAKING CHANGE:** Introduced hashing module [0b5cb49](https://github.com/emirror-de/axum-gate/commit/0b5cb496001fab85270bf8447a890780e4e2c91a)

- **💥 BREAKING CHANGE:** Renamed BasicGroup to Group [7416517](https://github.com/emirror-de/axum-gate/commit/741651789887c732107cb267f8f7b259c05a0d64)



### 🐞 Bug Fixes
- Fix and simplify doctests; correct imports and remove problematic permission validation section [10706e1](https://github.com/emirror-de/axum-gate/commit/10706e153399578b2d7975826c4dd79d20cb5a27)



### 📄 Documentation
- Add missing documentation for prometheus_metrics module - Add comprehensive module-level documentation with usage examples - Document all enums (JwtInvalidKind, AccountDeleteOutcome, SecretRestored, AccountInsertOutcome) and their variants - Document Metrics struct and all its fields - Document metrics() function with return value explanations - Resolves all missing documentation errors (22 items) in audit.rs - Maintains compliance with #![deny(missing_docs)] lint [cb4037c](https://github.com/emirror-de/axum-gate/commit/cb4037cd167f69833ec7e227390ade47aa02087e)



### 📝 Other Changes
- Updated CHANGELOG.md [65e1c75](https://github.com/emirror-de/axum-gate/commit/65e1c755bdd981953a85e551dc02095bf2592b7f)

- Updated Cargo.lock [568cbd5](https://github.com/emirror-de/axum-gate/commit/568cbd5bf9b61d8bbe30d9be4abbffba7a68bb66)

- Re-arranged use statements in permission-registry example [643656d](https://github.com/emirror-de/axum-gate/commit/643656d002f21668e9c7c41403e4d05e3ac7072b)

- Updated CHANGELOG.md [fd38e04](https://github.com/emirror-de/axum-gate/commit/fd38e041db45f98d55fb53384977758fb0000937)

- Updated Rust version to 2024 in permission-registry example [4f0b4c4](https://github.com/emirror-de/axum-gate/commit/4f0b4c4d8c41fdc49bfb8cc89bf3fd19e3ef116d)

- Added snippets about prelude and re-exported crates [69c8f1e](https://github.com/emirror-de/axum-gate/commit/69c8f1e0c2e4f3c9518c1e26e78e5d876b293976)

- Allowed unwrap and expect at certain test modules [5e1ac35](https://github.com/emirror-de/axum-gate/commit/5e1ac35b567d89aaa88b08aac0a2825321247e6a)

- Examples [29d88f1](https://github.com/emirror-de/axum-gate/commit/29d88f1d70acf1af4308c61a05a37c7f2701265f)

- Updated CHANGELOG.md [a9f4873](https://github.com/emirror-de/axum-gate/commit/a9f48735fc1b2e27562913e162d48bb38045c501)

- Updated README.md [c96b048](https://github.com/emirror-de/axum-gate/commit/c96b0489e76e51574e419ea5af4cd7cde0a2d05f)

- Added unwrap to convenient login check example [47faa87](https://github.com/emirror-de/axum-gate/commit/47faa87474c37fa17c1946f1f11e27b0e3ee4707)

- Extended prelude by RegisteredClaims and Permissions [2e26d5e](https://github.com/emirror-de/axum-gate/commit/2e26d5e7396bd605a012b45a341389de324e628d)

- Moved prelude to separate file [2745a14](https://github.com/emirror-de/axum-gate/commit/2745a1487007862ddfcf24523407715225efaf41)

- Now using cookie library directly instead of re-export [5ac5c80](https://github.com/emirror-de/axum-gate/commit/5ac5c80a57457b5fcfa5ec8bb2147262067d3d63)

- Removed serde_json re-export [0b6b666](https://github.com/emirror-de/axum-gate/commit/0b6b666d020c574957f03b729a33453854c7092e)

- Updated surrealdb example imports [364fa7c](https://github.com/emirror-de/axum-gate/commit/364fa7cba80374d086a6261d14f957d4ec88330a)

- Updated imports in simple-usage example [8f194bd](https://github.com/emirror-de/axum-gate/commit/8f194bdc2dffe98af6131480f314745ed4786369)

- Updated sea-orm example [478ee83](https://github.com/emirror-de/axum-gate/commit/478ee83d588baeccebcec9f3dfacd6d59a3008bb)

- Updated rate-limiting example [80cb059](https://github.com/emirror-de/axum-gate/commit/80cb05977708576fbc5538df828818cc70b96586)

- Updated permission-registry example [068f24f](https://github.com/emirror-de/axum-gate/commit/068f24f772799c49324b7f025c9ed84ca17c90c7)

- Updated distributed example [ec5f393](https://github.com/emirror-de/axum-gate/commit/ec5f393a730dead872099a7691267694b913f66e)

- Updated import statements at crate level [5a09b63](https://github.com/emirror-de/axum-gate/commit/5a09b6331743fab6a86ea3f6e0d4a6827f9b685a)

- Merge branch 'refactor/errors-module' into 'nightly' refactor: Updated errors module to the ddd structure See merge request lprobst/axum-gate!5 [d3a7510](https://github.com/emirror-de/axum-gate/commit/d3a75106c4bfacc47afcec88e4670ae6819c588c)

- Doc-tests [d73f335](https://github.com/emirror-de/axum-gate/commit/d73f33595b991e89d0345a97cff6fdff3b68821a)

- Removed unnecessary slashes in documentation [2bda856](https://github.com/emirror-de/axum-gate/commit/2bda856a940cc690d8e1b80f9d4f6f9dc086c156)

- Removed unnecessary blank line [9208a25](https://github.com/emirror-de/axum-gate/commit/9208a25782cbecd615cfba26f44b867c5aa4adf9)

- Applied clippy fix [5468aa5](https://github.com/emirror-de/axum-gate/commit/5468aa541925e7bd1de2a16e79a376b80d4d9105)

- Removed unused variants of secrets::errors [055de11](https://github.com/emirror-de/axum-gate/commit/055de11af166a3c3b70e0a488c1fefb504b2cf5d)

- Removed unused variants in permissions::errors [7613bea](https://github.com/emirror-de/axum-gate/commit/7613bea1217dd39890eb9b1d87a816b4952687ad)

- Removed unused variants in accounts::errors [798bb92](https://github.com/emirror-de/axum-gate/commit/798bb9230850e6d0378095195c6f4f468ef5915f)

- Removed unused codecs errors [597f119](https://github.com/emirror-de/axum-gate/commit/597f119026b12205ede0c56554fc17c58b5ab174)

- Removed unused variants in authz::errors [a001f20](https://github.com/emirror-de/axum-gate/commit/a001f20e032eca947719042370361b913bfed468)

- Removed unused variants in authn errors [fc89157](https://github.com/emirror-de/axum-gate/commit/fc89157a7bd7bf1495e9a37d2729b4f94fbe27cd)

- Removed some leftovers [f32a9fb](https://github.com/emirror-de/axum-gate/commit/f32a9fba977bfb3bd8bf3328d81b15f6a5c660a3)

- Removed unused variants of repository errors [77e6e24](https://github.com/emirror-de/axum-gate/commit/77e6e24cfa5e5595ebe4629357af45bdc5eb9d54)

- Removed unused error variants [9dc9bfb](https://github.com/emirror-de/axum-gate/commit/9dc9bfbaa7863be487bee4ee432ea8c78a4ce40e)

- Moved error modules to the top level categories [a9609e3](https://github.com/emirror-de/axum-gate/commit/a9609e3dff078dc0000de84fd118c58125e0ddf0)

- Removed unnecessary deny missing docs attributes [9dc15c3](https://github.com/emirror-de/axum-gate/commit/9dc15c336ab6ed4f5043363203bba60026f25aa7)

- Wrong module documentation of errors::authn [c9a1a51](https://github.com/emirror-de/axum-gate/commit/c9a1a510cc0e8b25004f9e0e7d474f5c6e30f96f)

- Updated errors module to the ddd structure [d9c3da1](https://github.com/emirror-de/axum-gate/commit/d9c3da12417b00cd12fe42df775a858c58bf5422)

- Updated custom-roles example [3b34498](https://github.com/emirror-de/axum-gate/commit/3b3449849a857f4e4a5e082e87e2ceb1f8789065)

- Added some comments to the custom-roles example [b9b8f7c](https://github.com/emirror-de/axum-gate/commit/b9b8f7c0e8e2bacb57cdd3e13fb0ccc821a50162)

- Prometheus example [e57fbab](https://github.com/emirror-de/axum-gate/commit/e57fbabd43913c657ac90a5f411565d89a7f5eae)

- CI workflows [0c5ce13](https://github.com/emirror-de/axum-gate/commit/0c5ce1389e5d421441e141e9ad240616c5830491)

- Doc-test [a1b08a8](https://github.com/emirror-de/axum-gate/commit/a1b08a873f4950fc009ae393643912b5975f36c0)

- Added type hint to Gate documentation example [eaa1b53](https://github.com/emirror-de/axum-gate/commit/eaa1b53d5769e1261e121035a44f936de81c982a)

- Updated CHANGELOG.md [709ba89](https://github.com/emirror-de/axum-gate/commit/709ba89f537a6a265ca2c26bf6c91a1a67fa5a68)

- Added github workflow [90dd0be](https://github.com/emirror-de/axum-gate/commit/90dd0be75a0597952e81af58ca809210c428cfa9)

- Updated gitlab-ci to nightly instead of main [f21cb99](https://github.com/emirror-de/axum-gate/commit/f21cb99ab4db04836cceb67bc945764da98564a5)

- Resolved clippy error about type complexity [9b1c43d](https://github.com/emirror-de/axum-gate/commit/9b1c43d728ecbad5b5ed0616ea766263d09118d9)

- Updated Cargo.lock [766db79](https://github.com/emirror-de/axum-gate/commit/766db7972777a68cfa853e336b4ed9bf38127666)

- Switched from jsonwebtoken/rust_crypto to /aws_lc_rs to remove RSA (RUSTSEC-2023-0071) from dependencies [6e2f7ff](https://github.com/emirror-de/axum-gate/commit/6e2f7ff2d19c7373fa0599981c7bdbc3b9350a2d)

- Updated Cargo.lock [4818596](https://github.com/emirror-de/axum-gate/commit/4818596f2b9b59f00be1cc9303db33f4383d5337)

- Downgraded to v1.0.0-rc.0 [d76fd46](https://github.com/emirror-de/axum-gate/commit/d76fd46eda430c12c2f5575de66966fe30d526d9)

- Updated to latest jsonwebtoken dependency [a1a88ae](https://github.com/emirror-de/axum-gate/commit/a1a88ae5e1a0d168f1b98fd4d99fd193d644b88d)

- Updated Cargo.lock [68dd4f4](https://github.com/emirror-de/axum-gate/commit/68dd4f44669c53b2f12b7bae734fb657948af45b)

- Restructured dependency organization [41158f9](https://github.com/emirror-de/axum-gate/commit/41158f916e390f877ca828cdf78686e96cd32c94)

- Updated .rules [5837b67](https://github.com/emirror-de/axum-gate/commit/5837b67564e1c402c8356bd27df297299b9a4baf)

- Added bearer gate documentation to Gate [e279caf](https://github.com/emirror-de/axum-gate/commit/e279caff76c283f2494a4f6688839adb962825ca)

- Added Account::has_role, is_member_of and has_permission convenience methods [b19dcf3](https://github.com/emirror-de/axum-gate/commit/b19dcf3abbff7bd7efa9d71b4d41543f0ad1e592)

- Added example for custom enum permissions [b04935f](https://github.com/emirror-de/axum-gate/commit/b04935fa129c6864a7d6c675659e75ff47d2b65d)

- Moved route_handlers to separate modules [3fbaf9c](https://github.com/emirror-de/axum-gate/commit/3fbaf9c817046f4154468a72cc3b5b535e46ffb5)

- Removed clippy warnings [b8e48e3](https://github.com/emirror-de/axum-gate/commit/b8e48e33b7f9cffe6aeb45e8cf447ca5e8e88da8)

- Updated README.md [57f02c4](https://github.com/emirror-de/axum-gate/commit/57f02c45c3d2dac021fc431e7b40012cdc7589fa)

- Updated to the latest state [083523f](https://github.com/emirror-de/axum-gate/commit/083523f8fba6ff072f1a0c834e075fa07f58e52a)

- Removed unused doc folder [19c8646](https://github.com/emirror-de/axum-gate/commit/19c8646832d1a8ab02c2e1fa1afaeabbbac45335)

- Updated prelude module [7c0b85e](https://github.com/emirror-de/axum-gate/commit/7c0b85e82e3107642a2fdd48c2bb3755986ae779)

- Now re-exporting axum_extra [d4fd4e5](https://github.com/emirror-de/axum-gate/commit/d4fd4e5f4ac92a6a1e256cbc0927d0823cffd950)

- Updated .rules [cbfc7b7](https://github.com/emirror-de/axum-gate/commit/cbfc7b7401004abe3731745868fb53f5616597da)

- Revert "feat: Added metrics jwt_remaining_ttl_seconds" This reverts commit e5dce3d1f90ea2a8e3ee0f146d1010fca7bcbe9e. [dc31802](https://github.com/emirror-de/axum-gate/commit/dc31802930e35558c301c6df2d1b8392c8f2f3eb)

- Added metrics jwt_remaining_ttl_seconds [e5dce3d](https://github.com/emirror-de/axum-gate/commit/e5dce3d1f90ea2a8e3ee0f146d1010fca7bcbe9e)

- Added outcomes of JWT validation for latency labeling [94d8e61](https://github.com/emirror-de/axum-gate/commit/94d8e61f640435221feabc2736c2075c1e3f99c5)

- Integrated metrics to bearer gate, Added authz latency histogram [955a22b](https://github.com/emirror-de/axum-gate/commit/955a22ba5d6a3d2fdab9b8601bc6948910230eae)

- Merge branch 'refactor/category-domain-design' into 'nightly' Refactor/category domain design See merge request lprobst/axum-gate!2 [5ebbaf6](https://github.com/emirror-de/axum-gate/commit/5ebbaf66c7be535f7eee530a950674530c7ae0a5)

- Updated README.md [af01808](https://github.com/emirror-de/axum-gate/commit/af018082ec655350d15f86ea47c6366069f80200)

- Updated .rules [e523190](https://github.com/emirror-de/axum-gate/commit/e523190f28c42e87b45bf043607957d6989b7731)

- Polished documentation and public API [3413ff6](https://github.com/emirror-de/axum-gate/commit/3413ff6c0c01275847b02f6f0dd6124b95c8b71d)

- Moved static_token_authorized module to gate::bearer [b4f278a](https://github.com/emirror-de/axum-gate/commit/b4f278a501e7bfef52f030d848a4e5eb489ff860)

- Moved as_permission_name module to permissions [f4f3b75](https://github.com/emirror-de/axum-gate/commit/f4f3b753c3096799387096072c19cdb576f57be2)

- Tests [28fcce1](https://github.com/emirror-de/axum-gate/commit/28fcce10e819e3d95e88cbc5f03d6fef2f96ce70)

- Updated .rules [45209fa](https://github.com/emirror-de/axum-gate/commit/45209fab4d05d2340f78f3a615d5947d343ccb51)

- Updated ruleset [3bb4164](https://github.com/emirror-de/axum-gate/commit/3bb41645d3e30807463c7ab4b3c98a1a4c193a29)

- Updated documentation [ed8da88](https://github.com/emirror-de/axum-gate/commit/ed8da887f7f9c912142120b2c8b5f57899d31945)

- Errors now unified in errors module [03d4cb8](https://github.com/emirror-de/axum-gate/commit/03d4cb8f095257490244941f7edda5ba4e2fb063)

- Further refactoring, only errors left [9c0b79a](https://github.com/emirror-de/axum-gate/commit/9c0b79ac063f51f33b72f1234a85d2efd540560e)

- Started moving to new structure [f178687](https://github.com/emirror-de/axum-gate/commit/f17868751384a4965d017e9d90ac611d9d0d0364)

- Merge branch 'wip/permission-mapping-repository' into 'nightly' Wip/permission mapping repository See merge request lprobst/axum-gate!1 [45446a5](https://github.com/emirror-de/axum-gate/commit/45446a534aa756e2fa8707f68bcddb6cd5e95650)

- Cargo clippy [912c85b](https://github.com/emirror-de/axum-gate/commit/912c85b642ae2756e9909bf3d44ca01866fcb23e)

- Removed bulk demo as it is not implemented yet [a6fd752](https://github.com/emirror-de/axum-gate/commit/a6fd752cbd515bca9801c5ebc3e701cc5313b42f)

- Removed comment [524fbb0](https://github.com/emirror-de/axum-gate/commit/524fbb0a5161d2d4878f41f6c331fe4343bc308e)

- Removed unnecessary sentence [947c1d9](https://github.com/emirror-de/axum-gate/commit/947c1d94a29b6b5bc81f5236b0b64032a120fb95)

- Updated documentation for Role [1a9fa83](https://github.com/emirror-de/axum-gate/commit/1a9fa8337d2c5b3af3013fd7a408fd99cc82a476)

- Missing demonstration of with_cookie_template [7f5d2b7](https://github.com/emirror-de/axum-gate/commit/7f5d2b7598ab3df93961aa1d5419261c6fdad478)

- Moved crate to workspace root [605d07d](https://github.com/emirror-de/axum-gate/commit/605d07de2cb2466ed3cac13abd730ab3681c7812)

- Fixed documentation warnings [cef3f07](https://github.com/emirror-de/axum-gate/commit/cef3f07bbd6bd48956d6e7ae7c6f82b7b2656459)

- Doc-tests [bcbbf22](https://github.com/emirror-de/axum-gate/commit/bcbbf2296f46632d856af15b37e2304059864085)

- Updated bearer gate implementation [6ceab5e](https://github.com/emirror-de/axum-gate/commit/6ceab5e755a24f216d5cd28a481cd37858411185)

- Fixed clippy warnings [6e66cdd](https://github.com/emirror-de/axum-gate/commit/6e66cdd042520b9507a8f5cb41fdde0e5ea1e1a8)

- Removed documentation part of SurrealPermissionMapping [a71de8a](https://github.com/emirror-de/axum-gate/commit/a71de8aacf8fef9104475c502c76d678a50f3221)

- Updated documentation for CookieGate::new_with_codec [c20e742](https://github.com/emirror-de/axum-gate/commit/c20e742057f6042bb37a6634eb07b55df6590baf)

- Moved CookieGate and CookieService to cookie module [a333143](https://github.com/emirror-de/axum-gate/commit/a333143e5c4ae5db5b4ec768a49a33f5c17cd8e1)

- Added optional user authentication configuration for Gate [14e0f15](https://github.com/emirror-de/axum-gate/commit/14e0f152f8bd82279ba28d0f4a2462e37effda32)

- Re-added validation in new function, updated documentation [29fabf5](https://github.com/emirror-de/axum-gate/commit/29fabf529e8e9ce9cd27fde0bb5de77ff2c8ea86)

- Removed constructor that creates inconsistency [e2d85d9](https://github.com/emirror-de/axum-gate/commit/e2d85d9874288dbdb54a5b2fb8c1eec8d70b364a)

- Removed unnecessary validation of PermissionId [be80432](https://github.com/emirror-de/axum-gate/commit/be80432b7c371da7825237f406a3e8c1239a1139)

- TableName is now gated to be only available when surrealdb or seaorm is activated [8af96db](https://github.com/emirror-de/axum-gate/commit/8af96dbbe1c6632186fa80ad1583a57a2a971ae4)

- Applied clippy fix [284f410](https://github.com/emirror-de/axum-gate/commit/284f4102480903e8cdd02472f709a3f0e4608118)

- Unified TableNames for databases [2b9bb0d](https://github.com/emirror-de/axum-gate/commit/2b9bb0dd46697fecd1f40361fe9cc62c993974b8)

- Updated table names for seaorm [6d9b606](https://github.com/emirror-de/axum-gate/commit/6d9b606eb5615e132842db362d3984d1e334d36d)

- Added implementation of PermissionMappingRepository for seaorm [baa9b10](https://github.com/emirror-de/axum-gate/commit/baa9b106fbc73380a2bed1ceedd3e48ea8d492f5)

- Updated documentation for LoginService [1e24b1a](https://github.com/emirror-de/axum-gate/commit/1e24b1a4fab0bdf323694525331b13c35eb41246)

- Updated surrealdb repository implementation [952731f](https://github.com/emirror-de/axum-gate/commit/952731f9ce0b87085f9081fbd9635bb72615be37)

- Updated .rules [18c6b09](https://github.com/emirror-de/axum-gate/commit/18c6b0989bd9be03b6e728492350c56c22526d71)

- Updated DatabaseScope default [7632352](https://github.com/emirror-de/axum-gate/commit/763235243a08b95894d466068daa2664488218df)

- Added PermissionMappingRepository implementation for surrealdb [71b2b77](https://github.com/emirror-de/axum-gate/commit/71b2b7719de368ab963f51da8b3c9b5d8fc48c76)

- Added surrealdb best practices to .rules [e6229b0](https://github.com/emirror-de/axum-gate/commit/e6229b0a9d38b33bf99b965306a17b09665d5b70)

- Simplified MemoryPermissionMapping implementation [75785ff](https://github.com/emirror-de/axum-gate/commit/75785ffc0d4a7b960c669eb5c58142f74f501be3)

- Removed PermissionMapping::original_string [d903556](https://github.com/emirror-de/axum-gate/commit/d9035567f114107bdc9c89c2eb9772bc8fd94469)

- Added PermissionMappingRepository to docs [b37112d](https://github.com/emirror-de/axum-gate/commit/b37112d14fe37e8bfcf88f4797110fb8e68dcefa)

- Moved prometheus export to integrations module [1c9db09](https://github.com/emirror-de/axum-gate/commit/1c9db09cf489c3644a00b7aa5aa6640a9105f92a)

- Updated .rules file [2a7d8e4](https://github.com/emirror-de/axum-gate/commit/2a7d8e45afa72c18d1b40efab2857cdc242e44e0)

- Made rust-analyzer happy by adding type for .into() call [b654824](https://github.com/emirror-de/axum-gate/commit/b654824ce3c425b7f79858304acabeabe85eaebf)

- Removed normalized string parameter from PermissionMapping::new [d6c2750](https://github.com/emirror-de/axum-gate/commit/d6c2750a49af75e51678c32b5236612e8c45b674)

- All doc-tests are now running and passing [b748825](https://github.com/emirror-de/axum-gate/commit/b748825be182ff54f796fb8b13cf4fbae1e543ad)

- Renamed `utils` module to `integrations` as it only contains third party code [14b49f1](https://github.com/emirror-de/axum-gate/commit/14b49f147309e58233502a31d233cec24a490516)

- Updated Cargo.lock [c981f4c](https://github.com/emirror-de/axum-gate/commit/c981f4c4d87db9efd24e55d72c48de4ab7b128ec)

- PermissionMappingRepository [614089d](https://github.com/emirror-de/axum-gate/commit/614089d8f29d4ba60560bba1bd2651724396360c)

- Updated Cargo.lock [48d8e7f](https://github.com/emirror-de/axum-gate/commit/48d8e7f5a36f8717df4f3e5d50f09d60b00c4469)

- Updated .rules file [0bd7b9c](https://github.com/emirror-de/axum-gate/commit/0bd7b9c41ad45c28ca972957a3f51dc72b601465)

- Added convenient method CookieGate::require_login [b4be1f9](https://github.com/emirror-de/axum-gate/commit/b4be1f9fd0888b59aa171ecac10089033c60c61e)

- Updated .rules [ee0375c](https://github.com/emirror-de/axum-gate/commit/ee0375c763c25e4e8e59a20ca6acc021c1fbde2d)

- Updated MSRV in README.md to 1.86 [f0654cd](https://github.com/emirror-de/axum-gate/commit/f0654cd6c9c34a3a2be47101467924ee27c6dc7d)

- Doctest in audit module [59484d6](https://github.com/emirror-de/axum-gate/commit/59484d6f89338d1d2f6a2a128823de1acf77ec1d)

- Updated flake version number [629d52f](https://github.com/emirror-de/axum-gate/commit/629d52f51c049281a54d8ffd1eda5b1cbc43fbbb)

- Updated SECURITY.md [95a26e5](https://github.com/emirror-de/axum-gate/commit/95a26e548fee0b45273fa6be835328e5bd6c13ee)

- Updated .rules file [5cc0875](https://github.com/emirror-de/axum-gate/commit/5cc08756121d56c67fa3ddf3f17e78afbe0e583d)

- Updated prometheus dependency [eaf9c19](https://github.com/emirror-de/axum-gate/commit/eaf9c199044421693b8df9a3b635bc257568a6a5)

- Removed clippy warnings [2452594](https://github.com/emirror-de/axum-gate/commit/2452594fa6929494f7a1eb719bf9278ebf54aa9e)

- Add Prometheus metrics feature, builder hooks, strum label enums, and example; re-export prometheus; instrument account insert success/failure [78dae20](https://github.com/emirror-de/axum-gate/commit/78dae20b2d41c622949145d09ac8456ea9432279)

- Updated SECURITY.md [a84d2bb](https://github.com/emirror-de/axum-gate/commit/a84d2bbfeaca5e15b19c8fc3da8580450b39553f)

- Updated .rules file [dd340dd](https://github.com/emirror-de/axum-gate/commit/dd340dd405cbe36267bd486e2f736b81c37e245e)

- Updated version to v1.0.0-rc.1 [1131e1c](https://github.com/emirror-de/axum-gate/commit/1131e1c70f9e7666cceb43c739b1ad0a770c7421)

- Added additional JWT secret management section to README.md [9eaf12e](https://github.com/emirror-de/axum-gate/commit/9eaf12edd058a0408faec429bf2b785658b9d68b)

- Limited visibility of modules to crate where not inteded to bleed to the outside [95f7edb](https://github.com/emirror-de/axum-gate/commit/95f7edbc4dc40b2bc162d787a56cc35c105ad606)

- Removed Cargo.lock from .gitignore to ensure correct Nix build [6f34fe2](https://github.com/emirror-de/axum-gate/commit/6f34fe2d7465f65c48747ff9bc4582db5867623c)

- Fixed doctests in advanced module [8fbad7e](https://github.com/emirror-de/axum-gate/commit/8fbad7e6becaeb5228302ca2af22ac420aa06ad6)

- Added planned feature section [ad4a982](https://github.com/emirror-de/axum-gate/commit/ad4a98279e27192e25b942b0d5dcf0c106cd5327)

- Updated cliff.toml [e1f0056](https://github.com/emirror-de/axum-gate/commit/e1f00564b4aae0289711a4ddbf0ea81ac30c4776)

- Redesigned cliff.toml [1d3c29f](https://github.com/emirror-de/axum-gate/commit/1d3c29f038507f011e06f2772957c64ada189e1e)

- Added security audit status to README.md [3729894](https://github.com/emirror-de/axum-gate/commit/37298943bb1e84315b3a45522653253891d7a367)

- Updated error messages [327381e](https://github.com/emirror-de/axum-gate/commit/327381e6658dbf293aa3361e44d88594e908192a)

- Made error messages more user friendly [ae92214](https://github.com/emirror-de/axum-gate/commit/ae9221460757bf29407260bd7def91c136173eda)

- Added cargo semver checks to gitlab-ci.yml [02627a0](https://github.com/emirror-de/axum-gate/commit/02627a0348ed86ba511b18b8f1130a8d055038b3)

- Removed access_policy_example test [5fc404d](https://github.com/emirror-de/axum-gate/commit/5fc404d1806d5575a8007dace4fd4b8ffcb3913c)

- Remove repository_additions module, moved content to repositories [6366a33](https://github.com/emirror-de/axum-gate/commit/6366a33e1f2f09d8aead9a651e6ed08003c525f1)

- GitLab CI configuration for examples [66018ba](https://github.com/emirror-de/axum-gate/commit/66018ba3f4f60452ff5dcae57f6a192847be4fdb)

- Temporarily added paste and rsa to ignore parameter of audit until solved by surrealdb and sea-orm [11cdf54](https://github.com/emirror-de/axum-gate/commit/11cdf54cae59d7f701402cf99f5f442fd31b0bdf)

- Using rust:latest image for CI [3c71769](https://github.com/emirror-de/axum-gate/commit/3c717697f00c21cc4be8b5cd48de7d9bc540da8a)

- Tried to fix timing attack vector test for CI [5aa12ee](https://github.com/emirror-de/axum-gate/commit/5aa12ee21e42c36f0b02534b90c6c14548b010a3)

- Cargo deny check [fd273b4](https://github.com/emirror-de/axum-gate/commit/fd273b4348962ef017d8d514a013d60e7ee8bf90)

- Clippy warnings [803308d](https://github.com/emirror-de/axum-gate/commit/803308d0d5f0a80250cf3cd3e9c0b87fee2e1c0d)

- Hardened timing attack protection test to work with CI [f8a8140](https://github.com/emirror-de/axum-gate/commit/f8a8140186fd6ba8aecc2b6107d102d14fc7ab73)

- Bool conversion test [a579d27](https://github.com/emirror-de/axum-gate/commit/a579d2773754cf1630740f0bad34649b8e4f88c6)

- Clippy errors [4c7ffc7](https://github.com/emirror-de/axum-gate/commit/4c7ffc7d699bf62671d45e61a46fa808647401b7)

- Moved test module to the bottom of the file in distributed example [062db6e](https://github.com/emirror-de/axum-gate/commit/062db6edbc8d7245da6afdbcb5139308d97177e2)

- Implemented FromIterator instead of custom from_iter function [26d0817](https://github.com/emirror-de/axum-gate/commit/26d081704cf7189a8324afe5b333305b5dad6452)

- Added Cargo.lock to gitignore [50738e3](https://github.com/emirror-de/axum-gate/commit/50738e33de5610acc883c4b1a51fb1f59f68a419)

- Rustfmt [b7c10e2](https://github.com/emirror-de/axum-gate/commit/b7c10e26e0874bf78539c9cfbe962601c787df08)

- Updated MSRV to 1.86 [745bf90](https://github.com/emirror-de/axum-gate/commit/745bf90fef1796680f0f84ef07a2f222f6d0848d)

- Updated GitLab CI to Rust 1.85 [f5be341](https://github.com/emirror-de/axum-gate/commit/f5be3418b431954a72b55f0411890e3e10dad660)

- Using resolver 2 [e4a80ab](https://github.com/emirror-de/axum-gate/commit/e4a80ab65c14a3f0cb3c59159ab0a5481d169205)

- Rust 2024 required [f51ff0a](https://github.com/emirror-de/axum-gate/commit/f51ff0ae935d1caa6c97469abaa28700708993e5)

- Removed Cargo.lock [93a6b9c](https://github.com/emirror-de/axum-gate/commit/93a6b9c6b52fb45a6d773951cd036fab99a83141)

- Updated examples to use 2021 instead of 2024 for backwards compatibility [7996956](https://github.com/emirror-de/axum-gate/commit/799695687a4e5da12b88c25e1d05392e4d6a8e63)

- Applied clippy fix [af439b8](https://github.com/emirror-de/axum-gate/commit/af439b8747db2f8524582cd0a842b97584ed80cd)

- Updated resolver to 2 instead of 3 [c0a4087](https://github.com/emirror-de/axum-gate/commit/c0a408757894ca2420dfa1c7f26cf31c6917bd6e)

- Added initial GitLab CI file [fe29804](https://github.com/emirror-de/axum-gate/commit/fe298046152c20c17410c9b70b96c54cdf1834c5)

- Updated documentation for validate_permissions macro [4a5d75a](https://github.com/emirror-de/axum-gate/commit/4a5d75a36396a002ee91c6d5c051cdb7308ac597)

- Re-arranged re-exports [9205118](https://github.com/emirror-de/axum-gate/commit/9205118cb7cf146fefe4ea410445281b7f88fa45)

- Re-added no_run to failing tests [fdd5d69](https://github.com/emirror-de/axum-gate/commit/fdd5d69e9c65718c24ae85e814bc33e5addae0d1)

- Updated MSRV and added policy to README.md [09baccb](https://github.com/emirror-de/axum-gate/commit/09baccb8aa748cf39cba5a2dfa550aaf3c93a7d5)

- Updated documentations [05fd819](https://github.com/emirror-de/axum-gate/commit/05fd8198d1c81db2b5e1c85d6a72ed87fa8aac48)

- Added rust-version to Cargo.toml [c10c99e](https://github.com/emirror-de/axum-gate/commit/c10c99e3d6fbc707296743606e82ba90636447d6)

- Bumped version to 1.0.0 [a2dc4c5](https://github.com/emirror-de/axum-gate/commit/a2dc4c5bbd64938603deae0dc4d2bbc3962763d3)

- Updated README.md [f5c0865](https://github.com/emirror-de/axum-gate/commit/f5c08657cff2a0d04ea522e2b845b921b011c82f)

- Updated TableNames documentation [95388ec](https://github.com/emirror-de/axum-gate/commit/95388ec80ed1d1dd7400511c63f7755eec80dd1a)

- Updated repository docs [74df9c6](https://github.com/emirror-de/axum-gate/commit/74df9c676089fb53bbd13d1fabd0d2eedeaca647)

- Removed stability notice for advanced module [82bc301](https://github.com/emirror-de/axum-gate/commit/82bc301e4fa2f99321cfbbef7bf2c9784e2d5ab9)

- Overhaul ValidationReport documentation and restore Default derive [020d00b](https://github.com/emirror-de/axum-gate/commit/020d00b12e5258f2647fa9e995007ba994285788)

- Improve CredentialsVerifier, HashingService, VerificationResult and Codec documentation [c4491ce](https://github.com/emirror-de/axum-gate/commit/c4491ce632ee83e3047c371b9cae370a88ccd0f4)

- Improve AccountRepository and SecretRepository documentation with semantics, security and usage guidance [a46d741](https://github.com/emirror-de/axum-gate/commit/a46d741b260db0e1ce81a11002d1b2becfe1fdd6)

- Updated LoginResult and LoginService [9e8be98](https://github.com/emirror-de/axum-gate/commit/9e8be98b3527b016b31af4a82ce93ef04692f2ca)

- Moved advanced module to a separate file [d960540](https://github.com/emirror-de/axum-gate/commit/d960540ecc6691764653117f417734458e811d03)

- Refactored and documented advanced module [4e676a1](https://github.com/emirror-de/axum-gate/commit/4e676a14b162366889c50232b04102b7815219f8)

- Failing tests, removed non_exhausting attribute from Error enum [1109617](https://github.com/emirror-de/axum-gate/commit/11096177b3c079bc4fb39bfbf7baf37326220c39)

- Updated documentation for repositories and storage root export [d998613](https://github.com/emirror-de/axum-gate/commit/d99861332f4d6806bc35c606207648775f52e56f)

- Restructured jwt module exports [9be02d0](https://github.com/emirror-de/axum-gate/commit/9be02d02472d4a415fd1ce38daa8eb95716a4a6c)

- Replaced directory .rules with a .rules file [ac81f49](https://github.com/emirror-de/axum-gate/commit/ac81f49559af8b54c2f081c821b8ca11266e16ff)

- Updated documentation for errors module [6daffd1](https://github.com/emirror-de/axum-gate/commit/6daffd13df4621965a4e3183e9f837c6a3703ad3)

- Renamed all error modules to errors [caa3a95](https://github.com/emirror-de/axum-gate/commit/caa3a956eec1d1c3e2c2ca5ddb6da878b94bffa1)

- Removed obsolete validate_permission_uniqueness function [1b05b60](https://github.com/emirror-de/axum-gate/commit/1b05b60986b3f72022902c7ccc476a7504a6417a)

- Updated Credentials documentation [d641659](https://github.com/emirror-de/axum-gate/commit/d6416598b36676d6dd424c89fc563ca64c275918)

- Updated Secret documentation [a616884](https://github.com/emirror-de/axum-gate/commit/a6168849927264db352bc6ea51fdc526eae35e97)

- Updated crate::advanced module exports and documentation [d4b6c79](https://github.com/emirror-de/axum-gate/commit/d4b6c7985e1123029b5c37b89de96b562f026c4a)

- Updated Argon2Hasher documentation [8c523db](https://github.com/emirror-de/axum-gate/commit/8c523dbf68f1d72c00205fad8aca2cdc57062b9d)

- Updated validation module documentation [2246f7a](https://github.com/emirror-de/axum-gate/commit/2246f7a9dc1664b9990b64a3b3ca7901c96d47ee)

- Updated crate root documentation [07deac1](https://github.com/emirror-de/axum-gate/commit/07deac1d8230f5955a3853cd376ba0a8e191fcb0)

- Updated authentication handlers section in crate root [5185336](https://github.com/emirror-de/axum-gate/commit/518533609976352765da697b7c08b8f9310cb7a2)

- Added .rules folder [66b3646](https://github.com/emirror-de/axum-gate/commit/66b3646225a9f4ac7b346b727005d6f30a5acc8c)

- Added rate-limiting example [6a14849](https://github.com/emirror-de/axum-gate/commit/6a14849f4f6d07da14564dbf11ee05fd979a32ab)

- Merged Gate::cookie_deny_all into Gate::cookie [422d3f8](https://github.com/emirror-de/axum-gate/commit/422d3f800d4df7043d98d16ec84bf44105783c4c)

- Updated documentation for JWT key management [42dd3a9](https://github.com/emirror-de/axum-gate/commit/42dd3a918c2a3131b2ad3a1811d26643df87df82)

- Added hint about JWT key regeneration, Fixed doc links [eb4042e](https://github.com/emirror-de/axum-gate/commit/eb4042e3904b71b230177b0da76368685f4deae8)

- Removed unnecessary comments [e2e8cb6](https://github.com/emirror-de/axum-gate/commit/e2e8cb6bc225859b2b123bfd48a6581a882ccaa7)

- Timing attack for non-existent users [fa56d1c](https://github.com/emirror-de/axum-gate/commit/fa56d1c369b439cb1095c10cc96c0d9eb7a68c0b)

- Implemented atomicity of removing secrets and accounts as close as possible [ef75c17](https://github.com/emirror-de/axum-gate/commit/ef75c17405c451632140bc52dea81147c98bb667)

- Updated documentation for CookieTemplateBuilder [a2dbf11](https://github.com/emirror-de/axum-gate/commit/a2dbf1135176638645f2909965132422a2bc75ac)

- Updated documentation for CookieTemplateBuilder [72a2503](https://github.com/emirror-de/axum-gate/commit/72a2503fcd5248507ef76767d49209a9e5084a73)

- Removed unused imports in examples [30fcf74](https://github.com/emirror-de/axum-gate/commit/30fcf745a5e8d7cd0f89ef8aa1dbe4e36497f9ef)

- Added CookieTemplateBuilder [d7b5ef4](https://github.com/emirror-de/axum-gate/commit/d7b5ef46d7c6e8a51af14030a7a89a45e41ac490)

- Added .envrc for direnv [a080188](https://github.com/emirror-de/axum-gate/commit/a0801881cb0caef80fa29485c1a7d21d41de907d)

- :default always returns DevFast variant on debug builds, HighSecurity on release builds [5820480](https://github.com/emirror-de/axum-gate/commit/5820480e2cedfb1ed39abb1dafb111e83e037822)

- Updated SECURITY.md [e04e21a](https://github.com/emirror-de/axum-gate/commit/e04e21a8044e6ea5a9c78592edb32122563fa97d)

- Fixed some import errors [e6aa7fd](https://github.com/emirror-de/axum-gate/commit/e6aa7fd47ba4dbd66b140c789993fcc4619dae87)

- Updated error to RoaringTreemap [4d8b2fd](https://github.com/emirror-de/axum-gate/commit/4d8b2fd267b699642e15d61403ac80cbbf08f9f7)

- Replaced RoaringBitmap by RoaringTreemap [1b1f16c](https://github.com/emirror-de/axum-gate/commit/1b1f16c0650c395d8848f4af6482f5f6a27fa59d)

- SurrealDB repository implementation [1693929](https://github.com/emirror-de/axum-gate/commit/1693929e4a392c675fa4c402f01ec5138cd1b19b)

- Login function no longer required Json encoded credentials [468c276](https://github.com/emirror-de/axum-gate/commit/468c2764a675052397c0d1476fa840a91f10d553)

- Removed unused import [d2c36ff](https://github.com/emirror-de/axum-gate/commit/d2c36fff746438e80058c325e012168da7b96da2)

- Simple usage example [6f3878b](https://github.com/emirror-de/axum-gate/commit/6f3878b2d1bc0da987d6ddee719c0f21d5c7ce72)

- Json serialization of the cookie value was not required [4e385b3](https://github.com/emirror-de/axum-gate/commit/4e385b3a04d43431de6a66b5f67a3b94fd6dc603)

- Updated simple-usage example [4e022d7](https://github.com/emirror-de/axum-gate/commit/4e022d79479ad77cf1f184ef407a977a79c4753e)

- Removed deleted demo definition in distributed example [dc62511](https://github.com/emirror-de/axum-gate/commit/dc6251159962b06b7ef02021de416754f53e27b1)

- CommaSeparatedValue is now gated behind storage-seaorm feature as it is the only application for it [17c8424](https://github.com/emirror-de/axum-gate/commit/17c8424df666b3c48ac296924d1ecc74255cda7e)

- Moved the simple usage example to be used within the workspace [4ee9a9a](https://github.com/emirror-de/axum-gate/commit/4ee9a9add82fa55a7f1bbdde7d8c15cac59cf89c)

- Removed unused code warning [22bd782](https://github.com/emirror-de/axum-gate/commit/22bd7824fb593a5461d0e0905dd94a595d49bc9d)

- Removed readme from permission validation example [43cbd64](https://github.com/emirror-de/axum-gate/commit/43cbd64dbb2ff942cca566874450e5d15156cc9e)

- Re-arranged use statements [3c8806a](https://github.com/emirror-de/axum-gate/commit/3c8806afc90abf702024fecc11c841a8e50021bc)

- AsPermissionName is now in domain::traits [c50ab89](https://github.com/emirror-de/axum-gate/commit/c50ab89766a8f011cb2e11839c2284727b5d8ace)

- Re-arranged use statements [ed0fc60](https://github.com/emirror-de/axum-gate/commit/ed0fc603a8ef00f6cf9859b25c52ad4299aabc6f)

- Updated visibility of crate internal function [3d5f8dc](https://github.com/emirror-de/axum-gate/commit/3d5f8dca232825b32619f99a25ce9d4e3098dea7)

- Redesigned README.md [5b9c22d](https://github.com/emirror-de/axum-gate/commit/5b9c22d23009965e8a0a740b01b238053bf46f11)

- Updated README.md [ffc96a8](https://github.com/emirror-de/axum-gate/commit/ffc96a80da8e5cf94a9397086ec12e911c5a4f52)

- Added timing-attack protection [7e133eb](https://github.com/emirror-de/axum-gate/commit/7e133eba5a44ecee0994de24db8306c5bba400bd)

- Updated README.md [4bb15dd](https://github.com/emirror-de/axum-gate/commit/4bb15dd9c1455aa6f93f2044ded96f59a7d89ebd)

- Updated README.md [5b19a12](https://github.com/emirror-de/axum-gate/commit/5b19a12e538f67e6e95c87af492e5b274b04cc7c)

- Updated crate documentation [7bed0ea](https://github.com/emirror-de/axum-gate/commit/7bed0eaf3bafecbcfdb28070f97ac431479613a0)

- Updated references to the new public API [c9c2947](https://github.com/emirror-de/axum-gate/commit/c9c29471bf63cbc8f6192fd41728b80587a89174)

- Updated documentation of public exports [fcc4ea1](https://github.com/emirror-de/axum-gate/commit/fcc4ea1ab81a15c16fd81936f27fc03236a4ffab)

- Removed unused files [7cd120b](https://github.com/emirror-de/axum-gate/commit/7cd120b0fef898aabf4ac05e31d3e41acb865003)

- Updated public API [ddf79ba](https://github.com/emirror-de/axum-gate/commit/ddf79ba4354fdf0f792718c070b8f7b7108d3fcd)

- Updated to the latest public crate API [9db1348](https://github.com/emirror-de/axum-gate/commit/9db13482092413a4022029fc8d511b7bf8534111)

- Refined public crate API [75a4fd7](https://github.com/emirror-de/axum-gate/commit/75a4fd73504a3ec80f0171fc3e66828e022d6d0b)

- Removed warning of unused imports [8f4c82b](https://github.com/emirror-de/axum-gate/commit/8f4c82b3a36c9f34bf12c9582ea16cccab3b8e8d)

- :from is now implemented for several types [e179f67](https://github.com/emirror-de/axum-gate/commit/e179f67b0f10045b1a15f092f4cf89503832759c)

- Added Permissions struct that replaces the raw usage of RoaringBitmap [c54df45](https://github.com/emirror-de/axum-gate/commit/c54df4537d774e2cfd2473d63c17971a4efa029b)

- Removed some unnecessary comments [2b07b97](https://github.com/emirror-de/axum-gate/commit/2b07b97ceb5c392c764e527eea79e4897fcce666)

- Removed unused code [5bb03a9](https://github.com/emirror-de/axum-gate/commit/5bb03a994daed99d0034126ab58a29206268866d)

- Tests [d5aa84c](https://github.com/emirror-de/axum-gate/commit/d5aa84c94b388901869c5d560972e8f294c0c25a)

- Renamed storage in infrastructure to repositories [3398b1b](https://github.com/emirror-de/axum-gate/commit/3398b1b6cac8d96e1392fecc999056aa399cd956)

- Updated README.md [1158424](https://github.com/emirror-de/axum-gate/commit/1158424be6b017e36adaf4f7441a09e8a952d26d)

- Removed unused document [6dd4cb4](https://github.com/emirror-de/axum-gate/commit/6dd4cb45033f912ca508288fc7b4c42bedbc87b3)

- Removed unused document [49b7077](https://github.com/emirror-de/axum-gate/commit/49b707740c629f986c11ff031cc4eb0f7e846740)

- Moved CommaSeparatedValue to separate module [8b27aa7](https://github.com/emirror-de/axum-gate/commit/8b27aa76724578864805cfee3bda8c8a51537453)

- Updated documentation [2b108ba](https://github.com/emirror-de/axum-gate/commit/2b108ba5cbe30f8551d3e91e8c798f80be4e91bd)

- Renamed methods in AuthorizationService to get a clean API [4569631](https://github.com/emirror-de/axum-gate/commit/4569631a2167a042cf87213ae8b9dc3d81e7f4fc)

- Implemented AccessPolicy within domain::services [c124936](https://github.com/emirror-de/axum-gate/commit/c124936cfa10fbede0ecc9eeabd28286065b3c76)

- Updated error implementation [6e20f22](https://github.com/emirror-de/axum-gate/commit/6e20f225fc9b37a37092f3eb36c81b8e5c0cfebd)

- Extracted JWT validation logic into separate service [c2cb06c](https://github.com/emirror-de/axum-gate/commit/c2cb06cd00641fe66ed41fe62d27383a04c27d8c)

- Removed dead code because of early return [b8e4974](https://github.com/emirror-de/axum-gate/commit/b8e4974a785a09f3be4ad86f1f57b85017ed6afa)

- Extracted authorization logic to domain layer [04f5031](https://github.com/emirror-de/axum-gate/commit/04f5031e445caf7817c5f7b4af7f75e1fd834cd4)

- Unused import warnings [f8af654](https://github.com/emirror-de/axum-gate/commit/f8af654cc4140cb5581dfaae675c1fb1df2d08ce)

- Moved VerificationResult from infrastructure to domain [ca46f8b](https://github.com/emirror-de/axum-gate/commit/ca46f8bea0f00f0d6662e6a304ee18e9646b1cad)

- Moved business logic of login and logout to application layer [e380290](https://github.com/emirror-de/axum-gate/commit/e380290647d6580f187ce773d0b28879296bdd8e)

- Moved Account*Services to application::accounts [ba1a848](https://github.com/emirror-de/axum-gate/commit/ba1a8486ca5370d0d9e0b31871c7384ee4c7acaf)

- Updated remaining structure double check markdown file [daa08f8](https://github.com/emirror-de/axum-gate/commit/daa08f88570591721158d0c95d3eb6593834813b)

- Removed compiler warnings [4e7a407](https://github.com/emirror-de/axum-gate/commit/4e7a40737e57e503c5df28840517bcc0eb4decc1)

- Moved codecs module from infrastructure to ports [b42e40b](https://github.com/emirror-de/axum-gate/commit/b42e40b8a52efc7fc97bf74f0be0a03ad0343edf)

- Moved credentials_verifier and hashing to ports [49555d2](https://github.com/emirror-de/axum-gate/commit/49555d2a99e3c31c9250fa09ff1b552f62477222)

- Moved infrastructure::services::secret_repository to ports:repositories::secret [ec2fb4a](https://github.com/emirror-de/axum-gate/commit/ec2fb4a1c7ed236ae8f1ec815f53c4d69aededa9)

- Moved infrastructure::services::account_repository to ports::repositories::account [478eff4](https://github.com/emirror-de/axum-gate/commit/478eff4a916c5570d72a477048227d2938daf617)

- Renamed storages to repositories [23a9a04](https://github.com/emirror-de/axum-gate/commit/23a9a04f8d459247aedf095a014a3ebe21e54bb6)

- Applied hexagonal architecture internally [bafac97](https://github.com/emirror-de/axum-gate/commit/bafac97a647c0a5ae9d2a8f63649b92d836897e9)

- Added structure double check markdown file [7a0756a](https://github.com/emirror-de/axum-gate/commit/7a0756a17b8a68309004f219e58d8d0d1d031e9f)

- Updated permissions example documentation [f1cc4ff](https://github.com/emirror-de/axum-gate/commit/f1cc4ff635d142e429a118c01406fa17bc39cdf7)

- Added documentation for working with accounts and permissions [7c41a26](https://github.com/emirror-de/axum-gate/commit/7c41a264dc81623e9809ae276b89a0e2b4232d6f)

- Moved documentation from validation to permissions module [d606773](https://github.com/emirror-de/axum-gate/commit/d6067732c9f77918cbe75a6645f9b4b5a7501d96)

- Merged ApplicationValidator::validate and ::validate_with_report [e20d1d5](https://github.com/emirror-de/axum-gate/commit/e20d1d53cbcb2f91729a9a1be68e588729d46804)

- Removed ValidationReport::duplicates [4e1e821](https://github.com/emirror-de/axum-gate/commit/4e1e821aa24b4001fc8bf6b8d71bcbf498a368a9)

- Further implemented permissions module [1140838](https://github.com/emirror-de/axum-gate/commit/11408384afa46636265e816b1528484b09fbfc93)

- Removed warning about hidden lifetime [0815294](https://github.com/emirror-de/axum-gate/commit/08152942fae58de253480870cd862ab90a0db803)

- Replaced custom SHA256 implementation by external crate [b0b834e](https://github.com/emirror-de/axum-gate/commit/b0b834e8258c50018d30462db0a5a213a0cb2143)

- Replaced const definitions for permissions in distributed example [508558f](https://github.com/emirror-de/axum-gate/commit/508558f47f3d633b3d607cf11a1713ce49601764)

- Further updated new permission module [bc9b762](https://github.com/emirror-de/axum-gate/commit/bc9b762f45add1f94608501321a40651709e9084)

- Started implementing the new permission system [065ca3a](https://github.com/emirror-de/axum-gate/commit/065ca3ae8fe518f9800ffb44d3710d96ddfb3def)

- Updated flake.lock [9595d59](https://github.com/emirror-de/axum-gate/commit/9595d598d818c681177cb5d3981de44b83ad0d22)

- Refactored flake.nix [7461c86](https://github.com/emirror-de/axum-gate/commit/7461c86366439eff98dc797beea575c5ac4e9aac)

- Updated Cargo.toml [16fc4fb](https://github.com/emirror-de/axum-gate/commit/16fc4fb188ab93b5fd623b2b80ad0c228dedefe4)

- Updated flake.lock [67349fd](https://github.com/emirror-de/axum-gate/commit/67349fdd5e2b66ea32e675e3bd046ca66391fff8)

- Moved axum-gate to crates folder [1e5a460](https://github.com/emirror-de/axum-gate/commit/1e5a46037ee387ed41805dc268b7adcbc314ea28)

- Added convenient builder option functions for JsonWebTokenOptions [7143adb](https://github.com/emirror-de/axum-gate/commit/7143adb07b16119e4a94e0ed72837dc3eb7a5202)

- Updated CHANGELOG.md [4ad0216](https://github.com/emirror-de/axum-gate/commit/4ad021644f15b76b2a842d6a6a63545ea8281aae)

- Version bump to v1.0.0-rc.0 [584ef22](https://github.com/emirror-de/axum-gate/commit/584ef22913828c04841d589c90b67036b9e2adf7)

- Updated README.md [a0b4f3d](https://github.com/emirror-de/axum-gate/commit/a0b4f3d90464799c7b7b4c5ec5441a3a46415bea)

- Gate is now a intermediary struct and works as kind of builder for a Gate [bf41702](https://github.com/emirror-de/axum-gate/commit/bf41702f165873461fa89cf2020a5945a83c8eea)

- Updated README.md [ac9b7ec](https://github.com/emirror-de/axum-gate/commit/ac9b7ec43ee98faafc7aff9b004e0c5f06b2220e)

- Removed unused gate::state module [b5f5834](https://github.com/emirror-de/axum-gate/commit/b5f5834402e2a665212a62bfc654b80e5c09aeae)

- Renamed gate::cookie to gate::cookie_service [23ed034](https://github.com/emirror-de/axum-gate/commit/23ed03430ba72c7351c24b2509893707ec5f122b)

- Moved GateService to cookie::CookieGateService in preparation for bearer auth support [d93c632](https://github.com/emirror-de/axum-gate/commit/d93c632ebd083b07ff74f52e98b47072b7c77240)

- Updated documentation, added route handler for extending permission set [da031a4](https://github.com/emirror-de/axum-gate/commit/da031a494341e425cfb97d2a5116e2f0ef728c99)

- Re-arranged use statements [7138a45](https://github.com/emirror-de/axum-gate/commit/7138a45297406a2f50393304071a275ae8205b1e)

- Moved login and logout example from README.md to the module documentation [7feb578](https://github.com/emirror-de/axum-gate/commit/7feb5782c47bb3bf09101b4c732834497590ec16)

- Removed security considerations part from README.md [230eff4](https://github.com/emirror-de/axum-gate/commit/230eff45a02d5ef854f2a6800c44eeabe5deecf1)

- Added initial PermissionSet implementation [4ba10e4](https://github.com/emirror-de/axum-gate/commit/4ba10e497c78dc4b1a4f922536151601d71e40be)

- Added integration test for authorization [0c9ac8d](https://github.com/emirror-de/axum-gate/commit/0c9ac8d250445fd9be22d0f18c320059caf42b5a)

- Added permission example to README.md [1f1721a](https://github.com/emirror-de/axum-gate/commit/1f1721aefaf935314a0b40e9fc582f7e66fe8cf9)

- Added Gate::grant_permissions that accepts a permission set [706e4cf](https://github.com/emirror-de/axum-gate/commit/706e4cf3b33837eab0cbdf1b1049126c05aac835)

- Fine-grained permission support in extend to roles and groups [6bc6ecf](https://github.com/emirror-de/axum-gate/commit/6bc6ecfa2b702a2211a8e1d16ee7795d61533052)

- Added initial, untested support for fine-grained permissions [1e35308](https://github.com/emirror-de/axum-gate/commit/1e35308857f6e3be874b066aa82ffe19f543b0c2)

- Updated README.md by default Gate behavior [7bd53d5](https://github.com/emirror-de/axum-gate/commit/7bd53d55f383ae7c4c81a2169f86f81713a7f186)

- Added hint that all requests are denied by default in Gate::new documentation [ecb0567](https://github.com/emirror-de/axum-gate/commit/ecb05671b36e391be67d1cc6437082856f53390d)

- Default behavior of a Gate is now to deny access [6cb6e90](https://github.com/emirror-de/axum-gate/commit/6cb6e90ebdc850d1e1b8f2cb4dcdbca6836738f4)

- The unauthorized future is created only once [da4c2d2](https://github.com/emirror-de/axum-gate/commit/da4c2d2a07c54937a870661fa33cbce748f39f79)

- Added permissions module in preparation for fine-grained permission support [4df38f0](https://github.com/emirror-de/axum-gate/commit/4df38f0ff8ebcaaa1a7b12ade9d8ce58e3d621ea)

- Removed unclear documentation for GateState [43fa629](https://github.com/emirror-de/axum-gate/commit/43fa6294c6fe04ea9f6fefd2c9aba57a4e07e834)

- Replaced GateState::default by ::new to prevent a hidden setting of not_before_time [212f796](https://github.com/emirror-de/axum-gate/commit/212f7963d6a7e8d8b98bd91e6d5fd863ca14776d)

- Removed unnecessary comment [86db51e](https://github.com/emirror-de/axum-gate/commit/86db51e72eea8ea805d189544b86248a88cf3976)

- Moved AccessScope from gate to separate module [3918f65](https://github.com/emirror-de/axum-gate/commit/3918f65aa2fb665ada2401f7820bcc16cd011b46)

- Revert "chore: Removed unnecessary result* from .gitignore" This reverts commit 72e64664171d5a92f3d305ed2d06c4d63b609ad3. [32e8243](https://github.com/emirror-de/axum-gate/commit/32e8243227c44390db2094e58902e8c4bbb6d70d)

- Added roaring dependency in preparation for fine-grained permission support [138d6f5](https://github.com/emirror-de/axum-gate/commit/138d6f500ef0703098bbfce8196913ff39d88fe2)

- Updated internal implementation of Gate in preparation for GateState and fine-grained permission support [f3deb88](https://github.com/emirror-de/axum-gate/commit/f3deb88e1f88ea45f02fae2ab880d2983c1db80b)

- Updated custom role and groups example [9628e1c](https://github.com/emirror-de/axum-gate/commit/9628e1c871155f2147c8e9209c557a7e434ad4cf)

- Added custom roles and groups example [dc536d9](https://github.com/emirror-de/axum-gate/commit/dc536d932cfe5219cd02fc18380cde02650b9b21)

- The iat claim is now set on creting the claims to Utc::now [0247304](https://github.com/emirror-de/axum-gate/commit/024730433a8fee34a4b40cf471b0ff8c5547127a)

- Removed README.md from sea-orm example [b81295b](https://github.com/emirror-de/axum-gate/commit/b81295b2cebafec3bb7b2c36d95bf73194f7dd12)

- Removed unnecessary result* from .gitignore [72e6466](https://github.com/emirror-de/axum-gate/commit/72e64664171d5a92f3d305ed2d06c4d63b609ad3)

- Added .env file to surrealdb example [fbf3775](https://github.com/emirror-de/axum-gate/commit/fbf3775ed7e7e2c721531998d8c44d460dd4ba4a)

- Added surrealdb example [6968a6b](https://github.com/emirror-de/axum-gate/commit/6968a6b9c0602ad314b507cf1a5b7a9cd8f6598e)

- Renamed sea-orm example, Now using the same storage instance for acc and sec [8753b65](https://github.com/emirror-de/axum-gate/commit/8753b6514605ccdd2484bdfa372ff980125e015b)

- Sea-orm example now working as well [83c55f1](https://github.com/emirror-de/axum-gate/commit/83c55f1773d65eb19e9295174adbddbd2731cd65)

- Models module of sea-orm is now public again [b77e5cc](https://github.com/emirror-de/axum-gate/commit/b77e5cc3fe3afa949e08784355afd4b25303e2cd)

- Updated README.md [073b361](https://github.com/emirror-de/axum-gate/commit/073b361e4afcdbbd48e44645ac15ad6e4123d9c4)

- Updated README.md [ba2e3c8](https://github.com/emirror-de/axum-gate/commit/ba2e3c876bce026ff0b9947161c5697c323e770a)

- Updated documentation [0903213](https://github.com/emirror-de/axum-gate/commit/09032132c9ee5e9737729936fd8cdf88002d9368)

- Updated surrealdb storage to new API [e48f8eb](https://github.com/emirror-de/axum-gate/commit/e48f8eb36fb0aa22edfc9459b19ef711198baa74)

- Secret now contains hashed value, Credentials stores only plain values [32a8324](https://github.com/emirror-de/axum-gate/commit/32a8324106ea3c8cf76a5a42fb1ac1902e85f4f6)

- **💥 BREAKING CHANGE:** Started outsourcing of secret hashing to secret itself [dbda109](https://github.com/emirror-de/axum-gate/commit/dbda109c64aefdad09908fa6fdf9104cb3ee9ed0)

- Support for sea-orm [bf2c216](https://github.com/emirror-de/axum-gate/commit/bf2c21654137708369e50b21c73263a2855c3475)

- Removed public-api module as it has been used for API design [80d7a26](https://github.com/emirror-de/axum-gate/commit/80d7a2696e39f277e8bff0607d85aae31859c838)

- Applied clippy fix [b692615](https://github.com/emirror-de/axum-gate/commit/b692615155eb356b6403b11669fb540037ab5379)

- Added surrealdb storage module [aa6c2f5](https://github.com/emirror-de/axum-gate/commit/aa6c2f589d4f7305c23ae1f1dc4806a442218c0e)

- Applied clippy fix [1e4ffcd](https://github.com/emirror-de/axum-gate/commit/1e4ffcdb87e958f25abb44aa1c926908540c93fd)

- Re-arranged use statements in distributed example [95ad7f2](https://github.com/emirror-de/axum-gate/commit/95ad7f261ccc084ac687a07ef958d0d1119891bb)

- Distributed example is now working with memory storage again [2a07049](https://github.com/emirror-de/axum-gate/commit/2a07049aecbf5cb5a9cd117181d8a3f48fd74926)

- Further working on the new API [588edee](https://github.com/emirror-de/axum-gate/commit/588edeec094840a04157b81c98181ad5c8e596c1)

- Still thinking about the new API [6bff05c](https://github.com/emirror-de/axum-gate/commit/6bff05c92d793dab8c7fe04e8a55386f401da606)

- Thinking about a complete new API [0d297af](https://github.com/emirror-de/axum-gate/commit/0d297af36dfe5cc2af8f0a7303bb965049ee1dc5)

- Added missing import statement in sea_orm storage module [da836fb](https://github.com/emirror-de/axum-gate/commit/da836fb3e50e1427c79166f3dafc6ba43b1d027c)

- Added SeaOrmStorage::new function [74bf982](https://github.com/emirror-de/axum-gate/commit/74bf982f02111dfe0a6244f7c73593acd7115e94)

- Implemented CredentialsVerifierService for SeaOrmStorage [1ede6a0](https://github.com/emirror-de/axum-gate/commit/1ede6a014b31079a24f7f33ef8f7b92f758b3ff1)

- Removed unnecessary Into<Vec<u8>> trait bound from route_handlers::login [7752642](https://github.com/emirror-de/axum-gate/commit/77526422c20052e892710555c90d1b589ba2eed8)

- Added missing link hint in sea-orm documentation [bf8fcf2](https://github.com/emirror-de/axum-gate/commit/bf8fcf26f2435a03629bf28a982f1800c6744dff)

- Removed unnecessary trait bounds of Passport [9bcf1c2](https://github.com/emirror-de/axum-gate/commit/9bcf1c28333b033de9860db4ae292500ee1ce6e2)

- Updated README.md [dabed5d](https://github.com/emirror-de/axum-gate/commit/dabed5d32d26afcfe474ac9d9285d55e5acfda49)

- Updated documentation [b9d77c2](https://github.com/emirror-de/axum-gate/commit/b9d77c20a03b34711d7b8f098857281409f0a332)

- Added issuer value to distributed example [48caaaa](https://github.com/emirror-de/axum-gate/commit/48caaaa2462da7349caf393e93cb74813e85ce1d)

- Gate now checks for issuer value [318ce27](https://github.com/emirror-de/axum-gate/commit/318ce2702617872ca59ec9023c22f68043431c80)

- Updated route_handler::login to the new JwtClaims API [72d5466](https://github.com/emirror-de/axum-gate/commit/72d54666bf537a4655e2735f19c78e029ccc2943)

- Added JwtClaims::has_issuer [a300fab](https://github.com/emirror-de/axum-gate/commit/a300fabc9e523c16470bddc58927bcbac7a428a7)

- JWT now requires issuer and expiration time [24e910f](https://github.com/emirror-de/axum-gate/commit/24e910f786bd03cb9ea9e36fbae4dbd04c79fb90)

- Role is now Debug [6312a42](https://github.com/emirror-de/axum-gate/commit/6312a42d5757ff05a17f7c478126c23151588b98)

- Passport in Gate now requires Debug [909b22f](https://github.com/emirror-de/axum-gate/commit/909b22f255e51a9d225656e0470940a84efc38a4)

- Account is now Debug [0c200c8](https://github.com/emirror-de/axum-gate/commit/0c200c856f3146a7ca08b95c740879397f45547f)

- Registered claim "iss" is now Option<String> instead of Option<HashSet<String>> [4b9fe9c](https://github.com/emirror-de/axum-gate/commit/4b9fe9c37ca30ab0428303acf772eae65b21de96)

- Updated Cargo.lock in distributed example [0dd424c](https://github.com/emirror-de/axum-gate/commit/0dd424c5a3cc7abe266d46a9daae6e1b495ef533)

- Updated distributed example [36a127c](https://github.com/emirror-de/axum-gate/commit/36a127cdce0333e449dd974974ab5c9243272556)

- Added hint about hashing to storage documentation [a1f8312](https://github.com/emirror-de/axum-gate/commit/a1f831206e3f1328317f442ae1eda2f59ba6bba6)

- Hashing is now done in sea-orm storage as well for credentials insertion and update [c6256be](https://github.com/emirror-de/axum-gate/commit/c6256be0f484f7c6ad19d45e7373bacb83c34c66)

- Updated sea-orm storage implementation to new API [b4c613f](https://github.com/emirror-de/axum-gate/commit/b4c613f03d02b068a59b791d454a843098710b15)

- Removed fully qualified syntax error in surrealdb storage [9a5d248](https://github.com/emirror-de/axum-gate/commit/9a5d248fcdd12d2a2eba5ed0c31e669c5990f034)

- Refactoring BasicRole [68e50a7](https://github.com/emirror-de/axum-gate/commit/68e50a7dad0ab8134d16955df3326efe9147011a)

- Removed unnecessary line from Cargo.toml [806f925](https://github.com/emirror-de/axum-gate/commit/806f9254cf2f53a7edda285a2b0b7086fb33acf0)

- Support for sea-orm, restructuring [6953e63](https://github.com/emirror-de/axum-gate/commit/6953e6365c0c499c86640ec699d5ff5734cb4345)

- Removed documentation warnings [7332e94](https://github.com/emirror-de/axum-gate/commit/7332e94c79ea6e149a0d00066e8f7d7c51b868fc)

- Added debug = false to profile.dev for faster debug builds [ce348dd](https://github.com/emirror-de/axum-gate/commit/ce348dd1a996ddeac1dffad30bef533eebe9bb50)

- Moved TableNames to storage module [7698656](https://github.com/emirror-de/axum-gate/commit/7698656e354d2040fa802233481bc595804d8cf4)

- MIT license [2ade8af](https://github.com/emirror-de/axum-gate/commit/2ade8af0e94b814755186fc71535592da837ef64)

- Maximum release level is now info [eb2e9bb](https://github.com/emirror-de/axum-gate/commit/eb2e9bb228750ee50fce2724ccaf8d2c82f5ccd0)

- Added BasicPassport::with_expires_at [46be5b4](https://github.com/emirror-de/axum-gate/commit/46be5b4013b1963b69764de3356362ffba9ec4bf)

- Removed unused PassportId [ae1ea87](https://github.com/emirror-de/axum-gate/commit/ae1ea870046a5ce2af481d453834b0e53f51292e)

- Password verification is now outsourced to surrealdb instance [bab23aa](https://github.com/emirror-de/axum-gate/commit/bab23aa7e28663c4145500338f8207bdf176396a)

- Updated flake.lock [fcdc37e](https://github.com/emirror-de/axum-gate/commit/fcdc37e21bf0f1e21210f270b6387a05343b6054)

- Updated examples to match the new API [c1ac42d](https://github.com/emirror-de/axum-gate/commit/c1ac42de84f5689d006ef9f9f8610547f3cb96d1)

- Added missing email field to surrealdb storage implementation [9086565](https://github.com/emirror-de/axum-gate/commit/908656561d448a8a597be786da67300044674bcc)

- Added BasicPassport::email field [a2c6297](https://github.com/emirror-de/axum-gate/commit/a2c6297266fe6e6294b3fce76a7b8b27800e1f1f)

- BasicPassport now uses generic for ID [0c069a0](https://github.com/emirror-de/axum-gate/commit/0c069a08ae39b6b56cbe55e05c5b4f83e7be2c74)

- Added Rust hint for logout example in README.md [751fe75](https://github.com/emirror-de/axum-gate/commit/751fe75537edf7c6cc31ecde90ae580497ba4c51)

- README is not included in lib.rs documentation [05cdad9](https://github.com/emirror-de/axum-gate/commit/05cdad9cb90df5e5b6effda9029a0270941bf9c5)

- Added security notices to README.md [a54eca6](https://github.com/emirror-de/axum-gate/commit/a54eca683aab666dcb40ef97007897a40152131f)

- Added LICENSE [47ce3c9](https://github.com/emirror-de/axum-gate/commit/47ce3c912588087be23b2a93e2d414220ea13bfb)

- Added license and notices [700c9b7](https://github.com/emirror-de/axum-gate/commit/700c9b721e2b8aef6ee53a2e3d5836749b22d7be)

- Added Rust hint to code examples in README.md [fc89a23](https://github.com/emirror-de/axum-gate/commit/fc89a23d0aad290fa2537dd3a21404f5f284f710)

- Added description from lib.rs to README.md [de326e6](https://github.com/emirror-de/axum-gate/commit/de326e6c549d0ce5ccce98621cc1f4cead424b56)

- Updated license to MIT/Apache-2 [d893fba](https://github.com/emirror-de/axum-gate/commit/d893fba229dd231745226028734115a86959271e)

- Removed comment from Cargo.toml [263f2c8](https://github.com/emirror-de/axum-gate/commit/263f2c86bdbc398bc130965edf49a13c5b9729ba)



### 🚲 Miscellaneous
- **💥 BREAKING CHANGE:** Removed DecodingKey and EncodingKey from prelude [bf35415](https://github.com/emirror-de/axum-gate/commit/bf354158530afdece3bf5794e0d4c284a00f3b41)

- **💥 BREAKING CHANGE:** Renamed update_permission_set to extend_permission_set [17dac0f](https://github.com/emirror-de/axum-gate/commit/17dac0fa468cee042eac5b9728ac121f2d8e6947)

- **💥 BREAKING CHANGE:** Some JWT properties are now mandatory [acae31b](https://github.com/emirror-de/axum-gate/commit/acae31b25134addf970ce2112a042733ef37c347)

- **💥 BREAKING CHANGE:** MemorySecretStorage::try_from is now ::from [6502a46](https://github.com/emirror-de/axum-gate/commit/6502a46611828c9d72df0f05306ea74b15db6388)

- **💥 BREAKING CHANGE:** CredentialsStorageService::store_credentials now returns the inserted credentials [08630e4](https://github.com/emirror-de/axum-gate/commit/08630e4bb300fb75fce658bb506fc449141198a3)

- **💥 BREAKING CHANGE:** Credentials::new now takes id as reference bounded by ToOwned [5268e6f](https://github.com/emirror-de/axum-gate/commit/5268e6f74b4410908da98f36990096cef9cee8a5)

- **💥 BREAKING CHANGE:** Removed requirement of full qualified syntax for PassportStorage implementation [f29e2af](https://github.com/emirror-de/axum-gate/commit/f29e2af59ad09fed194c8f5cee358c1b4c085123)



### 🛳 Features
- **💥 BREAKING CHANGE:** Added deny unwrap, expect and unsafe code [71de97c](https://github.com/emirror-de/axum-gate/commit/71de97cb3cc8e47f7ffb97dfc200506fbdd69153)

- Add tracing-based audit logging gated behind feature - Introduce audit module with spans/events (request, authz, JWT issues, account lifecycle) - Instrument gate, login/logout handlers, and account services - Remove unused audit functions and simplify module-level cfg gating - No secrets logged; structured fields only; feature-off has zero overhead *(audit)*  [3a87d25](https://github.com/emirror-de/axum-gate/commit/3a87d257ae2977c971d1a36ab6f7e6742267b230)

- Comprehensive security testing, code quality improvements, and documentation updates * Initial plan * Add comprehensive security tests and fix clippy warnings - Fixed all 10 clippy warnings for improved code quality - Added 44 new security-focused tests covering: * JWT manipulation and validation edge cases * Password hashing security and malformed input handling * Authorization bypass attempts and role escalation * Input validation against SQL injection and malformed data * Cookie security attributes and manipulation prevention * Storage layer isolation and concurrent access safety * Unicode/special character handling throughout system * Serialization security for sensitive data structures - Added timing attack awareness test (marked as ignore for CI stability) - Enhanced test coverage for edge cases and error conditions - All existing tests continue to pass Co-authored-by: emirror-de <30552361+emirror-de@users.noreply.github.com> * Add comprehensive security documentation and fix README example - Added SECURITY.md with detailed security considerations and best practices - Fixed README.md example to use consistent field names (user_id vs account_id) - Documented password security, JWT security, cookie security, and authorization security - Added guidance on timing attack considerations and security best practices - Included testing instructions for security test suites - All documentation examples validated and working Co-authored-by: emirror-de <30552361+emirror-de@users.noreply.github.com> * Address review comments: fix imports, remove unused code, use AccountDeleteService Co-authored-by: emirror-de <30552361+emirror-de@users.noreply.github.com> --------- Co-authored-by: copilot-swe-agent[bot] <198982749+Copilot@users.noreply.github.com> Co-authored-by: emirror-de <30552361+emirror-de@users.noreply.github.com> [7bc5e6e](https://github.com/emirror-de/axum-gate/commit/7bc5e6eda0870c627c2fd8c09bcdc29b452adc7b)

- **💥 BREAKING CHANGE:** Initial support for surrealdb [f807944](https://github.com/emirror-de/axum-gate/commit/f8079443898c7d459412f8c074e7b792377af48c)



## [0.1.0] - 2025-04-05
### 🏗 Refactoring
- **💥 BREAKING CHANGE:** API refinement [7eae628](https://github.com/emirror-de/axum-gate/commit/7eae62823128dbfa62ba41f7c7fd54a2415571c6)

- **💥 BREAKING CHANGE:** Some API improvements [835d7f6](https://github.com/emirror-de/axum-gate/commit/835d7f6543ca5289f28c8664f0f16904ae0c108a)



### 🐞 Bug Fixes
- Added Gate::with_cookie_template because it uses the wrong cookie otherwise [1b4f6fc](https://github.com/emirror-de/axum-gate/commit/1b4f6fcd65c5cc011d134daafddab1603d88274f)



### 📝 Other Changes
- Added CHANGELOG.md, Added cliff.toml [1c53048](https://github.com/emirror-de/axum-gate/commit/1c53048e779cdcc288ac422db6ffb149a3cc1d0a)

- Removed unnecessary as_bytes call in documentation [d1a6876](https://github.com/emirror-de/axum-gate/commit/d1a6876a537babdd4c7e765ed2bd2263e5bef3d6)

- Changed tokio feature full to sync [faf05d7](https://github.com/emirror-de/axum-gate/commit/faf05d7d5276d53e8259492f6c43b3997413d048)

- Removed auth_node example [a4e5995](https://github.com/emirror-de/axum-gate/commit/a4e5995ed6daf50ee692c92906ed0f574576038e)

- Added initial, small README.md [bb4c1ad](https://github.com/emirror-de/axum-gate/commit/bb4c1ada1055d0c9dadb11953f7e38f46ddd88d2)

- Updated keywords, categories and other properties in Cargo.toml [5df0d8b](https://github.com/emirror-de/axum-gate/commit/5df0d8ba78df6f2e0f4040ba0c43dc2f30747db9)

- Added licenses to deny.toml [0fdedf8](https://github.com/emirror-de/axum-gate/commit/0fdedf80a4670211bb7b6e61de761ac69a8880af)

- Updated some description [7c6acdc](https://github.com/emirror-de/axum-gate/commit/7c6acdc0a5321aec282eb2eda49871c54126979d)

- Added distributed example, remove auth_node example [71f0a40](https://github.com/emirror-de/axum-gate/commit/71f0a40ca1d2a2246662928e46a2c150373b9f5c)

- Added **/target to .gitignore [bfc967f](https://github.com/emirror-de/axum-gate/commit/bfc967fc01513d3dd4faf55564919ff4154479c8)

- It is now possible to pass a cookie template [72c247a](https://github.com/emirror-de/axum-gate/commit/72c247a471d18bcc766759f14e156496f76e81ea)

- Removed unrequired .as_bytes call from auth_node example [f5e8144](https://github.com/emirror-de/axum-gate/commit/f5e8144716f7c37d30f05a3fd1a4dbdac4ebbf83)

- Added second group to example [9c39047](https://github.com/emirror-de/axum-gate/commit/9c3904788296c6877962d32fef44b7aee1986f6d)

- Removed anonymous user because it does not make any sense [b8c4df0](https://github.com/emirror-de/axum-gate/commit/b8c4df0e8cd235cd79f6c8a898e5cf0bca4f0506)

- Added group scope to auth node example [15f27ff](https://github.com/emirror-de/axum-gate/commit/15f27ffaaec15257cbbd0eb04d4ee47722b65a8f)

- Moved authorization of minimum role into AccessScope [af4078b](https://github.com/emirror-de/axum-gate/commit/af4078b73f104c57e1d42cd42074636804b31a3e)

- Added small description of the crate [b4d17dc](https://github.com/emirror-de/axum-gate/commit/b4d17dcc45b1df5241d598721077e85d682678e1)

- It is now possible to have multiple users and groups in a Gate [e6481bb](https://github.com/emirror-de/axum-gate/commit/e6481bbdbc537886f71d98a6235328a0fd001eb5)

- Added group authorization [c0c7508](https://github.com/emirror-de/axum-gate/commit/c0c7508e2eb114d0ce1d60dd01ce7bcde415af6d)

- Moved roles::role_hierarchy to crate::access_hierarchy [c6a6606](https://github.com/emirror-de/axum-gate/commit/c6a6606f6b94190a96f5547040597fe98fb185d3)

- Added some documentation to the lib module [56ca140](https://github.com/emirror-de/axum-gate/commit/56ca1400a14bf6c409d365037f60dd829544f63c)

- Renamed example to auth_node [0b644f3](https://github.com/emirror-de/axum-gate/commit/0b644f34a98eee05214a61c22e787f8a61c25864)

- Removed some warnings [a911ebb](https://github.com/emirror-de/axum-gate/commit/a911ebbc7ca347d83ce2184fda059db604f594bc)

- Removed unused tracing-attributes dependency [a0ca70e](https://github.com/emirror-de/axum-gate/commit/a0ca70ed841f21455599874571b9a8d0e0cc5d48)

- Small refactor for cleanup, Fixed test [6b0e8d6](https://github.com/emirror-de/axum-gate/commit/6b0e8d62af4cd5bc4f08d5aff887ca7e71e2991f)

- Added secrets hasher service test [0b2a288](https://github.com/emirror-de/axum-gate/commit/0b2a288d184b7f42c698b4ef03c5c6d6e6ebd158)

- Initial working version with example [9cd1188](https://github.com/emirror-de/axum-gate/commit/9cd11881c61b28e9a4609feff22710c557bcdeb9)

- Initially able to create and deliver cookie with JWT [41d2c4d](https://github.com/emirror-de/axum-gate/commit/41d2c4dfdf3151615ec3de0ab5567b258d3e1b28)

- HashedCredentials need to be replaced by Credentials because hashing does not work for credential verification [25bfc9e](https://github.com/emirror-de/axum-gate/commit/25bfc9ec311c03154e34db2727c0de67d7f2ea18)

- Renamed module error to errors [010e4c8](https://github.com/emirror-de/axum-gate/commit/010e4c818e1fc04b2fb897ee2437f41b4ad9856d)

- Updated to development state [f2019b6](https://github.com/emirror-de/axum-gate/commit/f2019b6e3601e14e1b017c250e5d52565247b83e)

- Applied nixfmt, Updated to crane/master [d23b8bc](https://github.com/emirror-de/axum-gate/commit/d23b8bcfb9c31c3026b77169e8cdc166a6bd715f)

- Added role and role_hierarchy module [4a7fcee](https://github.com/emirror-de/axum-gate/commit/4a7fceee621da2c8fe656f85e3113cae51c96da7)

- Added initial project setup with Credentials struct [de6d7ff](https://github.com/emirror-de/axum-gate/commit/de6d7ff8c798019574051cb32e103a94d641a701)



### 🚲 Miscellaneous
- Implemented BasicGroup instead of using a pure String for consistency [10eb0b7](https://github.com/emirror-de/axum-gate/commit/10eb0b7c00383dfb84847959de5c8c7708b33448)



---

<!-- generated by git-cliff -->
