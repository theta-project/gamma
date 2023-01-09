# gamma!

the new bancho server for theta! built for scalability and speed

# configuration

configuration is done either through `gamma.toml`, or through environment variables. see `gamma.toml.example` for the available fields, and `src/settings.rs` / rustdoc for the environment variable names.

# development

you can use the `docker-compose.dev.yml` to set up the required other services, then copy `gamma.toml.example` to `gamma.toml`.
you can then `cargo run`, or use `nix run` / `nix build .#gamma`

# docker

you can build a docker image using `nix build .#docker`, then load it into docker with `docker load < result`.
