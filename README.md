# Odin Emulator
Odin Emulator is a server implementation for the With Your Destiny MMO, written in Rust.

## Goal
This is an educational project aimed at writing more Rust to improve my skills. One of my goals is to develop this project in a distinctly "Rusty" way. I previously created this project in C/C++, following a more procedural style with many global variables (it was decompiled from the original executable). I also aim to enhance my skills in writing tests, which I consider a crucial part of the development process.

## Database
The chosen primary database for this project is PostgreSQL, a fast and straightforward option. I already have a complete database structure for the game, so I plan to use it, but I am open to making changes if needed.

We have also added support for SQLite, primarily for testing purposes. This allows us to run in-memory tests without the need for a PostgreSQL instance, but SQLite will be supported as well in the project.

I am creating abstractions at the database layer, so the project could potentially support other ORM or even sqlx. While I am still evaluating SeaORM for this project, the simplicity of the queries might make an ORM unnecessary.

## Features
The project is in its early stages, and currently, no complete features have been implemented. At the moment, you can attempt to log into the server, where you’ll receive a message indicating that login failed (e.g., due to invalid password, invalid account, invalid client version, or banned account).

Configurations are still under development, so options such as client version (cliver) and key table customization are not yet available.

## Planned Features
- [x] Message encryption and decryption
- [x] Receive and parse messages
- [x] Login to the character select screen
- [x] Numeric token
    - [ ] Disconnect user when it sends more than 3 incorrect tokens
- [x] Create character
- [x] Delete character
- [ ] Enter world
- [ ] ...
