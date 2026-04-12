{ self, ... }:

{
  systems = self.lib.systems;
  perSystem =
    { pkgs, ... }:
    {
      packages.flake-root = pkgs.writeShellApplication {
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
    };
}
