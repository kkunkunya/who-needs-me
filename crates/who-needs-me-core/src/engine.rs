use std::{collections::BTreeMap, path::Path};

use crate::{Adapter, CoreError, CoreResult, Environment, Session, SessionKey};

pub struct Engine {
    adapters: Vec<Box<dyn Adapter>>,
}

impl Engine {
    pub fn new(adapters: Vec<Box<dyn Adapter>>) -> Self {
        Self { adapters }
    }

    pub fn collect_sessions(&self, environment: &Environment) -> CoreResult<Vec<Session>> {
        let mut sessions = BTreeMap::<SessionKey, Session>::new();

        for adapter in &self.adapters {
            let descriptor = adapter.descriptor();
            let root = environment.session_data_root(descriptor.provider())?;
            for artifact in adapter.discover(root)? {
                let session = adapter.parse(&artifact)?;
                let cwd = session
                    .metadata
                    .cwd
                    .as_deref()
                    .map(Path::new)
                    .unwrap_or_else(|| Path::new(""));
                if !environment.process_probe().is_alive(descriptor, cwd) {
                    continue;
                }

                let key = session.key();
                if sessions.insert(key.clone(), session).is_some() {
                    return Err(CoreError::DuplicateSession(format!(
                        "{:?}/{}",
                        key.provider, key.session_id
                    )));
                }
            }
        }

        Ok(sessions.into_values().collect())
    }
}
