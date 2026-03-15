use std::collections::VecDeque;

#[derive(Debug)]
pub struct ClientIdManager {
    available: VecDeque<usize>,
}
impl ClientIdManager {
    pub fn with_maximum(maximum: usize) -> Self {
        Self {
            available: (1..=maximum).collect(),
        }
    }

    pub fn add(&mut self) -> Option<usize> {
        self.available.pop_front()
    }

    pub fn remove(&mut self, client_id: usize) -> Result<(), ClientIdManagerError> {
        if self.available.contains(&client_id) {
            return Err(ClientIdManagerError::NotFound(client_id));
        }
        self.available.push_back(client_id);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ClientIdManagerError {
    #[error("Could not find specified session {0}")]
    NotFound(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_session_generates_client_ids() {
        let mut manager = ClientIdManager::with_maximum(1000);

        assert_eq!(manager.add(), Some(1));
        assert_eq!(manager.add(), Some(2));
    }

    #[test]
    fn it_is_limited_by_a_number_of_maximum_clients() {
        let mut manager = ClientIdManager::with_maximum(2);

        assert_eq!(manager.add(), Some(1));
        assert_eq!(manager.add(), Some(2));
        assert_eq!(manager.add(), None);
    }

    #[test]
    fn remove_session_from_list() {
        let mut manager = ClientIdManager::with_maximum(2);

        manager.add();
        assert!(manager.remove(1).is_ok());
    }

    #[test]
    fn remove_session_that_does_not_exist() {
        let mut manager = ClientIdManager::with_maximum(2);

        manager.add();
        assert_eq!(manager.remove(2), Err(ClientIdManagerError::NotFound(2)));
    }

    #[test]
    fn removed_id_is_reusable() {
        let mut manager = ClientIdManager::with_maximum(2);

        assert_eq!(manager.add(), Some(1));
        assert_eq!(manager.add(), Some(2));
        assert_eq!(manager.add(), None);

        manager.remove(1).unwrap();
        assert_eq!(manager.add(), Some(1));
    }
}
