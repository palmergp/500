# Rules of thumb for 500 (house rules)

Practical rules for our table's game, measured rather than folklore. Each rule says where its number comes from.

| Source | What it is |
|---|---|
| **Solver data** | 64,000 bidding decisions from 16,000 deals, each solved with all cards visible and adjusted down half a trick to match real play |
| **Lab** | Duplicate matches between bots |
| **Expert habits** | What The Counter, our strongest player, does across thousands of hands |

Details and re-run commands are in `docs/lab/README.md`. The rules assume our house rules: once-around bidding, mentions and 6-calls, no nullo, and defenders can win.

## The short list

**Bidding**

1. **Count your hand in the suit you'd name trumps:**
   - Joker 1½, right bower 1¼, left bower 1;
   - trump ace ¾, king ⅔, queen ½, each other trump ½;
   - side ace ⅔, guarded side king ⅕, void ⅕;
   - then **add 3** for partner and the kitty.
2. **Bid 7 when the count reaches about 7. Go up to 8 only at about 8½, and to 9 only at about 10.** Each extra
   level adds just 100 points, but falling one trick short costs both bids' values.
3. **Bid a little more than feels safe.** Timid bidding loses games under these rules.
4. **Adjust for the auction:**
   - partner's 6-call +¾, partner's mention of your suit +0.6, partner's bid of it +2¼;
   - partner's pass −0.9, any opponent's bid of your suit −2.
5. **No trumps:**
   - count Joker 2¼, each ace 1, guarded king ½, then add 3⅓;
   - an unstopped suit hurts far less than its reputation.
   - **Suit or no trumps? Bid no trumps mainly with the Joker.**
     - Without it, no trumps beats your suit only about 30% of the time.
     - With the Joker and 2 aces, it wins 79% of the time.
     - Above the 7 level, lean toward the suit.
   - **Length or strength?**
     - In a suit, a top trump (the Joker or a bower) is worth more than an extra small trump: about ¾ against ¼–½ of a trick.
     - In no trumps, a long suit counts only if the ace heads it.
6. **Compete for the bid.** The neural bidder declares twice as often as the rule-based expert and scores 12 points a hand more.
7. **Trust partner's bid.** Raise it or pass; don't pull partner into your suit.

**Kitty**

8. **Keep every trump and your aces; void a short side suit.**

**Card play**

9. **As declarer with 5+ trumps, open with your top trump; with fewer, cash a side ace.** Rarely lead a small trump, unless partner called 6.
   Count your top trumps (Joker, both bowers, ace): missing two, it's a toss-up with a side ace; missing three or
   four, lead a side ace or a singleton first.
    - **In no trumps,** cash side winners and keep the Joker.
10. **Don't rush the Joker.** The expert usually plays it in tricks 3–5.
11. **Second hand low,** almost always in no trumps.
12. **Trump the opponents' winners, not your partner's.** Do protect partner's trick when someone can still beat it.
13. **On defense, lead a side ace if you have one.** Never a low trump, and partner's mention isn't a reason to lead that suit.
14. **Holding the Joker but no bid? Call 6.** It's worth ¾ of a trick to your partner.
15. **Kitty discards come from your shortest side suits,** to make a void.

**Signals and counting**

16. **The first time you're out of a suit, throw off from your strongest side suit.** It tells partner where to lead.
17. **Count trumps (13 red, 12 black) and every suit, and remember who has thrown off:** that player is out of the suit for good.

---

## Bidding

### 1. Count your hand like this

For the suit you'd make trumps, add up:

| You hold | Tricks |
|---|---|
| Joker | **1½** |
| Right bower (jack of trumps) | **1¼** |
| Left bower (other jack of the same color) | **1** |
| Ace of trumps | **¾** |
| King of trumps | **⅔** |
| Queen of trumps | **½** |
| Each other trump (10 or lower) | **½** |
| Each side-suit ace | **⅔** |
| Each guarded side king (king plus at least one more card in the suit) | **⅕** |
| Each side void | **⅕** |
| Each side singleton | **⅛** |

Then **add 3** for your partner and the kitty. That total is roughly how many tricks your side will take if you declare.

*Source: solver data, linear fit.* The starting value was 3.1 tricks. Card values are accurate to ±0.02 tricks.

