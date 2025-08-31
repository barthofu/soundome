# Soundome

Le but de cette application est de :
1. Télécharger des fichiers audio depuis plusieurs sources (Spotify, Soundcloud, Youtube)
2. Synchroniser des playlists et des artistes depuis ces mêmes sources
3. Tagger les fichiers audio téléchargés afin qu'ils disposent des métadonnées nécessaires à leur classement dans une bibliothèque musicale
4. Gestion des fichiers et organisation de la bibliothèque musicale (dossier par artiste, album, etc.), tout en gardant une trace de la playlist d'origine si la track vient d'une playlist (dans la base de données, dans les metadata ou encore en physique avec des liens symboliques)
5. Reconnaissance automatique du genre d'une musique
6. Suppression des duplicats
7. Interface web de gestion des metadata si besoin (pour les fichiers non taggés automatiquement)


4. Persistence des infos dans une base de données :


Interface web:
1. Parcours des différentes playlists et/ou artistes qui sont synchronisés:
    - vue par artistes, puis album
    - vue par 

## Development

### Requirements


### Initialization

1. `make shell`
2. `cargo install diesel_cli --no-default-features --features sqlite`
3. `diesel setup`
4. `diesel migration run`

###
