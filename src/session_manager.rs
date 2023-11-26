use std::{collections::HashSet, hash};

#[derive(Debug)]
pub struct SessionManager<T: hash::Hash + Eq + PartialEq> {
    clients: HashSet<ValueWithClientId<T>>,
    maximum: usize,
}
impl<T> SessionManager<T>
where
    T: hash::Hash + Eq + PartialEq,
{
    pub fn with_maximum(maximum: usize) -> Self {
        Self {
            clients: Default::default(),
            maximum,
        }
    }

    pub fn add_session(&mut self, value: T) -> Result<usize, SessionManagerError> {
        if self.clients.iter().any(|x| x.0 == value) {
            return Err(SessionManagerError::Existent);
        }

        match self.next_client_id() {
            Some(client_id) => {
                self.clients.insert(ValueWithClientId(value, client_id));

                Ok(client_id)
            }
            None => Err(SessionManagerError::SessionFull),
        }
    }

    fn next_client_id(&self) -> Option<usize> {
        let existing_ids: HashSet<usize> = self.clients.iter().map(|v| v.1).collect();
        (1..=self.maximum).find(|&i| !existing_ids.contains(&i))
    }
}
impl<T: hash::Hash + Eq> Default for SessionManager<T> {
    fn default() -> Self {
        Self {
            clients: Default::default(),
            maximum: usize::MAX,
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SessionManagerError {
    #[error("No available slot")]
    SessionFull,

    #[error("The session is already present")]
    Existent,
}

#[derive(Default, Debug, Hash, PartialEq, Eq)]
pub struct ValueWithClientId<T>(T, usize);

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Hash, PartialEq, Eq)]
    struct ClientId(usize);

    #[test]
    fn add_session_generates_client_ids() {
        let mut session_manager = SessionManager::default();

        assert_eq!(session_manager.add_session(ClientId(1)), Ok(1));
        assert_eq!(session_manager.add_session(ClientId(2)), Ok(2));
    }

    #[test]
    fn it_is_limited_by_a_number_of_maximum_clients() {
        let mut session_manager = SessionManager::with_maximum(2);

        assert_eq!(session_manager.add_session(ClientId(1)), Ok(1));
        assert_eq!(session_manager.add_session(ClientId(2)), Ok(2));
        assert_eq!(
            session_manager.add_session(ClientId(3)),
            Err(SessionManagerError::SessionFull)
        );
    }

    #[test]
    fn client_already_on_list() {
        let mut session_manager = SessionManager::with_maximum(2);

        assert_eq!(session_manager.add_session(ClientId(1)), Ok(1));
        assert_eq!(
            session_manager.add_session(ClientId(1)),
            Err(SessionManagerError::Existent)
        );
    }
}
