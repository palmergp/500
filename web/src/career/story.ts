// Career mode's story, "The Kitty": who's in it, and each chapter's scenes and level. The game
// itself is plain 500; the story is told around it, in cutscenes before and after each level and a
// line from someone at the table after each hand. `{name}` in any text is what the player is called.
// The player is "you" throughout, never given pronouns.

import type { Ambience, Mood, Sfx } from "../storyAudio";

/** How a character is drawn (components/Portrait.tsx): face, hair, expression and clothes. */
export interface Look {
  skin: string;
  hair: string;
  hairStyle: "bun" | "quiff" | "bob" | "bald" | "fedora" | "flatcap" | "buzz" | "trucker" | "perm" | "crew" | "pompadour" | "long";
  eyes: "soft" | "sly" | "narrow" | "wide" | "stern";
  eyeColor?: string;
  brows: "arched" | "flat" | "angry" | "raised" | "worried";
  browColor?: string;
  mouth: "smile" | "smirk" | "grin" | "flat" | "frown" | "nervous" | "cheer";
  /** Lipstick. */
  lips?: string;
  beard?: "stubble" | "mustache" | "trim" | "full";
  beardColor?: string;
  /** Lines of age. */
  old?: boolean;
  glasses?: "reading" | "wire";
  /** A chain on the reading glasses. */
  chain?: boolean;
  scar?: boolean;
  /** A broken nose. */
  broken?: boolean;
  bigEars?: boolean;
  freckles?: boolean;
  earring?: boolean;
  pearls?: boolean;
  brooch?: boolean;
  goldTooth?: boolean;
  /** Chin width (about 8 to 18). */
  jaw?: number;
  top: "cardigan" | "leather" | "trench" | "tweed" | "pinstripe" | "peacoat" | "track" | "flannel" | "trenchTie" | "suitOpen" | "turtleneck";
  /** The garment (and a hat's) colour, what's under it, and a tie or stripe. */
  coat: string;
  shirt?: string;
  trim?: string;
  /** The rim of light around them. */
  accent: string;
  /** Broad shoulders. */
  big?: boolean;
}

export interface Character {
  name: string;
  /** Who they are, in a few words (the chapter list and tooltips). */
  role: string;
  /** The bot persona that plays their cards. */
  persona: string;
  look: Look;
}

export type CharacterId =
  | "danny" | "mae" | "roz" | "silas" | "vincent" | "bruno" | "sal" | "earl" | "dot" | "cole" | "lou" | "ingrid" | "astrid";