- **Trumps are what count.** Every trump is worth at least half a trick. A side king is worth less than a small trump.
- **The two jacks are worth more than the ace of trumps.**
- **Voids help less than you'd think** (⅕ of a trick), because you need spare trumps to use them.

### 2. Bid 7 at a count of about 7; go up to 8 at about 8½, and to 9 only at about 10

**When a bid beats passing.** A 7-level bid needs to be made about 60% of the time to pay, given that defenders
score 10 a trick either way. That happens from a count of about **7** (6¾ in clubs, diamonds and hearts, whose bids
are worth more).

**When to go up a level.** With no one bidding against you, the question is usually not "bid or pass?" but "how
high?". Each extra level adds only 100 points when you make it. But if you then take one trick too few, a plus
becomes a minus: with 8 tricks, 8♦ scores +280 and 9♦ scores −380, a swing of 660. So go up a level only when
you'd make the higher bid about **80%** of the time:

| Your count | Bid (no one competing) | Made at that count |
|---|---|---|
| under about 7 | pass, mention or 6 | |
| about 7 to 8 | 7 | 7: 65–85% |
| about 8½ to 9½ | 8 | 8: 78–92% |
| about 10 or more | 9 | 9: about 80–85% |

At a count of 9½ a 9 bid is made about 70% of the time, and still scores about 70 points less than 8 on average.

**Competition doesn't change it.** You might think a higher bid is worth it to keep the bid away from players
still to call. Played out over 20,000 deals, bidding 9 never paid at any count up to 10, from any seat
(`docs/lab/README.md`, finding 14). With players still to call, 8 pays from a count of about 8½. And after an
opponent has bid, pass unless your count is about 8½ or more.

*Source: solver data, 64,000 bidding decisions (`train/rules_of_thumb.py`).* Real play runs about half a trick
below the solver, so a deal the solver makes with exactly the bid counts as half made. An earlier version of this
rule counted those deals as failures, charging a whole trick, and said "7 at 7¼, 8 at 8¼": half a trick too
cautious for the 7 level.

### 3. Bid a little more than feels safe

A bot that adds about **0.7 tricks** to a careful estimate wins **57%** of games against the same bot without it. Bidding too little is the bigger mistake under these rules:

- A thrown-in deal scores nothing.
- A cautious bot lost **74% of games** to a normal one.

*Source: lab, 40,000-game sweeps* (`docs/lab/README.md`, findings 1 and 6).

### 4. What the auction tells you

Add these to your count for the strain you're considering:

| What happened | Suit bid | No trumps |
|---|---|---|
| Partner called 6 (they hold the Joker) | **+¾** | **+1** |
| Partner mentioned this strain | **+0.6** | **+0.7** |
| Partner bid this strain | **+2¼** | **+2¼** |
| Partner passed | **−0.9** | **−1.3** |
| An opponent called 6 | **−½** | **−0.7** |
| An opponent mentioned this strain | **−⅔** | **−0.7** |
| An opponent bid this strain | **−2** | **−2** |
| You speak last (dealer), with no opposing bid, mention or 6 in this strain | **+½** | **+0.6** |

*Source: solver data* (±0.02–0.1 tricks).

- **Partner's pass is real information.** It costs almost a trick, so don't stretch to bid after partner has passed.
- **Opponents' silence is information too.** As dealer, if neither opponent showed this strain, add half a trick.

### 5. No trumps

Count:

| You hold | Tricks |
|---|---|
| The Joker | **2¼** (it's every suit's top card) |
| Each ace | **1** |
| Each guarded king | **½** |
| Each queen with two guards | **¼** |
| Each extra card in a long suit headed by the ace | **⅛** |

Then **add 3⅓**. The bidding thresholds are the same as in rule 2.

*Source: solver data.*

**Surprise:** an unstopped suit cost almost nothing (+0.06 ± 0.05 tricks). That's against the usual "stoppers in every suit" advice. With the kitty to discard into and the Joker guarding any suit, gaps hurt less than you'd expect.

### 5a. Suit or no trumps? The Joker decides it

*Source: solver data* (107,000 hands, each solved in every strain; `train/strain_choice.py`). Each hand compares your longest trump suit with no trumps at the 7 level, under the house scoring.

