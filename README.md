# WireBox

### TONEX and AmpliTube 5 on Linux

WireBox is an unofficial Linux compatibility project designed to make **IK Multimedia TONEX** and **AmpliTube 5** available on Linux.

Inspired by projects such as Sober, WireBox aims to provide a simple, native-feeling Linux experience without requiring users to manually configure a traditional Windows compatibility environment.

The goal is simple:

**Install WireBox → Install TONEX or AmpliTube 5 → Play.**

> [!WARNING]
> **WireBox is an unofficial, community-developed project.**
>
> WireBox is **not affiliated with, endorsed by, sponsored by, maintained by, or otherwise associated with IK Multimedia in any way**.
>
> TONEX, AmpliTube, IK Multimedia, and all related trademarks, product names, and logos are the property of their respective owners.
>
> WireBox does not provide ownership or licensing of IK Multimedia software. Users are responsible for obtaining and properly licensing any proprietary software used with WireBox.
>
> **Use WireBox at your own risk.**

## What is WireBox?

WireBox is designed to provide a dedicated Linux compatibility environment for IK Multimedia's TONEX and AmpliTube 5.

Rather than recreating these applications or developing an alternative guitar amp and effects platform, WireBox focuses on making the existing Windows applications usable on Linux.

WireBox is intended to manage the compatibility environment, application configuration, and Linux integration required to run supported applications while keeping the underlying complexity out of the user's way.

### The Goal

```text
Linux
  ↓
WireBox
  ↓
TONEX / AmpliTube 5
```

The long-term goal is to make using TONEX and AmpliTube on Linux feel as close as possible to using them on Windows.

## Current State of the Project

WireBox is currently in **early development**.

The project is currently focused on establishing the core architecture and Linux integration required to support TONEX and AmpliTube 5.

### Current Development

* Linux application foundation
* Audio device detection
* Linux audio integration
* Core application architecture
* Initial compatibility work

### Planned

* TONEX support
* AmpliTube 5 support
* Application installation
* Application management
* Low-latency audio
* Audio interface integration
* MIDI and controller support
* Configuration management
* Persistent application data
* Application updates

> **Note:** WireBox is not yet a complete replacement for a traditional Windows compatibility setup. Compatibility and functionality will improve as development progresses.

## Supported Applications

### TONEX

WireBox aims to support the desktop version of **IK Multimedia TONEX**.

Planned functionality includes:

* TONEX application support
* Tone Model playback
* Tone Model management
* Presets
* Audio input/output
* Low-latency guitar processing
* MIDI controllers
* TONEX hardware integration where technically feasible

### AmpliTube 5

WireBox also aims to support **IK Multimedia AmpliTube 5**.

Planned functionality includes:

* AmpliTube 5 application support
* Amplifier models
* Effects
* Cabinets
* Signal chains
* Presets
* Audio input/output
* MIDI controllers
* Low-latency guitar processing

## Why WireBox?

Running Windows audio applications on Linux can require significant manual configuration.

Users may need to deal with:

* Wine prefixes
* Wine configuration
* Dependencies
* Audio routing
* Windows DLLs
* Registry configuration
* Application-specific workarounds
* Separate launch scripts
* MIDI configuration
* Controller configuration

WireBox aims to handle as much of this complexity as possible.

Instead of:

```text
Wine
 ↓
Prefix
 ↓
Dependencies
 ↓
Configuration
 ↓
TONEX
```

WireBox aims for:

```text
WireBox
 ↓
TONEX
```

The underlying compatibility technology may still involve components such as Wine or other compatibility technologies, but WireBox is intended to manage that complexity for the user.

## Audio

Low-latency audio is an important part of WireBox because TONEX and AmpliTube are intended to be used for real-time guitar processing.

WireBox aims to integrate with the Linux audio stack while providing a reliable and low-latency experience.

The intended audio workflow is:

```text
Guitar
   ↓
Audio Interface
   ↓
Linux Audio Stack
   ↓
TONEX / AmpliTube 5
   ↓
Linux Audio Stack
   ↓
Audio Interface
   ↓
Headphones / Speakers
```

WireBox sits around the application and compatibility environment rather than acting as the guitar-processing engine itself.

Audio performance will depend on the user's hardware, Linux audio configuration, application version, and system configuration.

## Linux Support

WireBox is built specifically for Linux, with broad distribution support as a core goal of the project.

### Initial Support

Our initial focus is on:

* **All Arch-based distributions**
* **Gentoo**
* **Fedora**

This includes distributions such as:

* Arch Linux
* CachyOS
* EndeavourOS
* Manjaro
* Other Arch-based distributions

### Future Support

As WireBox matures, we plan to expand support to additional major Linux distributions and ecosystems, including:

* Debian
* Ubuntu
* Linux Mint
* openSUSE
* Other major Linux distributions


Our goal is to make WireBox as distribution-agnostic as possible while maintaining a reliable and consistent experience across supported platforms.

Distribution support may vary depending on system libraries, package availability, audio configuration, and the packaging method used by WireBox.

## Future Projects

As the WireBox ecosystem matures, we may eventually develop a separate project focused on bringing the same concept to BSD-based operating systems.

This would be a separate project from WireBox rather than an extension of WireBox's Linux support.

## Architecture

