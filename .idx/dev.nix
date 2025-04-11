# To learn more about how to use Nix to configure your environment
# see: https://firebase.google.com/docs/studio/customize-workspace
{ pkgs, ... }: {
  # Which nixpkgs channel to use.
  channel = "stable-24.05"; # or "unstable"
  # Use https://search.nixos.org/packages to find packages
  packages = [
    pkgs.cargo
    pkgs.rustc
    pkgs.rustfmt
    pkgs.nodejs
  ];
  # Sets environment variables in the workspace
  env = {
    RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
  };
  idx = {
    # Search for the extensions you want on https://open-vsx.org/ and use "publisher.id"
    extensions = [];
    workspace = {
      onCreate = {
        install-dfx = "sh -ci \"$(curl -fsSL https://internetcomputer.org/install.sh)\" && source \"$HOME/.local/share/dfx/env\"";
        # Open editors for the following files by default, if they exist:
        default.openFiles = ["src/main.rs"];
      };
    };
    # Enable previews and customize configuration
    previews = {};
  };
}
