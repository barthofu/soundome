# Scénarios de téléchargement et d'organisation

## Download Spotify track via URL

Pour chaque track à importer :

1. Récupération des infos de la source
    1.1. Extraire l’URL et les métadonnées de la track depuis la source (Spotify, Soundcloud, Youtube…)
    1.2. Si la track provient d’une playlist, garder la référence (pour la reconstitution et le classement)

2. Déduplication niveau 1
    2.1. Si une track_ref avec la même external_url existe en base de données, passer à la track suivante

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
    - Rechercher et télécharger la track depuis le provider

6. Déduplication qualité
    - Si une track existe en base avec le même nom, artistes et année (ou via MusicBrainz), comparer la qualité :
        - Si la version existante est meilleure, passer à la track suivante
        - Sinon, continuer

7. Organisation
    - Déplacer la track dans la bibliothèque (`Artiste/Album/Track`)
    - Taguer le fichier avec les métadonnées enrichies
    - Ecrire les métadonnées dans la base de données
    - Créer un lien symbolique si la track appartient à une playlist

8. Sauvegarde
    - Sauvegarder toutes les informations en base de données (track, album, artistes, playlist d’origine…)

9. Gestion des erreurs et rollback
    - Si une étape échoue, annuler ou nettoyer les fichiers temporaires
    - Logguer l’erreur pour validation ou correction manuelle


