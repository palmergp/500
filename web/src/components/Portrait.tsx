import { useId, type ReactNode } from "react";
import type { Look } from "../career/story";

/** A colour lightened (amount > 0) or darkened (< 0), for shading. */
function shade(hex: string, amount: number): string {
  const n = parseInt(hex.slice(1), 16);
  const mix = (c: number) => Math.round(amount < 0 ? c * (1 + amount) : c + (255 - c) * amount);
  const [r, g, b] = [(n >> 16) & 255, (n >> 8) & 255, n & 255].map(mix);
  return `#${((1 << 24) | (r << 16) | (g << 8) | b).toString(16).slice(1)}`;
}

/** A light colour (gray or blond hair), whose brows are drawn darker. */
function bright(hex: string): boolean {
  const n = parseInt(hex.slice(1), 16);
  return 0.3 * ((n >> 16) & 255) + 0.59 * ((n >> 8) & 255) + 0.11 * (n & 255) > 140;
}

const HEAD = (jaw: number) =>
  `M66 94 C66 66 80 54 100 54 C120 54 134 66 134 94 C134 112 130 124 ${110 + jaw} 134 C${104 + jaw} 142 ${102 + jaw / 2} 147 100 147 C${98 - jaw / 2} 147 ${96 - jaw} 142 ${90 - jaw} 134 C70 124 66 112 66 94 Z`;
const TORSO = "M16 240 C18 198 42 178 76 172 L124 172 C158 178 182 198 184 240 Z";
const TORSO_BIG = "M0 240 C2 192 32 166 70 160 L130 160 C168 166 198 192 200 240 Z";
const LAPELS = ["M86 174 L100 212 L90 222 L68 186 Z", "M114 174 L100 212 L110 222 L132 186 Z"];
const SHIRT_V = "M85 172 L100 212 L115 172 Z";

/** Hair (and hats) behind the head: what shows around and below it. */
function hairBack(look: Look): string | null {
  switch (look.hairStyle) {
    case "bun":
      return "M60 102 C54 62 78 44 102 46 C128 46 148 64 140 104 C136 92 130 84 120 80 L80 80 C72 86 64 94 60 102 Z";
    case "bob":
      return "M59 106 C54 60 80 46 102 46 C126 46 148 62 141 108 L143 142 C130 145 126 136 128 120 L72 120 C74 136 70 145 57 142 Z";
    case "long":
      return "M58 100 C54 56 82 44 100 44 C118 44 146 56 142 100 L150 206 C136 202 132 172 132 122 L68 122 C68 172 64 202 50 206 Z";
    case "perm":
      return "M56 112 C46 78 62 46 100 42 C138 46 154 78 144 112 C140 100 134 90 126 84 L74 84 C66 90 60 100 56 112 Z";
    default:
      return null;
  }
}

