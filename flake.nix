rec {
  description = "Secret generation tool";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-25.11";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    { flake-parts, ... }@inputs:
    let
      name = "cryl";
    in
    flake-parts.lib.mkFlake
      {
        inherit inputs;
        specialArgs = {
          root = ./.;
        };
      }
      (
        {
          self,
          inputs,
          root,
          lib,
          ...
        }:
        {
          systems = [
            "x86_64-linux"
            "aarch64-linux"
          ];
          perSystem =
            { pkgs, system, ... }:
            let
              rust = (inputs.rust-overlay.lib.mkRustBin { } pkgs).stable.latest.default.override {
                extensions = [
                  "rustfmt"
                  "clippy"
                  "rust-analyzer"
                  "rust-src"
                ];
              };

              rustc = rust;
              cargo = rust;

              naersk' = pkgs.callPackage inputs.naersk {
                inherit rustc cargo;
              };

              flake-root = pkgs.writeShellApplication {
                name = "flake-root";
                text = ''
                  current="$PWD"
                  while [[ "$current" != "/" ]]; do
                    if [[ -f "$current/flake.nix" ]]; then
                      echo "$current"
                      exit 0
                    fi
                    current="$(dirname "$current")"
                  done
                  echo "no flake.nix found" >&2
                  exit 1
                '';
              };

              mkBuildInputs =
                pkgs: with pkgs; [
                  age
                  sops
                  nebula
                  openssl
                  mkpasswd
                  openssh
                  wireguard-tools
                  vault
                  vault-medusa
                  libargon2
                  ssss
                  cockroachdb
                  bubblewrap
                  nushell
                ];

              # Helper to get version string from a package
              getToolVersion =
                pkg:
                if pkg ? version then
                  pkg.version
                else if pkg ? name then
                  (builtins.elemAt (builtins.match "^([^-]+)-.*" pkg.name) 0)
                else
                  "unknown";

              # Generate sed commands to patch versions.rs
              mkVersionPatches =
                buildInputs:
                let
                  pkgNames = [
                    "age"
                    "sops"
                    "nebula"
                    "openssl"
                    "mkpasswd"
                    "openssh"
                    "wireguard-tools"
                    "vault"
                    "vault-medusa"
                    "libargon2"
                    "ssss"
                    "cockroachdb"
                    "bubblewrap"
                    "nushell"
                  ];
                  pkgVersions = lib.listToAttrs (
                    map (pkgName: {
                      name = pkgName;
                      value =
                        let
                          pkg = lib.findFirst (
                            p: p.pname == pkgName || (p ? name && lib.hasPrefix pkgName p.name)
                          ) null buildInputs;
                        in
                        if pkg != null then getToolVersion pkg else "unknown";
                    }) pkgNames
                  );
                in
                lib.concatStringsSep "\n" (
                  lib.mapAttrsToList (pkgName: version: ''
                    if [ -f "$sourceRoot/src/cryl/src/versions.rs" ]; then
                      sed -i "s|versions.insert(\"${pkgName}\", \"dev\")|versions.insert(\"${pkgName}\", \"${version}\")|g" "$sourceRoot/src/cryl/src/versions.rs"
                    fi
                  '') pkgVersions
                );

              buildInputs = mkBuildInputs pkgs;

              externalPackages =
                with pkgs;
                [
                  nil
                  nixfmt-rfc-style

                  rustc
                  cargo

                  markdownlint-cli
                  nodePackages.markdown-link-check
                  marksman

                  nodePackages.cspell

                  mdbook
                  nodePackages.prettier
                  nodePackages.vscode-langservers-extracted
                  nodePackages.prettier
                  nodePackages.yaml-language-server
                  taplo

                  fd
                  delta
                  cachix

                  release-plz
                ]
                ++ buildInputs;

              scripts = {
                run = ''
                  cd "$(flake-root)"

                  cargo run --bin ${name}
                '';
                format = ''
                  cd "$(flake-root)"

                  prettier --write .

                  # shellcheck disable=SC2046
                  nixfmt $(fd '.*.nix$' .)

                  cargo fmt --all
                  cargo clippy --fix --allow-dirty
                '';
                lint = ''
                  cd "$(flake-root)"

                  prettier --check .

                  cspell lint . --no-progress

                  # shellcheck disable=SC2046
                  nixfmt --check $(fd '.*.nix$' .)

                  markdownlint --ignore-path .markdownignore .
                  if [[ -z "''${NIX_BUILD_TOP:-}" ]]; then
                    # shellcheck disable=SC2046
                    markdown-link-check \
                      --config .markdown-link-check.json \
                      --quiet \
                      $(fd '.*.md' .)
                  fi

                  if [[ -z "''${NIX_BUILD_TOP:-}" ]]; then
                    taplo lint \
                      --schema "https://raw.githubusercontent.com/release-plz/release-plz/refs/tags/release-plz-v0.3.148/.schema/latest.json" \
                      .release-plz.toml
                  fi

                  if [[ -n "''${NIX_BUILD_TOP:-}" ]]; then
                    delta \
                      <(cat ./assets/schema.json) \
                      <(${lib.getExe self.packages.${system}.${name}} schema)
                  else
                    cargo clippy -- -D warnings
                    delta \
                      <(cat ./assets/schema.json) \
                      <(cargo run --quiet --bin ${name} -- schema)
                  fi
                '';
                test = ''
                  cargo test
                '';
              };

              scriptPackages = builtins.map (
                { name, value }:
                pkgs.writeShellApplication {
                  name = "dev-${name}";
                  runtimeInputs = externalPackages ++ [ flake-root ];
                  text = value;
                }
              ) (lib.attrsToList scripts);
            in
            {
              _module.args.pkgs = import inputs.nixpkgs {
                inherit system;
                config = {
                  allowUnfree = true;
                };
              };

              packages =
                let
                  unwrapped = naersk'.buildPackage (
                    let
                      cargoToml = builtins.fromTOML (builtins.readFile (lib.path.append root "src/${name}/Cargo.toml"));
                    in
                    {
                      inherit buildInputs;
                      src = root;
                      cargoBuildOptions =
                        prev:
                        prev
                        ++ [
                          "-p"
                          "${name}"
                        ];
                      name = cargoToml.package.name;
                      version = cargoToml.package.version;
                      postUnpack = ''
                        ${mkVersionPatches buildInputs}
                      '';
                    }
                  );

                  wrapped = pkgs.symlinkJoin {
                    name = "${name}-wrapped";
                    paths = [ unwrapped ];
                    nativeBuildInputs = [ pkgs.makeWrapper ];
                    postBuild = ''
                      wrapProgram $out/bin/${name} \
                        --prefix PATH : ${lib.makeBinPath buildInputs}
                    '';
                    meta = {
                      inherit description;
                      mainProgram = name;
                    };
                  };

                  # TODO: find a way to bundle the whole thing
                  bundled =
                    let
                      bundledTools = pkgs.symlinkJoin {
                        name = "${name}-tools";
                        paths = mkBuildInputs pkgs.pkgsStatic;
                      };
                    in
                    pkgs.runCommand "${name}-bundled" { buildInputs = [ pkgs.makeself ]; } ''
                      mkdir -p bundle_dir/bin

                      cp ${unwrapped}/bin/${name} bundle_dir/bin/
                      cp ${bundledTools}/bin/* bundle_dir/bin/

                      makeself bundle_dir ${name}.run "${description}" \
                        'export PATH=$PATH:$1/bin; exec ./bin/${name}'

                      mv ${name}.run $out
                    '';

                  docs =
                    pkgs.runCommand "${name}-docs"
                      {
                        src = self;
                        nativeBuildInputs = [ pkgs.mdbook ];
                      }
                      ''
                        mdbook build -d "$out" "$src/docs"
                      '';
                in
                {
                  unwrapped = unwrapped;
                  default = wrapped;
                  ${name} = wrapped;
                  standalone = bundled;
                  docs = docs;
                };

              apps =
                let
                  package = self.packages.${system}.default;

                  app = {
                    type = "app";
                    program = lib.getExe package;
                    meta.description = description;
                  };
                in
                {
                  default = app;
                  ${name} = app;
                };

              devShells.default = pkgs.mkShell {
                packages = externalPackages ++ scriptPackages;
              };

              checks.default =
                pkgs.runCommand "${name}-checks-default"
                  {
                    src = self;
                    nativeBuildInputs = externalPackages ++ [ flake-root ];
                  }
                  ''
                    cd "$src"
                    ${scripts.lint}
                    touch "$out"
                  '';
            };
        }
      );

  nixConfig = {
    extra-substituters = [
      "https://haras.cachix.org"
    ];
    extra-trusted-public-keys = [
      "haras.cachix.org-1:/HIo1JYqOIH1Nwk1EGXhuPPvDW0WekxIbY5CiXUZbYw="
    ];
  };
}