WireBox is designed as a dedicated compatibility environment rather than a replacement for TONEX or AmpliTube.

The architecture is intended to separate application-specific compatibility requirements from the host Linux system and provide a consistent environment for supported applications.

A simplified model is:

```text
┌─────────────────────────────────┐
│              Linux              │
├─────────────────────────────────┤
│             WireBox               │
│                                 │
│  Compatibility Environment      │
│  Application Management         │
│  Configuration                  │
│  Linux Integration              │
│  Audio Integration              │
├─────────────────────────────────┤
│       TONEX / AmpliTube 5       │
└─────────────────────────────────┘
```

The internal architecture is still under development and may change as compatibility work progresses.

## Installation

WireBox is currently under active development and does not yet provide a finalized end-user installation workflow.

Once WireBox reaches a sufficiently stable release, the intended experience will be:

```text
1. Install WireBox
2. Launch WireBox
3. Install or select TONEX / AmpliTube 5
4. Configure your audio interface
5. Launch the application
6. Play
```

Installation and application-management functionality will be documented here as it becomes available.

## Building

### Prerequisites

Development requirements currently include:

* Rust
* Cargo
* Git
* A supported Linux distribution
* Required system libraries

Additional dependencies may be required as development progresses.

### Build

Clone the repository:

```bash
git clone https://github.com/KaelixDevs/WireBox.git
cd WireBox
```

Build WireBox:

```bash
cargo build --release
```

Run the development build:

```bash
cargo run
```

## Development

Check the project:

```bash
cargo check
```

Format the source:

```bash
cargo fmt
```

Run tests:

```bash
cargo test
```

Run Clippy:

```bash
cargo clippy
```

## Roadmap

### Phase 1 — Foundation

* [x] Project architecture
* [x] Linux runtime
* [ ] Application management
* [ ] Configuration system
* [ ] Basic launcher
* [ ] Logging and diagnostics

### Phase 2 — TONEX

* [ ] TONEX installation
* [ ] TONEX launching
* [ ] TONEX compatibility
* [ ] Audio input/output
* [ ] Preset support
* [ ] MIDI support
* [ ] Controller support

### Phase 3 — AmpliTube

* [ ] AmpliTube 5 installation
* [ ] AmpliTube 5 launching
* [ ] AmpliTube 5 compatibility
* [ ] Audio input/output
* [ ] Preset support
* [ ] MIDI support
* [ ] Controller support

### Phase 4 — Audio

* [ ] PipeWire integration
* [ ] ALSA integration
* [ ] Low-latency configuration
* [ ] Automatic audio-device detection
* [ ] Buffer configuration
* [ ] Sample-rate configuration
* [ ] Audio diagnostics

### Phase 5 — User Experience

* [ ] Graphical installer
* [ ] Automatic configuration
* [ ] Application updates
* [ ] Runtime management
* [ ] Logs and diagnostics
* [ ] Per-application configuration
* [ ] Desktop integration

### Phase 6 — Advanced Compatibility

* [ ] Improved Windows API compatibility
* [ ] Hardware integration
* [ ] Advanced MIDI functionality
* [ ] Controller integration
* [ ] Plugin compatibility where technically feasible
* [ ] Additional IK Multimedia software where technically feasible

## Compatibility

WireBox does **not** guarantee compatibility with every version of TONEX or AmpliTube.

Compatibility may be affected by:

* Application version
* Linux distribution
* Kernel version
* CPU
* GPU
* Audio interface
* PipeWire/ALSA configuration
* Windows compatibility requirements
* DRM and licensing systems
* Online services
* Hardware-specific drivers

Compatibility may also change between application releases.

If an application works on one system but not another, please provide as much information as possible when reporting the issue.

## Reporting Issues

When opening an issue, include:

* Linux distribution
* Kernel version
* WireBox version or commit
* TONEX/AmpliTube version
* CPU
* GPU
* Audio interface
* PipeWire/ALSA configuration
* Relevant logs
* Steps to reproduce the problem

Please do **not** include:

* License keys
* Passwords
* Account credentials
* Authentication tokens
* Other sensitive information

## Contributing

Contributions are welcome.

To contribute:

1. Fork the repository.
2. Create a branch for your changes.
3. Make your changes.
4. Test them on Linux.
5. Submit a pull request.

For larger architectural changes, opening an issue before implementing the change is recommended.

## Disclaimer

WireBox is an independent, unofficial open-source project.

**WireBox is not affiliated with IK Multimedia in any way.**

It is not endorsed by, sponsored by, maintained by, or otherwise associated with IK Multimedia.

**TONEX**, **AmpliTube**, **IK Multimedia**, and all related trademarks, product names, and logos are the property of their respective owners.

WireBox does not claim ownership of, redistribute, or provide licenses for proprietary IK Multimedia software.

Users are responsible for obtaining and properly licensing any proprietary software they use with WireBox.

## The Goal

Linux is an increasingly capable platform for music production and guitarists, but many popular Windows-only applications remain difficult to use.

WireBox exists to close that gap.

No complicated setup.

No manually maintained compatibility environments.

No endless configuration guides.

Just:

```text
Install WireBox
     ↓
Install TONEX / AmpliTube 5
     ↓
Connect your interface
     ↓
Play
```

### The great guitar software migration to Linux.

**WireBox**
