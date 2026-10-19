//! The coach for the browser. Every function takes a save (seed + action log), rebuilds the game
//! itself, and analyzes from South's seat, so it can run in its own worker without slowing play.

use crate::table::{replay, replay_to_deal, SaveData, HUMAN};
use crate::view::card_view;
use fh_ai::coach::{dd_trace, expert_replay, Coach, Grade, OptionEval};
use fh_core::{Action, Card, Phase, Seat};
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
    /// Indices (in `view.legal`) of plays about as good as the best: a close call.
    pub close: Vec<usize>,
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
    hint_for(&parse(save)?, HUMAN)
}

/// What the coach would do now, for `seat` to act.
pub fn hint_for(save: &SaveData, seat: Seat) -> Result<HintView, String> {
    let game = replay(save, save.log.len())?;
    if game.to_act() != Some(seat) {
        return Err("it is not your turn".into());
    }
    let legal = game.legal_actions();
    let obs = game.observe(seat);
    let analysis = Coach::new(save.seed ^ save.log.len() as u64).analyze(&obs, &legal, None);
    let best = legal.iter().position(|&a| a == analysis.best).unwrap_or(0);
    Ok(HintView {
        best,
        best_label: label(analysis.best),
        options: options_view(&analysis.options, &legal),
        reasons: analysis.reasons,
        worlds: analysis.worlds,
        close: analysis.close.iter().filter_map(|a| legal.iter().position(|l| l == a)).collect(),
    })
}

/// Grades action `number` of the save (it must be one of South's, with a real choice).
pub fn grade(save: &str, number: usize) -> Result<Option<GradeView>, String> {
    grade_for(&parse(save)?, number, HUMAN)
}