/** Hair over the forehead, and hats. */
function hairFront(look: Look, id: string): ReactNode {
  const hair = look.hair;
  const dark = shade(hair, -0.35);
  const light = shade(hair, 0.35);
  const strand = { fill: "none", stroke: dark, strokeWidth: 1.2, strokeLinecap: "round" as const, opacity: 0.7 };
  switch (look.hairStyle) {
    case "bun":
      return (
        <>
          <circle cx="104" cy="42" r="17" fill={hair} />
          <path d="M92 34 Q104 28 116 38 M94 46 Q104 52 116 44" {...strand} />
          <path d="M68 92 C68 64 86 54 104 56 C124 58 135 72 133 90 C126 78 112 72 98 74 C86 76 74 82 68 92 Z" fill={hair} />
          <path d="M74 84 Q90 66 112 64 M84 76 Q100 64 122 70 M100 60 Q118 62 130 80" {...strand} />
          {/* A loose strand by the cheek. */}
          <path d="M70 92 Q66 104 70 116" fill="none" stroke={hair} strokeWidth="2" strokeLinecap="round" />
        </>
      );
    case "quiff":
      return (
        <>
          <path d="M66 96 C60 70 70 52 88 46 C96 36 116 36 126 46 C136 54 139 70 134 94 C130 80 124 74 114 72 C102 70 92 72 82 76 C74 80 70 88 66 96 Z" fill={hair} />
          <path d="M86 50 Q104 40 122 50 M80 60 Q100 50 128 60 M76 70 Q96 62 116 66" {...strand} />
          <path d="M113 71 Q109 77 105 80" fill="none" stroke={hair} strokeWidth="3" strokeLinecap="round" />
          <path d="M92 44 Q108 38 120 46" fill="none" stroke={light} strokeWidth="2" opacity="0.5" />
        </>
      );
    case "bob":
      return (
        <>
          <path d="M67 88 C67 62 84 50 100 50 C116 50 133 62 133 88 Z" fill={hair} />
          <path d="M80 60 Q100 50 122 60" fill="none" stroke={light} strokeWidth="2.5" opacity="0.35" />
        </>
      );
    case "bald":
      return (
        <>
          <path d="M64 96 C62 108 64 120 70 126 L73 104 C71 100 68 98 64 96 Z" fill={hair} />
          <path d="M136 96 C138 108 136 120 130 126 L127 104 C129 100 132 98 136 96 Z" fill={hair} />
          <ellipse cx="88" cy="66" rx="12" ry="6" fill="#fff" opacity="0.18" transform="rotate(-20 88 66)" />
        </>
      );
    case "fedora":
      return (
        <>
          <path d="M66 98 C64 86 66 78 70 74 L74 98 Z M134 98 C136 86 134 78 130 74 L126 98 Z" fill={hair} />
          <path d="M68 76 C66 46 80 32 100 32 C120 32 134 46 132 76 Z" fill={look.coat} />
          <path d="M90 36 Q100 46 110 36" fill="none" stroke={shade(look.coat, -0.4)} strokeWidth="2" />
          <rect x="68" y="64" width="64" height="9" fill={look.trim ?? "#000"} />
          <path d="M38 80 C60 66 140 66 162 80 C150 88 50 88 38 80 Z" fill={shade(look.coat, -0.15)} />
          <path d="M42 80 C62 70 138 70 158 80" fill="none" stroke={shade(look.coat, 0.25)} strokeWidth="1.5" opacity="0.6" />
        </>
      );
    case "flatcap":
      return (
        <>
          <path d="M66 94 L68 84 L74 96 Z M134 94 L132 84 L126 96 Z" fill={hair} />
          <path d="M64 84 C62 56 94 46 114 48 C134 50 142 64 138 84 Z" fill={shade(look.coat, 0.12)} />
          <path d="M70 70 Q100 58 132 68" fill="none" stroke={shade(look.coat, -0.3)} strokeWidth="1.5" />
          <path d="M60 86 C80 74 132 72 152 88 C132 93 88 93 60 86 Z" fill={shade(look.coat, -0.1)} />
        </>
      );
    case "buzz":
      return (
        <>
          <path d="M68 86 C68 60 86 54 100 54 C116 54 132 60 132 86 C120 76 80 76 68 86 Z" fill={hair} opacity="0.92" />
          <path d="M68 86 C68 60 86 54 100 54 C116 54 132 60 132 86 C120 76 80 76 68 86 Z" fill={`url(#${id}-stipple)`} />
        </>
      );
    case "trucker":
      return (
        <>
          <path d="M64 98 C62 90 64 84 68 80 L72 100 Z M136 98 C138 90 136 84 132 80 L128 100 Z" fill={hair} />
          <path d="M66 82 C64 54 86 44 102 44 C120 44 138 56 134 82 Z" fill={look.coat} />
          <path d="M80 82 C80 58 92 48 102 48 C112 48 122 58 122 82 Z" fill="#efe9dc" />
          <text x="101" y="72" textAnchor="middle" fontSize="10" fontWeight="700" fill={look.coat} fontFamily="Georgia, serif">
            ♣
          </text>
          <path d="M70 84 C90 76 126 76 150 90 C128 94 94 92 70 88 Z" fill={shade(look.coat, -0.2)} />
        </>
      );
    case "perm":
      return (
        <>
          {[
            [70, 80], [80, 64], [96, 56], [112, 58], [126, 68], [134, 84], [66, 98], [138, 100], [88, 70], [106, 68], [120, 76],
          ].map(([cx, cy]) => (
            <g key={`${cx}-${cy}`}>
              <circle cx={cx} cy={cy} r="11" fill={hair} />
              <path d={`M${cx - 5} ${cy} q5 -6 10 0`} fill="none" stroke={dark} strokeWidth="1.2" opacity="0.6" />
            </g>
          ))}
        </>
      );
    case "crew":
      return (
        <>
          <path d="M68 86 L68 66 C70 56 84 52 100 52 C116 52 130 56 132 66 L132 86 C120 78 80 78 68 86 Z" fill={hair} />
          <path d="M72 64 L128 64" stroke={light} strokeWidth="1.5" opacity="0.4" />
        </>
      );
    case "pompadour":
      return (
        <>
          <path d="M66 94 C60 66 70 42 94 36 C114 30 136 40 137 62 C138 76 136 86 134 94 C128 80 116 74 100 74 C86 74 74 80 66 94 Z" fill={hair} />
          <path d="M84 42 Q110 32 130 50" fill="none" stroke="#fff" strokeWidth="2.5" opacity="0.3" />
          <path d="M76 60 Q100 46 132 64 M72 74 Q96 62 126 72" {...strand} />
        </>
      );
    case "long":
      return (
        <>
          <path d="M66 98 C64 66 82 52 100 54 C118 52 136 66 134 98 C128 78 112 64 100 62 C88 64 72 78 66 98 Z" fill={hair} />
          <path d="M100 56 L100 64" stroke={dark} strokeWidth="1.2" />
          <path d="M80 70 Q90 62 98 62 M120 70 Q110 62 102 62" {...strand} />
        </>
      );
  }
}

