
{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain with WebAssembly support
    (rust-bin.stable.latest.default.override {
      targets = [ "wasm32-unknown-unknown" ];
    }) or (rustc.override {
      targets = [ "wasm32-unknown-unknown" ];
    }) or rustc
    cargo
    rustfmt
    clippy
    
    # WASM tools
    wasm-pack
    
    # LLVM tools for WebAssembly linking
    llvm
    lld
    
    # Node.js and npm for web development
    nodejs_20
    nodePackages.npm
    
    # Python for local server
    python3
    
    # Build essentials
    pkg-config
    openssl
    gcc
    binutils
    
    # Platform specific tools
    (if stdenv.isDarwin then darwin.apple_sdk.frameworks.Security else null)
    (if stdenv.isLinux then glibc else null)
  ];

  shellHook = ''
    echo "🦀 Rust + WASM development environment ready!"
    
    # Add LLVM tools to PATH for WebAssembly linking
    export PATH="${pkgs.llvm}/bin:${pkgs.lld}/bin:$PATH"
    
    # Set up Rust for WebAssembly
    export CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER="${pkgs.lld}/bin/lld"
    export CC="${pkgs.gcc}/bin/gcc"
    export AR="${pkgs.binutils}/bin/ar"
    
    # Add wasm32 target if not already added
    rustup target add wasm32-unknown-unknown 2>/dev/null || echo "wasm32 target already available"
    
    echo "Tools available:"
    echo "  - rustc $(rustc --version)"
    echo "  - cargo $(cargo --version)"
    echo "  - wasm-pack $(wasm-pack --version 2>/dev/null || echo 'installing...')"
    echo "  - lld $(lld --version 2>/dev/null | head -1 || echo 'available')"
    echo "  - node $(node --version)"
    echo "  - python3 $(python3 --version)"
    
    # Ensure wasm-pack is available
    if ! command -v wasm-pack &> /dev/null; then
      echo "Installing wasm-pack..."
      cargo install wasm-pack
    fi
  '';
}
