#[derive(Debug)]
pub struct Team {
    pub(crate) id: String,
    pub(crate) stage: u32,
    pub(crate) players_in_team: u32,
    pub(crate) users: Vec<String>,
}

impl Team {
    pub(crate) fn new(id: String, players_in_team: u32) -> Self {
        Self {
            id,
            stage: 1,
            players_in_team,
            users: vec![],
        }
    }

    pub(crate) fn add_user(&mut self, user_id: String) -> Result<&Self, &'static str> {
        if self.players_in_team == self.users.len() as u32 {
            return Err(MAXIMUM_PLAYERS_COUNT_ERROR);
        }

        self.users.push(user_id.clone());

        Ok(self)
    }

    pub(crate) fn add_users(&mut self, users_ids: Vec<String>) -> Result<&Self, &'static str> {
        if (self.players_in_team as usize) < users_ids.len() {
            return Err(MAXIMUM_PLAYERS_COUNT_ERROR);
        }

        self.users.append(users_ids.clone().as_mut());

        Ok(self)
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.players_in_team == self.users.len() as u32
    }

    pub(crate) fn increment_stage(&mut self) -> &Self {
        self.stage += 1;
        self
    }
}

pub const MAXIMUM_PLAYERS_COUNT_ERROR: &str = "The number of players may be exceeded";