| You hold | No trumps scores better |
|---|---|
| **No Joker** (any shape) | **about 30%**: bid your suit |
| **The Joker** | **55–80%** |
| The Joker + 2 aces + no suit longer than 4 | **79%** |
| 3+ aces, 3+ stoppers, no singleton | 62% |
| 0 aces | 22–28% |
| 2 aces | about 50% |
| 3+ aces | 63% |

- **Aces matter more than stoppers.** With the same number of aces, extra stoppers move the figure only a few points.
- **A long suit and the Joker?** No trumps is still better at the 7 level (76–80% with a 6-card suit). You give up only about a third of a trick, and no trumps is worth more. Without the Joker, bid the suit.
- **From the 8 level up, lean toward the suit.** No trumps is better only 44% of the time with the Joker, and 20% without it.

### 5b. Length or strength?

**In a suit bid, top trumps beat length.** Average tricks for your side, by trumps held and how many of the top three (Joker, right bower, left bower) you hold:

| Trumps | none | 1 | 2 | all 3 |
|---|---|---|---|---|
| 3 | 5.3 | 6.0 | 6.7 | 7.4 |
| 4 | 5.8 | 6.6 | 7.3 | 8.0 |
| 5 | 6.3 | 7.0 | 7.7 | 8.4 |
| 6 | 6.8 | 7.4 | 8.0 | 8.6 |
| 7 | 7.2 | 7.6 | 8.3 | 8.6 |

- **Swapping a small trump for one of the top three** adds about **0.7 tricks**, at any length.
- **One more small trump** adds **0.5** when you hold none of the top three, but only about **0.25** once you hold two of them.
- **Length matters most when your trumps are weak.** Past six trumps it adds little.
- **Three trumps including the Joker and both bowers** (7.4) are worth about as much as **seven small trumps** (7.2).

**In no trumps, length without the ace is worthless.** Average no-trump tricks by your longest suit:

| Longest suit | no ace | ace, no king | ace and king |
|---|---|---|---|
| 4 | 5.1 | 5.9 | 6.4 |
| 5 | 4.9 | 5.8 | 6.3 |
| 6 | 4.7 | 5.9 | 6.3 |
| 7+ | 4.3 | — | 6.4 |

With 10-card hands and the Joker against you, long suits rarely run. Count the Joker and your aces, not your length.

### 6. Compete for the bid: the neural bidder's lesson

The Oracle bids from a neural network trained on 112,000 solved bidding decisions.

| Bidder | Declares | Makes | Points gained per hand |
|---|---|---|---|
| The Oracle | **68%** of hands | 74% | **+12** over the rule-based expert |
| The Counter (rule-based expert) | 32% of hands | 78% | — |

- **Bid when the count says it pays,** even if that means going down a little more often.
- **Making a high share of your bids isn't the goal;** bidding the right ones is.
- *Source: lab, 400 duplicate deals* (finding 9).

### 7. Partner's bid is worth trusting

Pulling partner out of their suit into your own nearly always cost points in the lab. Raise partner's suit, or pass. Switch strains only with a much stronger hand.

*Source: lab* (finding 4).

---

## The kitty

### 8. Keep every trump, keep your aces, and use the rest to void a short side suit

A searched choice of discards, comparing options across sampled deals, did no better than this simple rule: **+0.1 points a hand**, statistically zero.

What each part is worth, solved with every card visible over 4,137 deals where the bidder takes the kitty:

| Choice | What it costs or gains | Ever better the other way? |
|---|---|---|
| Putting back a trump | **−0.7 tricks** | Never |
| Putting back a side ace | **−0.5 tricks** | Never |
| Emptying a side suit (making a void) | **+0.4 tricks** (+0.3 with 5 or fewer trumps, +0.5 with 7 or more) | Rarely (5% of deals) |
| Keeping a guarded king (K with a small card) | No difference (16% better, 18% worse) | Often |

So don't spend a discard keeping a king guarded: use it to make the void.

*Source: lab, 600 duplicate deals* (finding 8) *and `fh-lab declarer`* (finding 16).

---

## Card play

There are two kinds of evidence here:

