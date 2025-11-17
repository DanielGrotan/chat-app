# Chat App

![Rust](https://img.shields.io/badge/language-Rust-orange)

A simple client-server chat application built in **Rust**, using **Slint** for the GUI.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Usage](#usage)
- [Dependencies](#dependencies)

## Overview

This project is a basic chat application used to learn how to build a real-time messaging system in Rust. It consists of:

- A **server** that handles message routing and communication between clients
- A **client** with a GUI built using the Slint toolkit
- Shared components (in a `common` folder) for data models and protocol definitions.

## Features

- Real-time messaging between multiple clients
- Simple user interface
- Modular architecture: clear separation between client, server, and shared logic
- Cross-platform (Rust + Slint)

## Architecture

- **Server**: Listens for client connections, forwards messages, and manages connected users
- **Client**: GUI application using Slint for the interface, communicates with the server
- **Common**: Shared Rust code (e.g. message types, encoding, decoding) used by both client and server

## Getting Started

### Prerequisites

- Rust (with `cargo`)
- (Optional) Slint toolchain for GUI development

### Setup

1. Clone the repository:

```bash
git clone https://github.com/DanielGrotan/chat-app.git
cd chat-app
```

2. Build the project:

```bash
cargo build
```

## Usage

### 1. Run the server

```bash
cargo run --release -p server
```

This will start the chat server and listen for incoming client connections on `localhost:8080`.

### 2. Run the client

In a separate terminal/window:

```bash
cargo run --release -p client
```

This will launch the GUI chat client.

### 3. Chat

- Open multiple clients to simulate chat between users
- Enter the server address (e.g. `localhost:8080`), username, and start sending messages

## Dependencies

- **Rust** — core programming language  
  [rust-lang.org](https://www.rust-lang.org/)

- **Slint** — declarative GUI toolkit for Rust  
  [slint.dev](https://slint.dev/)

- **tokio** — asynchronous runtime for networking
  [crates.io](https://crates.io/crates/tokio)

- **bincode** — binary serialization for Rust  
  [crates.io](https://crates.io/crates/bincode)
