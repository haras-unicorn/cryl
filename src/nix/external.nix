{ self, inputs, ... }:

{
  systems = self.lib.systems;
  perSystem =
    { pkgs, lib, ... }:
    let
      rust = (inputs.rust-overlay.lib.mkRustBin { } pkgs).stable.latest.default.override {
        extensions = [
          "rustfmt"
          "clippy"
          "rust-analyzer"
          "rust-src"
        ];
      };

      packages = {
        inherit rust;

        inherit (pkgs)
          git
          nil
          nixfmt-rfc-style
          markdownlint-cli
          marksman
          mdbook
          taplo
          fd
          delta
          cachix
          release-plz
          ;

        inherit (pkgs.nodePackages)
          markdown-link-check
          cspell
          prettier
          vscode-langservers-extracted
          yaml-language-server
          ;
      };
    in
    {
      packages = lib.mapAttrs' (name: value: {
        inherit value;
        name = "external-${name}";
      }) packages;
    };
}
