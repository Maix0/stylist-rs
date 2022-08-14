{
  description = "A basic flake with a shell";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.rust-overlay.url = "github:oxalica/rust-overlay";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [(import rust-overlay)];
      };
    in {
      devShell = with pkgs;
        mkShell {
          nativeBuildInputs = [pkgs.bashInteractive];
          PATH_ADD = "/home/maix/.cargo/bin";
          buildInputs = [
            # Rust
            (rust-bin.stable.latest.default.override {
              targets = ["wasm32-unknown-unknown" "x86_64-unknown-linux-gnu"];
            })

            trunk
            wasm-bindgen-cli
            fastmod
          ];
          shellHook = ''
			PATH=$PATH_ADD:$PATH
          		alias "gen-dem-http"="nix shell nixpkgs#openjdk -c ./generate_dem_http.sh"
		  '';
        };
    });
}
