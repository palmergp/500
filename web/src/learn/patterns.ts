// The kinds of slips the coach tags (crates/ai/src/coach.rs, `slip_pattern`), with advice and the
// lesson that covers each. The Progress tab groups your graded slips by these to find leaks.

export interface Pattern {
  title: string;
  advice: string;
  /** A lesson id in lessons.ts. */
  lesson: string;
}

export const PATTERNS: Record<string, Pattern> = {
  overbid: {
    title: "Bidding too high",
    advice:
      "You bid more than the hand was worth, or bid when passing or a signal was better. Each extra level adds only 100 points, but one trick short costs both bids' values: go up a level only when you'd make it about 80% of the time.",
    lesson: "counting-hand",
  },
  underbid: {
    title: "Bidding too low",
    advice: "You passed or stopped short with a hand worth more. A thrown-in deal scores nothing: a 7 bid pays from a count of about 7.",
    lesson: "counting-hand",
  },
  strain: {
    title: "Choosing the trump suit",
    advice: "The right level, the wrong strain. Count each suit you could make trumps, and think about no trumps when you hold the Joker and aces with no long suit.",
    lesson: "strain",
  },
  "missed-signal": {
    title: "Missing a signal",
    advice: "With the Joker and not quite a bid, call 6; with a bower in your best suit, mention it. Your partner bids and leads better knowing.",
    lesson: "auction",
  },
  signal: {
    title: "Signals in the auction",
    advice: "Which signal to give, or whether to give one. Mentions show a bower (or aces for no trumps); a 6 shows the Joker.",
    lesson: "reading-auction",
  },
  "kitty-trump": {
    title: "Putting back trumps",
    advice: "Every trump is a likely trick: keep them all, and put back side cards instead.",
    lesson: "kitty",
  },
  "kitty-ace": {
    title: "Putting back aces",
    advice: "Side aces are winners. Put back low cards, ideally emptying a short suit so you can trump it.",
    lesson: "kitty",
  },
  kitty: {
    title: "Kitty choices",
    advice: "In a suit bid, empty a short side suit so you can trump it: worth about half a trick, more than keeping a king guarded. In no trumps it's the reverse: a void can't be trumped, so keep your guarded kings as stoppers and your length, and put back short suits with no ace or king first (a short queen stops little).",
    lesson: "kitty",
  },
  "draw-trumps": {
    title: "Not drawing trumps",
    advice: "As declarer, the coach would have led a trump here: drawing the opponents' trumps stops them trumping your winners, and can catch a lone top trump.",
    lesson: "declarer",
  },
  "trumps-early": {
    title: "Leading trumps at the wrong time",
    advice: "Drawing trumps isn't always first: sometimes you need to set up a side suit or keep trumps to ruff with.",
    lesson: "declarer",
  },
  "declarer-lead": {
    title: "Declarer's leads",
    advice: "Which side suit to lead as declarer: lead the top of touching honors, and set up long suits while you still hold trumps.",
    lesson: "declarer",
  },
  "defense-lead": {
    title: "Leads on defense",
    advice: "Lead your partner's suit (a mention or their first throw-off), or a side ace. Leading trumps on defense rarely helps.",
    lesson: "defense",
  },
  overtake: {
    title: "Overtaking your partner",
    advice: "Your partner was already winning the trick. Play low and save your high card for a trick that needs it.",
    lesson: "defense",
  },
  "missed-win": {
    title: "Not taking a trick you could win",
    advice: "You played a card that loses the trick when one that wins was better. Count what's left: is a later chance really coming?",
    lesson: "counting-cards",
  },
  "spent-winner": {
    title: "Spending a winner too early",
    advice: "You won a trick when playing low was better: the winner was worth more later, or partner would have won it anyway.",
    lesson: "counting-cards",
  },
  "throw-off": {
    title: "Choosing what to throw away",
    advice: "When you can't follow suit, throw from your strongest side suit the first time (our signal), and keep guards and winners.",
    lesson: "defense",
  },
  follow: {
    title: "Following suit",
    advice: "Which card to follow with: usually low when you can't win, and the cheapest card that wins when you can.",
    lesson: "counting-cards",
  },
};
