{ pkgs ? import <nixpkgs> { } }:
let
in
  pkgs.mkShell rec {
    buildInputs = with pkgs; [
      # Standard development tools
      pkg-config
      flex
      gperf
      bison
      cmake
      ninja

      # Libraries needed for ESP-IDF and libclang
      openssl
      libffi
      libusb1
      libclang
      stdenv.cc.cc.lib  # This provides libstdc++.so.6
      zlib
      ncurses

      # TUIs
      atac
      tuifimanager
      bluetui
      btop
      tenki
    ];

    #LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}:$LD_LIBRARY_PATH";
}