export const CHARACTERS: Record<CharacterId, Character> = {
  danny: {
    name: "Danny Reyes",
    role: "your partner since the sixth grade",
    persona: "gambler",
    look: { skin: "#c68a5e", hair: "#2a1a12", hairStyle: "quiff", eyes: "sly", brows: "arched", mouth: "smirk", beard: "stubble", jaw: 12, top: "leather", coat: "#5a3420", shirt: "#efe9dc", accent: "#f2a65a" },
  },
  mae: {
    name: "Mae Donnelly",
    role: "owns the diner; taught you to play",
    persona: "oracle",
    look: {
      skin: "#e8c2a0", hair: "#c9c4bd", hairStyle: "bun", eyes: "soft", eyeColor: "#4a6a7a", brows: "arched", mouth: "smile", lips: "#a8585e", old: true,
      glasses: "reading", chain: true, brooch: true, jaw: 9, top: "cardigan", coat: "#2f5f5a", shirt: "#f2ede2", accent: "#f2d38a",
    },
  },
  roz: {
    name: "Roz Vance",
    role: "Castellane's best player",
    persona: "oracle",
    look: { skin: "#efd3bf", hair: "#111214", hairStyle: "bob", eyes: "narrow", eyeColor: "#3b4a5a", brows: "flat", mouth: "flat", lips: "#a3122a", earring: true, jaw: 9, top: "trench", coat: "#3a3f47", shirt: "#141518", accent: "#9fd3ff" },
  },
  silas: {
    name: "Silas Grant",
    role: "the Professor, a legend who vanished",
    persona: "counter",
    look: {
      skin: "#8d5a3b", hair: "#e9e4da", hairStyle: "bald", eyes: "stern", brows: "flat", browColor: "#cfc8bb", mouth: "flat", beard: "trim", beardColor: "#e9e4da", old: true,
      glasses: "wire", scar: true, jaw: 11, top: "tweed", coat: "#6b5236", shirt: "#f2ede2", trim: "#6e1f2a", accent: "#ffcf8a",
    },
  },
  vincent: {
    name: "Vincent Castellane",
    role: "runs the Velvet Room",
    persona: "shark",
    look: { skin: "#d9a77f", hair: "#1a1410", hairStyle: "fedora", eyes: "sly", brows: "raised", mouth: "smirk", beard: "mustache", jaw: 12, top: "pinstripe", coat: "#1c2538", shirt: "#f4f1ea", trim: "#b3172c", accent: "#e0475b" },
  },
  bruno: {
    name: "Bruno",
    role: "Castellane muscle",
    persona: "steady",
    look: { skin: "#c99572", hair: "#3a2e26", hairStyle: "flatcap", eyes: "stern", brows: "angry", mouth: "frown", beard: "stubble", broken: true, jaw: 18, big: true, top: "peacoat", coat: "#3b3d40", shirt: "#151515", accent: "#c9c9c9" },
  },
  sal: {
    name: "Little Sal",
    role: "Bruno's shadow",
    persona: "rookie",
    look: { skin: "#e3b48f", hair: "#3b2a1c", hairStyle: "buzz", eyes: "wide", brows: "worried", mouth: "nervous", bigEars: true, freckles: true, jaw: 8, top: "track", coat: "#2f6b3a", trim: "#f2f2f2", accent: "#b6e08a" },
  },
  earl: {
    name: "Earl",
    role: "a Tuesday regular",
    persona: "rookie",
    look: { skin: "#e0ac88", hair: "#a9a39a", hairStyle: "trucker", eyes: "soft", brows: "flat", mouth: "smile", beard: "full", beardColor: "#b5afa6", old: true, jaw: 14, top: "flannel", coat: "#8a2a22", shirt: "#efe9dc", accent: "#ffb38a" },
  },
  dot: {
    name: "Dot",
    role: "a Tuesday regular",
    persona: "rookie",
    look: { skin: "#f0cfb3", hair: "#cfc3d6", hairStyle: "perm", eyes: "soft", brows: "arched", mouth: "cheer", lips: "#d46a8f", old: true, pearls: true, jaw: 9, top: "cardigan", coat: "#c45f8e", shirt: "#f6efe6", accent: "#ffb0d8" },
  },
  cole: {
    name: "Det. Harlan Cole",
    role: "a cop on the payroll",
    persona: "gambler",
    look: { skin: "#d8a383", hair: "#8a6a45", hairStyle: "crew", eyes: "narrow", brows: "angry", mouth: "frown", beard: "stubble", jaw: 16, top: "trenchTie", coat: "#a8895a", shirt: "#ece6d8", trim: "#2b2f3a", accent: "#e8d27a" },
  },
  lou: {
    name: "Lucky Lou",
    role: "says he's lucky",
    persona: "steady",
    look: { skin: "#b67a52", hair: "#0d0d0d", hairStyle: "pompadour", eyes: "sly", brows: "raised", mouth: "grin", goldTooth: true, jaw: 11, top: "suitOpen", coat: "#5b2f86", shirt: "#151515", accent: "#c8a2ff" },
  },
  ingrid: {
    name: "Ingrid Lindqvist",
    role: "a Moreau twin who counts everything",
    persona: "counter",
    look: { skin: "#f3dccb", hair: "#efe6c8", hairStyle: "long", eyes: "sly", eyeColor: "#5a7a9a", brows: "arched", mouth: "smirk", lips: "#8a3a4a", jaw: 8, top: "turtleneck", coat: "#1b1d21", shirt: "#0f1012", accent: "#f4f0d0" },
  },
  astrid: {
    name: "Astrid Lindqvist",
    role: "a Moreau twin who never takes a risk",
    persona: "rock",
    look: { skin: "#f3dccb", hair: "#efe6c8", hairStyle: "long", eyes: "stern", eyeColor: "#5a7a9a", brows: "flat", mouth: "flat", lips: "#7a3a46", jaw: 8, top: "turtleneck", coat: "#1b1d21", shirt: "#0f1012", accent: "#c0d8f0" },
  },
};

/** The scene behind the dialogue (styles.css `.bg-*`). */
export type Backdrop =
  | "diner" | "diner-dawn" | "street" | "poolhall" | "velvet" | "warehouse" | "car" | "boarding" | "lounge" | "pier" | "ballroom" | "penthouse";

/** A step of a cutscene. Speakers are characters, or "you". */
export type Step =
  | { bg: Backdrop; place: string }
  | { title: string; sub?: string }
  | { say: CharacterId | "you"; text: string }
  | { narrate: string }
  | { choose: { label: string; then: Step[] }[] }
  /** The score changes mood (each place has its own until then). */
  | { music: Mood }
  /** A sound, as the next line appears. */
  | { sfx: Sfx };

/** Each place's mood and sound, until a step says otherwise. */
export const PLACE_SOUND: Record<Backdrop, { mood: Mood; ambience: Ambience }> = {
  diner: { mood: "noir", ambience: "rain" },
  "diner-dawn": { mood: "dawn", ambience: "birds" },
  street: { mood: "noir", ambience: "rain" },
  poolhall: { mood: "tension", ambience: "pool" },
  velvet: { mood: "noir", ambience: "crowd" },
  warehouse: { mood: "tension", ambience: "hum" },
  car: { mood: "noir", ambience: "car" },
  boarding: { mood: "sad", ambience: "clock" },
  lounge: { mood: "noir", ambience: "crowd" },
  pier: { mood: "tension", ambience: "waves" },
  ballroom: { mood: "tension", ambience: "crowd" },
  penthouse: { mood: "tension", ambience: "city" },
};

const at = (bg: Backdrop, place: string): Step => ({ bg, place });
const card = (title: string, sub?: string): Step => ({ title, sub });
const n = (narrate: string): Step => ({ narrate });
const say = (who: CharacterId | "you", text: string): Step => ({ say: who, text });
const you = (text: string): Step => ({ say: "you", text });
const mus = (music: Mood): Step => ({ music });
const fx = (sfx: Sfx): Step => ({ sfx });
const choose = (...options: [string, ...Step[]][]): Step => ({ choose: options.map(([label, ...then]) => ({ label, then })) });

/** A line someone at the table says after a hand: by who declared it and whether it made. */
export interface Banter {
  /** Your side made its bid, or went down; the other side made theirs, or went down. */
  made: [CharacterId, string][];
  set: [CharacterId, string][];
  theyMade: [CharacterId, string][];
  theySet: [CharacterId, string][];
}

