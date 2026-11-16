import type { ReactNode } from "react";
import type { CallView, SeatIndex, TrickView, View } from "../engine";
import { SEAT_NAMES, SUIT_SYMBOL } from "../engine";
import { BidText } from "./BidText";
import { Card, CardBack } from "./Card";

const POSITION: Record<SeatIndex, string> = { 0: "south", 1: "west", 2: "north", 3: "east" };

const AVATAR: Record<string, string> = {
  human: "🧑",
  steady: "🧭",
  rookie: "🐣",
  rock: "🪨",
  gambler: "🎲",
  counter: "🧮",
  shark: "🦈",
  oracle: "🔮",
  natural: "⚡",
};

function lastCall(calls: CallView[], seat: SeatIndex) {
  return calls.find((c) => c.seat === seat);
}

const VOID_SUITS = ["S", "C", "D", "H"] as const;

/**
 * Finished tricks by the seat that keeps them: as at a real table, the first player on a side to
 * win a trick gathers that side's tricks in front of them, one pile per trick. `holding` leaves
 * out the trick still on the table.
 */
function trickPiles(view: View, holding: boolean): Record<SeatIndex, number[]> {
  const done = holding ? view.tricks.slice(0, -1) : view.tricks;
  const keeper: (SeatIndex | null)[] = [null, null];
  const piles: Record<SeatIndex, number[]> = { 0: [], 1: [], 2: [], 3: [] };
  done.forEach((trick, i) => {
    if (trick.winner === null) return;
    const side = trick.winner % 2;
    keeper[side] ??= trick.winner;
    piles[keeper[side]!].push(i);
  });
  return piles;
}

function Piles({ indices, onOpen }: { indices: number[]; onOpen?: (index: number) => void }) {
  if (indices.length === 0) return null;
  return (
    <div className="piles" role="group" aria-label={`${indices.length} ${indices.length === 1 ? "trick" : "tricks"} taken`}>
      {indices.map((i, n) => (
        <button
          key={i}
          type="button"
          className={`pile${n % 2 ? " crossed" : ""}`}
          title={`Trick ${i + 1}: see the tricks so far`}
          aria-label={`Trick ${i + 1}`}
          onClick={() => onOpen?.(i)}
          disabled={!onOpen}
        />
      ))}
    </div>
  );
}

interface PlateProps {
  view: View;
  seat: SeatIndex;
  showVoids: boolean;
  piles: number[];
  onTricks?: (index: number) => void;
}

function SeatPlate({ view, seat, showVoids, piles, onTricks }: PlateProps) {
  const voids = showVoids && view.phase === "playing" ? VOID_SUITS.filter((_, i) => view.shownOut[seat][i]) : [];
  const player = view.players[seat];
  const bidding = view.phase === "bidding";
  const call = bidding ? lastCall(view.calls, seat) : undefined;
  const latest = bidding && view.calls.length > 0 && view.calls[view.calls.length - 1].seat === seat;
  const declarer = view.contract?.declarer === seat;
  const toAct = view.toAct === seat && view.phase !== "gameOver";
  const backs = Math.min(view.handCounts[seat], 13);
  const classes = ["seat", `seat-${POSITION[seat]}`, toAct ? "to-act" : "", latest ? "just-called" : ""].filter(Boolean).join(" ");
  return (
    <div className={classes} title={player.blurb || undefined} aria-current={toAct ? "true" : undefined}>
      <div className="seat-name">
        <span className="avatar" aria-hidden="true">
          {AVATAR[player.persona] ?? "🙂"}
        </span>
        {player.name}
        {seat === 2 && <span className="tag">partner</span>}
        {view.dealer === seat && <span className="chip">dealer</span>}
        {declarer && (
          <span className="chip declarer">
            <BidText label={view.contract!.label} />
          </span>
        )}
      </div>
      <div className="seat-sub">
        {SEAT_NAMES[seat]}
        {toAct && <span className="turn">{seat === 0 ? "your turn" : bidding ? "to call" : "to play"}</span>}
        {voids.map((s) => (
          <span key={s} className={`void-chip${s === "D" || s === "H" ? " red" : ""}`} title={`Out of ${SUIT_SYMBOL[s]}`}>
            no {SUIT_SYMBOL[s]}
          </span>
        ))}
      </div>
      {seat !== 0 && (
        <div className="backs" aria-label={`${view.handCounts[seat]} cards`}>
          {Array.from({ length: backs }, (_, i) => (
            <span key={i} className="back-slot">
              <CardBack small />
            </span>
          ))}
        </div>
      )}
      <Piles indices={piles} onOpen={onTricks} />
      {call && (
        <div key={latest ? `new-${view.calls.length}` : "old"} className={`bubble ${call.kind}${latest ? " latest" : ""}`} aria-label={`${player.name}: ${call.label}`}>
          <BidText label={call.label} />
        </div>
      )}
    </div>
  );
}

