# Vec growth on Apple Silicon

Run `python3 performance/vec-growth/measure.py path/to/clust` with each compiler build on the same host. The harness builds a native optimized executable for each element type and runs it nine times. It counts global allocator calls from entry through cleanup and reports median elapsed time over the same interval, excluding compilation and process startup.

| 32,768 pushes | Before allocations | After allocations | Before median | After median |
| --- | ---: | ---: | ---: | ---: |
| u8 | 28 | 15 | 842 µs | 325 µs |
| Item with a destructor | 32,796 | 32,783 | 4,004 µs | 1,153 µs |

These measurements include element creation and destruction. Each of the 13 growth steps now allocates one destination and transfers one initialized prefix directly. Per-object allocations are included for Item. Timings are from one local machine and will vary with compiler version and load.
