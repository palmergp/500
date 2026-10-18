import type { ActionView, StrainCode, SuitCode } from "../engine";
import { isRed, strainLabel } from "../engine";

const STRAINS: StrainCode[] = ["S", "C", "D", "H", "NT"];
const SUITS: SuitCode[] = ["S", "C", "D", "H"];
const LEVELS = [7, 8, 9, 10];
const VALUE = (level: number, strain: StrainCode) => 40 + 20 * STRAINS.indexOf(strain) + 100 * (level - 6);

interface Props {
  legal: ActionView[];
  mustBid: boolean;
  onAct: (index: number) => void;
}

/** Pass, mentions, 6-calls and the 7–10 bid grid. Calls that aren't legal now are disabled. */
export function BiddingBox({ legal, mustBid, onAct }: Props) {
  const find = (kind: ActionView["kind"], strain?: StrainCode, level?: number) =>
    legal.find((a) => a.kind === kind && (strain === undefined || a.strain === strain) && (level === undefined || a.level === level));

  const button = (action: ActionView | undefined, label: string, strain: StrainCode | null, title: string) => (
    <button
      type="button"
      key={title}
      className={`call ${isRed(strain) ? "red" : ""}`}
      disabled={!action}
      title={title}
      onClick={() => action && onAct(action.index)}
    >
      {label}
    </button>
  );

  const pass = find("pass");
  return (
    <div className="bidding-box">
      {mustBid && <p className="note">Third deal with no bid: as dealer you must bid.</p>}
      <div className="call-row">
        <span className="row-label">Mention</span>
        {STRAINS.map((s) => button(find("mention", s), strainLabel(s), s, `Mention ${strainLabel(s)}: my strongest suit, not enough to bid`))}
      </div>
      <div className="call-row">
        <span className="row-label">6 (Joker)</span>
        {SUITS.map((s) => button(find("six", s), `6${strainLabel(s)}`, s, `6${strainLabel(s)}: I hold the Joker; this is my best suit`))}
      </div>
      {LEVELS.map((level) => (
        <div className="call-row" key={level}>
          <span className="row-label">{level}</span>
          {STRAINS.map((s) =>
            button(find("bid", s, level), `${level}${strainLabel(s)}`, s, `Bid ${level}${strainLabel(s)} (${VALUE(level, s)} points)`),
          )}
        </div>
      ))}
      <button type="button" className="call pass" disabled={!pass} onClick={() => pass && onAct(pass.index)}>
        Pass
      </button>
    </div>
  );
}