function TrickArea({ trick, sweepTo }: { trick: TrickView; sweepTo: SeatIndex | null }) {
  return (
    <div className={`trick-area${sweepTo !== null ? ` sweep sweep-${POSITION[sweepTo]}` : ""}`} aria-label="Current trick">
      {trick.plays.map((p) => (
        <div key={p.seat} className={`trick-card at-${POSITION[p.seat]}${trick.winner === p.seat ? " winning" : ""}`}>
          <Card card={p.card} />
        </div>
      ))}
      {trick.namedSuit && <div className="named">Joker names {SUIT_SYMBOL[trick.namedSuit]}</div>}
    </div>
  );
}

/** Which hand of the game this is. */
export function gameProgress(view: View): string {
  if (view.handLimit === null) return `First to ${view.target}`;
  const hand = view.handsPlayed + (view.phase === "gameOver" ? 0 : 1);
  return hand > view.handLimit ? `Extra hand ${hand - view.handLimit} (tied)` : `Hand ${Math.max(1, hand)} of ${view.handLimit}`;
}

/** The auction so far and the bid being played: kept beside the table, not under any one seat. */
export function InfoPanel({
  view,
  trumpsUnseen,
  onTricks,
  decisions,
  onDecisions,
}: {
  view: View;
  trumpsUnseen?: number | null;
  onTricks?: () => void;
  /** How many of your decisions the coach has graded this hand. */
  decisions?: number;
  onDecisions?: () => void;
}) {
  const c = view.contract;
  return (
    <section className="info-panel" aria-label="Bidding">
      <div className="info-game">{gameProgress(view)}</div>
      {view.phase === "bidding" && (
        <div className="auction" aria-label="Auction">
          <div className="auction-title">
            Bidding · deal {view.dealNumber} of 3{view.dealerForced ? " · dealer must bid" : ""}
          </div>
          <ol>
            {view.calls.map((call, i) => (
              <li key={i} className={`auction-row${i === view.calls.length - 1 ? " latest" : ""}`}>
                <span>{view.players[call.seat].name}</span>
                <strong className={call.kind}>
                  <BidText label={call.label} />
                </strong>
              </li>
            ))}
            {view.toAct !== null && (
              <li className="auction-row current">
                <span>{view.players[view.toAct].name}</span>
                <strong>…</strong>
              </li>
            )}
          </ol>
        </div>
      )}
      {c && (
        <div className="contract">
          <div className="bid-tile" aria-label={`Bid ${c.label} by ${view.players[c.declarer].name}`}>
            <strong className="bid-tile-bid">
              <BidText label={c.label} />
            </strong>
            <span className="bid-tile-text">
              <span>{c.declarer === 0 ? "Your bid" : `${view.players[c.declarer].name}'s bid`}</span>
              <span>
                {c.strain === "NT" ? (
                  "No trumps"
                ) : (
                  <>
                    Trumps <BidText label={SUIT_SYMBOL[c.strain]} />
                  </>
                )}
              </span>
            </span>
          </div>
          {view.phase !== "discarding" && (
            <div className="contract-tricks">
              Tricks: <b>{view.tricksWon[c.declarer % 2]}</b> of {c.level} needed · defenders <b>{view.tricksWon[1 - (c.declarer % 2)]}</b>
            </div>
          )}
          {trumpsUnseen != null && view.phase === "playing" && (
            <div className="trump-count" title="Trumps not in your hand, not yet played, and not among your discards">
              Trumps out: {trumpsUnseen}
            </div>
          )}
        </div>
      )}
      <div className="info-buttons">
        {onTricks && view.tricks.length > 0 && view.phase === "playing" && (
          <button type="button" className="past-tricks-button" onClick={onTricks} title="See every trick played so far this hand">
            Past tricks ({view.tricks.length})
          </button>
        )}
        {onDecisions && (
          <button type="button" className="past-tricks-button" onClick={onDecisions} title="The coach's grades for your bids and plays so far, with its reasons">
            Your decisions{decisions ? ` (${decisions})` : ""}
          </button>
        )}
      </div>
    </section>
  );
}

