import { useEffect, useRef } from "react";
import type { PlayerView, TrickView } from "../engine";
import { SUIT_SYMBOL } from "../engine";
import { Card } from "./Card";
import { Dialog } from "./Dialog";

interface Props {
  tricks: TrickView[];
  players: PlayerView[];
  /** Scroll to this trick (0-based) when opened. */
  focus?: number;
  onClose: () => void;
}

/** The tricks played so far this hand, each with who led, every card in order and who won it. */
export function TricksDialog({ tricks, players, focus, onClose }: Props) {
  const focusRef = useRef<HTMLLIElement>(null);
  useEffect(() => {
    focusRef.current?.scrollIntoView({ block: "nearest" });
  }, []);
  return (
    <Dialog title="Tricks this hand" onClose={onClose}>
      {tricks.length === 0 ? (
        <p>No tricks yet.</p>
      ) : (
        <ol className="past-tricks">
          {tricks.map((trick, i) => {
            const winner = trick.winner;
            const ours = winner !== null && winner % 2 === 0;
            return (
              <li key={i} ref={i === focus ? focusRef : undefined} className={i === focus ? "focused" : ""}>
                <div className="past-trick-head">
                  <strong>Trick {i + 1}</strong>
                  <span>{players[trick.leader].name} led</span>
                  {winner !== null && <span className={`won ${ours ? "ours" : "theirs"}`}>{players[winner].name} won it</span>}
                  {trick.namedSuit && <span>Joker names {SUIT_SYMBOL[trick.namedSuit]}</span>}
                </div>
                <div className="past-trick-cards">
                  {trick.plays.map((p) => (
                    <figure key={p.seat} className={p.seat === winner ? "winner" : ""}>
                      <Card card={p.card} />
                      <figcaption>{players[p.seat].name}</figcaption>
                    </figure>
                  ))}
                </div>
              </li>
            );
          })}
        </ol>
      )}
      <div className="actions">
        <button type="button" className="primary" onClick={onClose} autoFocus>
          Back to the game
        </button>
      </div>
    </Dialog>
  );
}
