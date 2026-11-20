# integer

Apply width and signedness at every integer operation. Checked and wrapping policies share bit representations while division and shifts enforce their own restrictions.

Module entry: [integer](../integer.rs).

| Source | Responsibilities or entry points |
| --- | --- |
| [arithmetic](arithmetic.rs) | `negate`, `bit_not`, `compare`, `binary`, `const_binary` |
| [division](division.rs) | `divide`, `shift` |
