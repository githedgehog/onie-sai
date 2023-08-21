# onie-sai

This is an implementation of [SAI](https://github.com/opencomputeproject/SAI) to be used in [ONIE](https://github.com/opencomputeproject/onie).
The goal is to enable network installation from within ONIE over front panel ports of switches.
However, for that to work one needs to initialize the switch ASIC to be able to create virtual interfaces in the OS which can be used as normal network interfaces.

## Requirements

Note, that this has a few requirements:

- you have to have a switch which can be programmed using SAI obviously
- you need a (heavily) modified version of ONIE to be able to use this, for example the following (but this is not an exhaustive list)
    - for example using the libsai for Broadcom from the SONiC project requires glibc
    - this means your whole ONIE needs to be glibc based
    - additional kernel modules as required by your SAI implementation
    - additional configuration files etc.pp. as required by your SAI implementation
    - modified ONIE scripts / init scripts which set up networking
- unfortunately, Hedgehog cannot make its version of ONIE open source or public at this point in time

## Components

Here is a breakdown of all the components that you can find in this repository

### sai / sai-sys

These lie at the heart of everything.
These are Rust libraries for SAI interaction.
They are structured like typical C FFI libraries in Rust:
`sai-sys` is the crate which is generated from the SAI C header files with `bindgen`.
`sai` is the more high-level library on using SAI with Rust.

### onie-sai-rpc / onie-sai-common

These are simply supporting crates for `onie-sai`.
For example, the RPC crate defines an RPC protocol (using ttrpc) which is used between `onie-said` and `onie-saictl`.

### onie-said

The core piece which provides the feature to allow installation from front panel ports in ONIE.
The `onie-said` crate contains a daemon which brings up all front panel ports as networking interfaces in ONIE.

**NOTE:** This is a library crate as it is being used in `onie-sai` to create a "busybox"-style single binary application of `onie-said` and `onie-saictl` together to save on space within ONIE.

### onie-saictl

This is a command line control utility for `onie-said`.
Apart from being mostly a debugging utility it provides some features which are actively being used in Hedgehog ONIE to configure the system.
For example, it has a wait command which is used in the `onie-said` init script to wait for `onie-said` to be ready before it moves on to running the ONIE networking initialization scripts.
It also contains some commands which can extract the network configuration of a port based on the potentially received configuration over LLDP.

**NOTE:** This is a library crate as it is being used in `onie-sai` to create a "busybox"-style single binary application of `onie-said` and `onie-saictl` together to save on space within ONIE.

### onie-sai

This is the crate that pieces `onie-said` and `onie-saictl` together into a single binary and determines which program to run based on the arg0 of the program.
This is usually referred to as `busybox`-style linking of programs as it requires to symbolically link `onie-said` and `onie-saictl` to `onie-sai` for the actual programs to work.

### xcvr / xcvr-sys

These are libraries to interact with the transceiver modules on switches (SFP/QSFP eeprom and others).
They are designed similarly in fashion to the SAI libraries.
The C definition for `xcvr-sys` can be found in `include/xcvr`.
This is a C based API to allow for potentially other implementations in the future.

### xcvr-pddf / xcvr-cel-seastone2 / ...

These are all libraries of device specific implementations for the `xcvr` library.
The most interesting one here is the PDDF implementation.
If your device is PDDF based, then there is no need for other device specific transceiver libraries.
The implementation internally makes use of the sysfs interface of the PDDF kernel modules, and can be configured with a configuration file.

### xcvrctl

`xcvrctl` is a standalone utility that uses the `xcvr` library and its device specific implementation to read and write/set the transceiver settings.
This is technically not a required utility to boot from front panel ports in ONIE, but it is for sure a useful debugging utility from within the ONIE shell.
Note that this standalone utility does also not require that `onie-said` is running.

### onie-lldp

`onie-said` has a builtin feature to automatically configure a network interface based on LLDP packets.
This is a Hedgehog specific protocol and works with Hedgehog SONiC devices on the other end or `systemd-networkd` configured interfaces on the other end.
However, the problem of using VM based ONIE for testing is that it does not have a switch ASIC and therefore does not require `onie-said` - or more correctly, it cannot run `onie-said`.

For that matter, this is a breakout crate containing this LLDP feature for VM-based ONIEs (like `MACHINE=kvm_x86_64`) containing the `onie-lldpd` and `onie-lldpctl` utilities.

## Building

**NOTE:** This requires the Linux operating system to be compiled.

The `Makefile` should be pretty self-explanatory, and it has a `make help` target to explain all of its targets.
This is a Rust-based project, so you will need to have Rust installed.

To get started, one wants to run

```shell
make init
```

to download and the Broadcom SAI libraries from the SONiC project.
Note that if you want to use this with a different SAI implementation, then you need to initialize this by yourself.

Then to build all software, there are dedicated Makefile targets for every specific piece, but to compile everything simply run:

```shell
make build
```
