import type { CallView, SeatIndex, TrickView, View } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { Card, CardBack } from "./Card";

const POSITION: Record<SeatIndex, string> = { 0: "south", 1: "west", 2: "north", 3: "east" };

function lastCall(calls: CallView[], seat: SeatIndex) {
  return calls.find((c) => c.seat === seat);
}

function SeatPlate({ view, seat }: { view: View; seat: SeatIndex }) {
  const player = view.players[seat];
  const call = view.phase === "bidding" ? lastCall(view.calls, seat) : undefined;
  const declarer = view.contract?.declarer === seat;
  const thinking = view.toAct === seat && !view.humanToAct;
  const backs = Math.min(view.handCounts[seat], 13);
  return (
    <div className={`seat seat-${POSITION[seat]}${thinking ? " thinking" : ""}`} title={player.blurb}>
      <div className="seat-name">
        {player.name}
        {seat === 2 && <span className="tag">partner</span>}
        {view.dealer === seat && <span className="chip">dealer</span>}
        {declarer && <span className="chip declarer">{view.contract!.label}</span>}
      </div>
      <div className="seat-sub">{SEAT_NAMES[seat]}</div>
      {seat !== 0 && (
        <div className="backs" aria-label={`${view.handCounts[seat]} cards`}>
          {Array.from({ length: backs }, (_, i) => (
            <CardBack key={i} small />
          ))}
        </div>
      )}
      {call && <div className={`bubble ${call.kind}`}>{call.label}</div>}
    </div>
  );
}

function TrickArea({ trick }: { trick: TrickView }) {
  return (
    <div className="trick-area" aria-label="Current trick">
      {trick.plays.map((p) => (
        <div key={p.seat} className={`trick-card at-${POSITION[p.seat]}${trick.winner === p.seat ? " winning" : ""}`}>
          <Card card={p.card} />
        </div>
      ))}
      {trick.namedSuit && <div className="named">Joker names {SUIT_SYMBOL[trick.namedSuit]}</div>}
    </div>
  );
}

function AuctionArea({ view }: { view: View }) {
  const first = ((view.dealer + 1) % 4) as SeatIndex;
  const order = [0, 1, 2, 3].map((i) => ((first + i) % 4) as SeatIndex);
  return (
    <div className="auction" aria-label="Auction">
      <div className="auction-title">
        Deal {view.dealNumber} of 3{view.dealerForced ? " · dealer must bid" : ""}
      </div>
      {order.map((seat) => {
        const call = lastCall(view.calls, seat);
        return (
          <div key={seat} className={`auction-row${view.toAct === seat ? " current" : ""}`}>
            <span>{view.players[seat].name}</span>
            <strong className={call?.kind}>{call ? call.label : view.toAct === seat ? "…" : ""}</strong>
          </div>
        );
      })}
    </div>
  );
}

interface Props {
  view: View;
  /** The trick to draw in the middle: the live one, or the one just completed. */
  trick: TrickView | null;
  /** Shown when the trump counter is on. */
  trumpsUnseen?: number | null;
  children: React.ReactNode;
}

/** The felt: three opponents' and partner's seats around the trick, your seat and hand below. */
export function TableView({ view, trick, trumpsUnseen, children }: Props) {
  const showTrick = view.phase === "playing" || (trick && trick.plays.length > 0);
  return (
    <div className="table">
      <SeatPlate view={view} seat={2} />
      <SeatPlate view={view} seat={1} />
      <div className="center">
        {view.contract && (
          <div className="contract">
            {view.players[view.contract.declarer].name}: <strong>{view.contract.label}</strong> · tricks {view.tricksWon[view.contract.declarer % 2]}
            /{view.contract.level}
            {trumpsUnseen != null && view.phase === "playing" && (
              <span className="trump-count" title="Trumps not in your hand, not yet played, and not among your discards">
                trumps out: {trumpsUnseen}
              </span>
            )}
          </div>
        )}
        {showTrick && trick ? <TrickArea trick={trick} /> : view.phase === "bidding" ? <AuctionArea view={view} /> : null}
        {view.phase === "discarding" && !view.humanToAct && (
          <div className="waiting">{view.players[view.contract!.declarer].name} is taking the kitty…</div>
        )}
      </div>
      <SeatPlate view={view} seat={3} />
      <div className="south">{children}</div>
    </div>
  );
}
