{ rustPlatform }:
rustPlatform.buildRustPackage {
  name = "dyd";
  src = ./.;
  cargoHash = "sha256-OMnw8AkdeiG9SGDgCiaDbPBcor3+E696Z171M/7rApM=";
}
