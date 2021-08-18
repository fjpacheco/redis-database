[![CI](https://github.com/taller-1-fiuba-rust/Rust-eze/actions/workflows/rust.yml/badge.svg)](https://github.com/taller-1-fiuba-rust/Rust-eze/actions/workflows/rust.yml)
=======
# Redis Database in Rust

## First Part

An implementation of [Redis database](https://redis.io/).

Redis is a (mainly) in-memory storage, used as a Database of type key-value in memory, as well as cache and message broker, with persistence option of the data.

This version implements:

- Client/Server architecture
- TCP Protocol
- Server multithreading
- Request and response [Redis Protocol](https://redis.io/topics/protocol)
- Three different types of data structures: strings, lists, sets
- Keys expiration
- Periodic persistence of data to disk storage (snapshots feature)
- Publishers/subscribers functionality
- Garbage collector: lazy and periodic deletion

All this project was made using technics as: Automated Unit Testing, Automated Integration Tests using a Redis client to the Rust language [(an external crate)](https://crates.io/crates/redis), Error Handling and versions control.

# Redis HTTP Monitor in Rust

## Second Part

The web server that receives requests from browsers, communicating with them through the HTTP/1.1 protocol. The description of this protocol is the one corresponding to [RFC 2616](https://datatracker.ietf.org/doc/html/rfc2616).

This version implements:

- Client/Server architecture
- HTTP Protocol
- Server multithreading (by a Threadpool)
- Datapacks and resources routing.
- Graphic Interface.

# Authors

Nombre | Padrón | Email
------ | ------| -------------
[FIRMAPAZ, Agustin Ezequiel](https://github.com/Agustinefe) | 105172 | afirmapaz@fi.uba.ar
[PACHECO, Federico Jose](https://github.com/fjpacheco) | 104541 | fpacheco@fi.uba.a
[PANETTA, Martina Agata](https://github.com/---) | --- | mpanetta@fi.uba.ar

This project is part of the subject [Taller de Programacion I (Deymonnaz)](https://taller-1-fiuba-rust.github.io/), [Facultad de Ingenieria](http://www.fi.uba.ar/), [Universidad de Buenos Aires](https://www.uba.ar/).
