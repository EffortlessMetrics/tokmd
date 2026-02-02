{
  description = "tokmd - Tokei-backed repo inventory receipts (Markdown/TSV/JSONL/CSV).";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
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
      # Used for builds where we want a minimal closure
      mkBuildSrc = craneLib: craneLib.path {
        path = ./.;
        filter = path: type:
          (craneLib.filterCargoSources path type)
          || (builtins.match ".*\\.html$" path != null);
      };

      # Full source for tests/checks - keeps fixtures, golden files, ignore files, etc.
      # cleanCargoSource is too restrictive; we need templates, test data, and snapshots
      mkCheckSrc = craneLib: pkgs: pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          let
            p = toString path;
            baseName = baseNameOf path;
          in
          # Keep standard Cargo sources
          (craneLib.filterCargoSources path type)
          # Keep HTML templates (for include_str!)
          || (builtins.match ".*\\.html$" path != null)
          # Keep test directories and their contents
          || (pkgs.lib.hasInfix "/tests/" p)
          # Keep docs directory (needed for schema validation tests)
          || (pkgs.lib.hasInfix "/docs/" p)
          # Keep snapshot files
          || (pkgs.lib.hasSuffix ".snap" baseName)
          # Keep proptest regression files
          || (pkgs.lib.hasSuffix ".proptest-regressions" baseName)
          # Keep gitignore files (used by tests)
          || (baseName == ".gitignore");
      };
    in
    {
      packages = forAllSystems (system:
        let
          pkgs = mkPkgs system;
          craneLib = mkCraneLib system;
          src = mkBuildSrc craneLib;

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
          pkgs = mkPkgs system;
          craneLib = mkCraneLib system;
          src = mkCheckSrc craneLib pkgs;
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
