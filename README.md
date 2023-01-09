# gamma
the new bancho server for theta! build for scalability and speed

# development

you can use the `docker-compose.dev.yml` to set up the required other services, then copy `.env.example` to `.env`.
you can then `cargo run`, or use `nix run` / `nix build .#gamma`

# docker

you can build a docker image using `nix build .#docker`, then load it into docker with `docker load < result`.
