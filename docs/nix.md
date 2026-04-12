# Nix

`cryl` Nix support comprises of a flake-parts module and a test module. These
modules are mostly interchangeable so you can use the same NixOS modules for
defining NixOS configurations and NixOS tests.

For example, the following flake uses the same NixOS module to configure a NixOS
configuration and a NixOS test:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-<nixpkgs-release>";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    sops-nix.url = "github:Mic92/sops-nix";
    sops-nix.inputs.nixpkgs.follows = "nixpkgs";

    cryl.url = "github:haras-unicorn/cryl";
    cryl.inputs.nixpkgs.follows = "nixpkgs";
    cryl.inputs.flake-parts.follows = "flake-parts";
    cryl.inputs.sops-nix.follows = "sops-nix";
  };

  outputs =
    {
      self,
      flake-parts,
      cryl,
      sops-nix,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { inputs, ... }:
      {
        imports = [ cryl.flakeModules.default ];

        cryl.enable = true;

        flake.nixosModules.default =
          { config, ... }:
          {
            sops.secrets.cryl-secret = {
              path = "/etc/cryl-secret-path";
            };

            cryl.enable = true;

            cryl.specification = {
              generations = [
                {
                  generator = "text";
                  arguments = {
                    text = "Hello from cryl!";
                    name = "cryl-secret";
                  };
                }
              ];
            };
          };

        flake.nixosConfigurations.machine = inputs.nixpkgs.nixosSystem {
          system = "x86_64-linux";
          modules = [
            sops-nix.nixosModules.sops
            self.nixosModules.cryl
            self.nixosModules.default
          ];
        };

        systems = [ "x86_64-linux" ];
        perSystem = { pkgs, ... }: {
          checks.machine = pkgs.testers.runNixOSTest {
            imports = [
              cryl.testModules.default
             ];

            cryl.enable = true;

            nodes.machine = {
              imports = [
                sops-nix.nixosModules.sops
                self.nixosModules.default
              ];
            };

            testScript = ''
              start_all()
              machine.succeed("""
                [ -f /etc/cryl-secret-path ]
              """)
              machine.succeed("""
                [ "$(cat /etc/cryl-secret-path)" = "Hello from cryl!" ]
              """)
            '';
          };
        };
      }
    );
}
```

Explanation for key lines of code going from top to bottom:

- `cryl.enable = true;` (inside flake-parts module): This adds a nixos module
  named "cryl" to your flake that configures your default SOPS file path for
  each NixOS configuration at "/sops/\<name of configuration\>.yaml".

  Additionally, it also adds a "cryl-\<name of configuration\>" package for each
  of your NixOS configurations that runs cryl with the generated specification
  and exports the SOPS file at its configured file path and an AGE secret file
  at "/age/\<name of configuration\>.txt".

  The package "cryl-default" is always created for your flake and it runs cryl
  with an aggregated specification of all of your NixOS configuration
  specifications isolating the import, generate and export stages with the
  "working-directory" importer, generator and exporter.

  Note, however, that the aforementioned packages need to be ran within your
  flake as they assume they can detect the root of your flake to put the
  generated SOPS files in their configured place.

  The aforementioned packages read a separate cryl output of your flake which
  can be inspected by running `nix eval .#cryl`.

- `sops.secrets.cryl-secret = { ...`: All files that are present in the
  directory of the NixOS configuration or test node are placed inside the
  generated SOPS file which is achieved by placing a SOPS file generator at the
  end of generation for each NixOS configuration or test node with the "deep"
  directory listing argument.

- `cryl.enable = true;` (inside NixOS module): This signals to the flake-parts
  and test modules that cryl is enabled for this nixosConfiguration or node and
  should be taken into account when generating the specifications for your flake
  or test.

- `flake.nixosConfigurations.machine = inputs.nixpkgs.nixosSystem { ...`: As
  mentioned above, this NixOS configuration will be picked up by the cryl flake
  module (because it has `cryl.enable = true` in its configuration) to create a
  "cryl-machine" package and a specification which will be aggregated into the
  main default specification and used in the "cryl-default" package

- `cryl.enable = true;` (inside test module): Much like the flake-parts module,
  the cryl test module also generates a separate cryl config with specifications
  and a `cryl.sops.package` that runs cryl with the default specification to
  automatically configure all test nodes to use SOPS files and AGE key files
  from it.

- `nodes.machine = { ...`: As stated before, the flake's NixOS module can be
  used with tests where the only difference is that there is no NixOS module
  specific to this test run like there is a NixOS module that you need to import
  for all of your NixOS configurations.

  This is because, in NixOS tests, cryl uses the defaults option to
  automatically include such a module into every test node which is not possible
  with NixOS configurations.

