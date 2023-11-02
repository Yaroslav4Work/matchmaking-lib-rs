pub mod pool;
use pool::{MinMax, Pool, PoolSettings};

use crate::pool::pool_matches;

fn main() {
    let user_1_id = "1".to_string();
    let user_2_id = "2".to_string();
    let user_3_id = "3".to_string();

    let team_1_id = user_1_id.clone();
    let team_2_id = user_2_id.clone();
    let team_3_id = user_3_id.clone();

    let mut pool = Pool::new(PoolSettings {
        players_in_team: 1,
        teams_in_pool: MinMax(2, 4),
        teams_in_match: MinMax(2, 2),
    });

    pool.set_team_completed_callback(|team| {
        let team_id = team.borrow().id.clone();
        println!("Team (ID: {team_id}) has been completed");
    });

    pool.set_match_scheduled_callback(|pool_match| {
        let pool_match_id = pool_match.borrow().id.clone();
        let teams_ids: Vec<String> = pool_match
            .borrow()
            .rivals
            .iter()
            .map(|team| team.borrow().id.clone())
            .collect();

        let teams_ids = teams_ids.join(", ");

        println!("Teams (with ID: {teams_ids}) in match (with ID: {pool_match_id})");
    });

    pool.set_next_stage_callback(|pool_matches| {
        let pool_matches_ids: Vec<String> = pool_matches
            .iter()
            .map(|pool_match| pool_match.borrow().id.clone())
            .collect();

        let pool_matches_ids = pool_matches_ids.join(", ");

        if let Some(pool_match) = pool_matches.get(0) {
            let stage = pool_match.borrow().stage;
            println!("Next stage: {stage} matches (with ids: {pool_matches_ids})");
        }
    });

    pool.set_pool_ended_callback(|pool| {
        let winner_id = pool.get_winner_id().unwrap();

        println!("Pool has been ended. Winner ID: {winner_id}");
    });

    dbg!(pool.add_team(team_1_id.clone()).unwrap());
    dbg!(pool
        .add_user_to_team(team_1_id.clone(), user_1_id.clone())
        .unwrap());

    dbg!(pool.add_team(team_2_id.clone()).unwrap());
    dbg!(pool
        .add_user_to_team(team_2_id.clone(), user_2_id.clone())
        .unwrap());

    dbg!(pool.add_team(team_3_id.clone()).unwrap());
    dbg!(pool
        .add_user_to_team(team_3_id.clone(), user_3_id.clone())
        .unwrap());

    dbg!(pool.start());

    dbg!(pool.set_match_winner(
        vec![team_1_id.clone(), team_2_id.clone()].concat(),
        team_1_id.clone()
    ));

    dbg!(pool.set_match_winner(
        vec![team_3_id.clone(), team_1_id.clone()].concat(),
        team_3_id.clone()
    ));
}
