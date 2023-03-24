// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::sync::{Arc, Mutex};

use anemo::PeerId;
use types::{PrimaryToWorker, WorkerToPrimary, WorkerToWorker};

/// Uses a Vec to allow running multiple Narwhal instances in the same process.
static LOCAL_PRIMARY_CLIENTS: Mutex<Vec<(PeerId, Arc<LocalPrimaryClient>)>> =
    Mutex::new(Vec::new());

/// Uses a Vec to support running multiple Narwhal workers.
static LOCAL_WORKER_CLIENTS: Mutex<Vec<(PeerId, Arc<LocalWorkerClient>)>> = Mutex::new(Vec::new());

pub struct LocalPrimaryClient {
    pub worker_to_primary: Arc<dyn WorkerToPrimary>,
}

impl LocalPrimaryClient {
    /// Sets the instance of LocalPrimarylient.
    pub fn add_global(primary_peer_id: PeerId, client: Arc<Self>) -> bool {
        let mut clients = LOCAL_PRIMARY_CLIENTS.lock().unwrap();
        if clients.iter().any(|(name, _)| name == &primary_peer_id) {
            return false;
        }
        clients.push((primary_peer_id, client));
        true
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

    /// Clears all LocalPrimarylient.
    pub fn clear_global() {
        let mut clients = LOCAL_PRIMARY_CLIENTS.lock().unwrap();
        clients.clear();
    }
}

pub struct LocalWorkerClient {
    pub primary_to_worker: Arc<dyn PrimaryToWorker>,
    pub worker_to_worker: Arc<dyn WorkerToWorker>,
}

impl LocalWorkerClient {
    /// Sets the instance of LocalWorkerClient.
    pub fn add_global(worker_peer_id: PeerId, client: Arc<Self>) -> bool {
        let mut clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        if clients.iter().any(|(name, _)| name == &worker_peer_id) {
            return false;
        }
        clients.push((worker_peer_id, client));
        true
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

    /// Clears all LocalWorkerClient.
    pub fn clear_global() {
        let mut clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        clients.clear();
    }
}
