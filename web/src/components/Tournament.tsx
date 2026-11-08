import { useState } from "react";
import type { Persona } from "../engine";
import { awards, HANDS, ROUNDS, roundComplete, roundPoints, standings, yourTable, type Award, type Tournament } from "../tournament";
import { Dialog } from "./Dialog";

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
  onClose: () => void;
  /** After a round: the round just played, to show its tables. */
  after?: number;
  /** Abandon this tournament and start a new one. */
  onRestart?: () => void;
}

/** The tournament's state: after a round, its results; always, the standings and the awards. */
export function TournamentDialog({ t, personas, onPlay, onClose, after, onRestart }: Props) {
  const waiting = after !== undefined && !roundComplete({ ...t, current: after });
  const { highGame, highHand } = awards(t);
  const done = t.current >= ROUNDS;
  return (
    <Dialog title={after !== undefined ? `Tournament: round ${after + 1} of ${ROUNDS}` : "Tournament"} wide onClose={onClose}>
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
      <AwardLine t={t} label="High game" award={highGame} personas={personas} />
      <AwardLine t={t} label="High hand" award={highHand} personas={personas} />
      {!done && !waiting && <YourTable t={t} r={t.current} personas={personas} />}
      <div className="actions">
        {onRestart && t.current > 0 && (
          <button
            type="button"
            onClick={() => {
              if (confirm("Abandon this tournament and start a new one?")) onRestart();
            }}
          >
            New tournament
          </button>
        )}
        <button type="button" onClick={onClose}>
          Close
        </button>
        <button type="button" className="primary" onClick={onPlay} disabled={waiting} autoFocus>
          {done ? "Final results" : after !== undefined ? "Next round" : `Play round ${t.current + 1}`}
        </button>
      </div>
      <p className="note">
        12 players at 3 tables, {ROUNDS} rounds of {HANDS} hands. Your side's score each round is added to both partners' totals, and you have a different
        partner every round. Each player has a randomly assigned playing style.
      </p>
    </Dialog>
  );
}

/** After the last round: third place, then second, then the winner, then the awards. */
export function FinalReveal({ t, personas, onDone }: { t: Tournament; personas: Persona[]; onDone: () => void }) {
  const [step, setStep] = useState(0);
  const list = standings(t);
  const at = (place: number) => list.filter((s) => s.place === place);
  const { highGame, highHand } = awards(t);
  const you = list.find((s) => s.participant.id === 0)!;
  const places = [3, 2, 1];
  const label = ["Third place", "Second place", "The winner"];
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
          <h3>Final standings</h3>
          <StandingsTable t={t} personas={personas} />
        </>
      )}
      <div className="actions">
        {step < 3 ? (
          <button type="button" className="primary" onClick={() => setStep(step + 1)} autoFocus>
            {["Reveal third place", "Reveal second place", "Reveal the winner"][step]}
          </button>
        ) : (
          <button type="button" className="primary" onClick={onDone} autoFocus>
            Done
          </button>
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
