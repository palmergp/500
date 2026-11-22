import { useEffect, useId, useMemo, useRef, useState } from "react";
import { characterName, fill } from "../career/career";
import { CHARACTERS, type Backdrop, type CharacterId, type Look, type Step } from "../career/story";

/** Characters per second while a line types out. */
const TYPE_SPEED = 60;

const reducedMotion = () => !!window.matchMedia?.("(prefers-reduced-motion: reduce)").matches;

/** A character as a noir silhouette: their outline (hair or hat), coat, and a rim of light in their
 * colour. */
export function Portrait({ look, className }: { look: Look; className?: string }) {
  const id = useId();
  const rim = { stroke: look.accent, strokeWidth: 2.5, strokeOpacity: 0.85 };
  const hair = { fill: "#0b0908", ...rim };
  const shoulders = look.big ? "M6 240 C12 176 48 154 100 152 C152 154 188 176 194 240 Z" : "M24 240 C30 184 60 162 100 160 C140 162 170 184 176 240 Z";
  return (
    <svg className={`portrait ${className ?? ""}`} viewBox="0 0 200 240" aria-hidden="true">
      <defs>
        <radialGradient id={`${id}-glow`} cx="50%" cy="45%" r="55%">
          <stop offset="0%" stopColor={look.accent} stopOpacity="0.38" />
          <stop offset="100%" stopColor={look.accent} stopOpacity="0" />
        </radialGradient>
      </defs>
      <ellipse cx="100" cy="120" rx="100" ry="120" fill={`url(#${id}-glow)`} />
      {look.style === "long" && <path d="M60 110 C54 60 84 50 100 50 C118 50 148 60 140 110 L148 178 C136 172 130 150 132 106 C122 84 80 84 68 106 C70 150 64 172 52 178 Z" {...hair} />}
      <path d={shoulders} fill={look.coat} {...rim} />
      <path d="M80 162 L100 198 L120 162" fill="none" stroke="rgba(255,255,255,0.22)" strokeWidth="3" />
      <rect x="88" y="130" width="24" height="34" fill="#17110f" />
      <ellipse cx="100" cy="100" rx="34" ry="42" fill="#1d1614" {...rim} />
      {look.beard && <path d="M68 108 C70 140 88 148 100 148 C112 148 130 140 132 108 C122 126 78 126 68 108 Z" fill="#d9d2c3" fillOpacity="0.75" />}
      {look.style === "slick" && <path d="M66 98 C62 62 86 52 104 54 C126 56 140 70 134 98 C128 80 112 72 96 74 C84 76 72 84 66 98 Z" {...hair} />}
      {look.style === "bob" && <path d="M62 106 C58 60 86 50 102 52 C124 52 144 66 140 108 L140 134 C132 130 130 112 130 100 C120 84 86 82 72 98 C70 112 70 128 62 134 Z" {...hair} />}
      {look.style === "bun" && (
        <>
          <circle cx="100" cy="46" r="15" {...hair} />
          <path d="M66 94 C66 64 86 56 100 56 C118 56 136 66 134 94 C124 78 78 78 66 94 Z" fill="#8f8a84" {...rim} />
        </>
      )}
      {look.style === "fedora" && (
        <>
          <path d="M70 72 C70 44 130 44 130 72 Z" {...hair} />
          <rect x="70" y="62" width="60" height="8" fill="#3a1015" />
          <ellipse cx="100" cy="74" rx="60" ry="10" {...hair} />
        </>
      )}
      {look.style === "cap" && (
        <>
          <path d="M64 86 C66 58 132 56 136 86 Z" {...hair} />
          <path d="M60 86 Q100 76 152 90 L150 97 Q100 84 62 92 Z" {...hair} />
        </>
      )}
      {look.style === "curly" &&
        [
          [70, 84],
          [80, 66],
          [98, 58],
          [116, 64],
          [130, 82],
          [66, 104],
          [134, 102],
        ].map(([cx, cy]) => <circle key={`${cx}-${cy}`} cx={cx} cy={cy} r="13" fill="#7a7470" {...rim} />)}
      {look.style === "crop" && <path d="M68 88 C68 62 88 56 100 56 C114 56 134 62 132 88 C120 78 80 78 68 88 Z" {...hair} />}
      {look.style === "long" && <path d="M66 100 C66 62 86 54 100 54 C116 54 136 62 134 100 C124 80 78 80 66 100 Z" {...hair} />}
      {look.style === "bald" && (
        <>
          <path d="M66 96 C64 108 66 116 70 120 L72 100 Z" fill="#d9d2c3" fillOpacity="0.7" />
          <path d="M134 96 C136 108 134 116 130 120 L128 100 Z" fill="#d9d2c3" fillOpacity="0.7" />
        </>
      )}
      {look.glasses && (
        <g fill="rgba(255,255,255,0.08)" stroke={look.accent} strokeWidth="2.5">
          <circle cx="86" cy="102" r="9" />
          <circle cx="114" cy="102" r="9" />
          <path d="M95 102 H105" />
        </g>
      )}
    </svg>
  );
}

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

/** The first step at or after `i` that's shown (a change of scene just sets it). */
function shownFrom(list: Step[], i: number): number {
  while (i < list.length && "bg" in list[i]) i++;
  return i;
}

/** The scene as of step `at`: the backdrop and place, and who's standing on the left and right
 * (a speaker keeps their side; a new one takes the side of whoever spoke less recently). */
function sceneAt(list: Step[], at: number) {
  let bg: Backdrop | null = null;
  let place = "";
  let slots: (CharacterId | null)[] = [null, null];
  let last: CharacterId | null = null;
  for (let i = 0; i <= at && i < list.length; i++) {
    const step = list[i];
    if ("bg" in step) {
      bg = step.bg;
      place = step.place;
      slots = [null, null];
      last = null;
    } else if ("say" in step && step.say !== "you") {
      const who = step.say;
      if (!slots.includes(who)) {
        const free = slots[0] === null ? 0 : slots[1] === null ? 1 : slots[0] === last ? 1 : 0;
        slots = free === 0 ? [who, slots[1]] : [slots[0], who];
      }
      last = who;
    }
  }
  return { bg, place, slots };
}

/**
 * A story scene, full screen: the place, the people in it, and their lines typed out one at a time
 * (a click or Next shows the whole line, then moves on). Some lines are your choice of what to say.
 */
export function Cutscene({ steps, name, label, onDone }: { steps: Step[]; name: string; label: string; onDone: () => void }) {
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
  const { bg, place, slots } = useMemo(() => sceneAt(list, at), [list, at]);
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
        <button type="button" className="cut-skip" onClick={onDone} title="Skip the rest of this scene">
          Skip scene ⏭
        </button>
      </div>
    </dialog>
  );
}
