import { gamesByPlayer, recap, ROUNDS, standings, type RecapGrade, type Tournament } from "../tournament";
import { BidText } from "./BidText";

/** How far the coach has got going over your games. */
export interface RecapProgress {
  round: number;
  done: number;
  total: number;
}

const PHASES: { phase: RecapGrade["phase"]; label: string }[] = [
  { phase: "bidding", label: "Bidding" },
  { phase: "discarding", label: "The kitty" },
  { phase: "playing", label: "Card play" },
];

function pct(n: number, of: number): string {
  return of > 0 ? `${Math.round((100 * n) / of)}%`.replace("-", "−") : "—";
}

function signed(n: number): string {
  return n > 0 ? `+${n}` : n < 0 ? `−${-n}` : "0";
}

function ordinal(n: number): string {
  const s = ["th", "st", "nd", "rd"];
  const v = n % 100;
  return n + (s[(v - 20) % 10] || s[v] || s[0]);
}

const good = (g: RecapGrade) => g.verdict === "best" || g.verdict === "good";
const slip = (g: RecapGrade) => g.verdict === "mistake" || g.verdict === "blunder";
const perDecision = (list: RecapGrade[]) => (list.length ? (list.reduce((sum, g) => sum + g.loss, 0) / list.length).toFixed(1) : "—");

/** Your tournament, looked back on: how you bid, where your points came from, and how well you
 * played by the coach's grades (filled in as the coach goes over your games). */
