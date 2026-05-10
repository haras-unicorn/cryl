{
  self,
  inputs,
  ...
}:

{
  flake.lib.systems = [
    "x86_64-linux"
    "aarch64-linux"
  ];

  flake.lib.scripts = {
    run = ''
      cd "$(flake-root)"

      cargo run --bin cryl
    '';
    format = ''
      cd "$(flake-root)"

      # NOTE: cat instead of cp
      # because it copies readonly and such attrs
      cat \
        "$(nix build \
          --no-link \
          --print-out-paths \
          .#docs-nixos-or-home-submodule)" \
        > ./docs/nixos-or-home-submodule.md
      # NOTE: cat instead of cp
      # because it copies readonly and such attrs
      cat \
        "$(nix build \
          --no-link \
          --print-out-paths \
          .#docs-flake-or-test-submodule)" \
        > ./docs/flake-or-test-submodule.md

      prettier --write .

      # shellcheck disable=SC2046
      nixfmt $(fd '.*.nix$' .)

      cargo fmt --all
      cargo clippy --fix --allow-dirty
    '';
    lint = ''
      cd "$(flake-root)"

      # NOTE: CI is always dirty
      if [[ -z "''${CI:-}" ]]; then
        if ! git diff --quiet; then
          echo "Please run with a clean working directory"
          exit 1
        fi
        # NOTE: cat instead of cp
        # because it copies readonly and such attrs
        cat \
          "$(nix build \
            --no-link \
            --print-out-paths \
            .#docs-flake-or-test-submodule)" \
          > ./docs/flake-or-test-submodule.md
        prettier --write ./docs/flake-or-test-submodule.md
        if ! git diff --quiet; then
          echo "NixOS docs options file './docs/flake-or-test-submodule.md' differs from generated."
          echo "Please regenerate NixOS option docs files with 'dev-format'"
          exit 1
        fi
        # NOTE: cat instead of cp
        # because it copies readonly and such attrs
        cat \
          "$(nix build \
            --no-link \
            --print-out-paths \
            .#docs-nixos-or-home-submodule)" \
          > ./docs/nixos-or-home-submodule.md
        prettier --write ./docs/nixos-or-home-submodule.md
        if ! git diff --quiet; then
          echo "NixOS docs options file './docs/nixos-or-home-submodule.md' differs from generated."
          echo "Please regenerate NixOS option docs files with 'dev-format'"
          exit 1
        fi
      fi

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

      if [[ -z "''${NIX_BUILD_TOP:-}" ]]; then
        cargo clippy -- -D warnings
        cargo test
        delta \
          <(cat ./assets/schema.json) \
          <(cargo run --quiet --bin cryl -- schema)
      fi
    '';
    nixos-test = ''
      test="$1"
      shift
      nix build \
        ".#checks.$(uname -m)-linux.test-''${test}.withSshBackdoor" \
        --option sandbox-paths /dev/vhost-vsock \
        "$@"
    '';
    nixos-test-interactive = ''
      test="$1"
      shift
      nix run \
        ".#checks.$(uname -m)-linux.test-''${test}.withSshBackdoor.driverInteractive" \
        "$@"
    '';
  };

  # TODO: add extra args and sandbox stuff here too
  # TODO: generate from schema
  flake.lib.submodules.specification =
    { lib, ... }:
    {
      options = {
        imports = lib.mkOption {
          type = lib.types.listOf lib.types.raw;
          default = [ ];
          description = ''
            Cryl imports specification value.
          '';
        };

        generations = lib.mkOption {
          type = lib.types.listOf lib.types.raw;
          default = [ ];
          description = ''
            Cryl generations specification value.
          '';
        };

        exports = lib.mkOption {
          type = lib.types.listOf lib.types.raw;
          default = [ ];
          description = ''
            Cryl exports specification value.
          '';
        };
      };
    };

  flake.lib.submodules.nixosOrHome =
    { lib, config, ... }:
    {
      options = {
        enable = lib.mkEnableOption "cryl";

        specification = lib.mkOption {
          type = lib.types.submodule self.lib.submodules.specification;
          default = { };
          description = ''
            Cryl specification for this nixos configuration or home configuration.
          '';
        };

        sops = {
          private = lib.mkOption {
            type = lib.types.str;
            default = "sops-private";
            description = "Name of the generated decrypted SOPS file";
          };

          public = lib.mkOption {
            type = lib.types.str;
            default = "sops-public";
            description = "Name of the generated encrypted SOPS file";
          };

          path = lib.mkOption (
            {
              type = lib.types.str;
              description = ''
                Encrypted SOPS file location relative to the root of the output.
                For tests relative to the root of the generated SOPS package
                and for flakes relative to the flake root.
              '';
            }
            // (if config.sops.defaultPath != null then { default = config.sops.defaultPath; } else { })
          );

          defaultPath = lib.mkOption {
            type = lib.types.nullOr lib.types.str;
            default = null;
            internal = true;
          };

          age = {
            private = lib.mkOption {
              type = lib.types.str;
              default = "age-private";
              description = "Name of the generated private age key";
            };

            public = lib.mkOption {
              type = lib.types.str;
              default = "age-public";
              description = "Name of the generated public age key";
            };

            export = lib.mkOption {
              type = lib.types.bool;
              default = config.sops.age.defaultExport;
              defaultText = lib.literalMD "`true` for tests and `false` for flakes";
              description = "Whether to export the private SOPS age file";
            };

            path = lib.mkOption (
              {
                type = lib.types.str;
                description = ''
                  Age private key location relative to the root of the output.
                  For tests relative to the root of the generated SOPS package
                  and for flakes relative to the flake root.
                  Make sure to add this path to gitignore.
                '';
              }
              // (if config.sops.age.defaultPath != null then { default = config.sops.age.defaultPath; } else { })
            );

            defaultExport = lib.mkOption {
              type = lib.types.bool;
              default = false;
              internal = true;
            };

            defaultPath = lib.mkOption {
              type = lib.types.nullOr lib.types.str;
              default = null;
              internal = true;
            };
          };
        };
      };
    };

  flake.lib.submodules.flakeOrTest =
    { lib, config, ... }:
    let
      nixosConfigurations = config.nixosConfigurations;

      # NOTE: this prefixes sops keys with "<user>-" for users that
      # share a sops file with nixos configurations
      # and should be documented properly
      mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories =
        crylSpecAttrset: attr: tag:
        builtins.concatMap (
          { name, value }:
          [
            {
              ${tag} = "working-directory";
              arguments.path = name;
            }
          ]
          ++ value.${attr}
          ++ [
            {
              ${tag} = "working-directory";
              arguments.path = "..";
            }
          ]
        ) (lib.attrsToList crylSpecAttrset);

      nixosConfigurationsWithCryl = lib.filterAttrs (
        _: nixosConfiguration: nixosConfiguration ? cryl && nixosConfiguration.cryl.enable
      ) nixosConfigurations;

      nixosConfigurationCrylConfig = builtins.mapAttrs (
        _: nixosConfiguration: nixosConfiguration.config.cryl
      ) nixosConfigurationsWithCryl;

      userCrylConfigsPerNixosConfigurationName = builtins.mapAttrs (
        nixosConfigurationName: nixosConfiguration:
        builtins.mapAttrs (_: userHomeManagerConfig: userHomeManagerConfig.cryl) (
          lib.filterAttrs
            (_: userHomeManagerConfig: userHomeManagerConfig ? cryl && userHomeManagerConfig.cryl.enable)
            (
              if nixosConfiguration.config ? home-manager then
                nixosConfiguration.config.home-manager.users
              else
                { }
            )
        )
      ) nixosConfigurations;

      nixosConfigurationCrylConfigsWithUsersWithSameSopsPath = builtins.mapAttrs (
        nixosConfigurationName: nixosConfigurationCrylConfig:
        let
          userCrylSpecsInConfigsWithSameSopsPath = builtins.mapAttrs (_: config: config.specification) (
            lib.filterAttrs (
              _: userCrylConfig: userCrylConfig.sops.path == nixosConfigurationCrylConfig.sops.path
            ) userCrylConfigsPerNixosConfigurationName.${nixosConfigurationName}
          );
        in
        {
          specification = {
            imports =
              nixosConfigurationCrylConfig.specification.imports
              ++ (mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories
                userCrylSpecsInConfigsWithSameSopsPath
                "imports"
                "importer"
              );
            generations =
              nixosConfigurationCrylConfig.specification.generations
              ++ (mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories
                userCrylSpecsInConfigsWithSameSopsPath
                "generations"
                "generator"
              );
            exports =
              nixosConfigurationCrylConfig.specification.exports
              ++ (mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories
                userCrylSpecsInConfigsWithSameSopsPath
                "exports"
                "exporter"
              );
          };
          sops = nixosConfigurationCrylConfig.sops;
        }
      ) nixosConfigurationCrylConfig;

      userCrylConfigsWithUniqueSopsPath = lib.concatMapAttrs (
        nixosConfigurationName: userCrylConfigs:
        lib.mapAttrs'
          (user: userCrylConfig: {
            name = "${nixosConfigurationName}-${user}";
            value = userCrylConfig;
          })
          (
            lib.filterAttrs (
              name: userCrylConfig:
              !(nixosConfigurationCrylConfig ? ${nixosConfigurationName})
              || (userCrylConfig.sops.path != nixosConfigurationCrylConfig.${nixosConfigurationName}.sops.path)
            ) userCrylConfigs
          )
      ) userCrylConfigsPerNixosConfigurationName;

      crylSpecs = builtins.mapAttrs (_: crylConfig: {
        imports = crylConfig.specification.imports;
        generations = crylConfig.specification.generations ++ [
          {
            generator = "age-key";
            arguments = {
              private = crylConfig.sops.age.private;
              public = crylConfig.sops.age.public;
            };
          }
          {
            generator = "sops";
            arguments = {
              renew = true;
              age = crylConfig.sops.age.public;
              private = crylConfig.sops.private;
              public = crylConfig.sops.public;
              secrets.type = "deep";
            };
          }
        ];
        exports = crylConfig.specification.exports ++ [
          {
            exporter = "copy";
            arguments = {
              listing = {
                type = "map";
                value = {
                  ${crylConfig.sops.path} = crylConfig.sops.public;
                }
                // lib.optionalAttrs crylConfig.sops.age.export {
                  ${crylConfig.sops.age.path} = crylConfig.sops.age.private;
                };
              };
              to = "$out";
            };
          }
        ];
      }) (nixosConfigurationCrylConfigsWithUsersWithSameSopsPath // userCrylConfigsWithUniqueSopsPath);
    in
    {
      options = {
        enable = lib.mkEnableOption "cryl";

        specifications = lib.mkOption {
          type = lib.types.attrsOf (lib.types.submodule self.lib.submodules.specification);
          default = { };
          description = "Cryl specification attrset";
        };

        sandboxed = lib.mkOption {
          type = lib.types.bool;
          default = config.defaultSandboxed;
          defaultText = lib.literalMD "`true` for flakes, `false` for tests";
          description = "Whether to run cryl sandboxed";
        };

        extraArgs = lib.mkOption {
          type = lib.types.listOf lib.types.str;
          default = [ ];
          description = "Extra cryl arguments";
        };

        shellInvocationForPath = lib.mkOption {
          type = lib.types.str;
          internal = true;
        };

        nixosConfigurations = lib.mkOption {
          type = lib.types.attrsOf lib.types.raw;
          internal = true;
        };

        defaultSandboxed = lib.mkOption {
          type = lib.types.bool;
          internal = true;
        };
      };

      config = {
        shellInvocationForPath = lib.mkIf config.enable (
          let
            extraArgs = lib.escapeShellArgs config.extraArgs;
            sandboxArgs = if config.sandboxed then "--binds $out --env out" else "--nosandbox";
          in
          "cryl path --envsubst ${sandboxArgs} ${extraArgs}"
        );

        specifications = lib.mkIf config.enable (
          lib.mkMerge [
            crylSpecs
            {
              default = {
                imports =
                  mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories crylSpecs "imports"
                    "importer";
                generations =
                  mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories crylSpecs "generations"
                    "generator";
                exports =
                  mergeCrylConfigAttrsetToSpecificationAttrInSeparateWorkingDirectories crylSpecs "exports"
                    "exporter";
              };
            }
          ]
        );
      };
    };

  flake.lib.submodules.test =
    let
      stateVersion = "25.11";
    in
    {
      imports = [ self.testModules.default ];

      node.pkgsReadOnly = false;

      defaults = {
        imports = [
          inputs.home-manager.nixosModules.default
          inputs.sops-nix.nixosModules.sops
        ];

        virtualisation.graphics = false;

        nixpkgs.config = {
          allowUnfree = true;
        };

        system.stateVersion = stateVersion;

        home-manager.sharedModules = [
          inputs.sops-nix.homeManagerModules.sops
          {
            home.stateVersion = stateVersion;
          }
        ];
      };
    };
}