/** The garment: its body, collar, and what's worn under it. */
function Top({ look, id }: { look: Look; id: string }) {
  const coat = look.coat;
  const dark = shade(coat, -0.3);
  const shirt = look.shirt ?? "#efe9dc";
  const trim = look.trim ?? "#7a1e2c";
  const torso = look.big ? TORSO_BIG : TORSO;
  // The turtleneck's collar goes over the neck (Portrait draws it); here, its front.
  const turtleneck = (color: string) => <path d={SHIRT_V} fill={color} />;
  const lapels = (color: string) => LAPELS.map((d) => <path key={d} d={d} fill={color} />);
  const tie = (loose: boolean) =>
    loose ? (
      <>
        <path d="M95 182 L105 182 L104 190 L96 190 Z" fill={trim} />
        <path d="M96 190 L104 190 L108 232 L101 240 L95 232 Z" fill={trim} />
      </>
    ) : (
      <>
        <path d="M95 172 L105 172 L103 180 L97 180 Z" fill={trim} />
        <path d="M97 180 L103 180 L106 214 L100 222 L94 214 Z" fill={trim} />
      </>
    );
  switch (look.top) {
    case "cardigan":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d="M84 172 L100 177 L116 172 L108 240 L92 240 Z" fill={shirt} />
          <path d="M84 171 L99 177 L91 190 Z M116 171 L101 177 L109 190 Z" fill={shirt} stroke={shade(shirt, -0.25)} strokeWidth="1" />
          <path d="M84 172 L92 240 M116 172 L108 240" stroke={dark} strokeWidth="2.5" />
          {[198, 216, 234].map((y) => (
            <circle key={y} cx={116 - (y - 172) * 0.12 - 3} cy={y} r="2.2" fill={shade(coat, 0.35)} />
          ))}
          <path d="M40 220 L44 240 M58 200 L62 240 M142 200 L138 240 M160 220 L156 240" stroke={dark} strokeWidth="1" opacity="0.5" />
        </>
      );
    case "leather":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d="M86 174 Q100 192 114 174 L112 240 L88 240 Z" fill={shirt} />
          <path d="M86 174 L80 240 M114 174 L120 240" stroke={dark} strokeWidth="2" />
          <path d="M70 176 L88 170 L93 198 L76 208 Z M130 176 L112 170 L107 198 L124 208 Z" fill={dark} />
          <path d="M118 184 L121 240" stroke="#c9c9c9" strokeWidth="1.5" strokeDasharray="2 2" />
          <path d="M40 200 Q52 184 70 180" fill="none" stroke="#fff" strokeWidth="3" opacity="0.15" />
        </>
      );
    case "trench":
      return (
        <>
          <path d={torso} fill={coat} />
          {turtleneck(shirt)}
          {lapels(dark)}
          <circle cx="78" cy="214" r="2.5" fill={dark} />
          <circle cx="122" cy="214" r="2.5" fill={dark} />
        </>
      );
    case "tweed":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d={torso} fill={`url(#${id}-tweed)`} />
          <path d={SHIRT_V} fill={shirt} />
          {lapels(dark)}
          <path d="M100 178 L87 171 L87 186 Z M100 178 L113 171 L113 186 Z" fill={trim} />
          <rect x="97" y="174" width="6" height="8" rx="1.5" fill={shade(trim, -0.25)} />
        </>
      );
    case "pinstripe":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d={torso} fill={`url(#${id}-stripes)`} />
          <path d={SHIRT_V} fill={shirt} />
          {tie(false)}
          {lapels(shade(coat, -0.2))}
          <path d="M132 198 L144 195 L142 204 Z" fill={trim} />
          <path d="M134 199 L139 192 L142 199 Z" fill={shade(trim, 0.2)} />
        </>
      );
    case "peacoat":
      return (
        <>
          <path d={torso} fill={coat} />
          {turtleneck(shirt)}
          <path d="M84 168 L100 214 L88 226 L58 180 Z M116 168 L100 214 L112 226 L142 180 Z" fill={dark} />
          {[200, 222].flatMap((y) => [80, 120].map((x) => <circle key={`${x}-${y}`} cx={x + (x < 100 ? -6 : 6)} cy={y + 10} r="3" fill={shade(coat, 0.3)} />))}
        </>
      );
    case "track":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d="M83 166 L117 166 L119 182 L81 182 Z" fill={shade(coat, -0.12)} />
          <path d="M100 166 L100 240" stroke="#cfcfcf" strokeWidth="2" />
          <path d="M78 176 C58 182 40 200 30 240 M84 178 C66 186 50 204 42 240" fill="none" stroke={trim} strokeWidth="3" />
          <path d="M122 176 C142 182 160 200 170 240 M116 178 C134 186 150 204 158 240" fill="none" stroke={trim} strokeWidth="3" />
        </>
      );
    case "flannel":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d={torso} fill={`url(#${id}-plaid)`} />
          <path d="M88 174 Q100 186 112 174 Z" fill={shirt} />
          <path d="M86 171 L100 182 L89 194 Z M114 171 L100 182 L111 194 Z" fill={dark} />
          <path d="M100 184 L100 240" stroke={dark} strokeWidth="1.5" />
          {[200, 220].map((y) => (
            <circle key={y} cx="103" cy={y} r="1.8" fill="#e8dcc8" />
          ))}
        </>
      );
    case "trenchTie":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d="M84 172 L100 214 L116 172 Z" fill={shirt} />
          <path d="M84 171 L98 182 L88 194 Z M116 171 L102 182 L112 194 Z" fill={shirt} stroke={shade(shirt, -0.3)} strokeWidth="1" />
          {tie(true)}
          <path d="M84 172 L100 216 L86 232 L58 184 Z M116 172 L100 216 L114 232 L142 184 Z" fill={dark} />
          <path d="M64 182 L52 166 L82 168 Z M136 182 L148 166 L118 168 Z" fill={shade(coat, 0.1)} />
        </>
      );
    case "suitOpen":
      return (
        <>
          <path d={torso} fill={coat} />
          <path d={SHIRT_V} fill={shirt} />
          <path d="M85 171 L97 180 L86 190 Z M115 171 L103 180 L114 190 Z" fill={shade(shirt, 0.15)} />
          <path d="M90 176 Q100 196 110 176" fill="none" stroke="#e8c15a" strokeWidth="1.8" />
          <circle cx="100" cy="194" r="2.6" fill="#e8c15a" />
          {lapels(shade(coat, -0.25))}
        </>
      );
    case "turtleneck":
      return (
        <>
          <path d={torso} fill={coat} />
          {turtleneck(shirt)}
          {lapels(shade(coat, 0.12))}
          <path d="M100 178 L100 198" stroke="#c9ccd2" strokeWidth="1" />
          <path d="M100 196 L104 202 L100 208 L96 202 Z" fill="#dfe3ea" />
        </>
      );
  }
}

