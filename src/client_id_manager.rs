use std::collections::HashSet;

#[derive(Debug)]
pub struct ClientIdManager {
    clients: HashSet<usize>,
    maximum: usize,
}
impl ClientIdManager {
    pub fn with_maximum(maximum: usize) -> Self {
        Self {
            clients: Default::default(),
            maximum,
        }
    }

    pub fn add(&mut self) -> Option<usize> {
        match self.next_client_id() {
            Some(client_id) => {
                self.clients.insert(client_id);
                Some(client_id)
            }
            None => None,
        }
    }

    pub fn remove(&mut self, client_id: usize) -> Result<(), ClientIdManagerError> {
        match self.clients.remove(&client_id) {
            true => Ok(()),
            false => Err(ClientIdManagerError::NotFound(client_id)),
        }
    }

    fn next_client_id(&self) -> Option<usize> {
        (1..=self.maximum).find(|&i| !self.clients.contains(&i))
    }
}
impl Default for ClientIdManager {
    fn default() -> Self {
        Self {
            clients: Default::default(),
            maximum: usize::MAX,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClientIdManagerError {
    #[error("No available slot")]
    SessionFull,

    #[error("Could not find specified session {0}")]
    NotFound(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_session_generates_client_ids() {
        let mut client_id_manager = ClientIdManager::default();

        assert_eq!(client_id_manager.add(), Some(1));
        assert_eq!(client_id_manager.add(), Some(2));
    }

    #[test]
    fn it_is_limited_by_a_number_of_maximum_clients() {
        let mut client_id_manager = ClientIdManager::with_maximum(2);

        assert_eq!(client_id_manager.add(), Some(1));
        assert_eq!(client_id_manager.add(), Some(2));
        assert_eq!(client_id_manager.add(), None);
    }

    #[test]
    fn remove_session_from_list() {
        let mut client_id_manager = ClientIdManager::with_maximum(2);

        client_id_manager.add();
        assert!(client_id_manager.remove(1).is_ok());
    }

    #[test]
    fn remove_session_that_does_not_exist() {
        let mut client_id_manager = ClientIdManager::with_maximum(2);

        client_id_manager.add();
        assert_eq!(
            client_id_manager.remove(2),
            Err(ClientIdManagerError::NotFound(2))
        );
    }
}
