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
            cargoSha256 = "haohCa9E8mKFQaycTyHoES54sJzi1vswcBb/rE5CRYg=";
            buildInputs = [ pkgs.ncurses ];
          };
      };

      devShells = {
        default = devShells.nixos-tree;

        nixos-tree = pkgs.mkShell {
          inputsFrom = [ packages.nixos-tree ];
          packages = [ pkgs.rustfmt pkgs.rust-analyzer ];

          CARGO_HOME =
          let
            crates-io-directory =
              pkgs.runCommandLocal "crates-io-directory" {} ''
                tar -xf ${packages.nixos-tree.cargoDeps}
                mv nixos-tree-vendor.tar.gz $out
              '';
          in
            pkgs.writeTextDir "config" ''
              [source.crates-io]
              directory = "${crates-io-directory}"
            '';
        };
      };
    });
}