function Eye({ cx, look }: { cx: number; look: Look }) {
  const id = useId();
  const y = 100;
  const left = cx < 100;
  const outer = cx + (left ? -8 : 8);
  const inner = cx + (left ? 7 : -7);
  const kind = look.eyes;
  const top = kind === "wide" ? 7.5 : kind === "narrow" || kind === "stern" ? 3.4 : kind === "sly" ? 4.4 : 5.6;
  const bottom = kind === "wide" ? 5 : 3.2;
  const tilt = kind === "sly" || kind === "narrow" ? -1.6 : 0;
  const lid = `M${inner} ${y} Q${cx} ${y - top * 1.7} ${outer} ${y + tilt}`;
  const shape = `${lid} Q${cx} ${y + bottom * 1.5} ${inner} ${y} Z`;
  const iris = look.eyeColor ?? "#2b1d14";
  return (
    <g>
      <clipPath id={id}>
        <path d={shape} />
      </clipPath>
      <path d={shape} fill="#efe7da" />
      <g clipPath={`url(#${id})`}>
        <circle cx={cx + (left ? 1 : -1)} cy={y - 0.5} r={kind === "wide" ? 3 : 3.6} fill={iris} />
        <circle cx={cx + (left ? 1 : -1)} cy={y - 0.5} r="1.5" fill="#0b0706" />
        <circle cx={cx - 0.5} cy={y - 2} r="1" fill="#fff" />
        {/* The upper lid's shadow across the eye. */}
        <path d={`M${inner - 2} ${y - 9} L${outer + (left ? -2 : 2)} ${y - 9} L${outer} ${y - 1} L${inner} ${y - 1.5} Z`} fill="#000" opacity="0.15" />
      </g>
      <path d={lid} fill="none" stroke="#1a1210" strokeWidth={kind === "soft" ? 1.6 : 2.2} strokeLinecap="round" />
      {look.lips && kind !== "soft" && <path d={`M${outer} ${y + tilt} l${left ? -3 : 3} -2.5`} stroke="#1a1210" strokeWidth="1.6" strokeLinecap="round" />}
      {kind === "stern" && <path d={`M${inner} ${y - 4} Q${cx} ${y - 6} ${outer} ${y - 3}`} fill="none" stroke="#1a1210" strokeWidth="1.2" opacity="0.6" />}
      {(kind === "soft" || look.old) && <path d={`M${inner} ${y + 5} Q${cx} ${y + 8} ${outer} ${y + 4}`} fill="none" stroke="#000" strokeWidth="1" opacity="0.25" />}
    </g>
  );
}

