//! Serial (concurrency = 1) background task executor.
//!
//! The executor owns a single worker thread that pulls `QueuedJob`s from a FIFO
//! channel and runs them one at a time. Its purpose is to guarantee that
//! Soundome only ever runs **one** heavy job at a time, which:
//!
//! - avoids SQLite `database is locked` errors caused by two writers racing
//!   for the exclusive database lock on concurrent playlist syncs,
//! - keeps external API usage (Spotify, SoundCloud, YouTube, MusicBrainz…)
//!   well below rate-limit thresholds by not fanning out,
//! - prevents `yt-dlp` / `ffmpeg` from stepping on each other when the host
//!   has limited CPU or bandwidth.
//!
//! Producers (routes, scheduler, boot recovery) call `enqueue_*` and get back
//! either the assigned `task_id` immediately (for fire-and-forget jobs) or a
//! oneshot receiver that resolves once the job finishes (for the synchronous
//! single-track download path).
//!
//! Task lifecycle in DB:
//! - `Pending` — inserted by the caller, sitting in the queue.
//! - `Running` — set by the worker just before it starts the job.
//! - `Completed` / `Failed` / `Cancelled` — set by the worker when the job
//!   returns.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use config::Config;
use domain::services::ServiceLayer;
use shared::errors::Error;
use shared::models::Track;
use shared::types::SoundomeResult;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::sync::oneshot;

use crate::utils::cancellation::CancellationRegistry;

/// A job scheduled on the serial executor.
///
/// Each variant carries every piece of data the worker needs to run the job
/// without touching Rocket state.
pub enum QueuedJob {
    PlaylistSync {
        task_id: i32,
        url: String,
        cancel_flag: Arc<AtomicBool>,
    },
    ArtistSync {
        task_id: i32,
        url: String,
        cancel_flag: Arc<AtomicBool>,
    },
    AlbumSync {
        task_id: i32,
        url: String,
        cancel_flag: Arc<AtomicBool>,
    },
    IngestDir {
        task_id: i32,
        ingest_dir: PathBuf,
    },
    /// Synchronous single-track download. The result is sent back through
    /// `responder`. No DB task row is used for this variant since the HTTP
    /// caller blocks on the response.
    SingleTrack {
        url: String,
        responder: oneshot::Sender<SoundomeResult<Track>>,
    },
}

/// Handle used by producers to enqueue jobs on the shared serial worker.
///
/// Cheap to clone (wraps an `mpsc::UnboundedSender`), lives inside
/// `rocket::State<Arc<TaskExecutor>>`.
pub struct TaskExecutor {
    sender: UnboundedSender<QueuedJob>,
}

impl TaskExecutor {
    /// Start the background worker thread and return a handle to enqueue jobs.
    ///
    /// The worker runs on its own dedicated OS thread with a current-thread
    /// Tokio runtime, so blocking downloads (yt-dlp) never stall the main
    /// Rocket runtime and the strict "one job at a time" invariant holds
    /// regardless of how producers call `enqueue_*`.
    pub fn start(services: Arc<ServiceLayer>, registry: Arc<CancellationRegistry>) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<QueuedJob>();

        std::thread::Builder::new()
            .name("soundome-task-executor".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("failed to build task-executor tokio runtime");
                rt.block_on(worker_loop(receiver, services, registry));
            })
            .expect("failed to spawn task-executor thread");

        Self { sender }
    }

    /// Enqueue a playlist sync. Non-blocking.
    pub fn enqueue_playlist_sync(
        &self,
        task_id: i32,
        url: String,
        cancel_flag: Arc<AtomicBool>,
    ) {
        self.send(QueuedJob::PlaylistSync {
            task_id,
            url,
            cancel_flag,
        });
    }

    /// Enqueue an artist sync. Non-blocking.
    pub fn enqueue_artist_sync(&self, task_id: i32, url: String, cancel_flag: Arc<AtomicBool>) {
        self.send(QueuedJob::ArtistSync {
            task_id,
            url,
            cancel_flag,
        });
    }

    /// Enqueue an album sync. Non-blocking.
    pub fn enqueue_album_sync(&self, task_id: i32, url: String, cancel_flag: Arc<AtomicBool>) {
        self.send(QueuedJob::AlbumSync {
            task_id,
            url,
            cancel_flag,
        });
    }

    /// Enqueue an ingest-dir job. Non-blocking.
    pub fn enqueue_ingest_dir(&self, task_id: i32, ingest_dir: PathBuf) {
        self.send(QueuedJob::IngestDir {
            task_id,
            ingest_dir,
        });
    }

    /// Enqueue a synchronous single-track download. Returns a receiver that
    /// resolves once the worker has processed the job. Awaiting this receiver
    /// naturally blocks the HTTP handler until the queue reaches this job,
    /// which is the intended behavior — no download ever bypasses the queue.
    pub fn enqueue_single_track(
        &self,
        url: String,
    ) -> oneshot::Receiver<SoundomeResult<Track>> {
        let (responder, rx) = oneshot::channel();
        self.send(QueuedJob::SingleTrack { url, responder });
        rx
    }

    fn send(&self, job: QueuedJob) {
        if let Err(err) = self.sender.send(job) {
            // The receiver only closes if the worker thread panicked, which
            // is fatal — surface it clearly.
            tracing::error!("Task executor channel closed unexpectedly: {}", err);
        }
    }
}

