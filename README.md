# Discket Ticket Bot

A Discord ticketing bot written in Rust, designed to facilitate anonymous questions in educational settings.

## Features

- Create tickets with a single reaction
- Create subjects for better ticket organization
- Role based access to tickets, to ensure anonymity
- Ticket claiming
- Reposting of ticket content to the designated channel

## Technologies Used

- Language: [Rust](https://www.rust-lang.org/)
    - DiscordAPI : [poise](https://github.com/serenity-rs/poise)
    - Database interaction: [sqlx](https://github.com/launchbadge/sqlx)
    - Async runtime: [tokio](https://github.com/tokio-rs/tokio)
- Database: [PostgreSQL](https://www.postgresql.org/)
