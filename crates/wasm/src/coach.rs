//! The coach for the browser. Every function takes a save (seed + action log), rebuilds the game
//! itself, and analyzes from South's seat, so it can run in its own worker without slowing play.

use crate::table::{replay, replay_to_deal, SaveData, HUMAN};
use crate::view::card_view;
use fh_ai::coach::{dd_trace, expert_replay, Coach, Grade, OptionEval};
use fh_core::{Action, Phase, Seat};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionView {
    /// Index in the legal actions at that point.
    pub index: usize,
    pub label: String,
    pub ev: f64,
    pub makes: Option<f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HintView {
    /// Index of the recommended action in `view.legal`.
    pub best: usize,
    pub best_label: String,
    pub reasons: Vec<String>,
    pub options: Vec<OptionView>,
    pub worlds: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GradeView {
    /// Which action of the game this grades (its position in the save log).
    pub action_number: usize,
    pub phase: &'static str,
    pub chosen_label: String,
    pub best_label: String,
    /// "best", "good", "inaccuracy", "mistake" or "blunder".
    pub verdict: &'static str,
    /// Expected points given up.
    pub loss: f64,
    pub reasons: Vec<String>,
    pub options: Vec<OptionView>,
    /// Which deal this decision belonged to (1-based, counting passed-out deals).
    pub deal: usize,
}

fn label(action: Action) -> String {
    action.to_string()
}

fn options_view(options: &[OptionEval], legal: &[Action]) -> Vec<OptionView> {
    options
        .iter()
        .take(4)
        .filter_map(|o| {
            let index = legal.iter().position(|&a| a == o.action)?;
            Some(OptionView { index, label: label(o.action), ev: o.ev, makes: o.makes })
        })
        .collect()
}

fn parse(save: &str) -> Result<SaveData, String> {
    serde_json::from_str(save).map_err(|e| e.to_string())
}

/// What the coach would do now, for South to act.
pub fn hint(save: &str) -> Result<HintView, String> {
    let save = parse(save)?;
    let game = replay(&save, save.log.len())?;
    if game.to_act() != Some(HUMAN) {
        return Err("it is not your turn".into());
    }
    let legal = game.legal_actions();
    let obs = game.observe(HUMAN);
    let analysis = Coach::new(save.seed ^ save.log.len() as u64).analyze(&obs, &legal, None);
    let best = legal.iter().position(|&a| a == analysis.best).unwrap_or(0);
    Ok(HintView {
        best,
        best_label: label(analysis.best),
        options: options_view(&analysis.options, &legal),
        reasons: analysis.reasons,
        worlds: analysis.worlds,
    })
}

/// Grades action `number` of the save (it must be one of South's, with a real choice).
pub fn grade(save: &str, number: usize) -> Result<Option<GradeView>, String> {
    let save = parse(save)?;
    let index = *save.log.get(number).ok_or("no such action")? as usize;
    let game = replay(&save, number)?;
    if game.to_act() != Some(HUMAN) {
        return Err("that action was not yours".into());
    }
    let legal = game.legal_actions();
    if legal.len() < 2 {
        return Ok(None);
    }
    let obs = game.observe(HUMAN);
    let chosen = legal[index];
    let grade: Grade = Coach::new(save.seed ^ number as u64).grade(&obs, &legal, chosen);
    Ok(Some(GradeView {
        action_number: number,
        phase: match obs.phase {
            Phase::Bidding => "bidding",
            Phase::Discarding => "discarding",
            Phase::Playing => "playing",
            Phase::GameOver => "gameOver",
        },
        chosen_label: label(chosen),
        best_label: label(grade.best),
        verdict: grade.verdict.label(),
        loss: grade.loss,
        options: options_view(&grade.analysis.options, &legal),
        reasons: grade.analysis.reasons,
        deal: game.history().len() + 1,
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracedPlayView {
    pub seat: usize,
    pub card: crate::view::CardView,
    pub par_after: u8,
    pub cost: u8,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceView {
    /// Declarer's tricks with perfect play by everyone, from the first lead.
    pub par: u8,
    pub plays: Vec<TracedPlayView>,
}

/// The double-dummy trace of deal `number` (hindsight: every card visible).
pub fn trace(save: &str, number: usize) -> Result<Option<TraceView>, String> {
    let save = parse(save)?;
    let game = replay(&save, save.log.len())?;
    let record = game.history().get(number - 1).ok_or("no such deal")?;
    let trump = record.contract.and_then(|(_, c)| c.trump());
    Ok(dd_trace(record).map(|(par, plays)| TraceView {
        par,
        plays: plays
            .into_iter()
            .map(|p| TracedPlayView { seat: p.seat.index(), card: card_view(p.card, trump), par_after: p.par_after, cost: p.cost })
            .collect(),
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpertView {
    /// What happened: points for your side minus theirs on this deal.
    pub actual: i32,
    /// The same cards with The Counter in your seat.
    pub expert: i32,
    pub expert_contract: Option<String>,
    pub expert_declarer: Option<usize>,
    pub expert_made: Option<bool>,
}

/// Luck vs skill: replays deal `number` with The Counter in South's seat.
pub fn expert(save: &str, number: usize) -> Result<ExpertView, String> {
    let save = parse(save)?;
    let full = replay(&save, save.log.len())?;
    let actual_record = full.history().get(number - 1).ok_or("no such deal")?;
    let (start, _) = replay_to_deal(&save, number)?;
    let players: [&str; 4] = std::array::from_fn(|i| save.players[i].as_str());
    let diff = |change: [i32; 2]| change[0] - change[1];
    let record = expert_replay(start, players, HUMAN, save.seed ^ 0xC0AC4 ^ number as u64).ok_or("replay failed")?;
    Ok(ExpertView {
        actual: diff(actual_record.score_change),
        expert: diff(record.score_change),
        expert_contract: record.contract.map(|(_, c)| c.to_string()),
        expert_declarer: record.contract.map(|(s, _)| s.index()),
        expert_made: record.contract.map(|(s, c): (Seat, _)| record.tricks_won[s.team().index()] >= c.level),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Table;

    #[test]
    fn hints_grades_traces_and_replays_from_a_save() {
        let mut table = Table::new(21, "steady", "steady", "steady").unwrap();
        let mut graded = 0;
        while table.game().history().iter().all(|h| h.contract.is_none()) {
            if table.human_to_act() {
                let save = serde_json::to_string(&table.save()).unwrap();
                let hint = hint(&save).unwrap();
                assert!(hint.best < table.view().legal.len());
                let number = table.action_count();
                table.act(hint.best).unwrap();
                let save = serde_json::to_string(&table.save()).unwrap();
                if let Some(g) = grade(&save, number).unwrap() {
                    assert_eq!(g.verdict, "best", "following the hint is best: {:?}", g.reasons);
                    graded += 1;
                }
            } else {
                table.step_bot();
            }
        }
        assert!(graded > 0);
        let save = serde_json::to_string(&table.save()).unwrap();
        let deal = table.game().history().len();
        let traced = trace(&save, deal).unwrap().expect("played deal");
        assert_eq!(traced.plays.len(), 40);
        let replayed = expert(&save, deal).unwrap();
        assert!(replayed.expert_contract.is_some() || replayed.expert == 0);
    }
}
