{ self, ... }:

{
  systems = self.lib.systems;
  perSystem =
    { lib, pkgs, ... }:
    let
      buildInputs = {
        inherit (pkgs)
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
          ceph
          ;
      };
    in
    {
      packages = lib.mapAttrs' (name: value: {
        name = "build-input-${name}";
        inherit value;
      }) buildInputs;
    };
}
