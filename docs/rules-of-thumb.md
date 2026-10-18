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
2. **Bid 7 when the count reaches about 7¼, and 8 at about 8¼.**
3. **Bid a little more than feels safe.** Timid bidding loses games under these rules.
4. **Adjust for the auction:**
   - partner's 6-call +¾, partner's mention of your suit +0.6, partner's bid of it +2¼;
   - partner's pass −0.9, any opponent's bid of your suit −2.
5. **No trumps:**
   - count Joker 2¼, each ace 1, guarded king ½, then add 3⅓;
   - an unstopped suit hurts far less than its reputation.
6. **Compete for the contract.** The neural bidder declares twice as often as the rule-based expert and scores 12 points a hand more.
7. **Trust partner's bid.** Raise it or pass; don't pull partner into your suit.

**Kitty**

8. **Keep every trump and your aces; void a short side suit.**

**Card play**

9. **As declarer, cash a side ace early, then draw trumps gradually.**
10. **Don't rush the Joker.** The expert usually plays it in tricks 3–5.
11. **Second hand low,** almost always in no trumps.
12. **Trump the opponents' winners, not your partner's.** Do protect partner's trick when someone can still beat it.
13. **On defense, lead low from a side suit more often than cashing an ace.**
14. **Holding the Joker but no bid? Call 6.** It's worth ¾ of a trick to your partner.
15. **Kitty discards come from your shortest side suits,** to make a void.

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

### 2. Bid 7 when your count reaches about 7¼, and 8 at about 8¼

At that point the contract makes about 60% of the time. That's what the house scoring needs to pay off, given that defenders score 10 a trick either way. A just-made 7-level contract needs about 60%, and an 8 about 56%.

*Source: solver data.* Contracts made 62% of the time at a count of 7¼, and 60% at 8¼.

### 3. Bid a little more than feels safe

A bot that adds about **0.7 tricks** to a careful estimate wins **57%** of games against the same bot without it. Bidding too little is the bigger mistake under these rules:

- A thrown-in deal scores nothing.
- A cautious bot lost **74% of games** to a normal one.

*Source: lab, 40,000-game sweeps* (`docs/lab/README.md`, findings 1 and 6).

### 4. What the auction tells you

Add these to your count for the strain you're considering:

| What happened | Suit contract | No trumps |
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

### 6. Compete for the contract: the neural bidder's lesson

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

*Source: lab, 300 duplicate deals* (finding 8).

---

## Card play

*Source: expert habits, The Counter in all four seats over 600 hands* (`fh-lab habits`). These describe what our strongest player does. They are strong guidance, not proofs: the expert plays by sampling deals and solving them with all cards visible, which has blind spots of its own.

### 9. As declarer, cash a side ace early, and draw trumps gradually

- **The opening lead.** Even with 6 or more trumps, the expert's opening lead is a side winner (usually an ace) **52%** of the time, and a trump only about **32%**.
- **Why:** a side ace is safest before the opponents have had a chance to throw that suit away and trump it.
- **After that,** about half the declarer's leads in tricks 2–3 are trumps, and **60%** from trick 4 on, while trumps are still out.
- **"Draw every trump first" isn't the expert's rule.** "Cash a side ace, then start drawing trumps" is closer.

### 10. Don't rush the Joker as declarer

In suit contracts the declarer's Joker is played:

| Tricks | Share |
|---|---|
| 1–2 | 24% |
| 3–5 | **48%** |
| 6–10 | 28% |

Draw the first round of trumps with lower ones if you can, and keep the Joker to regain control later.

### 11. Second hand low

When following suit second, the expert plays low:

| Contract | Plays low |
|---|---|
| Suit contracts | **64%** |
| No trumps | **94%** |

Save your high cards for when they capture something.

### 12. Trump the opponents' winners, not your partner's

When you can't follow suit:

| The trick | The expert |
|---|---|
| An opponent is winning and a trump would take it | **Trumps 88%** of the time |
| Partner is winning with a sure winner | **Discards 95%** of the time |
| Partner is winning but could still be beaten | Trumps **55%**: protect the trick when a later player might take it |

### 13. As a defender, lead low from a side suit more often than an ace

The defenders' first lead in a suit contract:

| Lead | Share |
|---|---|
| Low card from a side suit | **50%** |
| A side winner | **38%** |
| A trump | **11%** |

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

## Myths these numbers correct

- **"Count partner for two tricks."**
  - Partner plus the kitty are worth about **3** to the bidder.
  - Rule-of-thumb bots that counted 2 bid too little and lost.
- **"Double dummy is what you'll take."**
  - Solving with all cards visible overstates real declarer play by about **half a trick**, because nobody at the table can see the hidden cards.
  - *Source: lab, `calibrate`.*
