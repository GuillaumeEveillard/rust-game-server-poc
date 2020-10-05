[![Build Status](https://travis-ci.org/GuillaumeEveillard/rust-game-server-poc.svg?branch=master)](https://travis-ci.org/GuillaumeEveillard/rust-game-server-poc)

# Objectives

This is a toy project to play with Rust and see how to manage state synchronization between a server and its clients.
It's a little multiplayer game where players can move their character on a shared map. 

The main ideas are:
- The server is the source of truth.
- Each client receives update from the server.
- When a player does an action, the client shows the impact immediately without waiting for the server acknowledgement.
If the next update from the server is different from the expectation, the client does the reconciliation.

This logic is important if the latency between the server and the clients is high.
In this use case, we need the client to provide a fast "optimistic" feedback to the player while keeping the server the only source of truth.

# Technologies

The code is written in **Rust**. 
This project uses **gRPC** for the communication between the server and its clients.
The 2D engine is **Piston**.