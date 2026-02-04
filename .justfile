shell:
    nix develop --extra-experimental-features nix-command --extra-experimental-features flakes -c zsh

update:
    nix flake lock --update-input nixpkgs-master --extra-experimental-features nix-command --extra-experimental-features flakes

generate_diagrams:
    ./helpers/scripts/generate_diagrams_png.sh
