import type { HandView, PlayerView } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { Card } from "./Card";

interface Props {
  hand: HandView;
  players: PlayerView[];
}

/** All four hands as dealt, the kitty, the discards, the auction and every trick. */
export function HandReview({ hand, players }: Props) {
  const c = hand.contract!;
  return (
    <div className="review">
      <p>
        {players[c.declarer].name} ({SEAT_NAMES[c.declarer]}) played <strong>{c.label}</strong> for {c.value} and{" "}
        {hand.made ? "made it" : "went down"}, taking {hand.tricksWon[c.declarer % 2]} tricks.
      </p>
      <h3>Auction</h3>
      <p className="auction-line">
        {hand.calls.map((call, i) => (
          <span key={i}>
            {SEAT_NAMES[call.seat]}: <strong>{call.label}</strong>
          </span>
        ))}
      </p>
      <h3>Hands as dealt</h3>
      <div className="review-hands">
        {[2, 1, 3, 0].map((seat) => (
          <div key={seat} className="review-hand">
            <div className="who">
              {SEAT_NAMES[seat]} · {players[seat].name}
            </div>
            <div className="mini-hand">
              {hand.dealt[seat].map((card) => (
                <Card key={card.id} card={card} small />
              ))}
            </div>
          </div>
        ))}
      </div>
      <div className="review-hands">
        <div className="review-hand">
          <div className="who">Kitty</div>
          <div className="mini-hand">
            {hand.kitty.map((card) => (
              <Card key={card.id} card={card} small />
            ))}
          </div>
        </div>
        <div className="review-hand">
          <div className="who">Discarded by {SEAT_NAMES[c.declarer]}</div>
          <div className="mini-hand">
            {hand.discards.map((card) => (
              <Card key={card.id} card={card} small />
            ))}
          </div>
        </div>
      </div>
      <h3>Tricks</h3>
      <ol className="trick-list">
        {hand.tricks.map((trick, i) => (
          <li key={i}>
            {trick.plays.map((p) => (
              <span key={p.seat} className={p.seat === trick.winner ? "winner" : ""}>
                {SEAT_NAMES[p.seat][0]} {p.card.label}
                {p.card.suit === null && trick.namedSuit ? ` (naming ${SUIT_SYMBOL[trick.namedSuit]})` : ""}
              </span>
            ))}
          </li>
        ))}
      </ol>
    </div>
  );
}