export type Level =
  | {
      kind: "game";
      partner: CharacterId;
      west: CharacterId;
      east: CharacterId;
      /** A short game of this many hands (highest score wins), else a game to 500. */
      hands?: number;
      banter: Banter;
    }
  | {
      kind: "tournament";
      /** What the tournament is called in the story. */
      title: string;
      /** Finish in this place or better. */
      place: number;
      /** The other 11 players: story characters, or extras [name, persona]. */
      cast: (CharacterId | [string, string])[];
    };

export interface Chapter {
  title: string;
  /** A line under the title in the chapter list. */
  tagline: string;
  intro: Step[];
  level: Level;
  win: Step[];
  lose: Step[];
}

/** What a level asks of you, in a sentence. */
export function goal(level: Level): string {
  if (level.kind === "tournament") return `Finish in the top ${level.place} of 12 (8 rounds of 4 hands, a new partner every round).`;
  return level.hands ? `Win the game: ${level.hands} hands, highest score wins.` : "Win the game to 500.";
}

export const STORY_TITLE = "The Kitty";
export const STORY_BLURB = "Port Calloway. A diner, a deck of cards, and a debt that isn't what it seems.";

export const CHAPTERS: Chapter[] = [
  {
    title: "Tuesday Night",
    tagline: "Every Tuesday, after closing, the cards come out.",
    intro: [
      card("Chapter One", "Tuesday Night"),
      at("diner", "Mae's Diner, Port Calloway · 11:48 p.m."),
      n("Rain off the river. The neon in the window says MAE'S, except the S has been dead since spring."),
      n("You've worked the grill here three years. On Tuesdays, after the last coffee, Mae pushes two tables together and the cards come out."),
      say("mae", "Chairs down, {name}. Earl's been shuffling for ten minutes and he still can't do it."),
      say("earl", "I'm warming the deck up. It's a technique."),
      say("dot", "It's a stalling technique. He owes me two dollars from last week."),
      fx("bell"),
      n("The bell over the door. Danny Reyes, soaked to the skin, grinning like he's already won."),
      say("danny", "Am I late? Don't answer that. Deal me in, partner."),
      choose(
        ["You're always late.", say("danny", "And you always wait for me. That's what partners do.")],
        ["Where've you been?", say("danny", "Business. The kind with a lot of zeroes and not much in front of them."), n("He laughs. You don't ask again. You should have.")],
      ),
      say("mae", "Same as always. Four hands, small stakes, and nobody throws a chair."),
      say("mae", "And {name}? Count the trumps. I didn't teach you for nothing."),
      n("In the corner booth, a man in a gray coat orders black coffee and doesn't drink it. He watches the cards."),
    ],
    level: {
      kind: "game",
      partner: "danny",
      west: "earl",
      east: "dot",
      hands: 4,
      banter: {
        made: [["danny", "That's my partner!"], ["earl", "Aw, come on."], ["mae", "Now that's counting."]],
        set: [["danny", "We'll get it back. We always get it back."], ["mae", "You weren't counting, were you?"]],
        theyMade: [["dot", "Pay attention, kids."], ["earl", "The technique!"]],
        theySet: [["danny", "Down they go!"], ["dot", "Earl, what was that?"]],
      },
    },
    win: [
      n("The last trick falls your way. Earl groans. Dot collects in quarters."),
      say("dot", "Two dollars, Earl. Pay the house."),
      say("mae", "Lock up when you're done, you two."),
      n("She takes her coat. At the door she stops beside the corner booth, just for a second, and says something to the man in gray. Then she's gone into the rain."),
      n("The man in gray stands. On his way out, he drops a card on your table."),
      fx("card"),
      mus("tension"),
      n("The Joker. Across its face, in fountain pen, a phone number and four words: FRIDAY. BRING YOUR PARTNER."),
      say("danny", "Huh."),
      n("Danny picks it up too fast. He isn't grinning any more."),
      you("Danny. Who was that?"),
      say("danny", "Nobody. A fan."),
      n("He folds the Joker into his jacket. You've known Danny Reyes since the sixth grade. You know when he's lying."),
      n("You let it go. That's your first mistake."),
    ],
    lose: [say("earl", "Ha! The technique works!"), say("mae", "Again. And this time, count.")],
  },
  {
    title: "Forty Thousand",
    tagline: "Danny's waiting outside, and he isn't smiling.",
    intro: [
      card("Chapter Two", "Forty Thousand"),
      at("street", "Front Street · Friday, 2:15 a.m."),
      n("Danny is waiting under the pawnshop awning. Split lip. One eye swelling shut."),
      you("What happened to you?"),
      say("danny", "I walked into a door. The door's name is Bruno."),
      say("danny", "Okay. Okay. Here it is. I owe some people money."),
      you("How much?"),
      say("danny", "Forty."),
      you("Forty dollars?"),
      say("danny", "…Thousand. Forty thousand. To the Castellanes."),
      n("Everybody in Port Calloway knows the name. Nobody says it above a whisper."),
      say("danny", "But listen. I made a deal. Their two best players against us, one game. We win, the debt's gone."),
      you("And if we lose?"),
      say("danny", "Then I'd really appreciate it if we didn't lose."),
      choose(
        ["You bet on me without asking?", say("danny", "I bet on us. I always bet on us.")],
        ["Where are they?", say("danny", "The Anchor. Back room. They're already shuffling.")],
      ),
      at("poolhall", "The Anchor Pool Hall · the back room"),
      n("One bulb. One table. Smoke you could cut with a butter knife."),
      say("bruno", "So this is the cook. Sit down. Don't put your elbows on the felt."),
      say("sal", "Bruno hates elbows."),
      say("bruno", "Game's to 500. Whoever wins walks out. Whoever loses…"),
      n("He doesn't finish. He doesn't need to."),
    ],
    level: {
      kind: "game",
      partner: "danny",
      west: "bruno",
      east: "sal",
      banter: {
        made: [["danny", "Who's walking into doors now, Bruno?"], ["sal", "Bruno, they're beating us."]],
        set: [["bruno", "Tick tock, Danny."], ["danny", "Breathe. Just breathe."]],
        theyMade: [["bruno", "That's how it's done, cook."], ["sal", "Did you see that? Did you see me?"]],
        theySet: [["sal", "Bruno, that's bad, right?"], ["danny", "Ha! Elbows on the table, Bruno!"]],
      },
    },
    win: [
      n("Sal stares at the score pad like it insulted his mother. Bruno just nods, slowly."),
      say("bruno", "Huh. The cook can play."),
      fx("phone"),
      n("A phone rings behind the bar. Bruno answers, listens, says \"Yes, Mr. Castellane\" three times and hangs up."),
      say("bruno", "Debt's on hold. Not gone. On hold."),
      say("danny", "That wasn't the deal!"),
      say("bruno", "Deal changed. Mr. Castellane wants to meet your partner. Tomorrow night, the Velvet Room. Wear something nice, cook."),
      mus("noir"),
      fx("lighter"),
      n("Outside, the rain has stopped. Danny lights a cigarette with shaking hands."),
      say("danny", "I'm sorry, {name}. I swear I'll make this right."),
      n("You believe him. You've always believed him."),
    ],
    lose: [
      say("bruno", "Shame. Danny, you and me are going for a walk."),
      say("danny", "{name}… one more. Ask him for one more game."),
      say("bruno", "Fine. One more. I hate a walk in the rain."),
    ],
  },
  {
    title: "The Velvet Room",
    tagline: "Vincent Castellane makes an offer.",
    intro: [
      card("Chapter Three", "The Velvet Room"),
      at("velvet", "The Velvet Room, Harbor Street · Saturday, midnight"),
      n("Red velvet walls. A jazz trio that only plays in minor keys. Every table is a card table."),
      n("At the back, beneath a painting of a Joker in a crown, sits Vincent Castellane."),
      fx("glass"),
      say("vincent", "So you're the cook who beat my Bruno. Sit. Have a drink. Don't have a drink. I'm not your mother."),
      say("vincent", "Do you know what this city runs on, {name}? Not money. Not guns. Favors. And the Kitty."),
      you("The kitty? The three cards in the middle?"),
      say("vincent", "The three cards nobody's seen. The extra chance. It's also what we call our little tournament."),
      say("vincent", "Once a year the families sit down and play 500. Whoever wins the Kitty sets the terms for the year: the docks, the boats, the streets. We settle it with cards so we don't settle it with funerals."),
      say("vincent", "My player retired. Unexpectedly."),
      n("He says \"retired\" the way other men say \"drowned.\""),
      say("vincent", "Play for me. Win me the Kitty, and Danny's debt goes away. All of it."),
      choose(
        ["And if I say no?", say("vincent", "Then Danny's debt doubles, and Danny learns to swim.")],
        ["Why me?", say("vincent", "Because nobody knows your face. And because someone I trust tells me you count trumps the way a bank teller counts twenties."), n("Someone he trusts. You file that away.")],
      ),
      say("vincent", "But first, a little test. My friends here will play you and Danny."),
      n("Two men pull up chairs. One wears a cheap trench coat, with a detective's badge on his belt."),
      say("cole", "Detective Harlan Cole. Don't look so surprised. Cops play cards too."),
      say("lou", "Lucky Lou. I'm lucky. It's in the name."),
      say("vincent", "Win, and you're with me. Lose, and you were never here."),
    ],
    level: {
      kind: "game",
      partner: "danny",
      west: "cole",
      east: "lou",
      banter: {
        made: [["vincent", "Elegant."], ["cole", "Lucky."], ["danny", "Told you, Vince. Told you!"]],
        set: [["lou", "Now who's lucky?"], ["vincent", "Tsk."]],
        theyMade: [["cole", "A badge beats a spatula."], ["lou", "Lucky Lou!"]],
        theySet: [["lou", "My luck… where'd it go?"], ["vincent", "Harlan. Really."]],
      },
    },
    win: [
      fx("applause"),
      n("Vincent applauds, slowly."),
      say("vincent", "Beautiful. Did you see that, Harlan? Like a pianist."),
      say("cole", "I saw it. I don't like it."),
      say("vincent", "You don't like anything. Welcome to the family, {name}."),
      say("vincent", "Your first job: the qualifier. Next week, the warehouse on Pier Nine. Twelve players, eight rounds. Finish in the top four and you're in the Kitty."),
      say("danny", "And me?"),
      say("vincent", "Danny. Danny, Danny. In the qualifier everybody plays with everybody. You'll sit this one out."),
      n("Danny smiles. Under the table, his fist is clenched so hard his knuckles are white."),
      n("On the way out you glance back. Vincent is on the phone. You catch three words before the door swings shut: \"You were right.\""),
    ],
    lose: [say("vincent", "Disappointing. Harlan, show them out. The long way."), say("cole", "Hold on, Vince. Let's see them do it again. I want my money's worth.")],
  },
  {
    title: "Pier Nine",
    tagline: "The qualifier: twelve players, top four go through.",
    intro: [
      card("Chapter Four", "Pier Nine"),
      at("warehouse", "Pier Nine, Warehouse 4 · the qualifier"),
      n("Three card tables under hanging lamps. Twelve players. Guards at every door, and not the kind that check tickets."),
      n("Everybody plays with everybody: a new partner every round, eight rounds of four hands. The top four go to the Kitty."),
      say("cole", "Look who it is. The short-order genius."),
      you("Detective. Shouldn't you be arresting someone?"),
      say("cole", "Night off."),
      n("Across the room, a woman in a charcoal coat sits alone, writing in a small black notebook. She hasn't looked up once."),
      n("Through the loading door you'd swear you see a car you know: a green Buick with one headlight out. Like Mae's."),
      n("When you look again, it's gone."),
      fx("buzz"),
      n("Your phone buzzes. Danny: \"Knock em dead partner. Top 4. I'm counting on you. Literally.\""),
    ],
    level: {
      kind: "tournament",
      title: "The Pier Nine qualifier",
      place: 4,
      cast: ["cole", "lou", "roz", "bruno", "sal", ["Widow Marsh", "rock"], ["Father Quill", "counter"], ["Two-Bit Kenny", "gambler"], ["Hollis Crane", "shark"], ["Nell Sparrow", "natural"], ["Ox Brennan", "rookie"]],
    },
    win: [
      n("The final standings go up on a chalkboard. Your name is in the top four."),
      n("So is hers: the woman in the charcoal coat. Roz Vance."),
      say("vincent", "There's my investment! For the Kitty you'll have a permanent partner. The best I've got."),
      say("roz", "Vance. Roz, if you have to."),
      say("roz", "You hesitate on the third trick. Every time. Stop it."),
      you("Nice to meet you too."),
      say("vincent", "Roz has played for me three years. She has never lost a Kitty game."),
      say("roz", "I've never played a Kitty game."),
      say("vincent", "Exactly. Undefeated."),
      at("street", "The parking lot · 3 a.m."),
      n("Danny is waiting by your car."),
      say("danny", "Roz Vance? He replaced me with Roz Vance?"),
      say("danny", "Watch her, {name}. Nobody knows where she came from. Nobody."),
      n("He says it like a warning. Later, you'll wonder who the warning was really for."),
    ],
    lose: [
      n("The chalkboard goes up. Your name isn't in the top four."),
      say("vincent", "I don't pay for fifth place, {name}."),
      say("vincent", "Lucky for you, the families owe me a favor. There's another qualifier. Don't make me spend another favor."),
    ],
  },
  {
    title: "Cold Hands",
    tagline: "A new partner who never smiles.",
    intro: [
      card("Chapter Five", "Cold Hands"),
      at("car", "Roz's car · the coast road"),
      n("Roz drives the way she plays: precisely, silently, slightly over the limit."),
      say("roz", "Tonight we play the Lindqvist twins. They work for the Moreau family. The Moreaus want the docks."),
      say("roz", "Ingrid counts every card. Astrid never takes a risk. Together they've never gone down twice in a row."),
      you("Do you ever talk about anything but cards?"),
      say("roz", "No."),
      n("A long pause. Rain on the windshield."),
      say("roz", "Your friend Danny. How long have you known him?"),
      you("Since we were eleven. Why?"),
      say("roz", "No reason."),
      n("She has never once said anything for no reason."),
      at("lounge", "The Blue Lantern · Moreau territory"),
      n("The twins are identical, except that Ingrid smiles and Astrid doesn't."),
      say("ingrid", "Castellane's new pet. And Vance. We heard you were good."),
      say("astrid", "We heard wrong, probably."),
      say("roz", "Deal."),
    ],
    level: {
      kind: "game",
      partner: "roz",
      west: "ingrid",
      east: "astrid",
      banter: {
        made: [["roz", "Adequate."], ["ingrid", "A lucky kitty."]],
        set: [["roz", "Next hand. Forget this one."], ["astrid", "Hm."]],
        theyMade: [["ingrid", "Smile, Castellane's pet."], ["astrid", "As expected."]],
        theySet: [["roz", "Good. They're out of trumps; I counted."], ["ingrid", "…That was Astrid's fault."]],
      },
    },
    win: [
      say("ingrid", "…Well."),
      say("astrid", "I told you we heard wrong. About us."),
      at("car", "Roz's car · the drive back"),
      n("Roz says nothing for eleven miles. Then:"),
      say("roz", "Third trick. You didn't hesitate. Good."),
      n("From her, it's a parade."),
      n("When she drops you off, her notebook slides off the seat and falls open."),
      n("It isn't cards. It's names, dates, dollar amounts. Castellane. Moreau. And on the last page, circled twice: THE DEALER — WHO?"),
      choose(
        ["Hand it back. Say nothing.", n("She looks at you for a long moment."), say("roz", "Thank you.")],
        ["What is this?", say("roz", "Insurance."), n("She takes it back."), say("roz", "Goodnight, {name}.")],
      ),
    ],
    lose: [say("astrid", "Expected."), say("roz", "Again. And this time, trust my bids.")],
  },
  {
    title: "The Professor",
    tagline: "An old legend with an older grudge.",
    intro: [
      card("Chapter Six", "The Professor"),
      at("boarding", "St. Brendan's Boarding House · Sunday afternoon"),
      say("roz", "Before the Kitty, there's someone you need to meet."),
      n("Room 14. Books stacked to the ceiling. A chessboard with no pieces. A deck of cards worn soft as cloth."),
      n("The old man at the window doesn't turn around."),
      say("silas", "Vance. You brought me a cook."),
      say("roz", "I brought you the best natural player I've seen in ten years."),
      say("silas", "Everyone's a natural until the stakes are real."),
      n("He turns. Thin, sharp-eyed, a scar through one eyebrow."),
      say("silas", "Silas Grant. People used to call me the Professor. Mostly they stopped calling."),
      say("roz", "Silas won the Kitty four years running. Then he went to prison for fifteen."),
      say("silas", "For a murder I didn't commit, at a table I never left. We built the Kitty together, my partner and I. Then the Kitty only needed one of us."),
      you("Who was your partner?"),
      say("silas", "A name won't help you. Watching will. My partner smiled every time they lost a trick. Watch for the one who smiles when they lose."),
      fx("knock"),
      mus("noir"),
      n("A knock. Danny, holding a bag of oranges, looking lost."),
      say("danny", "Roz said to meet here? Hi. Danny. I brought fruit. Is that weird?"),
      say("silas", "Excellent. Now we have four."),
      n("He sweeps the books off the table."),
      say("silas", "{name} with me. Vance and the fruit man against us. Let's see what you're made of."),
    ],
    level: {
      kind: "game",
      partner: "silas",
      west: "roz",
      east: "danny",
      banter: {
        made: [["silas", "Ha! Like that!"], ["danny", "Whoa, partner. Easy on your old pal."]],
        set: [["silas", "Learn from it. Then forget it."], ["roz", "Third trick."]],
        theyMade: [["roz", "Pay attention."], ["danny", "Sorry! Sorry. Not sorry."]],
        theySet: [["silas", "The fruit man overbids."], ["roz", "Danny."]],
      },
    },
    win: [
      say("silas", "Ha! You see, Vance? Not a natural. A student. Students are better."),
      say("silas", "I'll teach you what I can before the Kitty. It isn't much time. It'll have to do."),
      n("As you leave, Danny lingers in the doorway, studying the old man."),
      say("danny", "Fifteen years. Did they ever find out who really did it?"),
      say("silas", "No."),
      say("silas", "But I know."),
      n("Danny laughs a little too loudly and says goodnight."),
      n("Later, you'll remember that laugh."),
    ],
    lose: [say("silas", "No. No. You're playing your cards. Play the table."), say("silas", "Again.")],
  },
  {
    title: "The Raid",
    tagline: "Three nights before the Kitty, the doors come down.",
    intro: [
      card("Chapter Seven", "The Raid"),
      at("velvet", "The Velvet Room · three nights before the Kitty"),
      n("A private game for show: Vincent's lieutenants against you and Danny. Silas sits in the corner with a club soda, watching. Roz is nowhere to be seen."),
      say("vincent", "Four hands, my friends. Short and sweet. I have dinner reservations."),
      n("Something's wrong. The jazz trio is packing up early. The bartender keeps checking the door."),
      say("silas", "{name}. The back exit. Remember where it is."),
      say("danny", "Relax, old man. Nobody raids the Velvet Room. Vincent owns half the precinct."),
      n("Harlan Cole isn't here tonight. Harlan Cole is always here."),
      say("vincent", "Deal."),
    ],
    level: {
      kind: "game",
      partner: "danny",
      west: "bruno",
      east: "lou",
      hands: 4,
      banter: {
        made: [["danny", "Ha! Pay up, Bruno!"], ["silas", "Good. Now keep your eyes on the door."]],
        set: [["bruno", "Not tonight, cook."], ["vincent", "My reservation, {name}."]],
        theyMade: [["lou", "Lucky Lou strikes again."], ["bruno", "Elbows."]],
        theySet: [["danny", "Down they go!"], ["silas", "Count, {name}. Always count."]],
      },
    },
    win: [
      mus("silence"),
      n("The last card hits the felt, and the front doors burst inward."),
      fx("crash"),
      mus("tension"),
      n("\"POLICE! EVERYBODY DOWN!\""),
      fx("sirens"),
      n("Chaos. Chips everywhere. The bass player dives behind the bar."),
      say("danny", "This way! Come on!"),
      fx("footsteps"),
      n("Danny grabs your arm and pulls you, not toward the back exit, but through the kitchen and a door you never knew existed."),
      n("He knew exactly where it was."),
      at("street", "The alley behind Harbor Street"),
      you("Where's Silas?"),
      n("Through the gap in the door: two officers walking the old man out in handcuffs. He doesn't struggle. He looks straight at the alley. Straight at Danny."),
      say("danny", "Somebody tipped them off. Somebody's a rat."),
      fx("buzz"),
      n("Your phone buzzes. An unknown number, one line: \"Don't trust anyone. Not even me. —R\""),
    ],
    lose: [say("vincent", "Pathetic. Again, before I lose my appetite.")],
  },
  {
    title: "The Kitty",
    tagline: "Every family in the city. Top three reach the final table.",
    intro: [
      card("Chapter Eight", "The Kitty"),
      at("pier", "Pier Nine · before dawn"),
      fx("foghorn"),
      n("Roz is waiting at the end of the pier, collar up, back to the water."),
      say("roz", "I don't have long. Listen carefully."),
      say("roz", "My name is Rosalind Vance. I'm a federal agent. I've been inside the Castellane family for three years."),
      choose(
        ["You're a cop?", say("roz", "Federal. It's worse. The paperwork alone.")],
        ["I knew it.", say("roz", "No, you didn't. But it's sweet of you to say so.")],
      ),
      say("roz", "Vincent isn't the top. He's a front. Someone above him has run the Kitty for twenty years. They call them the Dealer. Nobody's seen the Dealer's face."),
      say("roz", "The Dealer always watches the final in person. Reach the final table, and they'll be in the room. That's when we move."),
      you("And Silas?"),
      say("roz", "The raid wasn't mine. Someone wanted Silas off the board before the Kitty. Someone who knew he'd recognize the Dealer."),
      say("roz", "Someone close to you, {name}."),
      you("If you mean Danny, you're wrong."),
      say("roz", "I hope so. I really do. Now I have to disappear for a while. If anyone asks, I was arrested in the raid."),
      at("ballroom", "The Calloway Grand, the ballroom · the Kitty, day one"),
      n("Chandeliers. Every family in the city. Twelve players, eight rounds, a new partner every round. Only the top three reach the final table."),
      say("danny", "There you are! After the raid I couldn't find you anywhere."),
      say("danny", "Top three, partner. Then it's you and me at the final table. Like Tuesday nights."),
      n("He squeezes your shoulder. His hand is cold."),
    ],
    level: {
      kind: "tournament",
      title: "The Kitty",
      place: 3,
      cast: ["danny", "vincent", "ingrid", "astrid", "cole", ["Hollis Crane", "shark"], ["Nell Sparrow", "natural"], ["Father Quill", "counter"], ["Widow Marsh", "rock"], ["Duke Ferro", "oracle"], ["Madame Okoro", "natural"]],
    },
    win: [
      n("Your name climbs the board, past the Moreaus, past Hollis Crane, into the top three."),
      say("vincent", "Ha! Did I not say? Did I not say!"),
      n("The final is tomorrow night, in the penthouse."),
      mus("noir"),
      fx("buzz"),
      n("That night you can't sleep. At 3 a.m. your phone buzzes. Danny."),
      n("\"Meet me at the pier. Need to tell you something. Just you.\""),
    ],
    lose: [say("vincent", "Fourth is the first loser, {name}."), say("vincent", "The families will allow a replay. Once. Don't embarrass me twice.")],
  },
  {
    title: "Thirty Pieces of Silver",
    tagline: "Danny wants to talk. Just the two of you.",
    intro: [
      card("Chapter Nine", "Thirty Pieces of Silver"),
      at("pier", "Pier Nine · 3:40 a.m."),
      mus("sad"),
      fx("foghorn"),
      n("Fog. A foghorn somewhere past the breakwater. Danny sits on a crate, staring at the water."),
      say("danny", "Remember sixth grade? Mr. Pruitt caught me cheating at cards and you said the cards were yours. A week of detention, for me."),
      you("You said you'd pay me back."),
      say("danny", "Yeah. I'm about to."),
      mus("tension"),
      say("danny", "There's no debt, {name}. There never was."),
      n("The fog seems to get colder."),
      say("danny", "They came to me a year ago. They needed a player nobody knew. Clean. Good. Somebody they could own. I said I knew somebody."),
      say("danny", "The beating, the forty thousand, Bruno, all of it. A show, to get you to the Velvet Room."),
      you("The raid."),
      say("danny", "Silas recognized someone. He was going to talk. So I made a call."),
      choose(
        ["Why, Danny?", say("danny", "Because they pay better than the grill. And they don't lose, {name}. They never lose.")],
        ["Who are \"they\"?", say("danny", "You'll meet them tomorrow night. Everybody does, eventually.")],
      ),
      say("danny", "Here's the offer. Tomorrow, you throw the final. A few bad bids. Nobody will know. Your cut would buy Mae's diner ten times over."),
      you("And if I don't?"),
      say("danny", "Then you have to win your seat first. Vincent's called a challenge: the seat goes to whoever wins tonight. Me and Vincent, against you and whoever you can find at four in the morning."),
      say("danny", "I'm sorry, partner. I really am."),
      mus("sad"),
      fx("footsteps"),
      n("He walks off into the fog. You stand there a long time."),
      at("diner", "Mae's Diner · 4:30 a.m."),
      fx("bell"),
      n("The only light on in Port Calloway is Mae's. She's behind the counter with two cups of coffee already poured, as if she knew."),
      say("mae", "You look like somebody died."),
      you("Danny."),
      say("mae", "Ah."),
      n("She doesn't ask. She just takes off her apron."),
      say("mae", "Who do you need me to beat?"),
      you("Vincent Castellane. And Danny."),
      say("mae", "Well. I always did think that boy overbid."),
      say("mae", "Get your coat. I taught you everything you know, kid. Let's see if I taught you enough."),
    ],
    level: {
      kind: "game",
      partner: "mae",
      west: "vincent",
      east: "danny",
      banter: {
        made: [["mae", "That's my cook."], ["vincent", "Lucky."]],
        set: [["danny", "I know how you bid, {name}. I always have."], ["mae", "Shake it off."]],
        theyMade: [["vincent", "You see? Loyalty is expensive. Talent is cheap."], ["danny", "Sorry, partner."]],
        theySet: [["mae", "That boy always did overbid."], ["vincent", "Danny!"]],
      },
    },
    win: [
      say("vincent", "Impossible."),
      say("danny", "{name}, wait—"),
      n("You don't wait."),
      n("Mae gathers the cards, squares the deck and tucks it into her purse. Her hands are perfectly steady."),
      say("mae", "Go get some sleep. Big night tomorrow."),
      n("On her way out she pauses by Vincent's chair. He flinches. Vincent Castellane, flinching."),
      n("You tell yourself you imagined it."),
    ],
    lose: [say("danny", "Don't make me do this again, {name}."), say("mae", "Once more. And this time, don't look at Danny. Look at the cards.")],
  },
  {
    title: "The Dealer",
    tagline: "The final table. Everything on one game.",
    intro: [
      card("Chapter Ten", "The Dealer"),
      at("penthouse", "The Calloway Grand, the penthouse · the Kitty final"),
      n("Glass walls. The whole city below, lit up like a hand full of diamonds."),
      n("Every family boss in Port Calloway stands behind a velvet rope. At the head of the room, one chair sits empty."),
      say("vincent", "Ladies and gentlemen. Before we begin, the founder of the Kitty would like to say a few words."),
      mus("silence"),
      fx("elevator"),
      n("The elevator opens. Footsteps. Slow, and familiar."),
      fx("footsteps"),
      n("A gray bun. Reading glasses on a chain. The same cardigan she wore when she taught you to count trumps at the counter, twelve years ago."),
      fx("sting"),
      say("mae", "Hello, {name}."),
      you("…Mae?"),
      mus("tension"),
      say("mae", "Don't look at me like that. You've looked at me like that since you were ten. Like I hung the moon."),
      say("mae", "I built this. Silas and I, forty years ago. Then Silas got sentimental about rules, so I let him take the fall."),
      say("mae", "Vincent is my nephew. He's very good at being looked at. And Danny is mine. He always was."),
      say("mae", "And you. I've watched you since you were a kid flipping pancakes and counting cards in your head. I made you, {name}."),
      say("mae", "Last night was your final exam. You passed. Now lose this game, sit beside me, and when I'm gone the Kitty is yours. All of it."),
      choose(
        ["Never.", say("mae", "Everyone says that, the first time.")],
        ["Why me?", say("mae", "Because you're the only one who ever beat me at my own counter. Ten years old, and you counted my trumps.")],
      ),
      fx("elevator"),
      n("The elevator opens again."),
      say("silas", "Sorry I'm late. The federal government drives like my grandmother."),
      n("Silas Grant, in a borrowed suit, rubbing his wrists. Behind him, for half a second, a charcoal coat, before the doors slide shut."),
      say("silas", "Hello, Mae."),
      say("mae", "Silas. Still alive. How inconvenient."),
      say("silas", "The rules say a finalist may choose any partner. {name}, I'd be honored."),
      say("mae", "Then let's play. Danny, sit."),
      n("Danny takes the chair across from Mae. He won't look at you."),
      say("mae", "To 500. For everything."),
    ],
    level: {
      kind: "game",
      partner: "silas",
      west: "mae",
      east: "danny",
      banter: {
        made: [["silas", "Forty years I waited to see that face, Mae."], ["mae", "Hm. I did teach you that one."]],
        set: [["mae", "There it is. The hesitation on the third trick."], ["silas", "Steady. She wants you rattled."]],
        theyMade: [["mae", "Did you think I taught you everything I know?"], ["danny", "Sorry, partner."]],
        theySet: [["silas", "She's smiling, {name}. She always smiles when she loses."], ["mae", "Danny, dear. Count."]],
      },
    },
    win: [
      mus("silence"),
      n("The last trick. Yours."),
      n("For a long moment nobody in the penthouse breathes."),
      n("Then Mae Donnelly smiles. The smile Silas told you about."),
      say("mae", "Well. Look at that."),
      fx("elevator"),
      mus("tension"),
      n("The elevator doors open on a wall of FBI jackets. In front, badge raised: Roz."),
      say("roz", "Mary Ellen Donnelly, you're under arrest for racketeering, fraud, and the murder of Thomas Hale, nineteen eighty-four."),
      say("silas", "That was the one they gave me."),
      say("mae", "Took you long enough, Agent Vance. You think the Kitty is a room? It's a deck of cards. Somebody will always deal."),
      fx("cuffs"),
      n("Vincent is already in cuffs. Danny is edging toward the stairwell."),
      choose(
        ["Let him go.", n("You watch him go. At the door he looks back, and for a second he's eleven years old again."), say("danny", "I'm sorry, {name}. I really am."), n("Roz sighs and speaks into her radio. He doesn't get far.")],
        ["Danny. Stop.", n("He stops. His shoulders drop. When Roz cuffs him, he doesn't fight."), say("danny", "For what it's worth, you were always the better player.")],
      ),
      at("diner-dawn", "Mae's Diner · six months later"),
      n("The S in the neon sign works now. You fixed it yourself."),
      n("The deed came in the mail the week after the trial. No letter. Just the deed to the diner, signed over to you, and a Joker with one word on it in fountain pen: TUESDAY."),
      n("It's Tuesday."),
      say("silas", "Chairs down, {name}. I've been shuffling for ten minutes."),
      say("roz", "He's stalling. He owes me two dollars."),
      n("You push two tables together. Rain comes off the river. The cards come out."),
      n("In the corner booth, someone in a gray coat orders black coffee and doesn't drink it."),
      card("The End", "…for now."),
    ],
    lose: [say("mae", "Oh, kid. Sit beside me. It isn't too late."), say("silas", "Don't listen. Again. She's not as good as she thinks. Nobody is.")],
  },
];
