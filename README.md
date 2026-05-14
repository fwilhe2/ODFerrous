# ODFerrous

**ODFerrous** is a high-performance, type-safe Rust library designed to parse, manipulate, and serialize OpenDocument Format (ODF) files, including `.odt` (text), `.ods` (spreadsheets), and `.odp` (presentations).

## What’s in a Name?

The name **ODFerrous** is a two-fold pun designed to fit perfectly into the Rust ecosystem:

1. **ODF**: The acronym for the **OpenDocument Format**, the ISO-standardized XML-based file format for office applications.
2. **Ferrous**: From the Latin *ferrum* (iron). In chemistry, ferrous refers to iron-containing compounds. In the Rust community, "ferrous" and "oxide" are common naming tropes used to signal that a project is built with **Rust** (iron oxides).

Pronounced *oh-def-er-us*, it sounds like a sturdy, industrial-grade solution for your office document needs.

How to build and test
```bash
# Build library and CLI
make

# Run tests (unit + proptest)
make test

# Generate sample files (writes sample.fodt and sample.odt)
cargo run --bin generate

# Open sample.odt or sample.fodt with LibreOffice or run
make validate
```