function Brow({ cx, look }: { cx: number; look: Look }) {
  const left = cx < 100;
  const y = 89;
  const outer = cx + (left ? -10 : 10);
  const inner = cx + (left ? 9 : -9);
  const kind = look.brows === "raised" ? (left ? "arched-high" : "flat") : look.brows;
  const d = {
    arched: `M${outer} ${y + 2} Q${cx} ${y - 4} ${inner} ${y + 1}`,
    "arched-high": `M${outer} ${y} Q${cx} ${y - 8} ${inner} ${y - 1}`,
    flat: `M${outer} ${y + 1} Q${cx} ${y - 1} ${inner} ${y + 1}`,
    angry: `M${outer} ${y - 2} Q${cx} ${y - 1} ${inner} ${y + 4}`,
    worried: `M${outer} ${y + 3} Q${cx} ${y} ${inner} ${y - 3}`,
  }[kind];
  const color = look.browColor ?? (bright(look.hair) ? shade(look.hair, -0.3) : look.hair);
  return <path d={d} fill="none" stroke={color} strokeWidth="3.4" strokeLinecap="round" />;
}

function Mouth({ look }: { look: Look }) {
  const line = shade(look.skin, -0.55);
  const lips = look.lips ?? shade(look.skin, -0.18);
  const lipShapes = (
    <>
      <path d="M89 129 Q95 125.5 100 127.5 Q105 125.5 111 129 Q100 131 89 129 Z" fill={lips} />
      <path d="M89 129 Q100 137 111 129 Q100 132 89 129 Z" fill={shade(lips, 0.12)} />
    </>
  );
  switch (look.mouth) {
    case "smile":
      return (
        <>
          {look.lips && lipShapes}
          <path d="M88 128 Q100 136 112 127" fill="none" stroke={line} strokeWidth="2" strokeLinecap="round" />
          <path d="M112 127 l2 -2" stroke={line} strokeWidth="1.5" strokeLinecap="round" />
        </>
      );
    case "smirk":
      return (
        <>
          {look.lips && lipShapes}
          <path d="M89 131 Q101 134 111 126" fill="none" stroke={line} strokeWidth="2" strokeLinecap="round" />
          <path d="M111 126 l2.5 -1.5" stroke={line} strokeWidth="1.5" strokeLinecap="round" />
        </>
      );
    case "grin":
      return (
        <>
          <path d="M87 126 Q100 142 113 126 Q100 130 87 126 Z" fill="#2a0f0c" />
          <path d="M89 127 Q100 131 111 127 L110 131 Q100 134 90 131 Z" fill="#f4efe4" />
          {look.goldTooth && <rect x="103" y="128" width="4" height="4" fill="#e8c15a" />}
        </>
      );
    case "flat":
      return (
        <>
          {look.lips && lipShapes}
          <path d="M90 130 Q100 131 110 130" fill="none" stroke={line} strokeWidth="1.8" strokeLinecap="round" />
        </>
      );
    case "frown":
      return <path d="M89 133 Q100 127 111 133" fill="none" stroke={line} strokeWidth="2.2" strokeLinecap="round" />;
    case "nervous":
      return <path d="M89 130 Q94 127 99 131 Q104 135 111 129" fill="none" stroke={line} strokeWidth="2" strokeLinecap="round" />;
    case "cheer":
      return (
        <>
          <path d="M87 126 Q100 141 113 126 Q100 130 87 126 Z" fill="#3a1414" />
          <path d="M89 127 Q100 130.5 111 127 L110 130 Q100 132.5 90 130 Z" fill="#f4efe4" />
          <path d="M87 126 Q100 129 113 126" fill="none" stroke={look.lips ?? line} strokeWidth="2.4" strokeLinecap="round" />
        </>
      );
  }
}

