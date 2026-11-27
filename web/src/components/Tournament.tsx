import { useState } from "react";
import type { Persona } from "../engine";
import {
  awards,
  gamesByPlayer,
  HANDS,
  KINDS,
  ROUNDS,
  roundComplete,
  roundPoints,
  standings,
  tournamentKind,
  yourTable,
  type Award,
  type GameLine,
  type Tournament,
  type TournamentKind,
} from "../tournament";
import { Dialog } from "./Dialog";
import { TournamentRecap, type RecapProgress } from "./TournamentRecap";

function styleName(personas: Persona[], id: string): string {
  return id === "human" ? "" : (personas.find((p) => p.id === id)?.name ?? id);
}

function Who({ t, id, personas }: { t: Tournament; id: number; personas: Persona[] }) {
  const p = t.participants[id];
  return (
    <span className={id === 0 ? "you" : ""} title={styleName(personas, p.persona)}>
      {p.name}
      {id !== 0 && <span className="style"> ({styleName(personas, p.persona)})</span>}
    </span>
  );
}

function AwardLine({ t, label, award, personas }: { t: Tournament; label: string; award: Award | null; personas: Persona[] }) {
  if (!award) return null;
  return (
    <p className="award">
      <strong>{label}:</strong> {award.score} points, <Who t={t} id={award.players[0]} personas={personas} /> and <Who t={t} id={award.players[1]} personas={personas} /> (round{" "}
      {award.round + 1})
      {award.bid && <BidLine t={t} award={award} />}
    </p>
  );
}

/** A high hand's bid: "on Ada's 10♥ bid" for the declarers, "defending Finn's 8♠ bid (set)" for the defenders. */
function BidLine({ t, award }: { t: Tournament; award: Award }) {
  const bid = award.bid!;
  const declared = award.players.includes(bid.by);
  return (
    <span className="award-bid">
      , {declared ? "on" : "defending"}{" "}
      {bid.by === 0 ? "your" : `${t.participants[bid.by].name}'s`}{" "}
      <strong>{bid.label}</strong> bid{!declared && !bid.made && " (set)"}
    </span>
  );
}

