import { useState } from "react";
import type { HostInfo, Lobby as LobbyState, NetClient } from "../net";
import { SEAT_NAMES } from "../net";
import type { Speed } from "../settings";
import { Dialog } from "./Dialog";

interface Props {
  host: HostInfo;
  lobby: LobbyState | null;
  net: NetClient;
  connected: boolean;
  name: string;
  onName: (name: string) => void;
  /** Leave the host's table and play against bots on this device. */
  onSolo: () => void;
  /** Back to the game (when one is on and you're in it). */
  onClose?: () => void;
}

// Compass layout: North at the top, South at the bottom.
const GRID: Record<number, string> = { 2: "north", 1: "west", 3: "east", 0: "south" };

/** The host's table: who sits where, which bots fill the empty seats, and the start button. */
export function Lobby({ host, lobby, net, connected, name, onName, onSolo, onClose }: Props) {
  const [draft, setDraft] = useState(name);
  const [confirmEnd, setConfirmEnd] = useState(false);
  const you = lobby?.you ?? null;
  const playing = !!lobby?.playing;
  const commitName = () => {
    const clean = draft.trim();
    if (clean && clean !== name) onName(clean);
  };

  return (
    <Dialog title="Table" wide onClose={onClose}>
      <div className="lobby">
        <section className="join">
          <img src="./api/qr.svg" alt={`QR code for ${host.url}`} width={132} height={132} />
          <p>
            Friends on the same wifi scan the code, or open <strong className="url">{host.url.replace(/^https?:\/\//, "").replace(/\/$/, "")}</strong>{" "}
            in their browser.
          </p>
        </section>

        {!connected && <p className="lobby-warning">Reconnecting to the host…</p>}

        <form
          className="lobby-name"
          onSubmit={(e) => {
            e.preventDefault();
            commitName();
          }}
        >
          <label>
            <span>Your name</span>
            <input value={draft} maxLength={16} autoComplete="nickname" onChange={(e) => setDraft(e.target.value)} onBlur={commitName} />
          </label>
        </form>

        {lobby ? (
          <>
            <div className="lobby-seats" role="group" aria-label="Seats">
              {lobby.seats.map((seat, i) => {
                const mine = you === i;
                const partner = you !== null && !mine && i % 2 === you % 2;
                const away = seat.person && !seat.person.connected;
                return (
                  <div key={i} className={`lobby-seat ${GRID[i]}${mine ? " mine" : ""}`} aria-label={SEAT_NAMES[i]}>
                    <div className="lobby-seat-head">
                      {SEAT_NAMES[i]}
                      {mine && <span className="tag">you</span>}
                      {partner && <span className="tag">your partner</span>}
                    </div>
                    {seat.person ? (
                      <div className="lobby-person">
                        <span className={`dot${away ? " away" : ""}`} aria-hidden="true" />
                        {seat.person.name}
                        {away && <small> (away)</small>}
                      </div>
                    ) : (
                      <select
                        aria-label={`${SEAT_NAMES[i]} bot`}
                        value={seat.bot}
                        disabled={you === null}
                        onChange={(e) => net.setBot(i, e.target.value)}
                      >
                        {lobby.personas.map((p) => (
                          <option key={p.id} value={p.id}>
                            🤖 {p.name}
                          </option>
                        ))}
                      </select>
                    )}
                    <div className="lobby-seat-actions">
                      {mine ? (
                        <button type="button" onClick={() => net.stand()}>
                          {playing ? "Leave (a bot takes over)" : "Stand up"}
                        </button>
                      ) : (
                        (!seat.person || away) &&
                        !(playing && you !== null) && (
                          <button type="button" className={you === null ? "primary" : ""} onClick={() => net.sit(i)}>
                            {seat.person ? "Take this seat" : playing ? "Take over" : "Sit here"}
                          </button>
                        )
                      )}
                      {away && you !== null && !mine && (
                        <button type="button" onClick={() => net.setBot(i, seat.bot)}>
                          Let a bot play
                        </button>
                      )}
                    </div>
                  </div>
                );
              })}
              <div className="lobby-center" aria-hidden="true">
                North &amp; South
                <br />
                vs
                <br />
                East &amp; West
              </div>
            </div>

            <fieldset className="lobby-pace" disabled={you === null}>
              <legend>Pace</legend>
              {(["slow", "normal", "fast"] as Speed[]).map((pace) => (
                <label key={pace} className="inline">
                  <input type="radio" name="pace" checked={lobby.pace === pace} onChange={() => net.setPace(pace)} />
                  {pace}
                </label>
              ))}
            </fieldset>

            <div className="actions">
              <button type="button" onClick={onSolo}>
                Play solo instead
              </button>
              {playing ? (
                you !== null ? (
                  confirmEnd ? (
                    <>
                      <span className="confirm">End the game for everyone?</span>
                      <button type="button" className="danger" onClick={() => { setConfirmEnd(false); net.endGame(); }}>
                        End game
                      </button>
                      <button type="button" onClick={() => setConfirmEnd(false)}>
                        Keep playing
                      </button>
                    </>
                  ) : (
                    <>
                      <button type="button" onClick={() => setConfirmEnd(true)}>
                        End game…
                      </button>
                      {onClose && (
                        <button type="button" className="primary" onClick={onClose}>
                          Back to the game
                        </button>
                      )}
                    </>
                  )
                ) : (
                  <span className="confirm">A game is on: take over a bot's seat to join it.</span>
                )
              ) : (
                <button type="button" className="primary" disabled={you === null || !connected} onClick={() => net.start()}>
                  {you === null ? "Sit down to start" : "Deal"}
                </button>
              )}
            </div>
          </>
        ) : (
          <p>Connecting to the host…</p>
        )}
      </div>
    </Dialog>
  );
}
