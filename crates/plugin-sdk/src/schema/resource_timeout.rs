use std::time::Duration;

use serde::{Deserialize, Serialize};
const DEFAULT_READ_TIMEOUT: Duration = Duration::from_secs(60 * 5);
const DEFAULT_UPDATE_TIMEOUT: Duration = Duration::from_secs(60 * 30);
const DEFAULT_CREATE_TIMEOUT: Duration = Duration::from_secs(60 * 30);
const DEFAULT_DELETE_TIMEOUT: Duration = Duration::from_secs(60 * 60);

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourceTimeouts {
    create: Duration,
    delete: Duration,
    read: Duration,
    update: Duration,
}

impl ResourceTimeouts {
    pub fn new(create: Duration, delete: Duration, read: Duration, update: Duration) -> Self {
        ResourceTimeouts {
            create,
            delete,
            read,
            update,
        }
    }

    pub fn get_create_timeout(&self) -> Duration {
        self.create
    }
    pub fn get_delete_timeout(&self) -> Duration {
        self.delete
    }
    pub fn get_read_timeout(&self) -> Duration {
        self.read
    }
    pub fn get_update_timeout(&self) -> Duration {
        self.update
    }

    pub fn set_create_timeout(&mut self, timeout: Duration) {
        self.create = timeout;
    }
    pub fn set_delete_timeout(&mut self, timeout: Duration) {
        self.delete = timeout;
    }
    pub fn set_read_timeout(&mut self, timeout: Duration) {
        self.read = timeout;
    }
    pub fn set_update_timeout(&mut self, timeout: Duration) {
        self.update = timeout;
    }
}

impl Default for ResourceTimeouts {
    fn default() -> Self {
        ResourceTimeouts {
            create: DEFAULT_CREATE_TIMEOUT,
            delete: DEFAULT_DELETE_TIMEOUT,
            read: DEFAULT_READ_TIMEOUT,
            update: DEFAULT_UPDATE_TIMEOUT,
        }
    }
}