function FacialHair({ look }: { look: Look }) {
  const color = look.beardColor ?? look.hair;
  switch (look.beard) {
    case "stubble":
      return <path d="M70 110 C72 134 88 147 100 147 C112 147 128 134 130 110 C124 122 114 122 100 124 C86 122 76 122 70 110 Z" fill={shade(look.skin, -0.6)} opacity="0.22" />;
    case "mustache":
      return <path d="M88 124 Q100 119 112 124 Q106 123 100 123.5 Q94 123 88 124 Z" fill={color} stroke={color} strokeWidth="1.5" />;
    case "trim":
      return (
        <>
          <path d="M69 108 C70 136 86 151 100 151 C114 151 130 136 131 108 C127 122 118 128 112 127 Q100 123 88 127 C82 128 73 122 69 108 Z" fill={color} />
          <path d="M87 125 Q100 117 113 125 Q106 128 100 126 Q94 128 87 125 Z" fill={color} />
          <path d="M80 136 Q90 142 100 142 M100 146 Q110 144 120 136" fill="none" stroke={shade(color, -0.2)} strokeWidth="1" opacity="0.5" />
        </>
      );
    case "full":
      return (
        <>
          <path d="M65 104 C64 144 82 164 100 164 C118 164 136 144 135 104 C129 120 120 127 112 126 Q100 121 88 126 C80 127 71 120 65 104 Z" fill={color} />
          <path d="M86 125 Q100 116 114 125 Q107 129 100 127 Q93 129 86 125 Z" fill={shade(color, -0.1)} />
          <path d="M76 138 Q82 150 90 156 M100 150 L100 162 M124 138 Q118 150 110 156" fill="none" stroke={shade(color, -0.25)} strokeWidth="1.2" opacity="0.6" />
        </>
      );
    default:
      return null;
  }
}

