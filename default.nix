{ rustPlatform }:
rustPlatform.buildRustPackage {
  name = "dyd";
  src = ./.;
  cargoHash = "sha256-VduByUoO4aeGQfSpNcVxmJlYrpmGF6XLdC040YsDuO4=";
}
