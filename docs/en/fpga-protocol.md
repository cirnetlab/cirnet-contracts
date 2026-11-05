# FPGA control and protocol boundaries

English · [Tiếng Việt](../vi/fpga-protocol.md)

[Documentation index](index.md)

The [logical control schema](../../contracts/fpga-protocol/control.schema.json) defines a transport-independent `1.0-scaffold` envelope with `protocol_version`, `sequence` and `operation`. The Mini PC Hardware Service owns `FpgaTransport`. Browser and API reach the hardware boundary through the service. See the pinned [Lab-First architecture](https://github.com/cirnetlab/cirnet-remote/blob/0d038c7aed15798696223558b4e58f5ffe926aa9/docs/ARCHITECTURE.md).

| Operation | RTL test ID | Required state or effect |
| --- | ---: | --- |
| SAFE_STATE | 0 | Disable drive and routes, keeping FAULT latched. |
| SCAN_BEGIN | 1 | SAFE → SCANNING with drive disabled. |
| PREPARE | 2 | Requires successful completed scan, `safe_condition`, `plan_valid` and a nonzero clock tick setting. |
| ROUTE_APPLY | 3 | PREPARED + valid safe plan + `routing_ready` → ROUTED. |
| POWER_UP | 4 | Requires ROUTED, `protection_ready`, `watchdog_armed`, `routing_ready`, `safe_condition` and `plan_valid`. |
| ARM | 5 | Requires POWERED with a live protected permit. |
| START | 6 | Requires ARMED with a live protected permit and prepared clock. |
| STOP | 7 | Disable drive and routes, keeping FAULT latched. |
| RESET | 8 | Clear state and clock after fault inputs clear. Invalidate the earlier sequence. |
| STATUS | 9 | Acknowledge current status without enabling outputs. |

These IDs belong to RTL simulation and the logical control boundary. Invalid sequences fail to SAFE, with FAULT latched until RESET. The abstract errors are OK 0, INVALID_STATE 1, SAFETY_BLOCK 2, SCAN_FAILED 3, INVALID_COMMAND 4 and INTERLOCK_FAULT 5. Hardware RPC v2 has a separate error enum. `scan_success` means a usable observation. Topology correctness does not gate an otherwise safe wiring observation. The compiler sends approved endpoint and instrument allocations, never instructions to repair internal student wiring.

## Transport and register domains

The intended path is Mini PC → Hardware Service → FpgaTransport → selected Serial or USB-SPI adapter. Future USB3 or Ethernet adapters preserve the same bounded exchange and safety semantics. Current `SerialFpgaTransport` and `UsbSpiFpgaTransport` raise `HARDWARE_UNAVAILABLE`, including for safe-state requests. Virtual simulation does not implement a physical decoder.

Physical framing is `TBD_BY_HARDWARE_DESIGN`. It needs exact bridge and device parts, datasheets, pins and reset polarity, byte order, frame sizes and version negotiation, replay handling, corruption detection, timeouts and an independently enforced safe state and watchdog. Lost, corrupt or late commands must leave drive disabled.

The conceptual `1.0-reserved` register domains cover identification and capabilities, safe state/control/fault/watchdog, topology scan status, approved endpoint routing, protected drive status, clock configuration, Logic Analyzer, Scope 2CH, Generator 2CH, Trigger, Timestamp, SDRAM Arbiter and CDC handshakes. Addresses, widths, pins and binary ABI remain open. The scaffold exposes synchronous handshakes and permits. Fault and watchdog can asynchronously inhibit outputs and latch FAULT in the abstract controller. Physical synchronization, electrical cutoff, reset and timing require board-specific evidence.

## Version selection

CircuitGraph 1.0/1.1 and additive 1.2, Assignment 1.0 and additive 2.0, Physical Lab 1.0 and local Hardware RPC 2.0 have independent versions. The logical envelope, register domains, binary framing, firmware and hardware profile are separate domains as well. Absent physical drivers still report FPGA protocol `UNAVAILABLE` in StationCapabilities. Implemented, measured and compatible hardware is required before advertising physical capabilities.

See the original [command table](../../contracts/fpga-protocol/commands.md), [packet format](../../contracts/fpga-protocol/packet-format.md), [register map](../../contracts/fpga-protocol/register-map.md) and [version matrix](../../contracts/fpga-protocol/versions.md).
