{
  description = "lsor lightweight orm for rust";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/23.11";
    rust-flake-parts.url = "github:talo/rust-flake-parts";
    rust-flake-parts.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, rust-flake-parts, ... }:
    let version = "0.1.0";
    in flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ rust-flake-parts.flakeModule ];
      systems = [ "x86_64-linux" "aarch64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, ... }: {
        commonCraneArgs = {
          src = ./.;
          buildInputs = [ pkgs.cargo-sort pkgs.cargo-sweep ];
        };
        craneProjects.lsor = { inherit version; };
      };
    };
}
