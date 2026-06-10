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
          nixfmt
          markdownlint-cli
          marksman
          mdbook
          taplo
          fd
          delta
          cachix
          release-plz
          markdown-link-check
          cspell
          prettier
          vscode-langservers-extracted
          yaml-language-server
          cargo-edit
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
