import type { CardView } from "../engine";
import { SUIT_SYMBOL } from "../engine";

const SUIT_NAME = { S: "spades", C: "clubs", D: "diamonds", H: "hearts" } as const;

function describe(card: CardView): string {
  if (card.suit === null) return "Joker";
  let text = `${card.rank} of ${SUIT_NAME[card.suit]}`;
  if (card.bower) text += ` (${card.bower} bower)`;
  return text;
}

interface CardProps {
  card: CardView;
  onClick?: () => void;
  playable?: boolean;
  selected?: boolean;
  fromKitty?: boolean;
  small?: boolean;
}

export function Card({ card, onClick, playable, selected, fromKitty, small }: CardProps) {
  const symbol = card.suit ? SUIT_SYMBOL[card.suit] : "★";
  const classes = [
    "card",
    card.suit === null ? "joker" : card.red ? "red" : "black",
    playable ? "playable" : "",
    selected ? "selected" : "",
    small ? "small" : "",
    card.trump ? "trump" : "",
  ];
  return (
    <button
      type="button"
      className={classes.filter(Boolean).join(" ")}
      onClick={onClick}
      disabled={!onClick}
      aria-label={describe(card)}
      aria-pressed={selected || undefined}
    >
      <span className="corner">
        {card.suit && <span className="rank">{card.rank}</span>}
        <span className="suit">{symbol}</span>
      </span>
      <span className="pip">{card.suit ? symbol : "JOKER"}</span>
      {card.bower && (
        <span className="badge" title={`${card.bower === "right" ? "Right" : "Left"} bower: a trump`}>
          {card.bower === "right" ? "R" : "L"}
        </span>
      )}
      {fromKitty && <span className="kitty-tag">kitty</span>}
    </button>
  );
}

export function CardBack({ small }: { small?: boolean }) {
  return <span className={`card back${small ? " small" : ""}`} aria-hidden="true" />;
}
