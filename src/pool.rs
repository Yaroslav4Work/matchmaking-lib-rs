pub mod pool_matches;
pub mod teams;

use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use pool_matches::PoolMatch;
use teams::Team;

#[derive(Debug, Clone)]
pub struct MinMax(pub u32, pub u32);

#[derive(Debug, Clone)]
pub struct PoolSettings {
    pub players_in_team: u32,
    pub teams_in_pool: MinMax,
    pub teams_in_match: MinMax,
}

#[derive(Debug)]
pub(crate) struct Pool {
    settings: PoolSettings,

    teams: Vec<Rc<RefCell<Team>>>,
    matches: Vec<Rc<RefCell<PoolMatch>>>,

    stage: u32,

    is_started: bool,
    is_ended: bool,

    winner_team_id: Option<String>,

    team_completed_callback: Option<fn(team: Rc<RefCell<Team>>)>,
    next_stage_callback: Option<fn(matches: Vec<Rc<RefCell<PoolMatch>>>)>,
    match_scheduled_callback: Option<fn(pool_match: Rc<RefCell<PoolMatch>>)>,
    pool_ended_callback: Option<fn(pool: &Self)>,
}

impl Pool {
    pub fn new(settings: PoolSettings) -> Self {
        Self {
            settings,

            teams: vec![],
            matches: vec![],

            stage: 1,

            is_started: false,
            is_ended: false,

            winner_team_id: None,

            team_completed_callback: None,
            next_stage_callback: None,
            match_scheduled_callback: None,
            pool_ended_callback: None,
        }
    }

    pub fn set_team_completed_callback(&mut self, cb: fn(team: Rc<RefCell<Team>>)) -> &mut Self {
        self.team_completed_callback = Some(cb);
        self
    }

    pub fn set_next_stage_callback(
        &mut self,
        cb: fn(matches: Vec<Rc<RefCell<PoolMatch>>>),
    ) -> &mut Self {
        self.next_stage_callback = Some(cb);
        self
    }

    pub fn set_match_scheduled_callback(
        &mut self,
        cb: fn(pool_match: Rc<RefCell<PoolMatch>>),
    ) -> &mut Self {
        self.match_scheduled_callback = Some(cb);
        self
    }

    pub fn set_pool_ended_callback(&mut self, cb: fn(pool: &Self)) -> &mut Self {
        self.pool_ended_callback = Some(cb);
        self
    }

    pub fn get_winner_id(&self) -> Option<String> {
        if let Some(winner_id) = &self.winner_team_id {
            return Some(winner_id.clone());
        }

        None
    }

    pub fn add_team(&mut self, team_id: String) -> Result<&mut Self, &'static str> {
        if self.is_started {
            return Err("Pool game has been started");
        }

        if self.settings.teams_in_pool.1 == self.teams.len() as u32 {
            return Err("Teams maximum count already been reached");
        }

        let team = Team::new(team_id.clone(), self.settings.players_in_team.clone());

        self.teams.push(Rc::new(RefCell::new(team)));