- **Lead analysis:** every legal opening lead in 600 deals, scored on the same 32 likely layouts of the hidden cards (`fh-lab leads`). "Cost" means the average expected points a type of lead gives up against the best lead.
- **Expert habits:** what The Counter does over 600 hands (`fh-lab habits`).

Both use the expert's search, which has blind spots of its own, so read them as strong guidance rather than proofs. Where they disagree, the lead analysis wins, because it scores every option.

### 9. As declarer, lead a top trump with five or more; with fewer, cash a side ace

*Opening leads in suit bids:*

| You hold | Best opening lead | Then | Worst |
|---|---|---|---|
| **5+ trumps** | Your **top trump**: best 70–74% of the time, cost 2–4 | Side ace (cost 7–8) | Low trump (cost 24–32), low side card (cost 25–32) |
| **4 or fewer trumps** | A **side ace**: cost 7 | Top trump (cost 8½) | Low side card (cost 29) |

"Top trump" means the highest trump still out, other than the Joker (see rule 10).

**After the opening lead,** keep drawing trumps gradually while the opponents still have some. About half the expert's leads in tricks 2–3 are trumps, and 60% from trick 4 on.

**Count your top trumps.** Solved with every card visible over 4,137 deals, each lead was compared with leading
your top trump on the same deal. What mattered most was how many of the four top trumps you hold (the Joker,
both bowers and the ace of trumps):

| Top trumps you're missing | Lead | Compared with your top trump |
|---|---|---|
| 0 or 1 | **Your top trump** | A side ace −0.1 tricks, a low side card −0.25, a low trump −0.55 |
| 2 | **A toss-up:** top trump, side ace or singleton | Side ace +0.07, singleton +0.1, low trump −0.15 |
| 3 or 4 | **A side ace or a singleton** first | Side ace up to +0.27, singleton up to +0.37 (with 5 or fewer trumps) |

Drawing trumps pays when you hold the top ones: each round pulls two of theirs. When the top trumps are against
you, leading trump gives them the lead to attack your side suits. Take your side winners while they're still good.

*Source: `fh-lab declarer`* (`docs/lab/README.md`, finding 18).

**Should the bidder always lead trump?** No:

- With five or more trumps, yes, the top one.
- With four or fewer, cash a side ace first.
- A **small** trump is almost never the right opening lead (best only 21–30% of the time). There's one exception, in rule 9a.

### 9a. If partner called 6, a small trump becomes a good lead

When partner showed the Joker, leading a **small trump** was the best opening lead **62%** of the time, against about 25% normally (cost 8). A side ace was close (cost 5).

Leading small toward partner's Joker draws the opponents' trumps without spending your own top ones. This is a small sample (16 deals), so treat it as a strong hint.

**If partner mentioned your trump suit** (a bower), the usual rule stands: lead your top trump (cost 2).

### 9b. In no trumps, cash side winners, and never open with the Joker

- **Side winners:** an ace or other sure winner was the best opening lead **87%** of the time (cost 5).
- **The Joker:** leading it was the worst choice (cost 49). It's your stopper in whichever suit the opponents attack, so keep it.

### 10. Don't rush the Joker as declarer

In suit bids the declarer's Joker is played:

| Tricks | Share |
|---|---|
| 1–2 | 24% |
| 3–5 | **48%** |
| 6–10 | 28% |

Draw the first round of trumps with lower ones if you can, and keep the Joker to regain control later.

The lead analysis agrees. Holding the Joker as declarer:

| Opening lead | Cost |
|---|---|
| The next-highest trump | **4.7** |
| The Joker itself | 9.1 |
| A low trump | 40 |

### 11. Second hand low

When following suit second, the expert plays low:

| Bid | Plays low |
|---|---|
| Suit bids | **64%** |
| No trumps | **94%** |

Save your high cards for when they capture something.

### 12. Trump the opponents' winners, not your partner's

When you can't follow suit:

| The trick | The expert |
|---|---|
| An opponent is winning and a trump would take it | **Trumps 88%** of the time |
| Partner is winning with a sure winner | **Discards 95%** of the time |
| Partner is winning but could still be beaten | Trumps **55%**: protect the trick when a later player might take it |

### 13. As a defender, lead a side ace if you have one, and never a low trump

The defenders' first lead in a suit bid:

