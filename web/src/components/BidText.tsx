const SUIT_OF: Record<string, string> = { "♠": "S", "♣": "C", "♦": "D", "♥": "H" };

/** A call or bid ("8♥", "6NT", "Mention ♦") with its suit symbols in the suit's color. */
export function BidText({ label }: { label: string }) {
  return (
    <>
      {label.split(/([♠♣♦♥])/).map((part, i) =>
        SUIT_OF[part] ? (
          <span key={i} className={`suit-sym suit-${SUIT_OF[part]}`}>
            {part}
          </span>
        ) : (
          part
        ),
      )}
    </>
  );
}
