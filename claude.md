# Project Overview

## Repository Structure
This repository contains two implementations of an SSH connection management tool:

### 1. Rust Implementation (`./rust/`)
**Status**: Complete and feature-rich
**Key Features**:
- Interactive SSH host management
- Pre-command execution system
- SSH config file editing
- RSA key generation
- Cross-platform support (Windows, macOS, Linux)
- Comprehensive error handling

### 2. C++ Implementation (`./cpp/`)
**Status**: Basic implementation
**Key Features**:
- Interactive host selection
- SSH config parsing
- Windows-focused implementation
- External Inquirer library for UI

## Architecture Comparison

### Rust Version Advantages
- **More Features**: Pre-commands, key generation, config editing
- **Better Cross-platform**: Full OS support with proper path handling
- **Robust Error Handling**: Comprehensive error management
- **Modern Dependencies**: Uses modern Rust crates (inquire, regex, serde_json)

### C++ Version Characteristics
- **Simplicity**: Clean, minimal implementation
- **External UI Library**: Uses third-party Inquirer library
- **Windows Focus**: Primarily designed for Windows environment
- **Basic Functionality**: Core SSH connection features only

## Technical Stack

### Rust Stack
- **Language**: Rust
- **Dependencies**: inquire, regex, serde_json
- **Pattern**: Modular architecture with clear separation of concerns

### C++ Stack
- **Language**: C++
- **Dependencies**: Standard Library + external Inquirer
- **Pattern**: Object-oriented with Option class

## Recommended Usage
- **For full features**: Use Rust implementation
- **For simple connections**: C++ implementation works
- **Cross-platform needs**: Rust version is preferred
- **Windows-only**: Either implementation works

Both implementations serve the same core purpose but with different feature sets and implementation approaches.