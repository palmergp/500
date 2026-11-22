import { useEffect, useMemo, useRef, useState } from "react";
import { characterName, fill } from "../career/career";
import { CHARACTERS, PLACE_SOUND, type Backdrop, type CharacterId, type Step } from "../career/story";
import { storyAudio, type Mood } from "../storyAudio";
import { Portrait } from "./Portrait";

export { Portrait };

/** Characters per second while a line types out. */
const TYPE_SPEED = 60;

const reducedMotion = () => !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

/** Steps that aren't shown but set the scene: the place, the score's mood, a sound. */
const setsScene = (step: Step) => "bg" in step || "music" in step || "sfx" in step;

/** The backdrop: a gradient and a few shapes for each place (styles.css `.bg-*`). */
function Scene({ bg }: { bg: Backdrop | null }) {
  if (!bg) return <div className="cut-bg bg-black" aria-hidden="true" />;
  return (
    <div className={`cut-bg bg-${bg}`} aria-hidden="true">
      {(bg === "diner" || bg === "diner-dawn") && (
        <>
          <div className="cut-window">{bg === "diner" && <div className="cut-rain" />}</div>
          <div className="cut-neon">
            MAE&rsquo;<span className={bg === "diner" ? "dead" : ""}>S</span>
          </div>
          <div className="cut-counter" />
        </>
      )}
      {bg === "street" && (
        <>
          <div className="cut-lamp" />
          <div className="cut-rain" />
        </>
      )}
      {bg === "poolhall" && (
        <>
          <div className="cut-cone" />
          <div className="cut-felt" />
        </>
      )}
      {bg === "velvet" && <div className="cut-frame">🃏</div>}
      {bg === "warehouse" && (
        <div className="cut-lamps">
          <i />
          <i />
          <i />
        </div>
      )}
      {bg === "car" && (
        <>
          <div className="cut-windshield">
            <div className="cut-rain" />
          </div>
          <div className="cut-dash" />
        </>
      )}
      {bg === "boarding" && <div className="cut-beam" />}
      {bg === "lounge" && <div className="cut-lantern" />}
      {bg === "pier" && (
        <>
          <div className="cut-lamp" />
          <div className="cut-fog" />
          <div className="cut-fog two" />
        </>
      )}
      {bg === "ballroom" && (
        <div className="cut-chandeliers">
          <i />
          <i />
          <i />
        </div>
      )}
      {bg === "penthouse" && <div className="cut-city" />}
    </div>
  );
}

/** The first step at or after `i` that's shown (a change of scene, mood or a sound just happens). */
function shownFrom(list: Step[], i: number): number {
  while (i < list.length && setsScene(list[i])) i++;
  return i;
}

/** The scene as of step `at`: the backdrop and place, and who's standing on the left and right
 * (a speaker keeps their side; a new one takes the side of whoever spoke less recently). */
function sceneAt(list: Step[], at: number) {
  let bg: Backdrop | null = null;
  let place = "";
  let slots: (CharacterId | null)[] = [null, null];
  let last: CharacterId | null = null;
  let mood: Mood = "silence";
  for (let i = 0; i <= at && i < list.length; i++) {
    const step = list[i];
    if ("bg" in step) {
      bg = step.bg;
      place = step.place;
      mood = PLACE_SOUND[step.bg].mood;
      slots = [null, null];
      last = null;
    } else if ("music" in step) {
      mood = step.music;
    } else if ("say" in step && step.say !== "you") {
      const who = step.say;
      if (!slots.includes(who)) {
        const free = slots[0] === null ? 0 : slots[1] === null ? 1 : slots[0] === last ? 1 : 0;
        slots = free === 0 ? [who, slots[1]] : [slots[0], who];
      }
      last = who;
    }
  }
  return { bg, place, slots, mood };
}

/**
 * A story scene, full screen: the place, the people in it, and their lines typed out one at a time
 * (a click or Next shows the whole line, then moves on). Some lines are your choice of what to say.
 */