export function TournamentRecap({ t, progress }: { t: Tournament; progress: RecapProgress | null }) {
  const r = recap(t);
  if (r.games === 0) return <p className="note">This tournament was played before recaps were kept.</p>;
  const you = standings(t).find((s) => s.participant.id === 0)!;
  const games = (gamesByPlayer(t).get(0) ?? []).filter((g) => g !== null);
  const won = games.filter((g) => g.score > g.against).length;
  const total = r.points.yours + r.points.partner + r.points.defence;
  const counted = r.grades.filter((g) => !g.hinted);
  const hinted = r.grades.length - counted.length;
  const costliest = [...counted].filter((g) => g.loss >= 5).sort((a, b) => b.loss - a.loss).slice(0, 3);
  const playedRounds = (t.mine ?? []).map((g, round) => (g ? round : -1)).filter((round) => round >= 0);
  const sources: [string, number][] = [
    ["Your bids", r.points.yours],
    ["Your partners' bids", r.points.partner],
    ["Tricks on the opponents' bids", r.points.defence],
  ];
  return (
    <div className="recap">
      <div className="recap-tiles">
        <div className="recap-tile">
          <span>Place</span>
          <strong>{ordinal(you.place)}</strong>
          <small>of 12</small>
        </div>
        <div className="recap-tile">
          <span>Points</span>
          <strong>{you.total}</strong>
          <small>over {games.length} games</small>
        </div>
        <div className="recap-tile">
          <span>Games won</span>
          <strong>{won}</strong>
          <small>of {games.length}</small>
        </div>
        <div className="recap-tile">
          <span>Your bids made</span>
          <strong>{pct(r.made, r.declared)}</strong>
          <small>
            {r.made} of {r.declared}
          </small>
        </div>
      </div>

      <h3>The bidding</h3>
      <ul className="recap-lines">
        <li>
          You bid in <strong>{r.bid}</strong> of {r.hands} hands ({pct(r.bid, r.hands)}), and won the bid on <strong>{r.declared}</strong> (
          {pct(r.declared, r.hands)}).
        </li>
        <li>
          You made <strong>{r.made}</strong> of your {r.declared} bids ({pct(r.made, r.declared)}).
        </li>
        <li>
          Your partners won the bid {r.partnerDeclared} times and made {r.partnerMade} ({pct(r.partnerMade, r.partnerDeclared)}).
        </li>
        <li>
          The opponents won it {r.defended} times; your side beat {r.beaten} of those bids ({pct(r.beaten, r.defended)}).
        </li>
      </ul>

      <h3>Where your points came from</h3>
      <table className="score-pad recap-table">
        <thead>
          <tr>
            <th scope="col">From</th>
            <th scope="col">Points</th>
            <th scope="col">Share</th>
          </tr>
        </thead>
        <tbody>
          {sources.map(([label, points]) => (
            <tr key={label}>
              <th scope="row">{label}</th>
              <td>{signed(points)}</td>
              <td>{total > 0 ? pct(points, total) : "—"}</td>
            </tr>
          ))}
          <tr className="recap-total">
            <th scope="row">Total</th>
            <td>{signed(total)}</td>
            <td />
          </tr>
        </tbody>
      </table>
      <p className="note">A bid that goes down counts against its side, so a share can be negative.</p>

      <h3>How you played</h3>
      {r.ungraded > 0 && (
        <p className="waiting-text recap-progress">
          The coach is still going over your games
          {progress ? ` (round ${progress.round + 1}: ${progress.done} of ${progress.total} decisions)` : ""}. This fills in as it goes.
        </p>
      )}
      {counted.length === 0 ? (
        <p className="note">No graded decisions yet.</p>
      ) : (
        <>
          <p>
            The coach graded <strong>{counted.length}</strong> of your decisions: <strong>{pct(counted.filter(good).length, counted.length)}</strong> best or good,{" "}
            {counted.filter((g) => g.verdict === "inaccuracy").length} inaccuracies, {counted.filter((g) => g.verdict === "mistake").length} mistakes and{" "}
            {counted.filter((g) => g.verdict === "blunder").length} blunders. You gave up <strong>{perDecision(counted)}</strong> points a decision on average.
            {hinted > 0 && ` (${hinted} decision${hinted > 1 ? "s" : ""} you took a hint on ${hinted > 1 ? "aren't" : "isn't"} counted.)`}
          </p>
          <table className="score-pad recap-table">
            <thead>
              <tr>
                <th scope="col">Decisions</th>
                <th scope="col">Graded</th>
                <th scope="col">Best or good</th>
                <th scope="col" title="Points given up against the coach's choice, per decision">
                  Points given up
                </th>
                <th scope="col">Mistakes</th>
              </tr>
            </thead>
            <tbody>
              {PHASES.map(({ phase, label }) => {
                const list = counted.filter((g) => g.phase === phase);
                return (
                  <tr key={phase}>
                    <th scope="row">{label}</th>
                    <td>{list.length}</td>
                    <td>{pct(list.filter(good).length, list.length)}</td>
                    <td>{perDecision(list)}</td>
                    <td title="Mistakes and blunders">{list.filter(slip).length}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          <h4>Round by round: points given up per decision</h4>
          <div className="game-grid-scroll">
            <table className="score-pad recap-rounds">
              <thead>
                <tr>
                  {playedRounds.map((round) => (
                    <th key={round} scope="col" title={`Round ${round + 1} of ${ROUNDS}`}>
                      R{round + 1}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                <tr>
                  {playedRounds.map((round) => {
                    const list = counted.filter((g) => g.round === round);
                    const game = t.mine?.[round];
                    return (
                      <td key={round} title={`${list.length} decisions graded${game && !game.graded ? " so far" : ""}`}>
                        {perDecision(list)}
                      </td>
                    );
                  })}
                </tr>
              </tbody>
            </table>
          </div>
          {costliest.length > 0 && (
            <>
              <h4>Your costliest decisions</h4>
              <ol className="recap-costly">
                {costliest.map((g) => (
                  <li key={`${g.round}-${g.action}`}>
                    <span className="recap-where">
                      Round {g.round + 1}, hand {g.deal}
                      {g.trick ? `, trick ${g.trick}` : ""}
                    </span>
                    : you chose <BidText label={g.chosen} />; the coach preferred <BidText label={g.best} /> <strong>({Math.round(g.loss)} points)</strong>
                    {g.reason && <small>{g.reason}</small>}
                  </li>
                ))}
              </ol>
            </>
          )}
        </>
      )}
    </div>
  );
}
