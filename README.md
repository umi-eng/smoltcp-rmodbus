# `smoltcp` implementation for `rmodbus`

## Usage

Add an entry to your `Cargo.toml`:

```toml
[dependencies]
smoltcp-rmodbus = "0.1.0"
```

## Features

- `defmt-03`: Adds `defmt::Format` derives for all types and enables the equivalent feature for all dependencies that support it.

## Minimum supported Rust version

There will not yet be any guarantees for the minimum supported Rust version until this crate reaches maturity.

## References

- [MODBUS Application Protocol Specification v1.1b3 (PDF)](http://modbus.org/docs/Modbus_Application_Protocol_V1_1b3.pdf)
- [MODBUS over serial line specification and implementation guide v1.02 (PDF)](http://modbus.org/docs/Modbus_over_serial_line_V1_02.pdf)
- [MODBUS Messaging on TCP/IP Implementation Guide v1.0b (PDF)](http://modbus.org/docs/Modbus_Messaging_Implementation_Guide_V1_0b.pdf)