async fn worker_loop(
    mut receiver: UnboundedReceiver<QueuedJob>,
    services: Arc<ServiceLayer>,
    registry: Arc<CancellationRegistry>,
) {
    tracing::info!("Task executor started (serial, concurrency = 1)");

    while let Some(job) = receiver.recv().await {
        run_job(job, &services, &registry).await;
    }

    tracing::warn!("Task executor channel closed, worker exiting");
}

async fn run_job(
    job: QueuedJob,
    services: &Arc<ServiceLayer>,
    registry: &Arc<CancellationRegistry>,
) {
    let db_url = Config::get().database.url.clone();
    let conn = &mut database::init_connection(&db_url);

    match job {
        QueuedJob::PlaylistSync {
            task_id,
            url,
            cancel_flag,
        } => {
            mark_running(services, conn, task_id);
            let result = services
                .download_service
                .sync_playlist_from_url(&url, conn, Some(task_id), Some(cancel_flag))
                .await;
            finalize_task(services, registry, conn, task_id, result.map(|_| ()));
        }
        QueuedJob::ArtistSync {
            task_id,
            url,
            cancel_flag,
        } => {
            mark_running(services, conn, task_id);
            let result = services
                .download_service
                .sync_artist_from_url(&url, conn, Some(task_id), Some(cancel_flag))
                .await;
            finalize_task(services, registry, conn, task_id, result.map(|_| ()));
        }
        QueuedJob::AlbumSync {
            task_id,
            url,
            cancel_flag,
        } => {
            mark_running(services, conn, task_id);
            let result = services
                .download_service
                .sync_album_from_url(&url, conn, Some(task_id), Some(cancel_flag))
                .await;
            finalize_task(services, registry, conn, task_id, result.map(|_| ()));
        }
        QueuedJob::IngestDir {
            task_id,
            ingest_dir,
        } => {
            mark_running(services, conn, task_id);
            let result = services
                .download_service
                .ingest_local_dir(conn, &ingest_dir, task_id)
                .await;
            finalize_task(services, registry, conn, task_id, result);
        }
        QueuedJob::SingleTrack { url, responder } => {
            let result = services
                .download_service
                .download_track_from_url(&url, conn)
                .await;
            // Ignore send errors: the caller may have already given up (client
            // dropped the connection). Nothing else to do.
            let _ = responder.send(result);
        }
    }
}

fn mark_running(
    services: &Arc<ServiceLayer>,
    conn: &mut diesel::SqliteConnection,
    task_id: i32,
) {
    if let Err(e) = services.task_service.set_running(conn, task_id) {
        tracing::error!("Failed to set task {} as running: {}", task_id, e);
    }
}

fn finalize_task(
    services: &Arc<ServiceLayer>,
    registry: &Arc<CancellationRegistry>,
    conn: &mut diesel::SqliteConnection,
    task_id: i32,
    result: SoundomeResult<()>,
) {
    match result {
        Ok(()) => {
            if let Err(e) = services.task_service.set_completed(conn, task_id) {
                tracing::error!("Failed to mark task {} as completed: {}", task_id, e);
            }
        }
        Err(Error::Cancelled) => {
            tracing::info!("Task {} was cancelled", task_id);
            if let Err(e) = services.task_service.set_cancelled(conn, task_id) {
                tracing::error!("Failed to mark task {} as cancelled: {}", task_id, e);
            }
        }
        Err(e) => {
            tracing::error!("Task {} failed: {}", task_id, e);
            if let Err(e2) = services
                .task_service
                .set_failed(conn, task_id, &e.to_string())
            {
                tracing::error!("Failed to mark task {} as failed: {}", task_id, e2);
            }
        }
    }
    registry.remove(task_id);
}
