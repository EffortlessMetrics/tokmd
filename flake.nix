{
  description = "tokmd - Tokei-backed repo inventory receipts (Markdown/TSV/JSONL/CSV).";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crane, ... }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);

      workspaceCargo = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      version = workspaceCargo.workspace.package.version;

      mkPkgs = system: import nixpkgs { inherit system; };
      mkCraneLib = system: crane.mkLib (mkPkgs system);

      # Source filter that includes cargo sources plus HTML templates (for include_str!)
      mkSrc = craneLib: craneLib.path {
        path = ./.;
        filter = path: type:
          (craneLib.filterCargoSources path type)
          || (builtins.match ".*\\.html$" path != null);
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = mkPkgs system;
          craneLib = mkCraneLib system;
          src = mkSrc craneLib;

          commonArgs = {
            pname = "tokmd";
            inherit version src;
            strictDeps = true;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          tokmd = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p tokmd";
            doCheck = false;
          });

          tokmdWithAlias = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
            cargoExtraArgs = "-p tokmd --features alias-tok";
            doCheck = false;
          });
        in
        {
          default = tokmd;
          tokmd = tokmd;
          tokmd-with-alias = tokmdWithAlias;
        });

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.tokmd}/bin/tokmd";
        };
        tokmd = {
          type = "app";
          program = "${self.packages.${system}.tokmd}/bin/tokmd";
        };
      });

      checks = forAllSystems (system:
        let
          craneLib = mkCraneLib system;
          src = mkSrc craneLib;
          commonArgs = {
            inherit src;
            strictDeps = true;
          };
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        in
        {
          tokmd = self.packages.${system}.tokmd;
          clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          fmt = craneLib.cargoFmt { inherit src; };
          test = craneLib.cargoTest (commonArgs // { inherit cargoArtifacts; });
        });

      devShells = forAllSystems (system:
        let
          pkgs = mkPkgs system;
          craneLib = mkCraneLib system;
        in
        {
          default = craneLib.devShell {
            packages = [
              pkgs.rustc
              pkgs.cargo
              pkgs.rustfmt
              pkgs.clippy
              pkgs.rust-analyzer
              pkgs.cargo-insta
              pkgs.cargo-nextest
              pkgs.git
            ];
          };
        });

      formatter = forAllSystems (system: (mkPkgs system).alejandra);
    };
}
