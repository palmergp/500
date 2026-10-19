interface Props {
  /** A solo game saved on this device that can be picked up. */
  canContinue: boolean;
  /** Whether the app came from a LAN host, so friends can join. */
  hostAvailable: boolean;
  /** You're at the host's table already (and can go back to it). */
  atTable: boolean;
  onContinue: () => void;
  onSolo: () => void;
  onFriends: () => void;
  onLearn: () => void;
  onSettings: () => void;
}

const SUITS = ["♠", "♥", "♣", "♦"];

/** The first screen: pick how to play, learn, or change settings. */
export function MainMenu({ canContinue, hostAvailable, atTable, onContinue, onSolo, onFriends, onLearn, onSettings }: Props) {
  return (
    <main className="main-menu">
      <div className="menu-card">
        <h1>
          500
          <span className="menu-suits" aria-hidden="true">
            {SUITS.map((s) => (
              <span key={s} className={s === "♥" || s === "♦" ? "red" : ""}>
                {s}
              </span>
            ))}
          </span>
        </h1>
        <p className="menu-tag">Play 500 and learn to play it well.</p>
        <nav className="menu-buttons" aria-label="Main menu">
          {atTable && (
            <button type="button" className="primary" onClick={onFriends}>
              Back to the table
              <small>Your game with friends</small>
            </button>
          )}
          {canContinue && (
            <button type="button" className={atTable ? "" : "primary"} onClick={onContinue}>
              Continue
              <small>Pick up your game against the bots</small>
            </button>
          )}
          <button type="button" className={!canContinue && !atTable ? "primary" : ""} onClick={onSolo}>
            Play solo
            <small>You and a bot partner against two bots</small>
          </button>
          {!atTable && (
            <button type="button" onClick={onFriends} disabled={!hostAvailable}>
              Play with friends
              <small>
                {hostAvailable ? "Everyone on your wifi joins from their phone or laptop" : "Start the game with play.cmd to host friends on your wifi"}
              </small>
            </button>
          )}
          <button type="button" onClick={onLearn}>
            Learn
            <small>Lessons, a bidding trainer and play puzzles</small>
          </button>
          <button type="button" onClick={onSettings}>
            Settings
            <small>Coach, sound, speed and display</small>
          </button>
        </nav>
      </div>
    </main>
  );
}