interface Props {
  view: View;
  /** The trick to draw in the middle: the live one, or the one just completed. */
  trick: TrickView | null;
  /** Shown when the trump counter is on. */
  trumpsUnseen?: number | null;
  /** Sweep the finished trick toward its winner. */
  sweepTo?: SeatIndex | null;
  /** Mark the suits each player has shown out of. */
  showVoids?: boolean;
  /** A finished trick is still on the table, not yet gathered into a pile. */
  holding?: boolean;
  /** Opens the tricks played so far, at the given one. */
  onTricks?: (index?: number) => void;
  /** Opens your graded decisions; `decisions` is how many this hand. */
  onDecisions?: () => void;
  decisions?: number;
  /** Show the bid and auction panel in the table's top corner (when there's no side rail). */
  infoInCorner?: boolean;
  /** Shown in the middle instead of the trick: the bidding box, the discard button. */
  middle?: ReactNode;
  /** Coach panels shown above the middle (when there's no side rail). */
  dock?: ReactNode;
  /** Beside your seat: the status line and the Hint button. */
  status?: ReactNode;
  /** Below: your hand. */
  children: ReactNode;
}

/** The felt: partner and opponents around the trick, your seat and hand below. */
export function TableView({
  view,
  trick,
  trumpsUnseen,
  sweepTo = null,
  showVoids = false,
  holding = false,
  onTricks,
  onDecisions,
  decisions,
  infoInCorner,
  middle,
  dock,
  status,
  children,
}: Props) {
  const showTrick = view.phase === "playing" || (trick && trick.plays.length > 0);
  const piles = trickPiles(view, holding);
  const plate = (seat: SeatIndex) => <SeatPlate view={view} seat={seat} showVoids={showVoids} piles={piles[seat]} onTricks={onTricks} />;
  return (
    <div className="table">
      {infoInCorner && (
        <div className="table-info">
          <InfoPanel view={view} trumpsUnseen={trumpsUnseen} onTricks={onTricks ? () => onTricks() : undefined} decisions={decisions} onDecisions={onDecisions} />
        </div>
      )}
      {plate(2)}
      {plate(1)}
      <div className="center">
        {dock && <div className="dock">{dock}</div>}
        <div className="center-main">
          {middle ??
            (showTrick && trick ? (
              <TrickArea trick={trick} sweepTo={sweepTo} />
            ) : view.phase === "discarding" && !view.humanToAct ? (
              <div className="waiting">{view.players[view.contract!.declarer].name} is taking the kitty…</div>
            ) : null)}
        </div>
      </div>
      {plate(3)}
      <div className="south">
        <div className="south-bar">
          {plate(0)}
          {status}
        </div>
        {children}
      </div>
    </div>
  );
}
