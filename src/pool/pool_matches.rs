use std::{cell::RefCell, rc::Rc};

use super::teams::Team;

#[derive(Debug)]
pub struct PoolMatch {
    pub(crate) id: String,
    pub(crate) stage: u32,
    pub(crate) rivals: Vec<Rc<RefCell<Team>>>,
    pub(crate) winner: Option<Rc<RefCell<Team>>>,
}

impl PoolMatch {
    pub(crate) fn new(stage: u32, rivals: Vec<Rc<RefCell<Team>>>) -> Self {
        Self {
            id: rivals
                .iter()
                .map(|team| team.borrow().id.clone())
                .collect::<Vec<String>>()
                .concat(),
            stage,
            rivals,
            winner: None,
        }
    }

    pub(crate) fn set_winner(&mut self, team_id: String) -> Result<&mut Self, &'static str> {
        if let Some(found_team) = self
            .rivals
            .iter()
            .find(|curr_team| curr_team.borrow().id == team_id)
        {
            self.winner = Some(Rc::clone(&found_team));
        } else {
            return Err("Not found current team id in match");
        }

        return Ok(self);
    }
}
