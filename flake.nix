{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-master.url = "github:NixOS/nixpkgs/master";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nixpkgs-master, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        pkgs-master = import nixpkgs-master { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [
            # languages
            cargo
            rustc
            rustfmt
            pre-commit
            rustPackages.clippy
            rust-analyzer

            # tools
            pkg-config
            sqlite
            openssl
            d2
            inkscape
            ffmpeg
            pkgs-master.yt-dlp

            # utilities
            id3v2
            glibcLocales
            git-cliff

            # ai
            pkgs-master.opencode
            pkgs-master.rtk

          ];

          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";

          shellHook = ''
            # export TEMPDIR="$(mktemp -d /tmp/nix-shell-XXXXXX)"
            export PATH=$HOME/.cargo/bin:$PATH

            if [[ ! -d data ]]; then
              mkdir -p ./data
              if [[ ! -f ./data/soundome.db ]]; then
                cargo install diesel_cli --no-default-features --features sqlite
                diesel setup
                diesel migration run
              fi
            fi

            export PATH="/home/coder/.local/bin:$PATH"

            # Been moved to .zshrc
            # ssh -C -N -D 1080 vps.public || true &
          '';
        };
      }
    );
}
