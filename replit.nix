{ pkgs }: {
  deps = [
    pkgs.rustc
    pkgs.cargo
    pkgs.rust-analyzer
    pkgs.wasm-pack
    pkgs.llvm
    pkgs.lld
    pkgs.nodejs_20
    pkgs.python3
  ];

  env = {
    CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "${pkgs.lld}/bin/lld";
    PATH = "${pkgs.llvm}/bin:${pkgs.lld}/bin";
  };
}