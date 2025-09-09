# Soundome - Contexte pour IA

## Vue d'ensemble

Soundome est une application de gestion de bibliothèque musicale personnelle qui unifie différentes sources musicales (services de streaming, fichiers locaux) dans une bibliothèque bien organisée, enrichie et de haute qualité. L'application met l'accent sur l'organisation intelligente, la déduplication, l'enrichissement des métadonnées et la préservation de la meilleure qualité audio disponible.

## Objectifs du projet

- Créer une bibliothèque musicale unifiée à partir de sources diverses
- Minimiser l'intervention manuelle dans la gestion de la bibliothèque
- Maintenir une organisation cohérente et une qualité optimale
- Enrichir automatiquement les métadonnées des pistes
- Préserver les structures de playlists tout en évitant la duplication

## Architecture logicielle

### Composants principaux

1. **Système d'importation**
   - Gestion des imports depuis URLs (Spotify, YouTube, SoundCloud)
   - Gestion des imports de fichiers locaux
   - Validation et déduplication

2. **Système de métadonnées**
   - Extraction des métadonnées des sources
   - Enrichissement via MusicBrainz
   - Scoring de correspondance
   - Mise à jour des tags audio

3. **Système de stockage**
   - Base de données relationnelle pour métadonnées et références
   - Organisation du système de fichiers (Artiste/Album/Piste)
   - Gestion de cache et d'ingest

4. **Interface de validation**
   - Panel web pour validation manuelle des cas ambigus
   - Interface de résolution des conflits

5. **Gestionnaire de qualité**
   - Comparaison de qualité entre versions d'une même piste
   - Algorithmes de décision pour remplacement/conservation

### Flux de données

URLs/Fichiers → Extraction de métadonnées → Déduplication → Enrichissement → Organisation → Téléchargement/Déplacement → Tagging → Sauvegarde BD

### Structure du système de fichiers

/
├── cache/               # Cache temporaire de téléchargement
├── ingest/              # Dossier d'import pour fichiers locaux
└── library/             # Bibliothèque organisée
    ├── Artiste1/
    │   └── Album1/
    │       └── Piste1.mp3
    └── Artiste2/
        └── Album2/
            └── Piste2.mp3

## Flux de travail principaux

### Import depuis URL (ex: Spotify)

1. **Récupération des infos de la source**
   - Extraction d'URL et métadonnées
   - Conservation des références de playlist

2. **Déduplication niveau URL**
   - Vérification si l'URL existe déjà en base

3. **Identification des entités**
   - Album, artistes principaux, featuring

4. **Enrichissement des métadonnées**
   - Via MusicBrainz avec scoring de fiabilité

5. **Téléchargement**
   - Acquisition depuis le fournisseur approprié

6. **Déduplication niveau contenu**
   - Comparaison avec pistes existantes
   - Décision basée sur la qualité

7. **Organisation**
   - Placement dans la structure de bibliothèque
   - Création de liens symboliques pour playlists

8. **Sauvegarde et gestion d'erreurs**
   - Persistance en base de données
   - Rollback en cas d'échec

### Import de fichier local

1. **Analyse des métadonnées du fichier**
   - Extraction des tags existants

2. **Enrichissement si nécessaire**
   - Recherche MusicBrainz si métadonnées insuffisantes

3. **Déduplication**
   - Vérification de l'existence en bibliothèque

4. **Gestion des conflits**
   - Si meilleure qualité: remplacement
   - Si qualité égale: validation manuelle
   - Si qualité inférieure: archivage/suppression

5. **Organisation et indexation**
   - Placement dans la structure de bibliothèque
   - Mise à jour de la base de données

## Modèles de données clés

- **Track**: Représentation d'une piste musicale
- **Track_ref**: Référence externe vers une piste (URL Spotify, etc.)
- **Album**: Collection de pistes
- **Artist**: Créateur/interprète de musique
- **Playlist**: Collection ordonnée de pistes

## Défis techniques

- Détection précise des doublons avec variations de métadonnées
- Scoring fiable de la qualité audio
- Réconciliation d'identités d'artistes similaires
- Gestion des cas particuliers (compilations, remixes, versions)
- Balance entre automatisation et validation manuelle

## Technologies et dépendances

- Base de données relationnelle
- APIs externes (MusicBrainz, services de streaming)
- Système de tagging audio
- Interface web pour validation manuelle
- Algorithmes de comparaison audio et métadonnées

## Glossaire

- **Ingest**: Processus d'importation de fichiers dans le système
- **Track_ref**: Référence externe vers une piste musicale
- **Featuring**: Artistes additionnels participant à une piste
- **Score de correspondance**: Indice de fiabilité entre métadonnées et référence MusicBrainz
- **Déduplication**: Processus d'identification et gestion des doublons