        Ok(self)
    }

    pub fn get_team_by_id(&self, team_id: String) -> Result<Rc<RefCell<Team>>, &'static str> {
        if let Some(found_team) = self.teams.iter().find(|team| team.borrow().id == team_id) {
            return Ok(Rc::clone(found_team));
        }

        Err("Team with current id not found")
    }

    pub fn add_user_to_team(
        &self,
        team_id: String,
        user_id: String,
    ) -> Result<&Self, &'static str> {
        if self.is_started {
            return Err("Pool game has been started");
        }

        let team = self.get_team_by_id(team_id)?;

        team.as_ref().borrow_mut().add_user(user_id)?;

        if self.settings.players_in_team as usize == team.borrow().users.len() {
            if let Some(team_completed_callback) = self.team_completed_callback {
                team_completed_callback(Rc::clone(&team));
            }
        }

        Ok(self)
    }

    pub fn get_match_by_id(
        &self,
        match_id: String,
    ) -> Result<Rc<RefCell<PoolMatch>>, &'static str> {
        if let Some(found_match) = self
            .matches
            .iter()
            .find(|pool_match| pool_match.borrow().id == match_id)
        {
            return Ok(Rc::clone(found_match));
        }

        Err("Team with current id not found")
    }

    pub fn get_last_ended_stage(&self) -> u32 {
        let expected_matches = self
            .matches
            .iter()
            .filter(|pool_match| pool_match.borrow().stage == self.stage)
            .collect::<Vec<&Rc<RefCell<PoolMatch>>>>();

        let ended_matches: Vec<&&Rc<RefCell<PoolMatch>>> = expected_matches
            .iter()
            .filter(|pool_match| pool_match.borrow().winner.is_some())
            .collect();

        if ended_matches.len() < expected_matches.len() {
            return self.stage - 1;
        }

        self.stage
    }

    fn set_match_winner_by_ref(
        &self,
        pool_match: &mut RefMut<PoolMatch>,
        winner_team_id: &String,
    ) -> Result<(), &'static str> {
        pool_match.set_winner(winner_team_id.clone())?;

        Ok(())
    }

    fn check_winner(&self) -> Option<Rc<RefCell<Team>>> {
        if !self.is_started {
            return None;
        }

        let last_stage = self.get_last_ended_stage();

        let last_matches: Vec<&Rc<RefCell<PoolMatch>>> = self
            .matches
            .iter()
            .filter(|pool_match| pool_match.borrow().stage == last_stage)
            .collect();

        if last_matches.len() != 1 {
            return None;
        }

        if let Some(winner) = &last_matches[0].borrow().winner {
            return Some(Rc::clone(winner));
        }

        None
    }

    fn try_end_or_next(&mut self) -> Result<&Self, &'static str> {
        if let Some(winner) = self.check_winner() {
            self.winner_team_id = Some(winner.borrow().id.clone());
            self.is_ended = true;

            if let Some(pool_ended_callback) = self.pool_ended_callback {
                pool_ended_callback(&self);
            }
        } else {
            self.make_matches()?;
        }

        Ok(self)
    }

    pub fn set_match_winner(
        &mut self,
        match_id: String,
        winner_team_id: String,
    ) -> Result<&Self, &'static str> {
        if !self.is_started {
            return Err("Pool game has not started yet");
        }

        if self.is_ended {
            return Err("Pool game has been ended");
        }

        self.set_match_winner_by_ref(
            &mut self.get_match_by_id(match_id)?.as_ref().borrow_mut(),
            &winner_team_id,
        )?;

        self.get_team_by_id(winner_team_id)?
            .as_ref()
            .borrow_mut()
            .increment_stage();

        Ok(self.try_end_or_next()?)
    }

    fn teams_splice(
        &self,
        mut data: (Vec<Rc<RefCell<Team>>>, Vec<Vec<Rc<RefCell<Team>>>>),
    ) -> (Vec<Rc<RefCell<Team>>>, Vec<Vec<Rc<RefCell<Team>>>>) {
        let mut splice_to = self.settings.teams_in_match.1 as usize;
        let teams_len = data.0.len();

        if teams_len < self.settings.teams_in_match.0 as usize {
            return data;
        }

        if splice_to > teams_len {
            splice_to = teams_len;
        }

        data.1.push(data.0.splice(0..splice_to, []).collect());

        self.teams_splice(data)
    }

    fn make_match(&mut self, stage: u32, teams: Vec<Rc<RefCell<Team>>>) -> Rc<RefCell<PoolMatch>> {
        let pool_match = Rc::new(RefCell::new(PoolMatch::new(stage, teams)));

        self.matches.push(Rc::clone(&pool_match));

        Rc::clone(&pool_match)
    }

    fn get_unmatched_teams(&self) -> Vec<Rc<RefCell<Team>>> {
        let last_ended_stage = self.get_last_ended_stage();

        if last_ended_stage == self.stage - 1 {
            return Vec::new();
        }

        self.matches
            .iter()
            .filter(|pool_match| pool_match.borrow().stage == self.stage)
            .map(|pool_match| match &pool_match.borrow().winner {
                Some(winner) => Rc::clone(&winner),
                None => panic!("Match has no winner"),
            })
            .collect::<Vec<Rc<RefCell<Team>>>>()
    }

    fn make_matches(&mut self) -> Result<&Self, &'static str> {
        let mut unmatched_teams = Vec::new();

        if self.matches.len() == 0 {
            if self.settings.teams_in_pool.0 as usize > self.teams.len() {
                return Err("Has no minimal required count of teams");
            }

            unmatched_teams.append(
                &mut self
                    .teams
                    .iter()
                    .map(|team| Rc::clone(team))
                    .collect::<Vec<Rc<RefCell<Team>>>>(),
            )
        } else {
            unmatched_teams.append(&mut self.get_unmatched_teams());
            self.stage += 1;
        }

        let spliced_teams = self.teams_splice((
            unmatched_teams
                .iter()
                .map(|team_wrap| Rc::clone(&team_wrap))
                .collect(),
            Vec::new(),
        ));

        let mut next_stage_matches: Vec<Rc<RefCell<PoolMatch>>> = Vec::new();

        if spliced_teams.0.len() > 0 {
            for team in spliced_teams.0 {
                let team_clone = Rc::clone(&team);
                let pool_match = self.make_match(self.stage, vec![Rc::clone(&team_clone)]);

                next_stage_matches.push(Rc::clone(&pool_match));

                if let Some(match_scheduled_callback) = self.match_scheduled_callback {
                    match_scheduled_callback(Rc::clone(&pool_match));
                }

                self.set_match_winner_by_ref(
                    &mut pool_match.borrow_mut(),
                    &team_clone.borrow().id,
                )?;
            }
        }

        for teams_splice in spliced_teams.1 {
            if teams_splice.len() > 0 {
                let pool_match = self.make_match(self.stage, teams_splice);

                next_stage_matches.push(Rc::clone(&pool_match));

                if let Some(match_scheduled_callback) = self.match_scheduled_callback {
                    match_scheduled_callback(Rc::clone(&pool_match));
                }
            }
        }

        if let Some(next_stage_callback) = self.next_stage_callback {
            next_stage_callback(next_stage_matches);
        }

        Ok(self)
    }

    pub fn start(&mut self) -> Result<&Self, &'static str> {
        if self.is_started {
            return Err("Already started");
        }

        if self.is_ended {
            return Err("Already ended");
        }

        self.teams = self
            .teams
            .iter()
            .filter(|team| team.borrow().is_complete())
            .map(|team| Rc::clone(team))
            .collect::<Vec<Rc<RefCell<Team>>>>();

        self.is_started = true;

        self.make_matches()?;

        Ok(self)
    }
}
