# 🕵️ Project Agents & Quality Guardrails

To ensure **ODFerrous** remains the gold standard for ODF manipulation in Rust, every contribution must satisfy the following four conceptual agents. These agents represent the high-bar for code quality, safety, and specification compliance.

## 1. The Validator (Compliance Agent)

**Focus:** 100% ODF 1.4 Standards Compliance.

* **Primary Directive:** Ensure 100% ODF 1.4 schema compliance for both `.zip` packages and `.xml` flat files.


* **Flat-File Parity:** The agent must verify that a document saved as `.odt` (zipped) and `.fodt` (flat) contains semantically identical XML structures.

* **Standards:**
* **Schema Enforcement:** All serialization tests must validate their output against `./OpenDocument-v1.4-schema.rng` using a Relax NG validator (e.g., `jing` or a Rust-native wrapper).
* **Namespace Integrity:** Ensure correct usage of namespaces (`office:`, `text:`, `table:`, etc.) as defined in the ODF 1.4 spec.
* **Strict Round-tripping:** A document parsed and then re-serialized must remain schema-compliant, even if non-essential metadata is stripped.




* **Agent Tools:** `libxml2` (xmllint), custom `tests/schema_validation.rs`.

---

## 2. The Sentry (Testing & Mutation Agent)

**Focus:** Logic Integrity and Resilience.

* **Primary Directive:** Ensure that the test suite isn't just "covering" code, but actually *validating* it.


* **Standards:**
* **Mutation Testing:** We use **`cargo-mutants`**. If a line of code can be changed (e.g., swapping `>` for `<`) and the tests still pass, the Sentry rejects the PR.
* **Coverage Floor:** 95%+ line coverage is the baseline. We prioritize path coverage for complex XML state machines.


* **Fuzzing:** Use `cargo-fuzz` on the parser to ensure malformed ODF files do not cause panics or memory leaks.


* **Agent Tools:** `cargo-mutants`, `cargo-tarpaulin`, `cargo-fuzz`.

---

## 3. The Architect (API & Rust Quality Agent)

**Focus:** Idiomatic Design and Performance.

* **Primary Directive:** Maintain a "type-safe" and "ergonomic" API that prevents users from creating invalid documents at compile-time whenever possible.

* **Unified Abstraction:** The API must provide a seamless entry point (e.g., `OdfDocument::open()`) that auto-detects if the source is a ZIP container or a Flat XML file.


* **Standards:**
* **Zero-Panic Policy:** The library should return `Result` for all fallible operations. `unwrap()` is forbidden in the `src/` directory.
* **API Surface:** Keep the public API lean. Use `pub(crate)` for internal utilities to avoid "API bloat."
* **Documentation:** Every public-facing function must have a `# Examples` section and a `# Errors` section explaining when it might fail.
* **Performance:** Flat file parsing should skip ZIP decompression overhead while maintaining the same high-performance `quick-xml` pipeline.



* **Agent Tools:** `cargo clippy -- -D warnings`, `cargo-public-api`.

---

## 4. The Chronicler (Readability Agent)

**Focus:** Developer Experience and Specification Linking.

* **Primary Directive:** Bridging the gap between raw XML and the Rust developer.


* **Standards:**
* **Spec-Linking:** Doc comments for complex types must link directly to the relevant section of the ODF 1.4 specification.
* **Tutorial-Driven Readme:** The main `README.md` must remain up-to-date with a "Quick Start" that actually works.


* **Code Style:** Strict adherence to `rustfmt` with `unstable_features` for better readability (e.g., `imports_granularity = "Crate"`).


* **Agent Tools:** `cargo test --doc`, `markdownlint`.

---

### The Agent's Final Checklist

| Requirement | Responsible Agent | Verification Command |
| --- | --- | --- |
| **Schema Validation** | Validator | `cargo test --test schema_compliance` |
| **No Survived Mutations** | Sentry | `cargo mutants` |
| **Strict Linting** | Architect | `cargo clippy` |
| **Doc-Test Accuracy** | Chronicler | `cargo test --doc` |

> [!IMPORTANT]
> Failure to satisfy **The Validator** is a hard block. Even if the code is beautiful and the tests pass, if the output XML deviates from `./OpenDocument-v1.4-schema.rng`, the PR will not be merged.
