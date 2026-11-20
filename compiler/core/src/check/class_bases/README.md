# class_bases

Embed one owned base in a derived payload. Rewrite inherited bodies with their original lexical access and the complete receiver. Base construction and destruction must visit each component exactly once.

Module entry: [class_bases](../class_bases.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [ancestry](ancestry.rs) | `ancestors` |
| [fields](fields.rs) | `inherited_field_path` |
| [preparation](preparation.rs) | `prepare` |
| [rewrite](rewrite.rs) | `rewrite_block`, `rewrite` |
