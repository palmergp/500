// The curriculum. Content comes from docs/rules-of-thumb.md (measured in the lab) and
// docs/rules/house-rules.md. Keep the two in step: a number changed there should change here.

export interface QuizQuestion {
  q: string;
  options: string[];
  answer: number;
  explain: string;
}

export interface Lesson {
  id: string;
  title: string;
  summary: string;
  /** Paragraphs; a line starting with "• " is a bullet, "## " a subheading. */
  body: string[];
  quiz: QuizQuestion[];
}

export const LESSONS: Lesson[] = [
  {
    id: "cards",
    title: "1. The cards: bowers and the Joker",
    summary: "Which cards are trumps, and how the Joker behaves in no trumps.",
    body: [
      "## Trumps",
      "When a suit is trumps, its trumps rank: Joker, right bower (the jack of trumps), left bower (the other jack of the same color), then A K Q 10 9 … down to the lowest.",
      "• Hearts trumps: Joker, J♥, J♦, A♥, K♥, Q♥, 10♥ … 4♥: 13 trumps. A black trump suit has 12 (there are no black 4s).",
      "• The left bower is a trump, not a card of its own suit. With hearts trumps, J♦ follows a heart lead, never a diamond lead, and a hand whose only diamond is J♦ is void in diamonds.",
      "• The game marks bowers with R and L once trumps are set, and sorts them into the trump suit.",
      "## No trumps",
      "There are no bowers: all four jacks are ordinary. The Joker is a suit of its own and the only trump: it wins any trick it's played to, but you may play it only when you can't follow suit. If you lead it, you name the suit everyone else must follow.",
    ],
    quiz: [
      { q: "Hearts are trumps. Which card is higher?", options: ["A♥", "J♦"], answer: 1, explain: "J♦ is the left bower, third-highest trump after the Joker and J♥." },
      {
        q: "Hearts are trumps, diamonds are led, and your only diamond is J♦. What must you do?",
        options: ["Play J♦", "Play anything: you're void in diamonds"],
        answer: 1,
        explain: "J♦ is a trump here, so you have no diamonds. You may trump with it, or throw any card.",
      },
      {
        q: "No trumps. You hold the Joker and the 5♠, and spades are led. Can you play the Joker?",
        options: ["Yes", "No"],
        answer: 1,
        explain: "In no trumps the Joker may be played only when you can't follow suit.",
      },
    ],
  },
  {
    id: "auction",
    title: "2. The auction: calls and scoring",
    summary: "Once-around bidding, mentions, 6-calls, and what bids are worth.",
    body: [
      "Each player gets exactly one call, starting left of the dealer; the dealer speaks last.",
      "• Pass.",
      "• Mention a suit (or NT): \"this is my strongest suit, usually with a bower, but I can't bid\". Each mention must outrank the last (♠ < ♣ < ♦ < ♥ < NT).",
      "• 6 of a strain: \"I hold the Joker, and this is my best suit\". It never becomes the winning bid.",
      "• A bid of 7–10 in a strain. Once anyone calls 6 or bids, mentions and 6-calls close.",
      "If nobody bids 7, the same dealer redeals, up to twice; on the third deal the dealer must bid if nobody else has.",
      "## Scoring",
      "7♠ 140, 7♣ 160, 7♦ 180, 7♥ 200, 7NT 220, and each level up adds 100 (10♥ is 500). Make it and score the value; fall short and lose it. Defenders always score 10 a trick. Taking all ten tricks scores 250 if the bid is worth less. First side to 500 wins, defensive points included.",
    ],
    quiz: [
      { q: "Someone mentioned ♦. Can the next player mention ♣?", options: ["Yes", "No"], answer: 1, explain: "Mentions must outrank the last one: after ♦, only ♥ or NT." },
      { q: "What is 8♦ worth?", options: ["180", "280", "300"], answer: 1, explain: "7♦ is 180 and each level adds 100." },
      {
        q: "Partner called 6♥. What do you know?",
        options: ["They hold the Joker", "They hold six hearts", "They want to play 6♥"],
        answer: 0,
        explain: "A 6-call says \"I hold the Joker\"; the suit is their best suit. It's never a real bid.",
      },
    ],
  },
  {
    id: "counting-hand",
    title: "3. Counting your hand",
    summary: "A measured counting system and when a bid pays.",
    body: [
      "Count your hand in the suit you'd make trumps. These values were measured from 64,000 solved bidding decisions:",
      "• Joker 1½ · right bower 1¼ · left bower 1 · trump ace ¾ · trump king ⅔ · trump queen ½ · each other trump ½",
      "• Each side ace ⅔ · each guarded side king ⅕ · each side void ⅕ · each singleton ⅛",
      "• Then add 3 for your partner and the kitty.",
      "That total is roughly what your side will take as declarer. Bid 7 when it reaches about 7 (6¾ in a red suit or clubs): the bid is then made about 60% of the time, which is what the scoring needs to pay.",
      "## How high?",
      "Each extra level adds only 100 points, but one trick short turns a plus into a minus: with 8 tricks, 8♦ scores +280 and 9♦ scores −380. So with no one bidding against you, go up a level only when you'd make it about 80% of the time: 8 from a count of about 8½, and 9 only from about 10.",
      "Competition doesn't change it: played out over 20,000 deals, a higher bid to keep the bid away from players still to call never made 9 pay below a count of 10. And over an opponent's bid, pass unless your count is about 8½.",
      "## Bid a little more than feels safe",
      "Thrown-in deals score nothing, so timid bidding loses. In the lab, a bot that adds about 0.7 tricks to a careful estimate won 57% of its games against the same bot without it.",
      "The coach shows this count, card by card, whenever you ask for a hint or get a grade in the auction.",
    ],
    quiz: [
      {
        q: "You hold the Joker, J♥, three small hearts, the A♠ and a singleton club. Your heart count?",
        options: ["About 6", "About 8", "About 9½"],
        answer: 1,
        explain: "1½ + 1¼ + 3 × ½ + ⅔ + ⅛ + 3 ≈ 8.0: a 7♥ bid. 8♥ waits for about 8½.",
      },
      { q: "What is a side void worth on its own?", options: ["About ⅕ trick", "About 1 trick", "About 2 tricks"], answer: 0, explain: "A void helps only with spare trumps to use it: about ⅕ of a trick, less than a small trump." },
      { q: "From what count does a 7 bid start to pay?", options: ["6", "7", "8½"], answer: 1, explain: "About 7 (6¾ in a red suit or clubs): the bid is then made about 60% of the time." },
      {
        q: "Your diamond count is about 9½ and no one has bid. Which bid scores best on average?",
        options: ["8♦", "9♦", "10♦"],
        answer: 0,
        explain: "9♦ is made about 70% of the time, but taking exactly 8 turns +280 into −380. 8♦ scores about 70 points more on average; 9 pays from a count of about 10.",
      },
    ],
  },
  {
    id: "strain",
    title: "4. Suit or no trumps? Length or strength?",
    summary: "When no trumps pays, and what really makes tricks.",
    body: [
      "These numbers come from 107,000 hands, each solved in every strain and scored at the 7 level.",
      "## The Joker decides no trumps",
      "• Without the Joker, no trumps beats your longest suit only about 30% of the time, whatever your shape. Bid the suit.",
      "• With the Joker, no trumps wins 55–80% of the time. With the Joker and 2 aces, and no suit longer than 4, it wins 79% of the time.",
      "• Aces matter more than stoppers: 0 aces gives NT about 25%, 2 aces about 50%, 3+ aces 63%.",
      "• From the 8 level up, lean toward the suit. Even with the Joker, no trumps wins only 44% of the time there.",
      "## Lots of one suit?",
      "Without the Joker, bid it. With the Joker, no trumps is still often better at the 7 level, even with a 6-card suit. You lose only about a third of a trick, and no trumps is worth more points.",
      "## Length or strength in a suit bid",
      "• Top trumps win. Swapping a small trump for the Joker or a bower adds about ¾ of a trick.",
      "• One more small trump adds ½ a trick when you hold none of the top three trumps, but only ¼ once you hold two of them.",
      "• Three trumps that include the Joker and both bowers are worth about as much as seven small trumps.",
      "## Length in no trumps",
      "A long suit counts only if the ace heads it. Without the ace, a longer suit actually produced fewer no-trump tricks. With A-K at the top, length barely helped. Count the Joker and your aces.",
    ],
    quiz: [
      {
        q: "No Joker, three aces, a balanced hand. Suit or no trumps?",
        options: ["No trumps", "Your best suit"],
        answer: 1,
        explain: "Without the Joker, no trumps is right only about 30% of the time. Bid your best suit.",
      },
      {
        q: "You hold the Joker, two aces and no suit longer than four. Which strain?",
        options: ["No trumps", "Your four-card suit"],
        answer: 0,
        explain: "That pattern favored no trumps on 79% of hands.",
      },
      {
        q: "Hearts are trumps. Which adds more: a sixth small heart, or turning a small heart into the right bower?",
        options: ["The sixth small heart", "The right bower"],
        answer: 1,
        explain: "A top trump adds about ¾ of a trick; another small trump adds ¼–½.",
      },
      {
        q: "No trumps: your long suit is Q J 10 9 8 7 with no ace. How much does its length help?",
        options: ["A lot: it will run", "Very little"],
        answer: 1,
        explain: "Without the ace, a long suit rarely runs in no trumps: hands are only 10 cards and the opponents may hold the Joker.",
      },
    ],
  },
  {
    id: "reading-auction",
    title: "5. Reading the auction",
    summary: "What partner's and opponents' calls are worth.",
    body: [
      "Add these to your count for the strain you're considering (measured):",
      "• Partner called 6 (has the Joker): +¾ in a suit, +1 in no trumps.",
      "• Partner mentioned your strain: +0.6. Partner bid it: +2¼.",
      "• Partner passed: −0.9 (−1.3 in no trumps). Their pass is real information: don't stretch after it.",
      "• An opponent called 6: −½. An opponent mentioned your strain: −⅔. An opponent bid it: −2.",
      "• Speaking last as dealer, with no opposing interest in your strain: +½.",
      "## Compete, and trust partner",
      "The neural bidder (The Oracle) declares about twice as often as the rule-based expert and scores 12 points a hand more: bid when the count says it pays, even if you go down a little more often. But don't pull partner out of their suit into yours: raise it or pass.",
      "## Signal when you can't bid",
      "Holding the Joker but no bid? Call 6: it's worth ¾ of a trick to your partner. Holding a bower in a suit you can't bid? Mention it.",
    ],
    quiz: [
      { q: "Partner passed. Adjust your count by about…", options: ["+1", "0", "−0.9"], answer: 2, explain: "A pass shows weakness: about −0.9 in a suit bid." },
      {
        q: "You hold the Joker but your count is only 6½. Best call (if open)?",
        options: ["Pass", "6 in your best suit", "Bid 7 anyway"],
        answer: 1,
        explain: "Call 6 to show the Joker: it helps partner bid, and costs nothing.",
      },
    ],
  },
  {
    id: "kitty",
    title: "6. The kitty",
    summary: "What to put back.",
    body: [
      "Keep every trump and your aces. Use the three discards to shorten your shortest side suits: two discards from a doubleton leave you void, ready to trump that suit.",
      "In the lab, a search that compared discards across likely deals did no better than this simple rule.",
      "What each part is worth, solved over 4,000 deals: putting back a trump costs about ¾ of a trick, and putting back an ace about ½; neither was ever right. Making a void gains about ½ a trick, more the more trumps you hold. Keeping a king guarded makes no difference, so spend the discards on the void.",
    ],
    quiz: [
      {
        q: "Trumps are hearts. Which is the best discard plan?",
        options: ["A♠ and two small clubs", "Both cards of a club doubleton and a small diamond", "Three small trumps"],
        answer: 1,
        explain: "Void the doubleton and keep aces and trumps.",
      },
      {
        q: "You could keep K♣ guarded or put back both clubs to make a void. Which gains more?",
        options: ["Keep the king guarded", "Make the void", "No difference"],
        answer: 1,
        explain: "The void gains about ½ a trick on average; keeping the king guarded makes no difference.",
      },
    ],
  },
  {
    id: "declarer",
    title: "7. Playing the hand as declarer",
    summary: "Which card to lead first, and how partner's calls change it.",
    body: [
      "These tips come from scoring every possible opening lead in 600 deals, using the expert's search over likely layouts of the hidden cards.",
      "## Should the bidder always lead trump?",
      "• With five or more trumps, yes: lead your top trump (the highest still out, other than the Joker). It was the best lead about 72% of the time.",
      "• With four or fewer, cash a side ace first. Opponents rarely trump the first round of a suit.",
      "• Count your top trumps too (the Joker, both bowers and the ace of trumps). Holding three or four of them, lead trump. Missing two, a side ace or a singleton does about as well. Missing three or four, take your side winners first: leading trump hands them the lead.",
      "• Almost never open with a small trump: it was the best lead only about a quarter of the time, and on average gave up 24–32 points.",
      "## The Joker",
      "Don't lead it first. Holding the Joker, the next-highest trump was a better opening lead (it gave up about 5 points to the best lead, against 9 for the Joker). Keep the Joker to win a trick later and take back the lead when you need it. The expert usually plays it in tricks 3–5.",
      "## What partner told you",
      "• Partner called 6 (they hold the Joker): now a small trump is often the right lead (best 62% of the time). You draw trumps toward partner's Joker without spending your own top trumps.",
      "• Partner mentioned your trump suit (a bower): lead your top trump as usual.",
      "## After the first lead",
      "Keep drawing trumps gradually while the opponents still have some, mixing in your side winners. Count trumps as they fall (lesson 9): when none are left out, your side aces can't be trumped.",
      "## No trumps",
      "Cash your side winners first: an ace or sure winner was the best lead 87% of the time. Never open with the Joker: it's your stopper for whichever suit the opponents attack.",
    ],
    quiz: [
      {
        q: "You're declarer in hearts with six trumps, including both bowers and the A♥ (no Joker), and the A♠. Your opening lead?",
        options: ["The A♠", "Your top trump", "A small trump"],
        answer: 1,
        explain: "You hold three of the four top trumps: lead the top one. With five or more trumps it was the best lead about 72% of the time.",
      },
      {
        q: "You're declarer with five trumps but none of the Joker, the bowers or the ace of trumps. You hold the A♠. Lead?",
        options: ["The A♠", "Your top trump", "A small trump"],
        answer: 0,
        explain: "Missing all four top trumps, leading trump hands the opponents the lead. Cash the side ace first: it gained about ¼ of a trick.",
      },
      {
        q: "Partner called 6 during the auction and you're declarer. Which lead becomes much better than usual?",
        options: ["A small trump", "The Joker", "A low side card"],
        answer: 0,
        explain: "Partner holds the Joker, so a small trump draws the opponents' trumps toward it. It was best 62% of the time.",
      },
      {
        q: "You're declarer in no trumps holding the Joker. Open with it?",
        options: ["Yes, to take control", "No, keep it as a stopper"],
        answer: 1,
        explain: "Leading the Joker was the worst opening lead in no trumps.",
      },
    ],
  },
  {
    id: "defense",
    title: "8. Defense and signals",
    summary: "Opening leads, when to trump, and the discard signal.",
    body: [
      "## The first lead",
      "Lead a side ace if you have one: it was the best defensive lead 68% of the time. Otherwise lead a low card from a side suit. Never lead a low trump (it gave up 44 points on average) or the Joker.",
      "Partner's mention is not a lead signal: leading partner's mentioned suit was no better than average. The mention helps partner's side bid. To tell partner where to lead, there's the discard signal.",
      "## The discard signal",
      "The first time you can't follow suit and don't trump, throw a low card from your strongest side suit. Partner now knows where to lead when they run out of winners, and you should read partner's first throw-off the same way.",
      "• Example: spades are trumps, a diamond is led, you have no diamonds and hold A K 7 6 5 of hearts. Throw the 5♥.",
      "• Everyone sees the signal, opponents included. Against expert bots it cost nothing measurable, so use it.",
      "## Following and trumping",
      "• Second hand low: when following second, the expert plays low most of the time (94% in no trumps).",
      "• Trump the opponents' winners: when an opponent is winning and a trump would take the trick, the expert trumps 88% of the time.",
      "• Don't trump your partner's sure winner: the expert discards 95% of the time. But if partner's card could still be beaten by a later player, trumping to protect the trick is often right.",
    ],
    quiz: [
      {
        q: "Defending a suit bid, you hold the A♣. Your first lead?",
        options: ["A low trump", "The A♣", "Partner's mentioned suit"],
        answer: 1,
        explain: "A side ace was the best defensive lead 68% of the time; a low trump was the worst.",
      },
      {
        q: "You can't follow suit for the first time and won't trump. You hold K Q 8 4 of clubs and 9 6 of hearts. Which card?",
        options: ["9♥", "4♣", "K♣"],
        answer: 1,
        explain: "Throw a low card from your strongest side suit: the 4♣ tells partner to lead clubs.",
      },
      {
        q: "Partner's A♦ is winning the trick for sure and you're out of diamonds. Trump it?",
        options: ["Yes", "No, throw off"],
        answer: 1,
        explain: "Don't trump partner's sure winner. Discard, using the signal if it's your first throw-off.",
      },
    ],
  },
  {
    id: "counting-cards",
    title: "9. Counting cards",
    summary: "Keep track of trumps, suits, and who is out of what.",
    body: [
      "Good players count as they play. Three habits:",
      "• Count trumps. There are 13 in a red suit and 12 in a black suit, counting the Joker and both bowers. Each round, subtract what falls. When the count of trumps you haven't seen reaches zero, your remaining trumps are winners and your side aces are safe.",
      "• Count each suit. A side suit has 10 or 11 cards (minus the jack if it's the left bower). Watch how many have gone.",
      "• Notice who throws off. The moment a player fails to follow suit, they hold no more of it for the rest of the hand. Two players out of a suit means the rest are with the other two, or in the kitty.",
      "## Put it together",
      "From voids and counts you can slowly place the hidden cards. Example: hearts are trumps, three rounds have gone, West threw off on the second, and you haven't seen J♦ or K♥. Then they're with North or East (or discarded by the declarer), so leading a small trump won't catch them both with West.",
      "## Practice",
      "Turn on the counting quiz in the Coach menu: during play it asks how many trumps or suit cards are still out, and who is out of a suit, hiding the card tracker while you answer.",
    ],
    quiz: [
      { q: "Spades are trumps. How many trumps are in the pack?", options: ["10", "12", "13"], answer: 1, explain: "10 spades (5–A) + J♣ (left bower) + Joker = 12." },
      {
        q: "West failed to follow to a club. What does that tell you?",
        options: ["West has no clubs left", "West has the Joker", "Nothing for certain"],
        answer: 0,
        explain: "A player who can't follow suit holds none of it for the rest of the hand.",
      },
    ],
  },
  {
    id: "score",
    title: "10. Playing to the score",
    summary: "How the house rules on winning change your choices.",
    body: [
      "• Defenders can win the game with their 10 a trick. Near 500, every defensive trick matters, and you can go out on defense.",
      "• If both sides cross 500 on the same hand, the declarers count first and win.",
      "• There's no losing floor: a side can go any distance below zero. So when the opponents are close to 500, an aggressive bid that might go down is often worth it to stop them.",
    ],
    quiz: [
      {
        q: "You're at 480 and defending. Can you win the game this hand?",
        options: ["Yes, with two defensive tricks", "No, only by bidding"],
        answer: 0,
        explain: "Defensive points count under our rules: two tricks take you to 500.",
      },
    ],
  },
];
