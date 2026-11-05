# Local memory image 1.0

English · [Tiếng Việt](../vi/memory.md)

[Documentation index](index.md)

The [memory image schema](../../contracts/memory/memory-image.schema.json) defines an editor-local initial byte image. A generic MEMORY module stores it in `properties.memory`.

~~~json
{"version":"1.0","word_bits":8,"depth":4,"data":[0,1,65,255]}
~~~

Depth is 1–256, and each byte is an integer from 0 through 255. Consumers also enforce `data.length === depth` as a semantic check. The browser checks both before changing the graph. New instances start with 32 explicit zero bytes. Each instance owns its array. Hex Editor accepts space-separated two-digit bytes, loads and saves image JSON, and applies undo and redo through circuit history.

`ADDR`, `DATA_IN`, `DATA_OUT`, `WE`, `OE` and `CLK` are functional editor ports. They do not identify pins on a selected chip. Executable memory would require simulator rules for addressing, read and write timing, reset and persistence. The local image describes initial configuration. Physical RAM reads, simulated captures and SDRAM hardware are outside this contract. See the [original memory guide](../../contracts/memory/README.md).
