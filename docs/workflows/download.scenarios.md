# Scénarios de téléchargement et d'organisation

## Download Spotify track via URL

Pour chaque track à importer :

1. Déduplication track
    1.1. Si une track_ref avec la même external_url existe en base de données, passer à la track suivante

2. Récupération des infos de la source
    2.1. Extraire les métadonnées de la track depuis la source (Spotify, Soundcloud, Youtube…)
    2.2. Si la track provient d’une playlist, garder la référence (pour la reconstitution et le classement)

3. Identification des entités
    3.1. Identifier l’album et les artistes (principal + featuring)
    3.2. Pour chaque album/artiste, vérifier s’il existe en base :
        3.2.1. Si oui, associer l’id à la track
        3.2.2. Sinon, créer la référence correspondante

4. Enrichissement des métadonnées
    4.1. Récupérer les métadonnées via MusicBrainz (ou provider local)
    4.2. Si ref musicbrainz existe :
        4.2.1. L’associer à la track, et associer recursivement aux entités (album, artistes) qui n'ont pas été trouvés dans l'étape 3
        4.2.2. Passer à l’étape 5
    4.3. Si le score de correspondance est élevé, marquer comme “match complet”
    4.4. Si le score est faible, "à valider" (pour plus tard proposer une validation manuelle via le web panel)
    
5. Téléchargement
    5.5. Rechercher et télécharger la track depuis le provider

6. Déduplication qualité
    6.1. Si une track existe en base avec le même nom, artistes et année (ou via MusicBrainz), comparer la qualité :
        - Si la version existante est meilleure, passer à la track suivante
        - Sinon, continuer

7. Organisation
    7.1. Déplacer la track dans la bibliothèque (`Artiste/Album/Track`)
    7.2. Taguer le fichier avec les métadonnées enrichies

8. Sauvegarde
    8.1. Sauvegarder toutes les informations en base de données (track, album, artistes, playlist d’origine…)

9. Gestion des erreurs et rollback
    9.1. Si une étape échoue, annuler ou nettoyer les fichiers temporaires
    9.2. Logguer l’erreur pour validation ou correction manuelle

## Import d'un fichier local depuis le dossier d'ingest

Pour chaque fichier audio à importer depuis le dossier d'ingest :

1. Récupération des infos du fichier
    1.1. Extraire le chemin et les métadonnées présentes dans le fichier (tags ID3, etc.)

2. Vérification des métadonnées
    2.1. Si les métadonnées sont suffisantes (titre, artistes, album, année), passer à l'étape 3
    2.2. Sinon, tenter d'enrichir via MusicBrainz (en utilisant les tags ou le nom du fichier comme query)
    2.3. Si le score de correspondance est élevé, compléter les métadonnées
    2.4. Si le score est faible, marquer comme "à valider" (pour validation manuelle via le web panel)

3. Déduplication
    3.1. Vérifier si une track existe en base avec le même nom, artistes et année (ou via MusicBrainz)
        3.1.1. Si une version existe et est de meilleure qualité, passer au fichier suivant
        3.1.2. Si une version existe mais est de moins bonne qualité :
            - Remplacer le fichier existant par le nouveau (après confirmation ou selon politique de qualité)
            - Mettre à jour les métadonnées en base si besoin
            - Archiver ou supprimer l'ancienne version selon la configuration
        3.1.3. Si la version existante et le nouveau fichier sont de qualité équivalente :
            - Proposer une validation manuelle (web panel) pour choisir la version à conserver
        3.1.4. Si aucune version n'existe, continuer le workflow normalement

4. Organisation
    4.1. Déplacer le fichier dans la bibliothèque (`Artiste/Album/Track`)
    4.2. Taguer le fichier avec les métadonnées enrichies
    4.3. Écrire les métadonnées dans la base de données
    4.4. Créer un lien symbolique si le fichier appartient à une playlist

5. Sauvegarde
    5.1. Sauvegarder toutes les informations en base de données (track, album, artistes, playlist d’origine…)

6. Gestion des erreurs et rollback
    6.1. Si une étape échoue, annuler ou nettoyer les fichiers temporaires
    6.2. Logguer l’erreur pour validation ou correction manuelle


