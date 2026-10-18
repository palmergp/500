import type { HandView, PlayerView } from "../engine";
import { SEAT_NAMES } from "../engine";

interface Props {
  history: HandView[];
  players: PlayerView[];
  onReview: (hand: HandView) => void;
}

function signed(n: number) {
  return n > 0 ? `+${n}` : `${n}`;
}

/** Hand-by-hand scores, like a paper score sheet. */
export function ScorePad({ history, players, onReview }: Props) {
  const played = history.filter((h) => !h.passedOut);
  if (played.length === 0) return <p>No hands played yet.</p>;
  return (
    <table className="score-pad">
      <thead>
        <tr>
          <th>#</th>
          <th>Contract</th>
          <th>Tricks</th>
          <th>You &amp; {players[2]?.name}</th>
          <th>
            {players[1]?.name} &amp; {players[3]?.name}
          </th>
          <th />
        </tr>
      </thead>
      <tbody>
        {played.map((h, i) => {
          const c = h.contract!;
          const team = c.declarer % 2;
          return (
            <tr key={h.number}>
              <td>{i + 1}</td>
              <td>
                {c.label} by {SEAT_NAMES[c.declarer]} <span className={h.made ? "made" : "down"}>{h.made ? "made" : "down"}</span>
              </td>
              <td>{h.tricksWon[team]}</td>
              <td>
                {signed(h.scoreChange[0])} <strong>{h.scoresAfter[0]}</strong>
              </td>
              <td>
                {signed(h.scoreChange[1])} <strong>{h.scoresAfter[1]}</strong>
              </td>
              <td>
                <button type="button" className="link" onClick={() => onReview(h)}>
                  Review
                </button>
              </td>
            </tr>
          );
        })}
      </tbody>
    </table>
  );
}
