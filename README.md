# Shopping Cart WebAssembly Demo

This repository demonstrates implementing a shopping cart microservice using WebAssembly (Wasm) with multiple runtimes and programming languages. Goal is to compare a matured Java Spring Boot application with a similar WebAssembly approach.

## Overview

This project implements the same shopping cart microservice in multiple languages:

- **Languages**: Java, Rust, Go, TypeScript
- **WebAssembly Runtimes**: 
  - [Spin](https://developer.fermyon.com/spin/index) (Fermyon's WebAssembly runtime)
  - [WasmCloud](https://wasmcloud.com/) (Distributed WebAssembly platform)

## Repository Structure

- `/carts-java/` - Matured Java Spring Boot implementation of the shopping cart service
- `/spin-demo/` - Fermyon Spin-based implementations in multiple languages:
  - `/spin-demo/carts-rs/` - Rust implementation
  - `/spin-demo/carts-go/` - Go implementation
  - `/spin-demo/carts-ts/` - TypeScript implementation
  - `/spin-demo/order-be/` - Order backend service
- `/wasmcloud-demo/` - WasmCloud implementation
- `/k8s/` - Kubernetes deployment manifests
- `/ansible/` - Ansible playbooks for deployment
- `/terraform/` - Terraform configurations
- `/compose/` - Docker Compose files for local development and testing

## API Endpoints

All implementations expose the same RESTful API for shopping cart operations:

- `GET /carts-{impl}/{cartId}` - Get cart information
- `GET /carts-{impl}/{cartId}/items` - List items in cart
- `POST /carts-{impl}/{cartId}/items` - Add item to cart
- `PATCH /carts-{impl}/{cartId}/items` - Update item in cart
- `DELETE /carts-{impl}/{cartId}/items` - Delete all items from cart
- `DELETE /carts-{impl}/{cartId}/items/{itemId}` - Delete specific item from cart

Where `{impl}` is one of: `java`, `rs` (Rust), `go`, `ts` (TypeScript), or `wcrs` (WasmCloud Rust).

## Prerequisites

- Docker and Docker Compose
- Kubernetes cluster (for K8s deployments)
- Ansible
- Terraform
- Language-specific toolchains (depending on which implementation you want to build)
  - Rust toolchain with wasm32-wasi target
  - Go with TinyGo
  - Node.js and npm
  - Java with Maven

## License

This project is open source and available under the [MIT License](LICENSE).