/// Grades action `number` of the save, which must have been `seat`'s.
pub fn grade_for(save: &SaveData, number: usize, seat: Seat) -> Result<Option<GradeView>, String> {
    let index = *save.log.get(number).ok_or("no such action")? as usize;
    let game = replay(save, number)?;
    if game.to_act() != Some(seat) {
        return Err("that action was not yours".into());
    }
    let legal = game.legal_actions();
    if legal.len() < 2 {
        return Ok(None);
    }
    let obs = game.observe(seat);
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

/// Grades the whole kitty discard that starts at action `number` (South's).
pub fn grade_discards(save: &str, number: usize) -> Result<Option<GradeView>, String> {
    grade_discards_for(&parse(save)?, number, HUMAN)
}

/// Grades the three discards `seat` made starting at action `number`, as one decision. The grade
/// carries the first discard's action number.
pub fn grade_discards_for(save: &SaveData, number: usize, seat: Seat) -> Result<Option<GradeView>, String> {
    let mut game = replay(save, number)?;
    if game.to_act() != Some(seat) || game.phase() != Phase::Discarding {
        return Err("those weren't your discards".into());
    }
    let obs = game.observe(seat);
    if obs.discards.is_some_and(|d| !d.is_empty()) {
        return Err("grade the discards from the first one".into());
    }
    let mut chosen: Vec<Card> = Vec::new();
    for &index in save.log.iter().skip(number).take(3) {
        let legal = game.legal_actions();
        let Some(&Action::Discard(card)) = legal.get(index as usize) else { break };
        chosen.push(card);
        game.apply(Action::Discard(card)).map_err(|e| e.to_string())?;
    }
    if chosen.len() < 3 {
        return Err("the discards aren't finished".into());
    }
    let grade = Coach::new(save.seed ^ number as u64).grade_discard_set(&obs, &chosen);
    let text = |cards: &[Card]| cards.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ");
    Ok(Some(GradeView {
        action_number: number,
        phase: "discarding",
        chosen_label: text(&grade.chosen),
        best_label: text(&grade.plan),
        verdict: grade.verdict.label(),
        loss: grade.loss,
        options: vec![],
        reasons: grade.reasons,
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
    trace_for(&parse(save)?, number, HUMAN)
}

/// The trace of deal `number`, with seats as `viewer` sees them. The deal must be finished.
pub fn trace_for(save: &SaveData, number: usize, viewer: Seat) -> Result<Option<TraceView>, String> {
    let game = replay(save, save.log.len())?;
    let rotate = |s: Seat| (s.index() + 4 - viewer.index()) % 4;
    let record = game.history().get(number - 1).ok_or("no such deal")?;
    let trump = record.contract.and_then(|(_, c)| c.trump());
    Ok(dd_trace(record).map(|(par, plays)| TraceView {
        par,
        plays: plays
            .into_iter()
            .map(|p| TracedPlayView { seat: rotate(p.seat), card: card_view(p.card, trump), par_after: p.par_after, cost: p.cost })
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
    expert_for(&parse(save)?, number, HUMAN)
}

/// Luck vs skill for `seat`: deal `number` replayed with The Counter there. Other people's seats
/// are played by expert bots in the replay.
pub fn expert_for(save: &SaveData, number: usize, seat: Seat) -> Result<ExpertView, String> {
    let full = replay(save, save.log.len())?;
    let actual_record = full.history().get(number - 1).ok_or("no such deal")?;
    let (start, _) = replay_to_deal(save, number)?;
    let players: [&str; 4] = std::array::from_fn(|i| save.players[i].as_str());
    let (us, them) = (seat.team().index(), seat.team().other().index());
    let diff = |change: [i32; 2]| change[us] - change[them];
    let record = expert_replay(start, players, seat, save.seed ^ 0xC0AC4 ^ number as u64).ok_or("replay failed")?;
    Ok(ExpertView {
        actual: diff(actual_record.score_change),
        expert: diff(record.score_change),
        expert_contract: record.contract.map(|(_, c)| c.to_string()),
        expert_declarer: record.contract.map(|(s, _)| (s.index() + 4 - seat.index()) % 4),
        expert_made: record.contract.map(|(s, c): (Seat, _)| record.tricks_won[s.team().index()] >= c.level),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Table;

    #[test]
    fn drills_produce_positions_for_south() {
        let bid = drill_bidding(5).unwrap();
        assert_eq!(bid.view.phase, "bidding");
        assert!(bid.view.human_to_act);
        let puzzle = drill_play(5).unwrap();
        assert_eq!(puzzle.view.phase, "playing");
        assert!(puzzle.view.human_to_act);
        // The puzzle's answer is graded best by the same analysis.
        let h = hint(&puzzle.save).unwrap();
        let mut save: SaveData = serde_json::from_str(&puzzle.save).unwrap();
        let number = save.log.len();
        save.log.push(h.best as u16);
        let g = grade(&serde_json::to_string(&save).unwrap(), number).unwrap().unwrap();
        assert_eq!(g.verdict, "best");
    }

    /// Advice never depends on who the other players are: the same game with different personas
    /// in the other seats gets the same hints and grades.
    #[test]
    fn advice_ignores_who_the_other_players_are() {
        let mut table = Table::new(12, "steady", "steady", "steady").unwrap();
        let (mut checked, mut plays) = (0, 0);
        for _ in 0..400 {
            if table.game().phase() == Phase::GameOver || plays >= 2 {
                break;
            }
            if !table.human_to_act() {
                table.step_bot();
                continue;
            }
            let save = table.save();
            plays += usize::from(table.game().phase() == Phase::Playing && table.game().legal_actions().len() > 1);
            let mut others = save.clone();
            others.players = ["human", "gambler", "rookie", "shark"].map(String::from);
            let hint = |s: &SaveData| serde_json::to_string(&hint_for(s, HUMAN).unwrap()).unwrap();
            assert_eq!(hint(&save), hint(&others), "hint at action {}", save.log.len());
            table.act(0).unwrap();
            let (mut done, mut done_others) = (table.save(), table.save());
            done_others.players = others.players.clone();
            done.log.truncate(save.log.len() + 1);
            done_others.log.truncate(save.log.len() + 1);
            let grade = |s: &SaveData| serde_json::to_string(&grade_for(s, save.log.len(), HUMAN).unwrap()).unwrap();
            assert_eq!(grade(&done), grade(&done_others), "grade at action {}", save.log.len());
            checked += 1;
        }
        assert!(checked >= 3 && plays >= 2, "{checked} decisions, {plays} card plays");
    }

    #[test]
    fn the_three_discards_are_graded_together() {
        // South bids 10NT if it can, so it wins the bid, then puts back its first three cards.
        let mut table = Table::new(4, "rock", "rock", "rock").unwrap();
        while table.game().phase() != Phase::Discarding {
            if table.human_to_act() {
                let legal = table.game().legal_actions();
                let top = legal.iter().rposition(|a| matches!(a, Action::Call(fh_core::Call::Bid(_)))).unwrap_or(0);
                table.act(top).unwrap();
            } else {
                table.step_bot();
            }
        }
        assert_eq!(table.game().to_act(), Some(HUMAN));
        let first = table.action_count();
        for _ in 0..3 {
            table.act(0).unwrap();
        }
        let save = serde_json::to_string(&table.save()).unwrap();
        let grade = grade_discards(&save, first).unwrap().expect("a grade");
        assert_eq!(grade.action_number, first);
        assert_eq!(grade.chosen_label.split(' ').count(), 3);
        assert_eq!(grade.best_label.split(' ').count(), 3);
        assert!(!grade.reasons.is_empty());
        assert!(grade_discards(&save, first + 1).is_err(), "only from the first discard");
    }

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

// ---- Drills ---------------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DrillView {
    pub view: crate::view::View,
    pub save: String,
}

fn drill_view(table: &crate::table::Table) -> DrillView {
    DrillView { view: table.view(), save: serde_json::to_string(&table.save()).expect("save serializes") }
}

/// A bidding decision: a fresh deal with the bots calling until it is South's turn.
pub fn drill_bidding(seed: u64) -> Result<DrillView, String> {
    let mut table = crate::table::Table::new(seed, "oracle", "oracle", "oracle")?;
    while !table.human_to_act() {
        table.step_bot().ok_or("the bots couldn't reach your call")?;
    }
    Ok(drill_view(&table))
}

/// A card-play puzzle: hands are played by quick rule bots (one in South's seat too) until South
/// faces a decision where one play is clearly best, judged by the same analysis that grades the
/// answer. The moves leading up to it don't need to be expert; the puzzle position is checked
/// properly.
pub fn drill_play(seed: u64) -> Result<DrillView, String> {
    for attempt in 0..30u64 {
        let mut table = crate::table::Table::new(seed.wrapping_add(attempt * 7919), "steady", "steady", "steady")?;
        let mut south = fh_ai::make_bot("steady", seed ^ attempt).ok_or("no bot")?;
        let mut screen = fh_ai::SearchBot::new("screen", fh_ai::SearchConfig::FAST, seed ^ attempt);
        let start = table.game().history().len();
        let mut checked = 0;
        while table.game().history().len() == start && table.game().phase() != Phase::GameOver && checked < 4 {
            if !table.human_to_act() {
                table.step_bot();
                continue;
            }
            let game = table.game();
            let obs = game.observe(HUMAN);
            let legal = game.legal_actions();
            let tricks = obs.tricks.len();
            if obs.phase == Phase::Playing && (1..=7).contains(&tricks) && legal.len() >= 3 {
                // Cheap screen first, then the coach's own analysis.
                // Equivalent cards tie, so compare the best group with the next-best group.
                let ranked = screen.evaluate_plays(&obs, &legal);
                let best_ev = ranked[0].1;
                let next_ev = ranked.iter().map(|r| r.1).find(|&ev| ev < best_ev - 2.0);
                if next_ev.is_some_and(|ev| best_ev - ev >= 20.0) {
                    checked += 1;
                    let save = table.save();
                    let number = save.log.len();
                    let mut coach = Coach::new(save.seed ^ number as u64);
                    let analysis = coach.analyze(&obs, &legal, None);
                    let top = analysis.options.first().map_or(0.0, |o| o.ev);
                    let runner_up = analysis.options.iter().find(|o| o.ev < top - 2.0).map(|o| o.action);
                    if let Some(second) = runner_up {
                        let grade = Coach::new(save.seed ^ number as u64).grade(&obs, &legal, second);
                        if grade.verdict >= fh_ai::coach::Verdict::Mistake && grade.best == analysis.best {
                            return Ok(drill_view(&table));
                        }
                    }
                }
            }
            let action = south.choose(&obs, &legal);
            let index = legal.iter().position(|&a| a == action).unwrap();
            table.act(index)?;
        }
    }
    Err("no clear puzzle found in these deals; try again".into())
}
