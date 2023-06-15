{
  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages.${system};
    in
    rec {
      packages = {
        default = packages.nixos-tree;

        nixos-tree =
          pkgs.rustPlatform.buildRustPackage {
            name = "nixos-tree";
            src = nixpkgs.lib.cleanSource ./.;
            cargoLock.lockFile = ./Cargo.lock;
            buildInputs = [ pkgs.ncurses ];
          };
      };

      devShells = {
        default = devShells.nixos-tree;

        nixos-tree = pkgs.mkShell {
          inputsFrom = [ packages.nixos-tree ];
          packages = [ pkgs.rustfmt pkgs.rust-analyzer ];
          CARGO_HOME = pkgs.writeTextDir "config" ''
            [source.crates-io]
            directory = "${packages.nixos-tree.cargoDeps}"
          '';
        };
      };
    });
}
