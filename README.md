<h1 align="center">SNS Quicknode</h1>
<br />
<p align="center">
<img width="250" src="https://i.imgur.com/nn7LMNV.png"/>
</p>
<p align="center">
<a href="https://twitter.com/bonfida">
<img src="https://img.shields.io/twitter/url?label=Bonfida&style=social&url=https%3A%2F%2Ftwitter.com%2Fbonfida">
</a>
</p>
<br />

<p align="center">
<strong>
SNS Quicknode
</strong>
</p>

<div align="center">
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
</div>

## Overview

SNS-Quicknode is a Rust-based application that provides a web server for interacting with the Solana Name Service (SNS). It is designed to be used as an add-on for QuickNode, a platform that provides fast, reliable, and scalable blockchain nodes. More information about the Solana Name Service add-on can be found on the [QuickNode Marketplace](https://marketplace.quicknode.com/add-on/solana-name-service).

The application uses Actix-web for the web server and interfaces with a PostgreSQL database for data storage. It is containerized using Docker and can be deployed using Docker Compose.

## Getting Started

To get started with the project, you need to have Rust, Docker, and Docker Compose installed on your machine.

1. Clone the repository.
2. Build the Docker image using the provided Dockerfile.
3. Run the Docker container using Docker Compose.

## Code Structure

The codebase is organized into several modules:

- `src/lib.rs`: This is the main library file where the Actix-web server is set up and the routes are defined.
- `src/main.rs`: This is the entry point of the application.
- `src/config.rs`: This module handles the configuration of the application, reading from environment variables.
- `src/db.rs`: This module handles the connection to the PostgreSQL database.
- `src/error.rs`: This module defines the custom error type used throughout the application.
- `src/matrix.rs`: This module handles the interaction with the Matrix chat service.
- `src/provisioning.rs`: This module defines the provisioning routes and their handlers.
- `src/sns.rs`: This module defines the SNS routes and their handlers.

## Environment Variables

The application uses several environment variables for configuration. These are defined in the src/config.rs file.

## Docker Deployment

The application is containerized using Docker. The Dockerfile is provided in the root directory of the project. The Docker image is built using the build_container.sh script.

## Database Schema

The database schema is defined in the src/sql/schema.sql file. The application uses a single table named provisioning.

## Integration with QuickNode

This application is designed to be used as an add-on for QuickNode. To use it, you need to have a QuickNode account and a running Solana node. Once you have these, you can add the Solana Name Service add-on from the [QuickNode Marketplace](https://marketplace.quicknode.com/add-on/solana-name-service).
