{ inputs, self, ... }:

{
  systems = self.lib.systems;
  perSystem =
    {
      lib,
      pkgs,
      system,
      ...
    }:
    let
      rust = self.packages.${system}.external-rust;

      rustc = rust;
      cargo = rust;

      naersk' = pkgs.callPackage inputs.naersk {
        inherit rustc cargo;
      };

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

      buildInputs = builtins.map (name: self.packages.${system}.${name}) (
        builtins.filter (lib.hasPrefix "build-input") (builtins.attrNames self.packages.${system})
      );
    in
    {
      packages =
        let
          unwrapped = naersk'.buildPackage (
            let
              cargoToml = builtins.fromTOML (builtins.readFile "${self}/src/cryl/Cargo.toml");
            in
            {
              inherit buildInputs;
              src = lib.cleanSourceWith {
                src = self;
                filter =
                  path: type:
                  (lib.hasSuffix ".rs" path)
                  || (lib.hasSuffix ".toml" path)
                  || (lib.hasSuffix ".lock" path)
                  || (type == "directory");
              };
              cargoBuildOptions =
                prev:
                prev
                ++ [
                  "-p"
                  "cryl"
                ];
              name = cargoToml.package.name;
              version = cargoToml.package.version;
              postUnpack = ''
                ${mkVersionPatches buildInputs}
              '';
            }
          );

          wrapped = pkgs.symlinkJoin {
            name = "cryl-wrapped";
            paths = [ unwrapped ];
            nativeBuildInputs = [ pkgs.makeWrapper ];
            postBuild = ''
              wrapProgram $out/bin/cryl \
                --prefix PATH : ${lib.makeBinPath buildInputs}
            '';
            meta = {
              description = "Secret generation tool";
              mainProgram = "cryl";
            };
          };

          # TODO: find a way to bundle the whole thing
          bundled =
            let
              bundledTools = pkgs.symlinkJoin {
                name = "cryl-tools";
                paths = buildInputs;
              };
            in
            pkgs.runCommand "cryl-bundled" { buildInputs = [ pkgs.makeself ]; } ''
              mkdir -p bundle_dir/bin

              cp ${unwrapped}/bin/cryl} bundle_dir/bin/
              cp ${bundledTools}/bin/* bundle_dir/bin/

              makeself bundle_dir cryl.run "Secret generation tool" \
                'export PATH=$PATH:$1/bin; exec ./bin/cryl'

              mv cryl.run $out
            '';
        in
        {
          rust = rust;
          unwrapped = unwrapped;
          default = wrapped;
          cryl = wrapped;
          standalone = bundled;
        };
    };
}
