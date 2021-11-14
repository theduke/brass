{
  description = "wasm-coverage";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShell.${system} = pkgs.stdenv.mkDerivation {
        name = "wasm-coverage";
        inputs = with pkgs; [
          llvmPackages_latest.clang
        ];
      };
    };
}
