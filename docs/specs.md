# Specifications

Ce document sert de point d'entrée au projet, de pseudo-spécifications et de cahier des charges. Il a pour but d'expliciter avec précision les besoins et le périmètre du projet afin d'en résulter un ensemble fini de fonctionnalités à implémenter.

## Objectifs

### Objectifs fonctionnels finaux

1. Se procurer des fichiers audio depuis plusieurs sources (Local, Spotify, Soundcloud, Youtube) de manière automatique
2. Tagger* ces fichiers efficacement
3. Organiser ces mêmes fichiers dans une bibliothèque musicale sous la forme suivante: **Artiste** -> **Album/EP/etc** -> **Track**

*Par "tagger" j'entends ajouter les bonnes metadonnées au fichier

## Contraintes

1. Si la musique vient d'une playlist, en garder une trace pour la reconstituer afin de mixer
2. Tagging automatique mais si jamais cela ne marche pas, alors il faut un système de validation/edition manuel
3. Support multiplateforme Linux/Windows/MacOS ?
4. Compatible Beets
5. Permettre une liberté totale de modification directement sur le filesystem
6. Eviter les duplicats

## Solutions 

### Téléchargement

Le téléchargement se fait via 