- `testScript = '' ...`: Finally, we can test that cryl has successfully
  generated a SOPS file which is read by sops-nix upon activation to decrypt and
  link our secret to the desired path.

More test examples can be found in the
[Nix tests directory](https://github.com/haras-unicorn/cryl/tree/main/src/nix/test).

A detailed outline of all available options can be found in the following
chapters:

- [Flake or test options](./flake-or-test-submodule.md)
- [NixOS or home-manager options](./nixos-or-home-submodule.md)

## Home manager

Similarly, home-manager modules can use the `cryl.homeModules.default` module
for home configurations and tests.

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-<nixpkgs-release>";

    home-manager.url = "github:nix-community/home-manager/release-<nixpkgs release>";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    sops-nix.url = "github:Mic92/sops-nix";
    sops-nix.inputs.nixpkgs.follows = "nixpkgs";

    cryl.url = "github:haras-unicorn/cryl";
    cryl.inputs.nixpkgs.follows = "nixpkgs";
    cryl.inputs.flake-parts.follows = "flake-parts";
    cryl.inputs.sops-nix.follows = "sops-nix";
  };

  outputs =
    {
      self,
      flake-parts,
      cryl,
      home-manager,
      sops-nix,
      ...
    }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { inputs, ... }:
      {
        imports = [
          cryl.flakeModules.default
          home-manager.flakeModules.home-manager
        ];

        cryl.enable = true;

        flake.homeModules.default =
          { config, ... }:
          {
            sops.secrets.cryl-secret = {
              path = "/home/haras/cryl-secret-path";
            };

            cryl.enable = true;

            cryl.specification = {
              generations = [
                {
                  generator = "text";
                  arguments = {
                    text = "Hello from cryl!";
                    name = "cryl-secret";
                  };
                }
              ];
            };
          };

        flake.nixosModules.default = {
          # NOTE: only needed in test to login as user
          # to trigger sops-nix activation
          services.openssh.enable = true;
          environment.systemPackages = [ pkgs.sshpass ];

          users.groups.haras = { };

          users.users.haras = {
            home = "/home/haras";
            group = "haras";
            isNormalUser = true;
            initialPassword = "haras";
          };

          home-manager.users.haras =  self.homeModules.default;
        };

        flake.nixosConfigurations.machine = inputs.nixpkgs.nixosSystem {
          system = "x86_64-linux";
          modules = [
            {
              imports = [
                self.nixosModules.default
              ];

              home-manager.sharedModules = [
                sops-nix.homeManagerModules.sops
                self.homeModules.cryl
              ];
            }
          ];
        };

        systems = [ "x86_64-linux" ];
        perSystem = { pkgs, ... }: {
          checks.machine = pkgs.testers.runNixOSTest {
            imports = [
              cryl.testModules.default
             ];

            cryl.enable = true;

            nodes.machine = {
              imports = [
                self.nixosModules.default
              ];

              home-manager.sharedModules = [
                sops-nix.homeManagerModules.sops
              ];
            };

            testScript = ''
              start_all()

              # NOTE: starting sops-nix activation to
              # populate our secret
              machine.wait_for_unit("multi-user.target")
              machine.succeed("""
                sshpass -p haras ssh \
                  -o StrictHostKeyChecking=no \
                  haras@localhost \
                  'systemctl --user start --wait sops-nix'
              """)

              machine.succeed("""
                [ -f /home/haras/cryl-secret-path ]
              """)
              machine.succeed("""
                [ "$(cat /home/haras/cryl-secret-path)" = "Hello from cryl!" ]
              """)
            '';
          };
        };
      }
    );
}
```

As you can see, the home-manager module here is almost identical to the NixOS
module in the previous example. Some differences as opposed to the NixOS setup:

- Instead of configuring NixOS the user "haras" is configured with minimal NixOS
  configuration for the example to work.

- In addition, NixOS is configured with `openssh` and `sshpass` just to show the
  test working because `sops-nix` has to actually run for the user before the
  secret can be tested.

- Instead of using the `self.nixosModules.cryl` and `sops-nix.nixosModules.sops`
  modules in NixOS the example uses `self.homeModules.cryl` and
  `sops-nix.homeManagerModules.sops` in home-manager.

- The example puts the generated secret in the home directory of the machine
  user instead of `/etc` because that's more appropriate for user secrets.

As stated above, more examples can be found in the nix test directory of this
repository and more documentation can be found in the following chapters.