/**
 * A character, drawn in a noir comic style: flat colour with the far side of the face in shadow,
 * a rim of light in their colour, and their own face, hair and clothes.
 */
export function Portrait({ look, className }: { look: Look; className?: string }) {
  const id = useId().replace(/:/g, "");
  const head = HEAD(look.jaw ?? 11);
  const back = hairBack(look);
  const shadowSide = "M110 30 C128 56 140 100 126 160 L210 160 L210 30 Z";
  const skinDark = shade(look.skin, -0.25);
  const rim = (d: string) => <path d={d} fill="none" stroke={look.accent} strokeWidth="3" strokeOpacity="0.9" clipPath={`url(#${id}-rim)`} />;
  const torso = look.big ? TORSO_BIG : TORSO;
  return (
    <svg className={`portrait ${className ?? ""}`} viewBox="0 0 200 240" aria-hidden="true">
      <defs>
        <radialGradient id={`${id}-glow`} cx="50%" cy="42%" r="55%">
          <stop offset="0%" stopColor={look.accent} stopOpacity="0.4" />
          <stop offset="100%" stopColor={look.accent} stopOpacity="0" />
        </radialGradient>
        <clipPath id={`${id}-face`}>
          <path d={head} />
        </clipPath>
        <clipPath id={`${id}-rim`}>
          <rect x="124" y="0" width="90" height="240" />
        </clipPath>
        <pattern id={`${id}-stripes`} width="7" height="10" patternUnits="userSpaceOnUse">
          <path d="M3 0 L3 10" stroke="#fff" strokeWidth="0.8" opacity="0.22" />
        </pattern>
        <pattern id={`${id}-tweed`} width="6" height="6" patternUnits="userSpaceOnUse">
          <path d="M0 6 L3 0 M3 0 L6 6" stroke="#000" strokeWidth="0.7" opacity="0.25" fill="none" />
        </pattern>
        <pattern id={`${id}-plaid`} width="22" height="22" patternUnits="userSpaceOnUse">
          <path d="M0 6 H22 M0 9 H22 M6 0 V22 M9 0 V22" stroke="#1a0c08" strokeWidth="2" opacity="0.35" />
          <path d="M0 17 H22 M17 0 V22" stroke="#f5e6c8" strokeWidth="0.8" opacity="0.3" />
        </pattern>
        <pattern id={`${id}-stipple`} width="4" height="4" patternUnits="userSpaceOnUse">
          <circle cx="1" cy="1" r="0.6" fill="#000" opacity="0.35" />
        </pattern>
      </defs>
      <ellipse cx="100" cy="118" rx="100" ry="122" fill={`url(#${id}-glow)`} />

      {back && <path d={back} fill={look.hair} />}
      <Top look={look} id={id} />
      {rim(torso)}
      {/* The neck, under whatever collar the clothes have (drawn first, so the clothes sit over it where they cover it). */}
      <path d={look.big ? "M80 126 L79 164 L121 164 L120 126 Z" : "M85 126 L84 172 L116 172 L115 126 Z"} fill={look.skin} />
      <path d="M85 140 Q100 152 115 140 L115 126 L85 126 Z" fill="#000" opacity="0.28" />
      {(look.top === "trench" || look.top === "peacoat" || look.top === "turtleneck") && (
        <>
          <path d="M85 138 L85 178 L115 178 L115 138 Q100 145 85 138 Z" fill={look.shirt ?? "#151515"} />
          <path d="M86 150 Q100 156 114 150 M86 162 Q100 168 114 162" fill="none" stroke={shade(look.shirt ?? "#151515", 0.25)} strokeWidth="1" opacity="0.6" />
        </>
      )}
      {look.top === "trench" && <path d="M72 184 L79 148 L91 156 L92 192 Z M128 184 L121 148 L109 156 L108 192 Z" fill={shade(look.coat, 0.1)} stroke={shade(look.coat, -0.3)} strokeWidth="1" />}

      {/* Ears, then the head over them. */}
      <ellipse cx="66" cy="102" rx={look.bigEars ? 8 : 5} ry={look.bigEars ? 13 : 10} fill={look.skin} />
      <ellipse cx="134" cy="102" rx={look.bigEars ? 8 : 5} ry={look.bigEars ? 13 : 10} fill={skinDark} />
      <path d={head} fill={look.skin} />
      <g clipPath={`url(#${id}-face)`}>
        <path d={shadowSide} fill="#000" opacity="0.26" />
        {look.hairStyle === "fedora" && <rect x="50" y="74" width="100" height="22" fill="#000" opacity="0.35" />}
        {look.hairStyle === "flatcap" && <rect x="50" y="84" width="100" height="10" fill="#000" opacity="0.22" />}
        {look.freckles &&
          [[82, 112], [86, 115], [79, 116], [118, 112], [114, 115], [121, 116], [90, 113], [110, 113]].map(([x, y]) => (
            <circle key={`${x}-${y}`} cx={x} cy={y} r="0.9" fill={shade(look.skin, -0.35)} />
          ))}
      </g>
      {rim(head)}
      <path d={look.broken ? "M101 101 L98 106 L102 110 L95 116 Q99 119 104 117" : "M101 101 Q99 110 95 116 Q99 119 104 117"} fill="none" stroke={shade(look.skin, -0.45)} strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round" />
      <Eye cx={86} look={look} />
      <Eye cx={114} look={look} />
      <Brow cx={86} look={look} />
      <Brow cx={114} look={look} />
      {look.scar && <path d="M80 82 L91 98" stroke={shade(look.skin, 0.35)} strokeWidth="1.8" strokeLinecap="round" />}
      {look.old && (
        <path
          d="M80 75 Q100 71 120 75 M74 103 l-3 1.5 M126 103 l3 1.5 M89 119 Q85 126 87 132 M111 119 Q115 126 113 132"
          fill="none"
          stroke="#000"
          strokeWidth="1"
          opacity="0.22"
        />
      )}
      <Mouth look={look} />
      <FacialHair look={look} />
      {look.beard === "trim" && <Mouth look={{ ...look, mouth: "flat" }} />}
      {look.glasses === "reading" && (
        <g>
          <path d="M77 104 L95 104 Q95 112 86 112 Q77 112 77 104 Z M105 104 L123 104 Q123 112 114 112 Q105 112 105 104 Z" fill="#fff" fillOpacity="0.1" stroke="#d4af5a" strokeWidth="1.6" />
          <path d="M95 105 Q100 102 105 105" fill="none" stroke="#d4af5a" strokeWidth="1.4" />
          {look.chain && <path d="M77 105 C70 120 72 150 86 172 M123 105 C130 120 128 150 114 172" fill="none" stroke="#d4af5a" strokeWidth="1" strokeDasharray="1.5 1.5" />}
        </g>
      )}
      {look.glasses === "wire" && (
        <g fill="#fff" fillOpacity="0.08" stroke="#d8d2c4" strokeWidth="1.4">
          <circle cx="86" cy="100" r="9" />
          <circle cx="114" cy="100" r="9" />
          <path d="M95 99 Q100 96 105 99 M77 99 L67 97 M123 99 L133 97" fill="none" />
        </g>
      )}
      {hairFront(look, id)}
      {look.hairStyle === "fedora" && rim("M68 76 C66 46 80 32 100 32 C120 32 134 46 132 76 M38 80 C60 66 140 66 162 80")}
      {look.earring && <path d="M134 113 L134 118 M131 119 L134 124 L137 119 Z" stroke="#dfe3ea" strokeWidth="1" fill="#dfe3ea" />}
      {look.pearls && (
        <g fill="#f6f1e6" stroke="#cfc6b4" strokeWidth="0.5">
          {Array.from({ length: 11 }, (_, i) => {
            const a = Math.PI * (0.15 + (0.7 * i) / 10);
            return <circle key={i} cx={100 - Math.cos(a) * 17} cy={168 + Math.sin(a) * 12} r="2.2" />;
          })}
        </g>
      )}
      {look.brooch && (
        <g>
          <rect x="128" y="190" width="9" height="12" rx="1.5" fill="#f2ead6" stroke="#d4af5a" strokeWidth="1.2" transform="rotate(12 132 196)" />
          <circle cx="132.5" cy="196" r="1.8" fill="#b3172c" />
        </g>
      )}
    </svg>
  );
}