/** Everyone's totals, best first. */
function StandingsTable({ t, personas }: { t: Tournament; personas: Persona[] }) {
  const list = standings(t);
  return (
    <table className="score-pad standings">
      <thead>
        <tr>
          <th>Place</th>
          <th>Player</th>
          <th>Last round</th>
          <th>Total</th>
        </tr>
      </thead>
      <tbody>
        {list.map((s) => (
          <tr key={s.participant.id} className={s.participant.id === 0 ? "you-row" : ""}>
            <td>{s.place}</td>
            <td>
              <Who t={t} id={s.participant.id} personas={personas} />
            </td>
            <td>{s.last ?? "—"}</td>
            <td>
              <strong>{s.total}</strong>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** Every player's score in every round: a row each, best total first. Each score is tinted (blue
 * for points, deeper for more; red below zero) and the round's best is starred; a score's tooltip
 * names that table and lists each hand's bid, and choosing a player lists their games underneath. */
export function GameGrid({ t, personas }: { t: Tournament; personas: Persona[] }) {
  const list = standings(t);
  const games = gamesByPlayer(t);
  const [chosen, setChosen] = useState(0);
  const played = t.rounds.map((_, r) => list.some((s) => games.get(s.participant.id)![r] !== null));
  const rounds = t.rounds.map((_, r) => r).filter((r) => played[r]);
  const all = [...games.values()].flat().filter((g): g is GameLine => g !== null);
  const high = Math.max(1, ...all.map((g) => g.score));
  const low = Math.max(1, ...all.map((g) => -g.score));
  const best = t.rounds.map((_, r) => Math.max(...list.map((s) => games.get(s.participant.id)![r]?.score ?? -Infinity)));
  const name = (id: number) => (id === 0 ? "you" : t.participants[id].name);
  const tint = (score: number) =>
    score > 0
      ? `color-mix(in srgb, var(--score-up) ${Math.round(8 + (40 * score) / high)}%, transparent)`
      : score < 0
        ? `color-mix(in srgb, var(--score-down) ${Math.round(8 + (40 * -score) / low)}%, transparent)`
        : undefined;
  const describe = (r: number, g: GameLine) =>
    [
      `Round ${r + 1}: with ${name(g.partner)} against ${name(g.opponents[0])} and ${name(g.opponents[1])}, ${g.score} to ${g.against}${g.score > g.against ? ", won" : g.score < g.against ? ", lost" : ", tied"}`,
      ...g.hands.map(
        (h, i) => `Hand ${i + 1}: ${h.label} by ${name(h.by)}, ${h.made ? "made" : "set"} (${h.points > 0 ? "+" : h.points < 0 ? "−" : ""}${Math.abs(h.points)})`,
      ),
    ].join("\n");
  const theirs = games.get(chosen)!;
  if (rounds.length === 0) return <p className="note">No games finished yet.</p>;
  return (
    <div className="game-grid">
      <p className="game-grid-key">
        <span className="swatch up" aria-hidden="true" /> more points <span className="swatch down" aria-hidden="true" /> below zero <span aria-hidden="true">★</span> the round's best ·
        choose a player for their games
      </p>
      <div className="game-grid-scroll">
        <table className="score-pad">
          <thead>
            <tr>
              <th scope="col" title="Place">
                #
              </th>
              <th scope="col">Player</th>
              <th scope="col">Total</th>
              {rounds.map((r) => (
                <th key={r} scope="col" title={`Round ${r + 1}`}>
                  R{r + 1}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {list.map((s) => {
              const id = s.participant.id;
              return (
                <tr key={id} className={`${id === 0 ? "you-row" : ""}${id === chosen ? " chosen" : ""}`}>
                  <td>{s.place}</td>
                  <th scope="row">
                    <button type="button" className="link" aria-pressed={id === chosen} onClick={() => setChosen(id)}>
                      {id === 0 ? "You" : s.participant.name}
                    </button>
                  </th>
                  <td className="game-total">{s.total}</td>
                  {rounds.map((r) => {
                    const g = games.get(id)![r];
                    return g ? (
                      <td key={r} className="game-cell" style={{ background: tint(g.score) }} title={describe(r, g)}>
                        {g.score}
                        {g.score === best[r] && <span className="best-mark">★</span>}
                      </td>
                    ) : (
                      <td key={r} className="game-cell">
                        —
                      </td>
                    );
                  })}
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <h4>
        {chosen === 0 ? "Your games" : (
          <>
            <Who t={t} id={chosen} personas={personas} />
            's games
          </>
        )}
      </h4>
      <ol className="player-games">
        {rounds.map((r) => {
          const g = theirs[r];
          if (!g) return null;
          const result = g.score > g.against ? "won" : g.score < g.against ? "lost" : "tied";
          return (
            <li key={r}>
              <span className="pg-round">R{r + 1}</span>
              <span className="pg-who">
                with <strong>{name(g.partner)}</strong> against {name(g.opponents[0])} and {name(g.opponents[1])}
              </span>
              <span className={`pg-score ${result}`}>
                {g.score}–{g.against} {result}
                {g.score === best[r] && " ★"}
              </span>
            </li>
          );
        })}
      </ol>
    </div>
  );
}

function roundPointsAny(t: Tournament): boolean {
  return t.rounds.some((_, r) => roundPoints(t, r).size > 0);
}

/** Your table in round `r`: who partners you and who you face. */
export function YourTable({ t, r, personas }: { t: Tournament; r: number; personas: Persona[] }) {
  const table = t.rounds[r].tables[yourTable(t, r)];
  const [, west, north, east] = table.seats;
  return (
    <p>
      Round {r + 1} of {ROUNDS}: you partner <Who t={t} id={north} personas={personas} /> against <Who t={t} id={west} personas={personas} /> and{" "}
      <Who t={t} id={east} personas={personas} />, {HANDS} hands.
    </p>
  );
}

interface Props {
  t: Tournament;
  personas: Persona[];
  /** Play the current round's game (or, once all rounds are in, see the final results). */
  onPlay: () => void;
  /** Close the dialog: not offered after a round, whose only way on is the next round. */
  onClose: () => void;
  /** After a round: the round just played, to show its tables. */
  after?: number;
  /** The helpers on now (grading, the trump counter…), which would count as help. */
  helpers?: string[];
  /** After a round: to the main menu, the tournament kept for later. */
  onLeave?: () => void;
  /** End the tournament for good (any time before its results). */
  onQuit?: () => void;
  /** Turn every helper off (grading, the trump counter, the card tracker, out-of-suit marks), for
   * an unaided tournament. */
  onHelpOff?: () => void;
}

/** Whether this tournament can still be won unaided, and what would spoil it. */
function UnaidedNote({ t, helpers, onHelpOff }: { t: Tournament; helpers: string[]; onHelpOff?: () => void }) {
  if (t.assisted === undefined || t.current >= ROUNDS) return null;
  if (t.assisted) return <p className="note">You've had help in this tournament (a hint, an undo, Finish hand or a helper), so it can't count as unaided.</p>;
  return (
    <div className="note unaided-note">
      <p>
        🙌 Unaided so far: no hints, undos or Finish hand.
        {helpers.length > 0 && ` ${capitalize(listed(helpers))} ${helpers.length > 1 ? "are" : "is"} on in Settings: turn ${helpers.length > 1 ? "them off before you play, or they count" : "it off before you play, or it counts"} as help.`}
        {helpers.length === 0 && " Every helper is off."}
      </p>
      {helpers.length > 0 && onHelpOff && (
        <button type="button" onClick={onHelpOff} title="Turn off grading, the trump counter, the card tracker and out-of-suit marks, to play unaided">
          Turn off all help
        </button>
      )}
    </div>
  );
}

function listed(items: string[]): string {
  return items.length < 2 ? items.join("") : `${items.slice(0, -1).join(", ")} and ${items.at(-1)}`;
}

function capitalize(text: string): string {
  return text.charAt(0).toUpperCase() + text.slice(1);
}

/** A new tournament: who the other 11 players are. */
export function NewTournamentDialog({ onStart, onCancel, onRecap }: { onStart: (kind: TournamentKind) => void; onCancel: () => void; onRecap?: () => void }) {
  return (
    <Dialog title="New tournament" onClose={onCancel}>
      <p>
        12 players at 3 tables, {ROUNDS} rounds of {HANDS} hands, a new partner every round. Who are the other 11?
      </p>
      <div className="menu-buttons kind-choices">
        {KINDS.map((k, i) => (
          <button key={k.kind} type="button" onClick={() => onStart(k.kind)} autoFocus={i === 0}>
            <span>
              {k.icon} {k.title}
            </span>
            <small>{k.text}</small>
          </button>
        ))}
      </div>
      <div className="actions">
        {onRecap && (
          <button type="button" onClick={onRecap}>
            Your last tournament's recap
          </button>
        )}
        <button type="button" onClick={onCancel}>
          Cancel
        </button>
      </div>
    </Dialog>
  );
}

/** Your recap of a finished tournament, from the menu. */
export function RecapDialog({ t, progress, onClose }: { t: Tournament; progress: RecapProgress | null; onClose: () => void }) {
  return (
    <Dialog title="Your tournament" wide onClose={onClose}>
      <TournamentRecap t={t} progress={progress} />
      <div className="actions">
        <button type="button" className="primary" onClick={onClose} autoFocus>
          Close
        </button>
      </div>
    </Dialog>
  );
}

/** The tournament's state: after a round, its results; always, the standings and the awards. */
export function TournamentDialog({ t, personas, onPlay, onClose, after, helpers = [], onLeave, onQuit, onHelpOff }: Props) {
  const waiting = after !== undefined && !roundComplete({ ...t, current: after });
  const { highGame, highHand } = awards(t);
  const done = t.current >= ROUNDS;
  const kind = KINDS.find((k) => k.kind === tournamentKind(t))!;
  // After a round there's nowhere to go but the next one (closing it left a finished table).
  const closable = after === undefined;
  return (
    <Dialog title={after !== undefined ? `Tournament: round ${after + 1} of ${ROUNDS}` : "Tournament"} wide onClose={closable ? onClose : undefined}>
      {t.career ? (
        <p className="tournament-kind">
          🃏 <strong>{t.career.title}.</strong> {t.career.goal}
        </p>
      ) : (
        <p className="tournament-kind">
          {kind.icon} <strong>{kind.title} tournament.</strong> {kind.text}
        </p>
      )}
      {after !== undefined && (
        <>
          <h3>This round's tables</h3>
          <ul className="round-tables">
            {t.rounds[after].tables.map((table, i) => (
              <li key={i}>
                <Who t={t} id={table.seats[0]} personas={personas} /> &amp; <Who t={t} id={table.seats[2]} personas={personas} />{" "}
                <strong>{table.result ? table.result.scores[0] : "…"}</strong>
                {" · "}
                <Who t={t} id={table.seats[1]} personas={personas} /> &amp; <Who t={t} id={table.seats[3]} personas={personas} />{" "}
                <strong>{table.result ? table.result.scores[1] : "…"}</strong>
                {!table.result && <span className="waiting-text"> still playing…</span>}
              </li>
            ))}
          </ul>
          {waiting && <p className="waiting-text">Waiting for the other tables to finish their hands…</p>}
          {!waiting && roundPoints(t, after).get(0) !== undefined && (
            <p>
              You scored <strong>{roundPoints(t, after).get(0)}</strong> this round.
            </p>
          )}
        </>
      )}
      <h3>Standings</h3>
      <StandingsTable t={t} personas={personas} />
      {roundPointsAny(t) && (
        <details className="game-grid-details">
          <summary>Every game so far</summary>
          <GameGrid t={t} personas={personas} />
        </details>
      )}
      <AwardLine t={t} label="High game" award={highGame} personas={personas} />
      <AwardLine t={t} label="High hand" award={highHand} personas={personas} />
      {!done && !waiting && <YourTable t={t} r={t.current} personas={personas} />}
      <UnaidedNote t={t} helpers={helpers} onHelpOff={onHelpOff} />
      <div className="actions">
        {onQuit && !done && (
          <button
            type="button"
            title="Quit this tournament: it ends here. Start a new one from the main menu."
            onClick={() => {
              if (confirm("Quit this tournament? It ends here, and its standings are lost.")) onQuit();
            }}
          >
            Quit
          </button>
        )}
        {!closable && onLeave && (
          <button type="button" onClick={onLeave} title="Leave the tournament for now: pick it up again from the main menu">
            Main menu
          </button>
        )}
        {closable && (
          <button type="button" onClick={onClose}>
            Close
          </button>
        )}
        <button type="button" className="primary" onClick={onPlay} disabled={waiting} autoFocus>
          {done ? "Final results" : after !== undefined ? "Next round" : `Play round ${t.current + 1}`}
        </button>
      </div>
      <p className="note">
        12 players at 3 tables, {ROUNDS} rounds of {HANDS} hands. Your side's score each round is added to both partners' totals, and you have a different
        partner every round.
      </p>
    </Dialog>
  );
}

/** After the last round: third place, then second, then the winner, then the awards; then your
 * own recap. */
export function FinalReveal({ t, personas, onDone, progress }: { t: Tournament; personas: Persona[]; onDone: () => void; progress: RecapProgress | null }) {
  const [step, setStep] = useState(0);
  // Straight from your last game: the other two tables may still be playing out.
  if (!t.rounds.every((round) => round.tables.every((table) => table.result))) {
    return (
      <Dialog title="Tournament results" wide>
        <p className="waiting-text">The other tables are finishing their last games…</p>
        <div className="actions">
          <button type="button" className="primary" disabled>
            Reveal third place
          </button>
        </div>
      </Dialog>
    );
  }
  const list = standings(t);
  const at = (place: number) => list.filter((s) => s.place === place);
  const { highGame, highHand } = awards(t);
  const you = list.find((s) => s.participant.id === 0)!;
  const places = [3, 2, 1];
  const label = ["Third place", "Second place", "The winner"];
  if (step > 3) {
    return (
      <Dialog title="Your tournament" wide>
        <TournamentRecap t={t} progress={progress} />
        <div className="actions">
          <button type="button" onClick={() => setStep(3)}>
            Back to the results
          </button>
          <button type="button" className="primary" onClick={onDone} autoFocus>
            Done
          </button>
        </div>
      </Dialog>
    );
  }
  return (
    <Dialog title="Tournament results" wide>
      <div className="podium">
        {places.slice(0, step).map((place, i) => (
          <div key={place} className={`podium-step place-${place}`}>
            <span className="podium-label">{label[i]}</span>
            {at(place).length === 0 ? (
              <span className="note">(shared above)</span>
            ) : (
              at(place).map((s) => (
                <span key={s.participant.id} className="podium-name">
                  <Who t={t} id={s.participant.id} personas={personas} /> · {s.total} points
                </span>
              ))
            )}
          </div>
        ))}
      </div>
      {step >= 3 && (
        <>
          <AwardLine t={t} label="High game" award={highGame} personas={personas} />
          <AwardLine t={t} label="High hand" award={highHand} personas={personas} />
          <p>
            You finished <strong>{ordinal(you.place)}</strong> of 12 with {you.total} points.
          </p>
          <h3>Final standings, game by game</h3>
          <GameGrid t={t} personas={personas} />
        </>
      )}
      <div className="actions">
        {step < 3 ? (
          <button type="button" className="primary" onClick={() => setStep(step + 1)} autoFocus>
            {["Reveal third place", "Reveal second place", "Reveal the winner"][step]}
          </button>
        ) : (
          <>
            <button type="button" onClick={onDone}>
              Done
            </button>
            <button type="button" className="primary" onClick={() => setStep(4)} autoFocus>
              Your recap
            </button>
          </>
        )}
      </div>
    </Dialog>
  );
}

function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return n + (s[(v - 20) % 10] || s[v] || s[0]);
}
