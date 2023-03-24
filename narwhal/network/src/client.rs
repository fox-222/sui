// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use anemo::{rpc::Status, types::response::StatusCode, PeerId, Request, Response};
use tokio::time::{sleep, timeout};
use types::{PrimaryToWorker, WorkerSynchronizeMessage, WorkerToPrimary, WorkerToWorker};

/// Uses a Vec to allow running multiple Narwhal instances in the same process.
static LOCAL_PRIMARY_CLIENTS: Mutex<Vec<(PeerId, Arc<LocalPrimaryClient>)>> =
    Mutex::new(Vec::new());

/// Uses a Vec to support running multiple Narwhal workers.
static LOCAL_WORKER_CLIENTS: Mutex<Vec<(PeerId, Arc<LocalWorkerClient>)>> = Mutex::new(Vec::new());

pub struct LocalPrimaryClient {
    worker_to_primary: Arc<dyn WorkerToPrimary>,
    shutdown: AtomicBool,
}

impl LocalPrimaryClient {
    pub fn new(worker_to_primary: Arc<dyn WorkerToPrimary>) -> Self {
        Self {
            worker_to_primary,
            shutdown: AtomicBool::new(false),
        }
    }

    /// Sets the instance of LocalPrimarylient.
    pub fn set_global(primary_peer_id: PeerId, client: Arc<Self>) {
        let mut clients = LOCAL_PRIMARY_CLIENTS.lock().unwrap();
        // Replace the existing client with the same key, or appends to the end of the list.
        if clients.iter_mut().any(|(name, c)| {
            if name == &primary_peer_id {
                *c = client.clone();
                return true;
            }
            false
        }) {
            return;
        }
        clients.push((primary_peer_id, client));
    }

    /// Gets the instance of LocalPrimarylient.
    pub fn get_global(primary_peer_id: &PeerId) -> Option<Arc<Self>> {
        let clients = LOCAL_PRIMARY_CLIENTS.lock().unwrap();
        for (name, c) in clients.iter() {
            if name == primary_peer_id {
                return Some(c.clone());
            }
        }
        None
    }

    /// Marks the specified LocalPrimarylient as shutdown.
    pub fn shutdown_global(primary_peer_id: &PeerId) {
        let mut clients = LOCAL_PRIMARY_CLIENTS.lock().unwrap();
        assert!(clients
            .iter_mut()
            .any(|(key, client)| if key == primary_peer_id {
                client.shutdown();
                true
            } else {
                false
            }));
    }

    fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
    }

    pub async fn wait_for(primary_peer_id: &PeerId) -> Result<Arc<Self>, Status> {
        timeout(Duration::from_secs(10), async {
            loop {
                if let Some(client) = Self::get_global(primary_peer_id) {
                    return client;
                }
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| {
            Status::new_with_message(
                StatusCode::InternalServerError,
                format!("Primary {primary_peer_id} has not started!"),
            )
        })
    }
}

pub struct LocalWorkerClient {
    primary_to_worker: Arc<dyn PrimaryToWorker>,
    worker_to_worker: Arc<dyn WorkerToWorker>,
    shutdown: AtomicBool,
}

impl LocalWorkerClient {
    pub fn new(
        primary_to_worker: Arc<dyn PrimaryToWorker>,
        worker_to_worker: Arc<dyn WorkerToWorker>,
    ) -> Self {
        Self {
            primary_to_worker,
            worker_to_worker,
            shutdown: AtomicBool::new(false),
        }
    }

    /// Sets the instance of LocalWorkerClient.
    pub fn set_global(worker_peer_id: PeerId, client: Arc<Self>) {
        let mut clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        // Replace the existing client with the same key, or appends to the end of the list.
        if clients.iter_mut().any(|(name, c)| {
            if name == &worker_peer_id {
                *c = client.clone();
                return true;
            }
            false
        }) {
            return;
        }
        clients.push((worker_peer_id, client));
    }

    /// Gets the instance of LocalWorkerClient.
    pub fn get_global(worker_peer_id: &PeerId) -> Option<Arc<Self>> {
        let clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        for (name, c) in clients.iter() {
            if name == worker_peer_id {
                return Some(c.clone());
            }
        }
        None
    }

    /// Marks the specified LocalPrimarylient as shutdown.
    pub fn shutdown_global(primary_peer_id: &PeerId) {
        let mut clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        assert!(clients
            .iter_mut()
            .any(|(key, client)| if key == primary_peer_id {
                client.shutdown();
                true
            } else {
                false
            }));
    }

    fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
    }

    pub async fn wait_for(worker_peer_id: &PeerId) -> Result<Arc<Self>, Status> {
        timeout(Duration::from_secs(10), async {
            loop {
                if let Some(client) = Self::get_global(worker_peer_id) {
                    return client;
                }
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| {
            Status::new_with_message(
                StatusCode::NotFound,
                format!("Worker {worker_peer_id} has not started!"),
            )
        })
    }

    pub async fn synchronize(
        &self,
        request: Request<WorkerSynchronizeMessage>,
    ) -> Result<Response<()>, anemo::rpc::Status> {
        if self.shutdown.load(Ordering::Acquire) {
            return Err(Status::new_with_message(
                StatusCode::NotFound,
                "Worker has shutdown!",
            ));
        }
        self.primary_to_worker.synchronize(request).await
    }
}
