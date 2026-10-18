import { useMemo } from "react";
import type { HandView } from "../engine";

/** A short banner when a hand is scored, and confetti when you win the game. */
export function Celebration({ hand, gameWinner }: { hand: HandView; gameWinner: 0 | 1 | null }) {
  const c = hand.contract;
  const ours = c ? c.declarer % 2 === 0 : false;
  const good = ours === !!hand.made;
  const text =
    gameWinner === 0
      ? "You win!"
      : gameWinner === 1
        ? "Game over"
        : !c
          ? ""
          : ours
            ? hand.made
              ? `Made ${c.label}!`
              : `Down in ${c.label}`
            : hand.made
              ? `They made ${c.label}`
              : `Set them! ${c.label}`;
  const pieces = useMemo(
    () =>
      Array.from({ length: 70 }, (_, i) => ({
        left: Math.random() * 100,
        delay: Math.random() * 0.8,
        duration: 2.2 + Math.random() * 1.6,
        hue: [45, 0, 210, 130, 280][i % 5],
        spin: Math.random() * 720 - 360,
      })),
    [],
  );
  return (
    <div className="celebration" aria-hidden="true">
      <div className={`banner ${good ? "good" : "bad"}`}>{text}</div>
      {gameWinner === 0 &&
        pieces.map((p, i) => (
          <span
            key={i}
            className="confetti"
            style={
              {
                left: `${p.left}%`,
                animationDelay: `${p.delay}s`,
                animationDuration: `${p.duration}s`,
                background: `hsl(${p.hue} 85% 60%)`,
                "--spin": `${p.spin}deg`,
              } as React.CSSProperties
            }
          />
        ))}
    </div>
  );
}