| Lead | Best | Cost |
|---|---|---|
| **A side ace or winner** | **68%** | **8** |
| A low card from a side suit | 50% | 18 |
| Partner's mentioned suit | 34% | 20 |
| The Joker | 23% | 36 |
| A low trump | 17% | **44** |

- **Partner's mention isn't a lead signal.** Leading partner's mentioned suit is no better than average: the mention helps you bid, not lead. Partner's **discard signal** (rule 16) is the lead-directing one.
- *(The Counter's habits lean toward low side cards, 50% of its first leads. The full lead analysis prefers the ace.)*

### 14. Signal when you hold the Joker

Holding the Joker, the expert:

| Call | Share |
|---|---|
| Bid | **49%** |
| Called 6 to show it | **32%** |
| Passed (usually because someone had already bid, which closes signals) | **19%** |

If you have the Joker and can't bid, call 6. It's worth **¾ of a trick** to your partner (rule 4).

### 15. The kitty: shorten your shortest side suits

The expert's first discard comes from:

| Side suit | Share |
|---|---|
| Doubleton | **59%** |
| Singleton | **30%** |
| Three or more cards | 10% |

Two discards from a doubleton leave you void and able to trump.

---

## Signals and counting

### 16. The discard signal: throw off from your strongest side suit the first time you're out

**The convention.** The first time you can't follow suit and don't trump, discard a low card from your strongest side suit. It tells partner where to lead when they run out of winners. Every bot here uses it and reads it.

*Source: lab, 400 duplicate deals* (finding 11). For the expert bots the convention was a wash: **−0.8 ± 5 points a hand**.

- **It costs a little:** the signal sometimes gives up a useful card in your best suit.
- **It's worth little to an expert bot,** which already places most cards from the bidding and play.
- **Human partners get more out of it.** They can't infer as well, so the signal tells them something they wouldn't otherwise know. Use it freely; it isn't costing you anything measurable.

### 17. Count every suit, and notice every throw-off

**Count trumps.** There are 13 trumps in a red suit and 12 in a black suit, counting the Joker and both bowers.

- Subtract each one as it falls.
- When the trumps you haven't seen reach zero, your remaining trumps and side aces are safe.

**Count side suits.** A side suit has 10 or 11 cards, minus its jack when that jack is the left bower.

**Watch who throws off.** A player who fails to follow suit is out of it for the rest of the hand.

- Two players out of a suit means the rest are with the other two, or in the kitty.
- Combined with the bidding (who called 6, who mentioned what), this slowly places the hidden cards.

**What it's worth: about 12 points a hand.** The expert bot was matched against an identical bot that didn't
notice who had shown out of a suit, over 600 deals played both ways. Noticing was worth **+12 points a hand**
(95% range +5 to +19), and made bids 73% of the time against 68%. That's more than any single bidding or lead
rule measured here.

The game can show who is out of each suit, and a tracker of what's still out. The counting quiz turns those aids off and asks you instead.

---

### 18. Near the end, bid to the score

Each option was played out over 6,000 deals at each score, and valued by your side's chance of winning the game
(`fh-lab levels --us N --them N`; `docs/lab/README.md`, finding 20).

| Score | Best bid | Why |
|---|---|---|
| **They're at 460, you're at 300** | **Bid even a light hand** (count 6½), at the **cheapest level that takes you to 500** if made | Passing lets them win on defense with 4 tricks. Bidding raised your chances from about 32% to 43% at a count of 6½, and from 39% to 66% at 8 |
| **You're at 460, they're at 300** | **7 is enough**; over an opponent's bid, pass unless strong | Any made bid wins, and so do 4 defensive tricks |
| **Both at 450** | **Bid 7, even light** | Any made bid wins, and the declarers count first: 52% against 43% for passing at a count of 6½ |

Away from the end, the usual levels (rule 2) apply.

---

## Myths these numbers correct

- **"Count partner for two tricks."**
  - Partner plus the kitty are worth about **3** to the bidder.
  - Rule-of-thumb bots that counted 2 bid too little and lost.
- **"Double dummy is what you'll take."**
  - Solving with all cards visible overstates real declarer play by about **half a trick**, because nobody at the table can see the hidden cards.
  - *Source: lab, `calibrate`.*