export function Cutscene({
  steps,
  name,
  label,
  music,
  sound,
  onMusic,
  onSound,
  onDone,
}: {
  steps: Step[];
  name: string;
  label: string;
  /** The score (the story-music setting), and the place's sound and effects (the sound setting). */
  music: boolean;
  sound: boolean;
  onMusic: (on: boolean) => void;
  onSound: (on: boolean) => void;
  onDone: () => void;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  const [list, setList] = useState(steps);
  const [at, setAt] = useState(() => shownFrom(steps, 0));
  const [typed, setTyped] = useState(0);
  /** The line was clicked through: shown whole, though it was still typing. */
  const [whole, setWhole] = useState(false);
  useEffect(() => {
    const dialog = ref.current;
    if (dialog && !dialog.open) dialog.showModal();
    return () => dialog?.close();
  }, []);
  const step: Step | undefined = list[at];
  const { bg, place, slots, mood } = useMemo(() => sceneAt(list, at), [list, at]);

  // The sound: the score and the place's sound follow the scene; effects and a title's sting play
  // as their moment comes.
  useEffect(() => {
    storyAudio.start();
    return () => storyAudio.stop();
  }, []);
  useEffect(() => storyAudio.setMusic(music), [music]);
  useEffect(() => storyAudio.setSound(sound), [sound]);
  useEffect(() => {
    storyAudio.setMood(mood);
    storyAudio.setAmbience(bg ? PLACE_SOUND[bg].ambience : "none");
  }, [mood, bg]);
  const played = useRef(-1);
  useEffect(() => {
    for (let i = played.current + 1; i <= at && i < list.length; i++) {
      const s = list[i];
      if ("sfx" in s) storyAudio.sfx(s.sfx);
      else if ("title" in s) storyAudio.sfx("sting");
    }
    played.current = Math.max(played.current, at);
  }, [at, list]);
  const text = !step ? "" : "say" in step ? fill(step.text, name) : "narrate" in step ? fill(step.narrate, name) : "";
  useEffect(() => {
    setWhole(false);
    if (reducedMotion() || text.length === 0) {
      setTyped(Number.POSITIVE_INFINITY);
      return;
    }
    setTyped(0);
    const started = performance.now();
    const timer = setInterval(() => {
      const n = Math.floor(((performance.now() - started) / 1000) * TYPE_SPEED);
      setTyped(n);
      if (n >= text.length) clearInterval(timer);
    }, 30);
    return () => clearInterval(timer);
  }, [text, at]);
  const finished = whole || typed >= text.length;

  const advance = () => {
    if (!step) return onDone();
    if ("choose" in step) return;
    if (!finished) {
      setWhole(true);
      return;
    }
    const next = shownFrom(list, at + 1);
    if (next >= list.length) onDone();
    else setAt(next);
  };
  const pick = (option: { label: string; then: Step[] }) => {
    // Your words take the choice's place, and what follows from them comes next.
    setList([...list.slice(0, at), { say: "you", text: option.label }, ...option.then, ...list.slice(at + 1)]);
  };

  const speaker = step && "say" in step ? step.say : null;
  const shown = finished ? text : text.slice(0, typed);
  return (
    <dialog
      ref={ref}
      className="cutscene"
      aria-label={label}
      onCancel={(e) => e.preventDefault()}
    >
      <Scene bg={bg} />
      {step && "title" in step ? (
        <div className="cut-title" onClick={advance}>
          <h2>{step.title}</h2>
          {step.sub && <p>{step.sub}</p>}
          <button type="button" className="cut-continue" autoFocus onClick={(e) => (e.stopPropagation(), advance())}>
            Continue ▸
          </button>
        </div>
      ) : (
        <div className="cut-stage" onClick={advance}>
          {place && <div className="cut-place">{place}</div>}
          {slots.map((who, side) =>
            who ? (
              <div key={`${side}-${who}`} className={`cut-actor ${side === 0 ? "left" : "right"}${who === speaker ? " speaking" : ""}`} title={CHARACTERS[who].name}>
                <Portrait look={CHARACTERS[who].look} />
              </div>
            ) : null,
          )}
          <div className={`cut-box${speaker ? "" : " narration"}${speaker === "you" ? " you" : ""}`} onClick={(e) => step && "choose" in step && e.stopPropagation()}>
            {speaker && (
              <div className="cut-name" title={speaker === "you" ? undefined : CHARACTERS[speaker].role}>
                {characterName(speaker, name)}
              </div>
            )}
            {step && "choose" in step ? (
              <div className="cut-choices" role="group" aria-label="What do you say?">
                {step.choose.map((option, i) => (
                  <button key={option.label} type="button" autoFocus={i === 0} onClick={() => pick(option)}>
                    {option.label}
                  </button>
                ))}
              </div>
            ) : (
              <p className="cut-text">
                <span aria-hidden="true">{shown}</span>
                <span className="visually-hidden">{text}</span>
              </p>
            )}
            {!(step && "choose" in step) && (
              <button type="button" className="cut-next" onClick={(e) => (e.stopPropagation(), advance())} autoFocus aria-label="Next">
                {finished ? "▼" : "…"}
              </button>
            )}
          </div>
        </div>
      )}
      <div className="cut-controls">
        <button
          type="button"
          className={`cut-toggle${music ? "" : " off"}`}
          aria-pressed={music}
          aria-label="Story music"
          title={music ? "Music on (click to turn it off)" : "Music off (click to turn it on)"}
          onClick={() => onMusic(!music)}
        >
          🎵
        </button>
        <button
          type="button"
          className={`cut-toggle${sound ? "" : " off"}`}
          aria-pressed={sound}
          aria-label="Sound"
          title={sound ? "Sound on (click to mute)" : "Sound off (click to turn it on)"}
          onClick={() => onSound(!sound)}
        >
          {sound ? "🔊" : "🔇"}
        </button>
        <button type="button" className="cut-skip" onClick={onDone} title="Skip the rest of this scene">
          Skip scene ⏭
        </button>
      </div>
    </dialog>
  );
}
