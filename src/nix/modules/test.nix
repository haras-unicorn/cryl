{ self, ... }:

let
  crylSelf = self;
in
{
  flake.testModules = rec {
    default = cryl;
    cryl =
      {
        nodes,
        lib,
        config,
        pkgs,
        ...
      }:
      let
        cfg = config.cryl;

        testConfig = config;
      in
      {
        options = {
          cryl = lib.mkOption {
            type = lib.types.submodule (
              {
                lib,
                config,
                pkgs,
                ...
              }:
              {
                imports = [
                  crylSelf.lib.submodules.flakeOrTest
                ];

                options = {
                  sops = {
                    specifications = lib.mkOption {
                      type = lib.types.listOf (lib.types.enum (builtins.attrNames config.specifications));
                      default = [ "default" ];
                      description = ''
                        Specifications to run as part of the build command for the SOPS files package.
                      '';
                    };

                    package = lib.mkOption {
                      type = lib.types.package;
                      readOnly = true;
                      description = ''
                        Generated SOPS files package.
                      '';
                    };
                  };
                };

                config = {
                  nixosConfigurations = nodes;
                  defaultSandboxed = false;
                };
              }
            );
            default = { };
            description = "Cryl output for this test";
          };
        };

        config = {
          cryl.sops.package = lib.mkIf cfg.enable (
            let
              specifications =
                builtins.map ({ name, value }: pkgs.writers.writeTOML "${name}-cryl-specification.toml" value)
                  (
                    builtins.filter ({ name, ... }: builtins.elem name cfg.sops.specifications) (
                      lib.attrsToList cfg.specifications
                    )
                  );

              invocations = builtins.concatStringsSep "\n" (
                builtins.map (specification: "${cfg.shellInvocationForPath} ${specification}") specifications
              );
            in
            pkgs.runCommand "cryl-sops-package"
              {
                nativeBuildInputs = [ crylSelf.packages.${pkgs.stdenv.hostPlatform.system}.default ];
              }
              ''
                mkdir -p $out
                ${invocations}
              ''
          );

          defaults = lib.mkIf cfg.enable (
            {
              lib,
              config,
              options,
              ...
            }:
            let
              cfg = config.cryl;
              sopsPackage = testConfig.cryl.sops.package;
            in
            {
              options.cryl = lib.mkOption {
                type = lib.types.submodule {
                  imports = [ crylSelf.lib.submodules.nixosOrHome ];

                  sops.defaultPath = "sops/${config.networking.hostName}.yaml";
                  sops.age.defaultPath = "age/${config.networking.hostName}.txt";
                };
                default = { };
                description = "Cryl config for this node";
              };

              config = lib.mkMerge [
                (lib.mkIf cfg.enable (
                  if !(options ? sops) then
                    { }
                  else
                    {
                      sops.defaultSopsFile = "${sopsPackage}/${cfg.sops.path}";
                      sops.age.keyFile = "/etc/sops/age.txt";
                      environment.etc."sops/age.txt".source = "${sopsPackage}/${cfg.sops.age.path}";
                    }
                ))
                (
                  if !(options ? home-manager) then
                    { }
                  else
                    {
                      home-manager.sharedModules = [
                        (
                          {
                            lib,
                            config,
                            osConfig,
                            options,
                            ...
                          }:
                          let
                            cfg = config.cryl;
                          in
                          {
                            options.cryl = lib.mkOption {
                              type = lib.types.submodule {
                                imports = [ crylSelf.lib.submodules.nixosOrHome ];

                                sops.defaultPath =
                                  if osConfig ? cryl && osConfig.cryl.enable then
                                    osConfig.cryl.sops.path
                                  else
                                    "sops/${osConfig.networking.hostName}-${config.home.username}.yaml";

                                sops.age.defaultPath =
                                  if osConfig ? cryl && osConfig.cryl.enable then
                                    osConfig.cryl.sops.age.path
                                  else
                                    "age/${osConfig.networking.hostName}-${config.home.username}.txt";
                              };
                              default = { };
                              description = "Cryl config for this home configuration";
                            };

                            config = lib.mkIf cfg.enable (
                              if !(options ? sops) then
                                { }
                              else
                                {
                                  sops.defaultSopsFile = "${sopsPackage}/${cfg.sops.path}";
                                  sops.age.keyFile = "${config.xdg.dataHome}/sops/age.txt";
                                  xdg.dataFile."sops/age.txt".source = "${sopsPackage}/${cfg.sops.age.path}";
                                }
                            );
                          }
                        )
                      ];
                    }
                )
              ];
            }
          );
        };
      };
  };
}
