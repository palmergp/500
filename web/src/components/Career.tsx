import { useState } from "react";
import { nextChapter, type CareerState } from "../career/career";
import { CHAPTERS, goal, STORY_BLURB, STORY_TITLE } from "../career/story";
import { Dialog } from "./Dialog";

interface Props {
  career: CareerState;
  /** The chapter under way has its game or tournament in progress, to pick up. */
  resumable: boolean;
  onName: (name: string) => void;
  /** Play a chapter: from its opening scene, or (trying again) straight into the level. */
  onPlay: (chapter: number, skipIntro: boolean) => void;
  /** A finished chapter's opening or ending scene again. */
  onWatch: (chapter: number, which: "opening" | "ending") => void;
  /** Helper settings now on (they keep a chapter from counting as unaided), and turning them off. */
  helpers: string[];
  onHelpOff: () => void;
  onResume: () => void;
  onReset: () => void;
  onClose: () => void;
}

const NUMBERS = ["One", "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Eleven", "Twelve"];

/** Story mode: the story's chapters, where you are in it, and the way into each level. */
/** What "unaided" means, for the 🙌 marks. */
const UNAIDED = "Beaten with no help: no hints, undos or Finish hand, and grading, the trump counter, the card tracker and out-of-suit marks all off.";

export function CareerDialog({ career, resumable, onName, onPlay, onWatch, helpers, onHelpOff, onResume, onReset, onClose }: Props) {
  const [naming, setNaming] = useState(!career.name);
  const [draft, setDraft] = useState(career.name);
  const next = nextChapter(career);
  const playing = career.playing;
  return (
    <Dialog title="Story" wide onClose={onClose}>
      <div className="career">
        <header className="career-head">
          <h3 className="career-title">{STORY_TITLE}</h3>
          <p className="career-blurb">{STORY_BLURB}</p>
        </header>
        {naming ? (
          <form
            className="career-name"
            onSubmit={(e) => {
              e.preventDefault();
              const name = draft.trim().slice(0, 24);
              if (!name) return;
              onName(name);
              setNaming(false);
            }}
          >
            <label>
              What do they call you at the table?
              <input value={draft} onChange={(e) => setDraft(e.target.value)} maxLength={24} autoFocus placeholder="Your name" />
            </label>
            <button type="submit" className="primary" disabled={!draft.trim()}>
              {career.name ? "Save" : "Begin"}
            </button>
          </form>
        ) : (
          <>
            <p className="career-you">
              Playing as <strong>{career.name}</strong>{" "}
              <button type="button" className="link-button" onClick={() => setNaming(true)}>
                Rename
              </button>
            </p>
            {next >= CHAPTERS.length && <p className="career-end">The end… for now. Play any chapter again.</p>}
            <ol className="career-chapters">
              {CHAPTERS.map((ch, i) => {
                const done = career.done.includes(i);
                const locked = i > next;
                const current = playing?.chapter === i;
                const retry = current && playing.attempts > 0 && !resumable;
                return (
                  <li key={ch.title} className={`career-chapter${done ? " done" : ""}${locked ? " locked" : ""}${i === next ? " next" : ""}`}>
                    <div className="career-chapter-text">
                      <span className="career-number">
                        Chapter {NUMBERS[i]} {done && <span aria-label="finished">✓</span>}
                        {done && career.unaided.includes(i) && (
                          <span className="career-unaided" role="img" aria-label="beaten unaided" title={UNAIDED}>
                            🙌
                          </span>
                        )}
                        {locked && <span aria-label="locked">🔒</span>}
                      </span>
                      <strong>{locked ? "???" : ch.title}</strong>
                      {!locked && <span className="career-tagline">{ch.tagline}</span>}
                      {!locked && (
                        <small className="career-goal">
                          {ch.level.kind === "tournament" ? "🏆 " : "🂠 "}
                          {goal(ch.level)}
                        </small>
                      )}
                      {retry && <small className="career-tries">Tries so far: {playing.attempts}</small>}
                    </div>
                    {!locked && (
                      <div className="career-chapter-actions">
                        {current && resumable ? (
                          <button type="button" className="primary" onClick={onResume}>
                            Continue
                          </button>
                        ) : retry ? (
                          <>
                            <button type="button" onClick={() => onPlay(i, false)} title="See the chapter's opening scene again, then play">
                              Watch the scene
                            </button>
                            <button type="button" className="primary" onClick={() => onPlay(i, true)}>
                              Try again
                            </button>
                          </>
                        ) : (
                          <>
                            {done && (
                              <>
                                <button type="button" onClick={() => onWatch(i, "opening")} title="Watch the chapter's opening scene again">
                                  Opening
                                </button>
                                <button type="button" onClick={() => onWatch(i, "ending")} title="Watch the chapter's ending scene again">
                                  Ending
                                </button>
                              </>
                            )}
                            <button type="button" className={i === next ? "primary" : ""} onClick={() => onPlay(i, false)}>
                              {done ? "Play again" : "Play"}
                            </button>
                          </>
                        )}
                      </div>
                    )}
                  </li>
                );
              })}
            </ol>
            <div className="note career-unaided-note">
              <span aria-hidden="true">🙌</span> marks a chapter you've beaten with no help: no hints, undos or Finish hand, and the helpers off.
              {helpers.length > 0 && ` Right now ${helpers.join(", ")} ${helpers.length > 1 ? "are" : "is"} on (Settings), so a chapter won't count.`}
              {helpers.length > 0 && (
                <div>
                  <button type="button" onClick={onHelpOff} title="Turn off grading, the trump counter, the card tracker and out-of-suit marks, to play unaided">
                    Turn off all help
                  </button>
                </div>
              )}
            </div>
          </>
        )}
        <div className="actions">
          {(career.done.length > 0 || playing) && (
            <button
              type="button"
              onClick={() => {
                if (confirm("Start the story over from Chapter One? Your chapters are cleared.")) onReset();
              }}
            >
              Start over
            </button>
          )}
          <button type="button" onClick={onClose}>
            Close
          </button>
        </div>
      </div>
    </Dialog>
  );
}
