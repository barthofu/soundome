enum State {
    Fetching,
    Searching,
    Downloading,
    Tagging,
    Organizing,
    Completed,
    Error(String)
}
pub mod workflows;

use config::AppConfig;
// On suppose que ces modules existent dans le monorepo et exposent les fonctions nécessaires.
use fetcher;
use downloader;
use shared::{errors::Error, models::track::Track};
use tagger;
// use organizer;

const SCORE_THRESHOLD: u32 = 50;

/// Contexte qui stocke les données intermédiaires du workflow pour un track.
#[derive(Debug, Default)]
pub struct WorkflowContext {
    pub url: String,
    pub track: Track,
    pub search_result: String,
    // pub file_metadata: Option<tagger::FileMetadata>,
    // pub tagged_track: Option<tagger::TaggedTrack>,
}

/// Enum représentant les états d'un workflow de traitement d'un track.
enum WorkflowState {
    Start,
    FetchTrack,
    SearchTrack,
    DownloadTrack,
    GetFileMetadata,
    DetermineMetadata,
    ScoreMatching,
    // ValidateWeb,
    TagTrack,
    // DuplicateDetection,
    // QualityComparison,
    // MoveTrack,
    // SaveTrackInfo,
    // ArtistSimilarityDetection,
    Finished,
}

/// Sous-FSM pour le traitement d'un track.
/// Elle encapsule toutes les étapes communes (fetch, download, tag, organiser, etc.).
pub async fn orchestrate_track_fsm(url: &str, config: &AppConfig) -> Result<WorkflowContext, Error> {
    let mut context = WorkflowContext {
        url: url.to_string(),
        ..Default::default()
    };
    let mut state = WorkflowState::Start;

    loop {
        state = match state {
            WorkflowState::Start => WorkflowState::FetchTrack,
            WorkflowState::FetchTrack => {
                context.track = fetcher::get_track_from_url(&context.url, &config)?;
                WorkflowState::SearchTrack
            },
            WorkflowState::SearchTrack => {
                let download_url = downloader::search_track(&context.track, &config).await;
                match download_url {
                    Some(url) => {
                        context.search_result = url;
                        WorkflowState::DownloadTrack
                    },
                    None => {
                        eprintln!("Erreur lors de la recherche de la piste: {:?}", e);
                        WorkflowState::Finished
                    }
                }

                WorkflowState::DownloadTrack
            },
            WorkflowState::DownloadTrack => {
                let search_result = context.search_result.as_ref().unwrap();
                let downloaded = downloader::download_track(search_result).await?;
                context.downloaded_track = Some(downloaded);
                WorkflowState::GetFileMetadata
            },
            WorkflowState::GetFileMetadata => {
                let downloaded_track = context.downloaded_track.as_ref().unwrap();
                let metadata = tagger::get_metadata_from_file(downloaded_track).await?;
                context.file_metadata = Some(metadata);
                WorkflowState::DetermineMetadata
            },
            WorkflowState::DetermineMetadata => {
                let downloaded_track = context.downloaded_track.as_ref().unwrap();
                let file_metadata = context.file_metadata.as_ref().unwrap();
                let track_object = if tagger::sufficient_metadata(file_metadata) {
                    tagger::get_music_brainz_metadata_from_track_object(downloaded_track).await?
                } else {
                    tagger::get_music_brainz_metadata_from_query(downloaded_track).await?
                };
                context.track_object = Some(track_object);
                WorkflowState::ScoreMatching
            },
            WorkflowState::ScoreMatching => {
                let track_object = context.track_object.as_ref().unwrap();
                let score = tagger::score_matching(track_object);
                if score < SCORE_THRESHOLD {
                    WorkflowState::ValidateWeb
                } else {
                    WorkflowState::TagTrack
                }
            },
            // WorkflowState::ValidateWeb => {
            //     let track_object = context.track_object.as_ref().unwrap();
            //     tagger::web_validation(track_object).await?;
            //     WorkflowState::TagTrack
            // },
            WorkflowState::TagTrack => {
                let track_object = context.track_object.take().unwrap();
                let tagged = tagger::tag_track(track_object).await?;
                context.tagged_track = Some(tagged);
                WorkflowState::DuplicateDetection
            },
            // WorkflowState::DuplicateDetection => {
            //     let tagged_track = context.tagged_track.as_ref().unwrap();
            //     if organizer::duplicate_detection(tagged_track) {
            //         WorkflowState::QualityComparison
            //     } else {
            //         WorkflowState::MoveTrack
            //     }
            // },
            // WorkflowState::QualityComparison => {
            //     let tagged_track = context.tagged_track.as_ref().unwrap();
            //     if organizer::quality_comparison(tagged_track) {
            //         WorkflowState::MoveTrack
            //     } else {
            //         return Err(anyhow::anyhow!(
            //             "Nouvelle version moins bonne, workflow stoppé."
            //         ));
            //     }
            // },
            // WorkflowState::MoveTrack => {
            //     let tagged_track = context.tagged_track.as_ref().unwrap();
            //     organizer::move_track(tagged_track).await?;
            //     WorkflowState::SaveTrackInfo
            // },
            // WorkflowState::SaveTrackInfo => {
            //     let tagged_track = context.tagged_track.as_ref().unwrap();
            //     organizer::save_track_info_to_db(tagged_track).await?;
            //     WorkflowState::ArtistSimilarityDetection
            // },
            // WorkflowState::ArtistSimilarityDetection => {
            //     let tagged_track = context.tagged_track.as_ref().unwrap();
            //     if let Some(similar_artist) = organizer::artist_similarity_detection(tagged_track) {
            //         if organizer::artist_web_validation(&similar_artist).await? {
            //             organizer::merge_artists(tagged_track, similar_artist).await?;
            //         }
            //     }
            //     WorkflowState::Finished
            // },
            WorkflowState::Finished => break,
        };
    }

    Ok(())
}

/// Méthode publique pour traiter un track unique.
/// Utilise le FSM défini ci-dessus.
pub async fn download_track(url: &str) -> Result<()> {
    orchestrate_track_fsm(url).await
}

/// Fonction auxiliaire pour extraire les URLs des pistes depuis l'URL d'une playlist.
/// Dans une implémentation réelle, cette fonction utilisera la logique de parsing propre à ton projet.
async fn get_tracks_from_playlist(playlist_url: &str) -> Result<Vec<String>> {
    // Pour l'exemple, on retourne une liste fictive.
    Ok(vec![
        format!("{}/track1", playlist_url),
        format!("{}/track2", playlist_url),
        format!("{}/track3", playlist_url),
    ])
}

/// Méthode publique pour traiter une playlist complète.
/// Elle itère sur chaque track de la playlist en réutilisant le FSM de traitement.
pub async fn download_playlist(playlist_url: &str) -> Result<()> {
    let track_urls = get_tracks_from_playlist(playlist_url).await?;

    for url in track_urls {
        println!("Traitement de la piste: {}", url);
        // Optionnel : lancer les workflows en parallèle avec tokio::spawn pour plus de performance.
        if let Err(e) = orchestrate_track_fsm(&url).await {
            eprintln!("Erreur lors du traitement de {}: {:?}", url, e);
            // Ici, on peut choisir de continuer sur les autres pistes ou d'arrêter la playlist.
        }
    }

    Ok(())
}
