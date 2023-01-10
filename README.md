# gamma!

the new bancho server for theta! built for scalability and speed

# configuration

configuration is done either through `gamma.toml`, or through environment variables. see `gamma.toml.example` for the available fields, and `src/settings.rs` / rustdoc for the environment variable names.

# development

you can use the `docker-compose.dev.yml` to set up the required other services, then copy `gamma.toml.example` to `gamma.toml`.
you can then `cargo run`, or use `nix run` / `nix build .#gamma`

# docker

you can build a docker image using `nix build .#docker`, then load it into docker with `docker load < result`.

# proxying traffic

you can change the domain that osu uses by passing it the `-devserver` argument. it will then access several subdomains of that base, which you should set via dns or via `/etc/hosts`. eg for `-devserver localhost`, it will try:
  - `a.localhost`
  - `b.localhost`
  - `c.localhost`
  - `c[1-7].localhost`
  - `ce.localhost`
  - `osu.localhost`

as well as this, osu's security is kinda bad - it forces you to use tls 1.0, which most server software really doesn't want you to do. debian, nix, and arch's nginx packages all seem to be built without support for this.

locally, you can use `mitmproxy` (which nix installs for you): ie `sudo ./scripts/proxy.sh`. this does change certain headers, etc it shouldn't, but it will work for the most part. you will also need to import the CA that it auto generates in `~/.mitmproxy/mitmproxy-ca-cert.pem`.

for deployment, you'll need to run a proper reverse proxy set to support tlsv1.
