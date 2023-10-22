{
  outputs = { self, nixpkgs }:
    let
      forAllSystems = generate:
        nixpkgs.lib.genAttrs [
          "aarch64-darwin"
          "x86_64-linux"
        ]
          (system: generate nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.callPackage ./default.nix { };
      });
    };
}
