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
      "7♠ 140, 7♣ 160, 7♦ 180, 7♥ 200, 7NT 220, and each level up adds 100 (10♥ is 500). Make it and score the value; fall short and lose it. Defenders always score 10 a trick. Taking all ten tricks scores 250 if the bid is worth less. You win by making a bid that takes you to 500 or more: defensive points count toward your score but can't win the game. A side at −500 or worse loses.",
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
      "• Each side ace ⅔ · each side king beside its ace ⅓ · each guarded side king ⅕ (⅛ with just one guard) · each side void ⅕ · each singleton ⅛ · ⅛ more for a second short side suit (void or singleton)",
      "Side queens count nothing in a suit bid: even guarded, one wins a trick only about one time in seven.",
      "• Then add 3 for your partner and the kitty.",
      "That total is roughly what your side will take as declarer. Bid 7 when it reaches about 7 (6¾ in a red suit or clubs): the bid is then made about 60% of the time, which is what the scoring needs to pay.",
      "## How high?",
      "Each extra level adds only 100 points, but one trick short turns a plus into a minus: with 8 tricks, 8♦ scores +280 and 9♦ scores −380. So with no one bidding against you, go up a level only when you'd make it about 80% of the time: 8 from a count of about 8½. And stop at 8: a 9 bid almost never pays, even at a count of 11.",
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
        explain: "9♦ is made about 70% of the time, but taking exactly 8 turns +280 into −380. 8♦ scores about 70 points more on average, and 9 almost never pays.",
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
      "• Partner called 6 (has the Joker): +1⅓ if it's your suit, +0.6 in your left bower's suit (same color), +¼ in another suit, +0.8 in no trumps.",
      "• Partner mentioned your strain: +0.4. Partner mentioned another suit: −⅔ (their strength is elsewhere; −1⅛ for no trumps). Partner bid your strain: +2¼.",
      "• Partner passed: −1 (−1.6 in no trumps), or −¾ if you hold the Joker (so it wasn't the Joker they lacked). Their pass is real information: don't stretch after it.",
      "• Partner bid another strain: +½ (+1 for no trumps). A strong partner helps your suit too.",
      "• An opponent called 6 in your suit: −1 (they hold the Joker and length there). In your left bower's suit: −0.4. In another suit: −⅛. An opponent mentioned your strain: −½; another suit: +¼ (their strength isn't in yours); NT: no change. An opponent bid your strain: −2.",
      "• Speaking last as dealer, with no opposing interest in your strain: +½.",
      "• Speaking last when nobody has bid or called 6, without the Joker yourself: +1¼ more. The first two callers would have shown the Joker, and a third holding it would usually have bid, so it's in the kitty about four times in five.",
      "## Compete, and trust partner",
      "The neural bidder (The Oracle) declares about twice as often as the rule-based expert and scores 12 points a hand more: bid when the count says it pays, even if you go down a little more often. But don't pull partner out of their suit into yours: raise it or pass.",
      "## Signal when you can't bid",
      "Holding the Joker but no bid? Call 6: it tells partner where the Joker is. Strength in a suit you can't bid? Mention it: a bower with two more trumps tells partner about 1⅓ tricks, four trumps without a bower about 1, a bower with only one other card about ⅔.",
      "Once partner has called, though, just pass: partner can't act on a mention or 6 any more, and in the lab it made no difference.",
    ],
    quiz: [
      { q: "Partner passed. Adjust your count by about…", options: ["+1", "0", "−1"], answer: 2, explain: "A pass shows weakness: about −1 in a suit bid." },
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
      "No suit you can empty? If a king with small cards is the only suit you can cut down, put back the small cards and leave the king alone: about ¼ of a trick better than keeping it guarded. The small cards were losers; once the king has won or fallen to the ace, you trump the suit.",
      "## In no trumps",
      "It's the other way round: you can't trump a void, so making one gains nothing. Keep your stoppers instead: a guarded king (a king with a small card beside it) was worth about ⅕ of a trick. Keep your aces and the length under them, and put back low cards from suits with no honours.",
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
      {
        q: "Spades are trumps. You hold K♣ 9♣ 7♣ 6♣, five low hearts and no diamonds. Put back three clubs, or three hearts?",
        options: ["Three clubs, leaving K♣ alone", "Three low hearts, keeping K♣ guarded", "No difference"],
        answer: 0,
        explain: "Leaving the king bare is about ¼ of a trick better: the small clubs were losers, and once the king has won or fallen you trump clubs.",
      },
      {
        q: "Now you're playing no trumps. Keep K♣ guarded, or put back both clubs to empty the suit?",
        options: ["Keep the king guarded", "Empty the suit", "No difference"],
        answer: 0,
        explain: "In no trumps a void can't be trumped, so it gains nothing, while the guarded king stops their clubs: about ⅕ of a trick.",
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
      "Cash a side ace early rather than late: as the hand goes on the opponents run out of side suits, and an ace led after trick 6 was trumped 39% of the time (7% in the first three tricks). In the first three tricks, leading your top trump or cashing a side ace came out even when hands were played out both ways, even with many trumps out. Leading a side king early is the weak play: it ran into the ace more often than not.",
      "## No trumps",
      "Cash your side winners first: an ace or sure winner was the best lead 87% of the time. Don't lead the Joker early: in the first three tricks it cost about 0.2 tricks. Remember it isn't a stopper: you may play it only when you can't follow suit. Late in the hand (from about trick 7) leading it is better: a sure trick, the lead back, and the suit you name makes the others give up cards in it.",
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
        options: ["Yes, to take control", "No, cash side winners first"],
        answer: 1,
        explain: "Leading the Joker was the worst opening lead in no trumps. Save it for late in the hand (about trick 7 on), when leading it gains.",
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
      "## Second and third hand",
      "Third hand high: when partner leads and the declarer's side is winning, play the cheapest card that takes the trick. And when the declarer's side leads a jack, queen or king that you can beat, cover it. Each was worth almost half a trick in the lab, and right nearly every time.",
      "Second hand low is the habit, especially in no trumps. In a suit bid, taking the trick cheaply when you can was as good on average.",
      "## The discard signal",
      "The first time you can't follow suit and don't trump, throw a low card from your strongest side suit. Partner now knows where to lead when they run out of winners, and you should read partner's first throw-off the same way.",
      "Don't pay much for it: if the signal card is one you need (a guard or a winner), keep it. In the lab, bots that gave up real value to signal lost about 3 points a hand; the bots now signal only when it's nearly free.",
      "• Example: spades are trumps, a diamond is led, you have no diamonds and hold A K 7 6 5 of hearts. Throw the 5♥.",
      "• Everyone sees the signal, opponents included. Against expert bots it cost nothing measurable, so use it.",
      "## Following and trumping",
      "• Second hand low: when following second, the expert plays low most of the time (94% in no trumps).",
      "• Trump the opponents' winners: when an opponent is winning and a trump would take the trick, the expert trumps 88% of the time.",
      "• Don't trump your partner's sure winner: the expert discards 95% of the time. But if partner's card could still be beaten by a later player, trumping to protect the trick is often right.",
      "• Make your winners easy to read: from A-K, play the ace first, then the king. Partner can see the ace is a winner and won't trump it; a king first leaves partner guessing where the ace is.",
    ],
    quiz: [
      {
        q: "Defending a suit bid, you hold the A♣. Your first lead?",
        options: ["A low trump", "The A♣", "Partner's mentioned suit"],
        answer: 1,
        explain: "A side ace was the best defensive lead 68% of the time; a low trump was the worst.",
      },
      {
        q: "You can't follow suit for the first time and won't trump. You hold K Q 8 5 of clubs and 9 6 of hearts. Which card?",
        options: ["9♥", "5♣", "K♣"],
        answer: 1,
        explain: "Throw a low card from your strongest side suit: the 5♣ tells partner to lead clubs.",
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
      "How much is it worth? In the lab, the expert bot beat an identical bot that didn't notice who was out of a suit by about 12 points a hand, and made its bids 73% of the time against 68%: more than any single bidding or lead rule.",
      "## Put it together",
      "From voids and counts you can slowly place the hidden cards. Example: hearts are trumps, three rounds have gone, West threw off on the second, and you haven't seen J♦ or K♥. Then they're with North or East (or discarded by the declarer), so leading a small trump won't catch them both with West.",
      "## Practice",
      "Turn on the counting quiz in Settings: during play it asks how many trumps or suit cards are still out, who is out of a suit, and which is the highest card of a suit not yet played, hiding the card tracker while you answer. You can set how often it asks.",
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
      "• You win only on a hand you bid: make your bid and finish with 500 or more. Defenders still score 10 a trick, so a side can pass 500 on defense, but it plays on until it makes a bid. Once you're past 500, any bid you make wins.",
      "• A side at −500 or worse loses, so going down on a bid can end the game. Near −500, a risky bid can lose it outright.",
      "• Only the declarers can win or lose on a hand, so both can never happen at once.",
      "## How to bid near the end (measured over 6,000 deals at each score)",
      "• They're near 500 (say 460) and you're not: bid, even with a light hand (a count of about 6½). They can't win on defense, but any bid they make will, so take the auction away from them. From a count of 7½, bid the level that takes you to 500 if you make it: at 300, that's 8 in spades, clubs or diamonds, or 7 in hearts or no trumps.",
      "• You're near 500 and they're not: any bid you make wins, so bid 7 (don't climb higher). Over an opponent's bid, bid 8 rather than pass: your defensive tricks won't win the game, but their made bid might.",
      "• Both near 500: bid 7 even with a light hand. Any made bid wins.",
      "## In a four-hand game",
      "There's no 500 to race to: after four hands the higher score wins. So play every hand for points, as early in a normal game. A sacrifice that goes down to stop the opponents' bid just hands them the points. The coach plays four-hand games this way.",
    ],
    quiz: [
      {
        q: "You're at 480 and defending. Can you win the game this hand?",
        options: ["Yes, with two defensive tricks", "No, only by bidding"],
        answer: 1,
        explain: "Defensive tricks still score, so you may pass 500, but you only win on a hand you bid and make. Once you're past 500, any bid you make wins.",
      },
      {
        q: "They're at 460, you're at 300, and your diamond count is about 8. What's your bid?",
        options: ["Pass", "7♦", "8♦"],
        answer: 2,
        explain: "8♦ (280) takes you to 580 and wins if you make it; 7♦ wouldn't reach 500. Passing hands them the auction, and any bid they make wins. At a count of 8, bidding 8 won about 67% of games, 7 about 61%, passing 43%.",
      },
    ],
  },
];
