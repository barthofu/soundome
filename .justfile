shell:
    nix develop --extra-experimental-features nix-command --extra-experimental-features flakes -c zsh

generate_diagrams:
    ./helpers/scripts/generate_diagrams_png.sh
