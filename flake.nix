{
  description = "the new bancho server for theta";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        buildInputs = with pkgs; [
          openssl
          protobuf
          pkg-config
        ];

        craneLib = crane.lib.${system};
        gamma = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;

          inherit buildInputs;
        };

        docker = pkgs.dockerTools.buildLayeredImage {
          name = "gamma";
          tag = "latest";
          created = "now";

          config = {
            Cmd = "${gamma}/bin/gamma";
          };
        };
      in
      {
        checks = {
          inherit gamma;
        };

        packages = {
          inherit gamma docker;

          default = gamma;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = gamma;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          nativeBuildInputs = with pkgs; buildInputs ++ [
            cargo
            clippy
            rustc
            rust-analyzer
            rustfmt
            mitmproxy
          ];
        };
      });
}
