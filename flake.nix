{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, rust-overlay, ... }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          stable = pkgs.mkShell {
            packages = [ pkgs.rust-bin.stable.latest.default ];
          };
        in {
          default = stable;
          inherit stable;
          nightly = pkgs.mkShell {
            packages = [
              (pkgs.rust-bin.selectLatestNightlyWith (toolchain:
                toolchain.default.override {
                  extensions = [ "miri" "rust-src" ];
                }))
            ];
          };
        });
    };
}
