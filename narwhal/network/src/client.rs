// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::{sync::{Arc, Mutex}, collections::BTreeMap};

use crypto::NetworkPublicKey;
use types::{PrimaryToWorker, WorkerToWorker};

/// Uses a map to allow running multiple Narwhal instances in the same process.
/// TODO: after Rust 1.66, use BTreeMap::new() instead of wrapping it in an Option.
static LOCAL_WORKER_CLIENTS: Mutex<Vec<(NetworkPublicKey, Arc<LocalWorkerClient>)>> =
    Mutex::new(Vec::new());

pub struct LocalWorkerClient {
    primary_to_worker: Arc<dyn PrimaryToWorker>,
    worker_to_worker: Arc<dyn WorkerToWorker>,
}

impl LocalWorkerClient {
    /// Sets the instance of LocalWorkerClient.
    pub fn add_global(worker_name: NetworkPublicKey, client: Arc<Self>) -> bool {
        let mut clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        if clients.iter().any(|(name, c)| name == &worker_name) {
            return false;
        }
        clients.push((worker_name, client));
    }

    /// Gets the instance of LocalWorkerClient.
    pub fn get_global(worker_name: &NetworkPublicKey) -> Option<Arc<Self>> {
        let clients = LOCAL_WORKER_CLIENTS.lock().unwrap();
        for (name, c) in clients.iter() {
            if name == worker_name {
                return Some(c.clone());
            }
        }
        None
    }
}