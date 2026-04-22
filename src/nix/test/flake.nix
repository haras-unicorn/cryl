{
  lib,
  config,
  self,
  inputs,
  ...
}:

{
  options.cryl = {
    test = {
      issue6353Resolved = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = lib.literalMD ''
          Enable when issue
          ["Allow getFlake of in-store paths in pure mode."](https://github.com/NixOS/nix/issues/6353)
          gets resolved to test this code path as well.
        '';
      };
    };
  };

  config = lib.mkIf config.cryl.test.issue6353Resolved {
    systems = self.lib.systems;
    perSystem =
      {
        pkgs,
        system,
        lib,
        ...
      }:
      let
        text = "Hello from cryl!";
        textShellArg = lib.escapeShellArg text;
        name = "cryl-test-text";
        path = "/etc/test/${name}";
        pathShellArg = lib.escapeShellArg path;
        node = "machine";

        initialFlake = pkgs.writeTextFile {
          name = "cryl-test-flake.nix";
          destination = "/flake.nix";
          text = ''
            {
              inputs = {
                nixpkgs.url = "path:${inputs.nixpkgs}";

                flake-parts.url = "path:${inputs.flake-parts}";
                flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

                sops-nix.url = "path:${inputs.flake-parts}";
                sops-nix.inputs.nixpkgs.follows = "nixpkgs";

                cryl.url = "path:${self}";
                cryl.inputs.nixpkgs.follows = "nixpkgs";
                cryl.inputs.flake-parts.follows = "flake-parts";
                cryl.inputs.sops-nix.follows = "sops-nix";
              };

              outputs =
                {
                  self,
                  flake-parts,
                  cryl,
                  ...
                }@inputs:
                flake-parts.lib.mkFlake { inherit inputs; } (
                  { inputs, ... }:
                  {
                    imports = [ cryl.flakeModules.default ];

                    flake.nixosModules.default =
                      { config, ... }:
                      {
                        imports = [ self.nixosModules.cryl ];

                        sops.secrets.${name} = {
                          path = "${path}";
                        };
                        sops.age.keyFile = "/etc/sops/age.txt";
                        # NOTE: you should put this in your flake yourself and
                        # never have the actual age file in your flake
                        environment.etc."sops/age.txt".source =
                          "''${self}/''${config.cryl.sops.age.path}";

                        cryl.enable = true;

                        cryl.specification = {
                          generations = [
                            {
                              generator = "text";
                              arguments = {
                                text = "${text}";
                                name = "${name}";
                              };
                            }
                          ];
                        };
                      };

                    flake.nixosConfigurations.${node} = inputs.nixpkgs.nixosSystem {
                      system = "${system}";
                      modules = [ self.nixosModules.default ];
                    };
                  }
                );
            }
          '';
        };

        flakePackage = pkgs.runCommand "test-cryl-flake" { } ''
          mkdir -p $out
          cat ${initialFlake}/flake.nix > $out/flake.nix
          cd $out
          ${lib.getExe (builtins.getFlake "path:${initialFlake}").packages.cryl-default}
        '';

        flake = builtins.getFlake "path:${flakePackage}";
      in
      {
        checks.test-flake = pkgs.testers.runNixOSTest {
          name = "test-flake";

          nodes.${node} = {
            imports = [ flake.nixosModules.default ];
          };

          testScript = ''
            start_all()

            ${node}.succeed("""[ -f ${pathShellArg} ]""")
            ${node}.succeed("""[ "$(cat ${pathShellArg})" = ${textShellArg} ]""")
          '';
        };
      };
  };
}
