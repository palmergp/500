# 500 (Five Hundred): Rules Specification and Variants

Research document 01 · Researched 2026-10-17 · Scope: the trick-taking game Five Hundred (Euchre family). "Rummy 500" / "500 Rum" is a different game and is not covered.

---

## Contents

1. [How to read this document](#1-how-to-read-this-document)
2. [Recommended default ruleset ("AU-Standard")](#2-recommended-default-ruleset-au-standard)
3. [Rules in detail, with variants](#3-rules-in-detail-with-variants)
4. [Regional variations](#4-regional-variations)
5. [Variant toggles](#5-variant-toggles)
6. [Edge cases a digital implementation must decide](#6-edge-cases-a-digital-implementation-must-decide)
7. [Conflicts the user should decide](#7-conflicts-the-user-should-decide)
8. [Sources](#8-sources)

---

## 1. How to read this document

**Purpose.** This is the rules baseline for the engine. Section 2 is a default ruleset written to be implemented directly. Sections 3 and 4 give every rule with its variants and sources. Section 5 turns the variants into configuration toggles and presets. Sections 6 and 7 list what the sources leave open or disagree on.

**Citations.** Bracketed keys such as [pagat] or [nz2026] are links to the source URL; the full list with notes is in section 8. Wikipedia is cited only where it adds something; much of its variant material is tagged as unsourced, and this document says so where it matters.

**No single authority exists.** pagat's maintainer writes that "there is no official and universally accepted set of rules for Australian 500". He names the play of the Joker, the bidding procedure, misère, and the scoring by the contractor's opponents as the main disputed areas [pagat]. The closest things to official written rules are:
- the Clubs New Zealand tournament rules [nz2026];
- the Quebec FADOQ tournament rules [fadoq];
- the historical United States Playing Card Company (USPCC) rules, as reproduced in Hoyle editions [hsg] [foster1914] [hgm].

Digital implementations (NeuralPlay, Trickster, Board Game Arena) are cited as evidence of what players expect, not as authorities.

**Prevalence labels** used in sections 3 to 5:

| Label | Meaning |
|---|---|
| **Standard** | The main rule in pagat's Australian description, consistent with most other sources. |
| **Common** | Reported by several independent sources, or the default in several digital implementations. |
| **Some** | Reported as a variant ("some players …") by pagat or another source. |
| **Rare** | A single source, a named house rule, or unsourced on Wikipedia. |
| **Regional** | Documented for one body or place (NZ clubs, St Paul MN, Youngstown OH, Quebec, Tasmania). |
| **Historical US** | Original USPCC-era rules from Hoyle editions, 1904 to 1924. |

Nobody has surveyed how widely each rule is played. The labels describe the sources, not counted players.

**Terminology.**
- **Contractor** (also declarer or maker): the player who wins the auction.
- **Kitty** (also widow, blind, middle, *veuve*): the face-down cards left over after the deal.
- **Eldest:** the player on the dealer's left.
- **Right bower:** the jack of trumps. **Left bower:** the other jack of the same colour.
- **Odd tricks:** pagat's term for tricks won by the defenders.
- **Out the back door:** losing by falling to −500.

---

## 2. Recommended default ruleset ("AU-Standard")

This is the standard Australian four-handed game as pagat describes it [pagat]. Where pagat is silent, the most common practice fills the gap. Rules marked **(decide)** are where sources conflict; see section 7. The rule IDs (R1, R2, …) are meant to be referenced from code and tests.

### 2.1 Table and cards

- **R1.** Four players in two fixed partnerships, partners sitting opposite. Dealing, bidding and play all go clockwise [pagat].
- **R2.** The first dealer is chosen at random. After each hand the deal passes one seat clockwise [pagat].
- **R3.** The pack has 43 cards [pagat] [hsg] [catsatcards]:
  - ♥ and ♦: A K Q J 10 9 8 7 6 5 4 (11 cards each);
  - ♠ and ♣: A K Q J 10 9 8 7 6 5 (10 cards each);
  - one Joker. In Australia it is called the "bird", because the Australian 500 pack shows a kookaburra [pagat].
- **R4.** Each player gets 10 cards. The remaining 3 form the kitty, face down [pagat].
- **R5.** The deal goes clockwise, starting with eldest [hsg] [zpag]. The pattern is 3 to each player, 1 to the kitty, 4 to each, 1 to the kitty, 3 to each, 1 to the kitty [pagat].
  - In software the pattern is cosmetic. A uniformly random shuffle gives the same distribution whatever the pattern, so it matters only for animation.
- **R6.** There is no redeal for a weak hand, such as one with no face cards [nz2026] [play500]. Physical misdeals (exposed card, wrong count) cannot happen in software.

### 2.2 Rank of cards

- **R7. Suit contracts.** With trump suit T, trumps rank from high to low [pagat]:
  1. Joker;
  2. jack of T (right bower);
  3. jack of the other suit of the same colour (left bower);
  4. A K Q 10 9 8 … down to the lowest card of T.

  "The joker and left bower behave in all respects as members of the trump suit" [pagat]. This includes following suit: the left bower is not a card of its printed suit. The other suit of the trump colour therefore has no jack.
- **R8.** Non-trump suits rank A K Q J 10 9 … down to 5 (black suits) or 4 (red suits). The same-colour suit has no jack [pagat].
- **R9.** In the 43-card pack there are 13 trumps when a red suit is trumps and 12 when a black suit is [play500]. Table 3.1b gives the full breakdown.
- **R10. No trumps (NT), misère and open misère.** All four suits rank A K Q J 10 …, with jacks in their natural place. The Joker follows R25 to R31 [pagat].

### 2.3 Bidding

- **R11.** Eldest calls first, then calls go clockwise [pagat].
- **R12.** A call is either Pass or a bid. A bid is one of:
  - a number of tricks from 6 to 10 plus a trump suit or No Trumps;
  - Misère;
  - Open Misère.

  Every bid must rank higher than the bid before it [pagat].
- **R13.** Bids rank, lowest to highest [pagat]:

  6♠ 6♣ 6♦ 6♥ 6NT · 7♠ 7♣ 7♦ 7♥ 7NT · **Misère** · 8♠ 8♣ 8♦ 8♥ 8NT · 9♠ 9♣ 9♦ 9♥ 9NT · 10♠ 10♣ 10♦ · **Open Misère** · 10♥ 10NT

  Table 3.5a gives the values and lists which bids each one beats.
- **R14.** Misère may be bid only when the current highest bid is a 7-level bid. pagat: "Misere cannot be bid directly over a six bid, or when no one has made a positive bid" [pagat]. Every 8-bid outranks misère, so misère can never be bid over an 8-bid either.
- **R15.** Open Misère may be bid over any lower bid, including as the first bid of the auction [pagat].
- **R16.** A player who has passed may not bid again in that auction [pagat].
- **R17.** Partners may overcall each other. No source forbids it, and Board Game Arena states it explicitly [bga].
- **R18.** The auction ends when all players except one have passed. The last (highest) bid is the contract, and the player who made it is the contractor. After the others pass, the contract is fixed: the contractor may not raise or change it [pagat].
- **R19.** If all four players pass, the cards are thrown in with no score, and the deal passes to the next dealer [pagat] [catsatcards] [hsg].

### 2.4 Kitty

- **R20.** The contractor picks up the three kitty cards without showing them to anyone, then discards any three of the 13 cards face down [pagat]. This applies to every contract, including Misère and Open Misère.
  - The discards may include cards just taken from the kitty [pagat].
  - The discards are out of play for the hand and count for nothing.
  - No other player sees the kitty or the discards, the contractor's partner included [pagat] [nz2026].

### 2.5 Play

- **R21.** In every contract the contractor leads to the first trick, and the winner of each trick leads to the next [pagat].
- **R22.** A player must follow suit if able, where "suit" means the card's suit under R7 or R25. A player who cannot follow may play any card, subject to the Joker rules R27 and R28 [pagat].
- **R23.** The trick goes to the highest trump played. If no trump was played, it goes to the highest card of the suit led. In NT and misère an un-nominated Joker wins any trick it is played to [pagat].
- **R24. Misère and Open Misère.**
  - Both are played at no trumps. The contractor plays alone.
  - The contractor's partner lays their hand face down and takes no part, so each trick has three cards [pagat].
  - In Open Misère the contractor lays the remaining cards face up once the first trick is complete, and plays the rest of the hand exposed [pagat].

### 2.6 The Joker in NT, Misère and Open Misère

- **R25. Nomination.** A contractor who holds the Joker may nominate a suit for it. This must be done "at the start of play, before the lead to the first trick" [pagat]. The Joker is then the highest card of that suit for the whole hand and behaves exactly like a card of that suit. Nothing requires the contractor to hold other cards of the nominated suit [pagat].
- **R26. Un-nominated Joker.** If the Joker is not nominated (including whenever any other player holds it), it belongs to no suit. It is the highest card in the pack and wins any trick it is played to [pagat].
- **R27.** When another player has led, a player may play the Joker only if they have no card of the suit led [pagat].
- **R28.** In Misère and Open Misère, a player who cannot follow suit and holds the Joker must play it. In NT they may play it or discard another card [pagat].
- **R29.** The Joker may be led. The leader names a suit, which the others must follow if they can, and the Joker wins the trick. The named suit must not have been led earlier in the hand [pagat].
- **R30.** Once all four suits have been led, the Joker may not be led except to the last trick [pagat].
- **R31.** This follows from R26 to R30 and needs no separate code: a misère contractor who keeps the Joker without nominating it will lose. The Joker must be played eventually, and it then wins a trick [pagat].

### 2.7 End of play

- **R32. Suit and NT contracts.** All ten tricks are played, because the defenders' trick points (R37) and the slam (R35) depend on them. The engine may auto-complete once no remaining card can change any score.
- **R33. Misère and Open Misère.** The hand ends as soon as the contractor wins a trick. No source states this. Under R37 nothing else can change once the contract has failed, so it affects only display and statistics. (decide)

### 2.8 Scoring

- **R34.** Contracts are valued from the Avondale table (table 3.8a) [pagat] [hgm] [hsg].
- **R35. Contract made.** The contractors score the bid value. Overtricks score nothing [pagat]. The exception is a slam: if they take all 10 tricks on a bid worth less than 250 (8♠ or below), they score 250 instead [pagat]. There is no slam bonus on 8♣ or higher [pagat].
- **R36. Contract failed.** The contractors lose the bid value. Scores may go negative [pagat].
- **R37. Defenders.** In suit and NT contracts the defenders score 10 for each trick they win, whether or not the contract is made [pagat]. In Misère and Open Misère they score nothing [pagat].
- **R38. Misère contracts.** Misère scores +250 if the contractor takes no tricks and −250 otherwise. Open Misère scores +500 or −500 on the same basis [pagat].
- **R39. Must bid to win.** Trick points won while defending can never win the game [pagat]. Default: defending trick points cannot take a team above 490 [nz2026] [trickster-rules] [neuralplay] [bga] [catsatcards] [play500]:
  - if the team's score is below 490, new score = min(old score + 10 × tricks, 490);
  - if it is 490 or more, defending tricks score nothing.

  (decide: pagat's main text has no cap, only the "can't win by defending" principle; see 3.9.)

### 2.9 End of game

- **R40.** A team wins if, at the end of a hand in which it made its contract, its score is 500 or more [pagat].
- **R41.** A team whose score is −500 or worse at the end of a hand loses, and the other team wins ("going out the back door") [pagat].
- **R42.** Under R39 to R41, at most one team can meet a win or loss condition in any hand: only the contractors can win, and only the contractors can lose points. No tie-break rule is needed. If the cap is turned off, see 3.9.

**Score granularity.** Every score in this ruleset is a multiple of 10. Engines can store scores as integer tens. The one exception is the five-player half-share of a 250 slam; see edge case E28.

---

## 3. Rules in detail, with variants

### 3.1 Players, partnerships and packs

**Table 3.1a: Packs by player count**

| Players | Pack | Composition | Hand | Kitty | Sides | Sources |
|---|---|---|---|---|---|---|
| 2 | 43 | As four-player | 10 in hand plus table piles | 3 | One against one, with complex table piles | [pagat] |
| 3 | 33 | A to 7 in each suit, plus Joker | 10 | 3 | Contractor alone against a temporary partnership of the other two; scores kept per player | [pagat] [hgm] [hsg] |
| **4 (standard)** | **43** | Red A to 4, black A to 5, plus Joker | **10** | **3** | Fixed pairs, partners opposite | [pagat] [hsg] [catsatcards] |
| 4 (no Joker) | 42 | As above without the Joker | 10 | 2 | Fixed pairs | [pagat] [hsg] [zpag] |
| 4 (St Paul, Minnesota) | 45 | 52 cards minus 2s and 3s, plus Joker | 10 | 5 | Fixed pairs | [pagat] [denexa] |
| 4 (Quebec) | 46 | 52 cards minus 2s and 3s, plus **two** Jokers | 10 | 6 | Fixed pairs | [fadoq] [zpag] [pagat] |
| 5 | 53 | Full 52 plus Joker | 10 | 3 | Contractor calls a card to find a secret partner, or plays alone | [pagat] [hsg] |
| 6 | 63 | 52, plus 11s and 12s of all suits, plus 13s of ♥ and ♦, plus Joker | 10 | 3 | Two teams of three in alternating seats (Australia), or three pairs (US) | [pagat] [hsg] [zpag] |

Notes:
- **US Hoyle wording.** Hoyle's Standard Games (Laird & Lee, ©1908–1924, "courtesy of the United States Playing Card Company") describes the four-hand pack as "42-card pack, A (high) to 4 (low), deleting two 4's", and says "the joker may or may not be added" to any pack [hsg]. pagat says the early USPCC leaflets made the Joker optional [pagat]. A Quebec account of the American game names the black 4s as the ones removed [zpag].
- **Wikipedia variant (unsourced).** The 4♠ and 4♦ are removed instead, and the remaining four whose colour matches the trump suit becomes a trump, so there are always 13 trumps [wiki]. Rare.
- **History of the six-handed pack.** It predates the 1904 copyright: pagat says "In 1897 13's were added to the red suits to make a 63-card pack" [pagat].

**Table 3.1b: Suit composition of the 43-card pack**

| Contract | Trump suit (high to low) | Same-colour suit | Other two suits |
|---|---|---|---|
| ♥ trumps | 13: Joker, J♥, J♦, A K Q 10 9 8 7 6 5 4 | ♦: 10 cards (no jack) | ♠ 10, ♣ 10 |
| ♦ trumps | 13: Joker, J♦, J♥, A K Q 10 9 8 7 6 5 4 | ♥: 10 cards (no jack) | ♠ 10, ♣ 10 |
| ♠ trumps | 12: Joker, J♠, J♣, A K Q 10 9 8 7 6 5 | ♣: 9 cards (no jack) | ♥ 11, ♦ 11 |
| ♣ trumps | 12: Joker, J♣, J♠, A K Q 10 9 8 7 6 5 | ♠: 9 cards (no jack) | ♥ 11, ♦ 11 |
| NT and misère | none (Joker special) | — | ♥ 11, ♦ 11, ♠ 10, ♣ 10, Joker 1 |

These are arithmetic from R3, and agree with [play500].

**Six-player pack ranking.**
- The 11s, 12s and 13s rank "above the 10 and below the picture cards" [pagat]. Hoyle's ranking is "A (high), K, Q, J, 12, 11, 10, 9, etc." [hsg]. In plain suits this gives A K Q J 13 12 11 10 9 … 2, with 13s only in ♥ and ♦.
- A French-language Quebec account of the American six-hand game instead puts 13, 12 and 11 between the Q and the J in the two off-colour suits [zpag]. Rare.
- With hearts as trumps there are 18 trumps; with spades, 17 (arithmetic).
- If a 63-card pack is made from two ordinary packs, "the first played beats the second" when identical cards meet in a trick [pagat]. This does not arise in software.

**Partnerships and partner selection**
- **Three players.** The contractor plays alone against the other two, who form a temporary partnership; scores are kept individually [pagat]. Each defender scores 10 for each trick they personally win (US) [pagat] [hgm].
- **Five players, Australia** [pagat]:
  - After discarding, the contractor either announces that they are playing alone, or names "any specific card other than the joker or a bower".
  - The holder of that card becomes the partner but "must not say anything to reveal who they are". The partnership becomes known when the card is played.
  - Called-card variants:
    - no trump may be called;
    - any card except the Joker may be called;
    - "in some circles" a contractor secretly plays alone by calling a card they hold or have discarded;
    - books mention naming a player, or calling a card whose holder announces at once.
  - Misère and Open Misère are always played alone against the other four.
  - Scoring: "If the contractor calls a partner, they each win or lose half the value of the contract." A lone contractor "wins or loses the full contract value alone".
  - Winning: "If two players simultaneously reach 500 or more by winning a contract they both win."
  - pagat's five-player section does not say how defenders score; the four-player rule implies 10 per trick each.
- **Five players, US.** In pagat's American section the bidder names a specific player, not a card [pagat]. Hoyle lists three methods [hsg] [zpag]:
  - the bidder names a partner, who "can not refuse";
  - one partner on a bid of six or seven, two partners on a bid of eight, nine or ten;
  - the bidder calls the holder of a named card, who stays hidden until the card falls, or in some places announces at once.
- **Five-player Wikipedia note.** Wikipedia says misère and open misère are "usually not allowed" five-handed, and that scoring may be "full or half points each depending on the variant" [wiki].
- **Six players.**
  - Australia: two partnerships of three, partners sitting alternately. In misère "both partners of the contractor put down their cards and take no part" [pagat].
  - US Hoyle: "six-hand, three pairs of partners" [hsg]. The Quebec account seats the three pairs A1 B1 C1 A2 B2 C2 [zpag].

### 3.2 Dealing

| Rule set | Pattern | Kitty dealt | Sources |
|---|---|---|---|
| **Australia (standard)** | 3 each, 1 kitty, 4 each, 1 kitty, 3 each, 1 kitty | One card after each round of packets | [pagat] |
| US original (USPCC/Hoyle) | 3 each, widow, 4 each, 3 each | All three at once, after the first round | [hsg] [pagat] |
| NZ clubs | "3-4-3 or 3-2-3-2" (2024: also 3-3-4) | "Single cards to be dealt to the kitty (3). Last card goes to the dealer, not the kitty." | [nz2026] [nz2024] |
| St Paul (45 cards) | 3 each, 3 middle, 2 each, 2 middle, 3 each, 2 each | 5-card middle | [pagat] |
| Quebec FADOQ (46 cards) | 3 each, 2 widow, 2 each, 2 widow, 3 each, 2 widow, 2 each | 6-card widow | [fadoq] |
| Quebec traditional (46 cards) | 3 each, 3 widow (after the dealer's own packet), 4 each, 3 each, last 3 to widow | 6-card widow | [zpag] |

**First dealer.**
- Australia: random [pagat].
- US: draw cards, lowest deals; the ace is low and the Joker lowest of all [hsg] [catsatcards].
- St Paul: the first player to draw a jack deals [pagat].
- NZ clubs: "Scorer shall be first to deal"; when cutting, ties are broken with hearts high and spades low [nz2026].
- Quebec: lowest card deals [fadoq] [zpag].

**Rotation.** The deal passes clockwise (to the left) in every source [pagat] [hsg] [nz2026] [fadoq].

**Redeals for weak hands.**
- Standard: none. NZ clubs: "A player dealt no picture cards cannot throw in the cards for a new deal; this is a legal hand" [nz2026].
- Bryce Francis rejects redeals for hands without court cards [play500]. The Quebec account says a hand with neither ace nor face card is not a misdeal [zpag].
- Wikipedia: "in some versions" a player without a face card can call a misdeal [wiki].
- NeuralPlay offers an optional misdeal, callable during the first round of bidding by a player holding no ace, face card or Joker; the same dealer redeals [neuralplay].
- St Paul, "Ace – no face": a player with exactly one ace and no jack, queen, king or Joker may propose abandoning the deal [pagat] [denexa].
  - If the partner agrees, the hand is redealt; the opponents have no say.
  - If the partner refuses, play goes on, but that team may not bid Nullo or Grand Nullo.

**Physical misdeals** (exposed card, uncut pack, wrong card count) are covered by NZ clubs [nz2026], Hoyle [hsg] and the Quebec account [zpag]. They cannot occur in software.

### 3.3 Rank of cards

R7 to R10 give the standard ranking. Variants:
- Hoyle mentions that some players rank the trump ace above the bowers, which it calls "unusual" [hsg]. Rare.
- **"J5" no trumps:** in each suit the jack outranks the ace, and the Joker remains the only trump [wiki]. NeuralPlay offers it as an option [neuralplay]. Rare.
- **Two Jokers** (46-card pack):
  - Quebec: the white Joker (*la blanche*, *Fou Blanc*) outranks the coloured one [zpag] [wiki]. FADOQ lists "Blanche / Bonhomme" without stating an order [fadoq].
  - NeuralPlay and Trickster make the black Joker the higher [neuralplay] [trickster].
  - Make the order configurable.

### 3.4 The Joker in no trumps and misère

Section 2.6 gives the standard Australian rule. Every variant found:

**Table 3.4: Joker variants (NT, misère, open misère)**

| # | Question | Variant | Prevalence | Sources |
|---|---|---|---|---|
| J1 | May the contractor give the Joker a suit before play? | Yes, in NT, misère and OM (R25) | **Standard** AU | [pagat] |
| | | Must declare its suit; if not declared, it is treated as if another player held it | Some | [catsatcards] |
| | | Not in NT; still allowed in misère | Some | [pagat] |
| | | Never: the Joker cannot belong to a suit | Historical US | [pagat] (American section) |
| J2 | When may it be played to another player's lead? | Only when void in the suit led (R27) | **Standard** everywhere | [pagat] [nz2026] [hsg] [zpag] [trickster] |
| | | At any time | Rare | [neuralplay] (option); [gamerules] (reader comment) |
| | | Not on a suit the holder has already shown void in | Some ("many players") | [pagat] |
| | | Explicitly allowed on a suit already discarded on: the holder "may lay off a led suit and use the joker later on to trump in on that same suit" | Regional (NZ clubs) | [nz2026] |
| | | Played on a suit already renounced, it has "no winning value" | Historical (Hoffmann, 3-hand) | [hgm] |
| | | Only as "either the first or last card you play of any suit" (Gadd) | Rare | [pagat] [wiki] |
| | | "May not 'renege' with the joker" except when it is the last card | Rare (unsourced) | [wiki] |
| J3 | Must a void player play it? | Misère and OM: yes. NT: optional (R28) | **Standard** AU | [pagat] |
| | | Never compulsory | Historical US; Quebec | [pagat] [zpag] |
| J4 | Leading it: which suits may be named? | A suit not previously led (R29) | **Standard** AU | [pagat] [catsatcards] |
| | | Any suit except one the leader has shown void in | Historical US; some AU; QLD | [pagat] [hgm] [qld500] [denexa] |
| | | Any suit, "regardless of his previous play", including one the leader still holds | Regional (NZ clubs); USPCC-era Hoyle | [nz2026] [hsg] [wiki] |
| | | Only a suit that has already been led (by the contractor) | Regional (Quebec) | [zpag] [fadoq] |
| J5 | Leading it: on which tricks? | Until all four suits have been led, then only to the last trick (R30) | **Standard** AU | [pagat] |
| | | Any trick | Historical US; NZ | [pagat] [nz2026] |
| | | Never, except to the last trick (McNabb) | Rare | [pagat] |
| | | "Only in the first or last trick" | Rare | [gamerules] |
| J6 | Misère contractor holding the Joker | May keep it and nominate a suit (R25) | **Standard** AU | [pagat] |
| | | Must discard it ("lay it away in his discard") | Historical US (Nullo); QLD local rule | [hsg] [qld500] |
| J7 | Misère defender holding the Joker | Must play it when void (R28) | **Standard** AU | [pagat] |
| | | "It is the last card played" | Rare (QLD 2-player local rule) | [qld500] |
| J8 | The Joker's rank in misère | Highest card | **Standard** | [pagat] |
| | | Lowest card, not a trump | Some (app options) | [trickster-rules] [neuralplay] |

**The Joker in the kitty.**
- Australia: no special rule. The contractor takes it with the kitty and may keep it, nominate it (in NT or misère) or discard it, since the discard is "any three cards" [pagat].
- US Nullo: a bidder "holding the joker, or getting it in the widow, must discard it" [hsg].
- Quebec FADOQ: a taker who holds one Joker and finds the other in the widow must show it to the table, but only if the contract is 6 or 7 [fadoq].
- NeuralPlay has an option that places the highest Joker in the kitty before the deal [neuralplay]. Rare.

**The Joker in suit contracts.** It is the highest trump and a member of the trump suit [pagat]. NZ clubs: it "must be played if necessary to follow suit and can be played even if you have suit cards still in your hand" [nz2026]. Read "suit cards" as other trumps; the rule does not let a player who can follow a non-trump lead play the Joker instead. When a non-trump suit is led, a player who can follow must do so and may not play the Joker.

**Quebec, two Jokers.** In trumps both Jokers are trumps. In NT they belong to no suit and may be played only when void [zpag]. FADOQ: a player holding both Jokers "must bid at least 8" [fadoq].

### 3.5 Bidding

**Table 3.5a: Standard Australian bid ladder** [pagat]

| Rank | Bid | Avondale value | Notes |
|---|---|---|---|
| 1 | 6♠ | 40 | Lowest bid |
| 2 | 6♣ | 60 | |
| 3 | 6♦ | 80 | |
| 4 | 6♥ | 100 | |
| 5 | 6NT | 120 | |
| 6 | 7♠ | 140 | Misère becomes legal once a 7-bid is the high bid |
| 7 | 7♣ | 160 | |
| 8 | 7♦ | 180 | |
| 9 | 7♥ | 200 | |
| 10 | 7NT | 220 | |
| 11 | **Misère** | 250 | Beats every bid up to 7NT. Legal only over a 7-level bid (R14) |
| 12 | 8♠ | 240 | Beats misère even though its value is lower |
| 13 | 8♣ | 260 | |
| 14 | 8♦ | 280 | |
| 15 | 8♥ | 300 | |
| 16 | 8NT | 320 | |
| 17 | 9♠ | 340 | |
| 18 | 9♣ | 360 | |
| 19 | 9♦ | 380 | |
| 20 | 9♥ | 400 | |
| 21 | 9NT | 420 | |
| 22 | 10♠ | 440 | |
| 23 | 10♣ | 460 | |
| 24 | 10♦ | 480 | |
| 25 | **Open Misère** | 500 | Beats every bid up to 10♦, misère included. Legal at any time (R15) |
| 26 | 10♥ | 500 | Beats open misère despite the equal value |
| 27 | 10NT | 520 | Highest bid |

Rank and value differ in two places: 8♠ (240) beats misère (250), and 10♥ (500) beats open misère (500). Engines must therefore compare bids by their position in an explicit ordered list, never by value.

**Table 3.5b: Placement and value variants for misère and open misère**

| Variant | Misère | Open Misère | Prevalence | Sources |
|---|---|---|---|---|
| **AU standard** | 250. Between 7NT and 8♠. Only over a 7-level bid | 500. Between 10♦ and 10♥. Any time | **Standard**; matches BGA. NeuralPlay's Australian preset uses the same entry rules; Trickster matches for misère only | [pagat] [bga] [neuralplay] [trickster-rules] [catsatcards] [gamerules] |
| By value (American "Nullo"; Paul Pinn) | 250. Above 8♠, below 8♣ | No open nullo in the historical US sources; if added at 500 it ties with 10♥ | Historical US; some AU | [pagat] [hsg] [zpag] |
| Misère before any 7-bid | Any time | — | Some (common in 3-handed) | [pagat] [wiki] |
| Misère not allowed | — | — | Some AU; **NZ clubs** | [pagat] [nz2026] |
| Open misère only after an 8-bid | — | Needs a prior 8-bid | Some | [pagat] [trickster-rules] |
| Open misère between 9NT and 10♠ (pagat "some"); QLD local rule adds "only after a 7-bid" | QLD: only after a 7-bid; any 8-bid beats it | Between 9NT and 10♠, so any 10-bid beats it | Some; QLD local rule | [pagat] [qld500] |
| Open misère above 10♥ | — | Only 10NT beats it | Common in apps (Trickster default) | [trickster-rules] |
| Open misère the highest bid | — | Above 10NT | Some | [pagat] [britannica] |
| Open misère equal in rank to 10♥ or to 10NT | — | Ties with 10♥ or 10NT | Some | [pagat] |
| Tasmanian police | Only after a 7-bid | 520, equal in rank to 10NT | Regional | [pagat] |
| Roger Knights | 150, between 7♠ and 7♣ | 250, between 8♠ and 8♣ (plus "misère without pickup", 350, between 9♠ and 9♣) | Rare | [pagat] |
| "Reduced" (BGA options) | 130, between the 6 and 7 levels | 330, between the 8 and 9 levels | Rare | [bga] [wiki] |
| Trickster value options | 210 (only 7NT overcalls), 230 (any 8-bid overcalls), 250 | 330, 430, 500 or 520 | App options | [trickster-rules] |
| Brent Easton (5-player) | Banned | 230; "Super Open Misère" 430, in which the opponents see and play the hand | Rare | [pagat] |
| Quebec Nullo | 500, equal to 10♥. The partner passes the bidder one card; no widow | — | Regional | [zpag] |

**Table 3.5c: Auction procedure variants**

| Aspect | Variant | Prevalence | Sources |
|---|---|---|---|
| Rounds | As many rounds as needed; a pass is final (R16) | **Standard** AU | [pagat] |
| | One round: each player bids once or passes | Historical US; Quebec | [pagat] [hsg] [fadoq] [zpag] |
| Bidding after passing | Allowed; the auction continues until three passes in a row | Some | [pagat] |
| | Allowed only if the high bidder changes suit ("New Zealand rules") | Regional (NZ clubs) | [nz2026] [trickster-rules] |
| Raising your own bid | Not allowed (R18). US: "A player cannot raise his own bid if all other players pass" | **Standard**; historical US; Quebec | [pagat] [hsg] [zpag] |
| | Allowed after the others pass, but not if nobody else bid. A change of suit lets the others bid again; a higher trick count in the same suit does not | Some | [pagat] [wiki] |
| | NZ: with two or more bidders, the holder of the bid "may change suit once or increase the bid". A sole first-round bidder may increase once without changing suit. Either way the others may re-enter | Regional | [nz2026] |
| | Any player may change their call at any time if the change escalates; a change of suit reopens bidding | Rare | [play500] |
| Minimum bid | 6 | **Standard** | [pagat] |
| | Inkle: 6-level bids only from the first two bidders, to signal partner; an inkle cannot win the auction. If nobody bids 7 or more, the same dealer redeals (St Paul) | Regional US | [pagat] [trickster] [neuralplay] [bga] |
| | Minimum 7 | App preset ("Canadian") | [neuralplay] |
| | Holder of both Jokers must bid at least 8 | Regional (Quebec) | [fadoq] |
| All pass | Thrown in, no score; the deal passes | **Standard** AU; historical US | [pagat] [catsatcards] [hsg] |
| | Same dealer redeals | Regional (St Paul) | [pagat] |
| | Redeal by the same dealer at most twice, then the deal passes | Rare (unsourced) | [wiki] |
| | Hand played at NT without the kitty; eldest leads; each side scores 10 per trick | Regional (NZ clubs); US by agreement; 3-hand Hoyle | [nz2026] [pagat] [hsg] [hgm] |
| | The dealer must bid ("stick the dealer") | Rare | [wiki] [neuralplay] |
| Calls out of turn or too low | NZ: an out-of-turn bid stands; a too-low bid must be raised in that suit | Regional; not needed in software | [nz2026] |

**Bidding conventions** (signalling agreements, not rules; relevant to AI styles):
- Youngstown description bids: "6 No Trump" shows the Joker, "6 No" two aces, a 6-bid in a suit that suit's bower [pagat].
- St Paul "No" bids show the Joker or split bowers [pagat].
- NeuralPlay offers "6NT shows a joker" as an option [neuralplay].

**Other special bids** (all Rare or Regional; off by default):
- Blind misère, bid before looking at the cards; Wikipedia gives 1000 (unsourced) [wiki] [gamerules].
- Grand Nullo ("Granola"), St Paul. Worth 510: beats 10♥, loses to 10NT. It may be bid only if the bidder's partner opened Nullo [pagat] [denexa].
- Grand Slam call after the kitty: 500 in a suit, 520 in NT. The partner passes one card and sits out [pagat] [wiki].
- Hi/Lo ("Even Stevens"): take exactly 5 tricks, usually worth 350 [wiki].
- Patastrophe: double open misère, worth 1000 [wiki].
- "Prendre la partie" (Quebec): a 10-trick call that, if made, scores 1000 and ends the game [zpag].
- "Sans kitty" bids [gamerules], and Roger Knights's "misère without pickup" [pagat].

### 3.6 The kitty

- **Australia:** see R20 [pagat].
- **NZ clubs** [nz2026]:
  - "The Kitty may be viewed by the maker (winning bidder) only, who may, or may not, exchange the 3 cards."
  - The kitty and discards "may not be viewed or altered after the first card has been led".
  - The maker revealing a kitty card, or the partner looking at one, forfeits the bid and gives the opponents 100 points.
- **US:** the bidder "may retain part or all or none of the cards taken up" [hsg]. Hoyle Modernized: nobody may inspect the discards [hgm].
- **Quebec:** six-card widow, six discards [fadoq] [zpag]; the second-Joker display rule is in 3.4 [fadoq].
- **Misère and the kitty:**
  - Standard AU: the contractor takes the kitty as for any contract [pagat].
  - Trickster has an option: the misère contractor takes the kitty always, never, or unless the contract is open misère [trickster-rules].
  - Roger Knights scores "misère without pickup" as a separate bid [pagat].
  - Quebec Nullo uses no widow: the partner passes one card and the bidder discards one [zpag].
- **NZ all-pass hand:** the kitty is not picked up [nz2026].

### 3.7 Play of the hand

**Opening lead.**
- The contractor leads to trick 1, in Australia [pagat], in Quebec [fadoq] [zpag] and in early US rules [foster1914] [hsg].
- Hoyle: "(In some localities player at dealer's left leads.)" [hsg]. NeuralPlay offers this as an option [neuralplay].
- Nullo (US): "Bidder leads" [hsg].
- NZ all-pass NT hand: "The lead is to the left of the dealer" [nz2026].

**Following suit.**
- Every source requires players to follow suit if able; otherwise any card may be played [pagat] [hsg] [nz2026] [zpag].
- Hoyle: "It is not obligatory to lead trumps" [hsg].

**The misère partner.**
- Australia: lays their cards face down and takes no part [pagat]. In the six-player game, both partners do so [pagat].
- Quebec Nullo: the partner first gives the bidder their lowest or least dangerous card; the bidder then discards one card [zpag].
- Trickster offers a "Both Hands" option in which the bidder also plays the partner's hand face up [trickster-rules]. Rare.

**When the open-misère hand is exposed.**
- After the first trick: "after the first trick has been played, the contractor arranges his cards face up on the table" [pagat] [wiki] [bga] [gamerules]. **Standard**.
- Before the opening lead: "the hand is spread faceup before the opening lead" [britannica]. Some.

**Early termination.**
- pagat gives no rule [pagat].
- US: "If any player scores out during play of a hand, balance of hand is not played, unless the bidder can win out" [hsg]. Wikipedia cites the same rule to the USPCC's 1948 *Official Rules of Card Games* [wiki]. This matters because US defenders can win on trick points.
- Quebec FADOQ: all cards must be played before deciding whether the contract succeeded [fadoq].
- St Paul style: "The hand must be played out to the end", nullo hands included [denexa].
- Britannica, where defenders score per trick taken by the misère declarer: "misères must be played right through" [britannica].
- NeuralPlay has an option to end the hand once the contract result and all scores are determined [neuralplay].

**Claims and laydowns.** NZ clubs treat a maker who "lays down or shows the remaining balance of their hand" as losing the bid, with 100 points to the opponents. If the defenders lay down, the makers score the bid [nz2026]. Claims are therefore not part of tournament play. In software they can only be an optional convenience, verified by a solver.

**Revokes and penalties.** None of these apply when the engine enforces legal play.
- NZ: if the makers renege, the hand ends; they lose the bid and the opponents get 100. If the defenders renege, the makers score the bid and the defenders score nothing [nz2026].
- US: a revoking bidder is set back the bid. If an opponent revokes, the bidder scores the bid and the offending side scores nothing [hsg] [hgm].

### 3.8 Scoring

**Table 3.8a: Avondale schedule** (standard everywhere today) [pagat] [hgm] [hsg] [catsatcards] [neuralplay]

| Tricks | ♠ | ♣ | ♦ | ♥ | NT |
|---|---|---|---|---|---|
| 6 | 40 | 60 | 80 | 100 | 120 |
| 7 | 140 | 160 | 180 | 200 | 220 |
| 8 | 240 | 260 | 280 | 300 | 320 |
| 9 | 340 | 360 | 380 | 400 | 420 |
| 10 | 440 | 460 | 480 | 500 | 520 |
| Misère | 250 | | | | |
| Open Misère | 500 | | | | |

The formula is value = 100 × (tricks − 6) + 40 + 20 × suitIndex, with ♠ = 0, ♣ = 1, ♦ = 2, ♥ = 3, NT = 4. Hoyle Modernized calls it "the best and most modern" schedule, in which no two suit bids are equal [hgm]. Wikipedia dates the USPCC's Avondale table to 1906, introduced "to remove bidding irregularities" [wiki]. The FADOQ PDF prints 200 for 10♥, evidently a typo for 500 [fadoq].

**Table 3.8b: Original 1904 schedule** [foster1914] [hsg]

| Tricks | ♠ | ♣ | ♦ | ♥ | NT |
|---|---|---|---|---|---|
| 6 | 40 | 60 | 80 | 100 | 120 |
| 7 | 80 | 120 | 160 | 200 | 240 |
| 8 | 120 | 180 | 240 | 300 | 360 |
| 9 | 160 | 240 | 320 | 400 | 480 |
| 10 | 200 | 300 | 400 | 500 | 600 |

In the original schedule, bids are ranked by value: "a bid of seven in hearts outranks eight in clubs" [foster1914]. Some bids tie, such as 7♣, 6NT and 8♠, all worth 120 [hsg]. Wikipedia's "Original" table appears transposed relative to both primary sources.

**Other schedules.**
- **Inverted:** the same as the original but with ♣ lowest, then ♠, ♥, ♦, NT [hsg] [catsatcards]. Rare.
- **"Perfect":** each Avondale value minus 20, so 6♠ = 20 and 10NT = 500 [wiki] [neuralplay]. Rare.

**Scoring rules and variants**

| Item | Rule | Prevalence | Sources |
|---|---|---|---|
| Contract made | + bid value; overtricks score nothing | **Standard** | [pagat] [hsg] [zpag] |
| Slam (all 10 tricks) | Bids worth less than 250 score 250 instead; no bonus on 8♣ or higher | **Standard**; historical US | [pagat] [foster1914] [hsg] [hgm] [play500] |
| | No slam bonus | Some | [pagat] |
| | +100 for all ten tricks on any bid under ten (e.g. 8♣ with all ten tricks = 360) | Regional (Youngstown) | [pagat] |
| | A slam scores the value for the tricks actually taken | Rare (Quebec account, American variant) | [zpag] |
| Contract failed | − bid value | **Standard** | [pagat] |
| | The opponents score the bid value instead; no negative scores | Regional (Tasmanian police; traditional Quebec; NeuralPlay "Canadian") | [pagat] [zpag] [neuralplay] |
| Defenders, suit/NT | +10 per trick won, whether or not the contract is made | **Standard**; historical US; FADOQ | [pagat] [hsg] [fadoq] |
| | Only when the contract fails | Some | [pagat] |
| | Never | Some ("a few players") | [pagat] |
| Defenders, misère/OM | Nothing | **Standard** AU | [pagat] |
| | +10 per trick the misère contractor takes | Historical US | [pagat] [hsg] [britannica] |
| | +10 per trick they take | Rare (Quebec account of US Nullo) | [zpag] |
| | The bid value (500) if the Nullo fails | Regional (Quebec Nullo) | [zpag] |
| Misère / OM | ±250 / ±500 | **Standard** | [pagat] |
| Penalty points | 100 to the opponents for maker infractions; may take a team out | Regional (NZ clubs) | [nz2026] |
| Card points | "1000" and "1500" games add card values to tricks | Rare | [hsg] [zpag] |

### 3.9 Winning and losing

**Table 3.9: End-of-game rule sets**

| Rule set | Win | Can defenders win on trick points? | Defending-points cap | Both reach 500 in one hand | Lose | Sources |
|---|---|---|---|---|---|---|
| **AU, pagat main text** | Reach 500+ "as a result of winning a contract" | No: "further hands are played until one team wins or loses" | None stated; points presumably keep accruing | Only the contractors can win | −500 or worse | [pagat] [pagat-end] |
| **AU common; NZ clubs 2025–26** | As above | No | **490**: "No more 10-point tricks can be taken beyond 490" | Only the contractors can win | −500 | [nz2026] [nz2025] [trickster-rules] [neuralplay] [bga] [catsatcards] [play500] |
| NZ clubs 2021–2024; some AU | As above | No | 460 | — | −500 | [nz2024] [nz-remit] [pagat] |
| Some AU | As above | No | No trick points at all once at 400+ | — | −500 | [pagat] |
| Historical US | First to 500 | **Yes** | None | The bidder counts first and wins if he makes his bid; otherwise the first player to reach 500 on tricks wins | −500 loses [zpag] [pagat] | [pagat] [hsg] [hgm] [foster1914] [zpag] |
| US partnership spread rule | "if one side is in the hole the other side wins if it get[s] 500 ahead" | — | — | — | — | [hsg] |
| Tasmanian police | First to 1000 | No trick points | — | — | No negative scores | [pagat] |
| Quebec traditional | First to 1000 or more; a team reaching 1000 while the opponents have 0 is a "Chicago" | — | — | — | No negative scores | [zpag] |
| Quebec FADOQ tournament | 8 deals per game; points summed across the tournament | — | — | — | — | [fadoq] |
| NZ clubs timed | "10 minutes duration or a maximum of 4 hands": first to 500, or highest score; ties possible | No | 490 | — | −500 | [nz2026] |
| Variant | The back door applies "only if the other team is not in the negative" | — | — | — | — | [wiki] (unsourced) |
| Variant | If both teams pass 500, the higher score wins | Yes | — | Highest wins | — | [wiki] [trickster-rules] |

**Why NZ moved from 460 to 490.** In timed games, the 460 cap could hand a hand-four win to the team that was behind. The 2025 remit shows the example and proposes 490 "to reduce any possible unfair outcomes" [nz-remit].

**Consequences of pagat's no-cap reading.** A team can sit at 500 or more from defending without having won. It then wins with any successful contract. If it fails a contract it may stay above 500, still without winning. The 490 cap avoids these states, which is one reason to choose it as the default (section 7, C1).

**Three and five players (US).** If two opponents of the bidder pass 500 in the same hand, "the player who first wins the trick that takes his score over 500 wins" [hgm] [pagat]. This requires trick-by-trick score tracking.

### 3.10 Summary of three-, five- and six-player differences from R1–R42

| Topic | 3 players | 5 players | 6 players |
|---|---|---|---|
| Pack / kitty | 33 / 3 | 53 / 3 | 63 / 3 |
| Sides | Contractor against two; individual scores | Called-card partner (half shares) or alone (full value) | 2 × 3 alternating (AU) or 3 × 2 (US) |
| Misère | Contractor against two (no partner to sit out); often allowed before a 7-bid [wiki] | Always alone against four [pagat] | Both partners sit out [pagat] |
| Defender points | Each defender scores their own tricks [pagat] [hgm] | Each defender scores their own tricks (US) [hsg] | Per team |
| Win | Individual; first player to 500 on a contract (AU) [pagat] | Several players may win together [pagat] | Per team |

---

## 4. Regional variations

### 4.1 Australia

- **Status.** "Five Hundred is now the national card game of Australia", although it was invented in the USA and the USPCC copyrighted the rules in 1904 [pagat].
- **Main description.** The standard rules are pagat's Australian four-handed rules (section 2) [pagat].
- **Documented Australian house rules** (pagat, "some players …") [pagat]:
  - Joker: J1 to J5 variants.
  - Bidding: bidding after passing; raising your own bid.
  - Misère: banned; allowed before any 7-bid; open misère only after an 8-bid.
  - Open misère ranking: between 9NT and 10♠, highest of all, or equal to 10♥ or 10NT; value 520.
  - Roger Knights's misère schedule.
  - Defenders: capped at 490, 460 or 400; scoring only when the contract fails; never scoring.
  - No slam bonus.
- **Tasmanian police variant** (Robert Reid) [pagat]:
  - The bid value is scored by the contractors if they win and by the opponents if they lose.
  - No trick points; "There are no negative scores".
  - Game is 1000.
  - Misère only after a 7-bid; open misère counts 520 and ranks equal to 10NT.
- **Queensland "500 for 2" local rules** (two-player tournaments; "Same rules apply as for normal 4-player 500" otherwise) [qld500]:
  - Misère and open misère only after a 7-bid; an 8-bid beats misère; any 10-bid beats open misère.
  - Joker lead: may not name a suit the leader has "previously claimed to be void in".
  - In misère the bidder must discard the Joker; a defender's Joker "is the last card played".
  - Defenders' trick points "cannot be used to win a game"; slam 250 on bids below 250.
- **Digital defaults.**
  - NeuralPlay "Australian" preset: misère after a 7-level bid, open misère any time, values 250 and 500, multi-round bidding, Avondale, 500-point game [neuralplay].
  - Trickster Australian: multi-round bidding; "must bid to win" defaults to Yes (defenders count only to 490) [trickster-rules].
  - Board Game Arena Australian bidding: misère only after a 7-bid; a team "can't win unless it won the bid … maximum score is 490" [bga].

### 4.2 New Zealand

- pagat: "by and large the same versions of 500 are played in both New Zealand and Australia" [pagat].
- **Clubs NZ singles tournament rules** (2024, 2025, 2026). In "singles", partnerships rotate between games [nz2026] [nz2025] [nz2024]:
  - **No misère or open misère:** "Misere and Open Misere are not legal bids."
  - **All pass:** "the Kitty is not picked up and the hand is played as NO Trumps. The lead is to the left of the dealer. 10 points are awarded for each trick taken. (Note the hand is not thrown in for a re-deal)."
  - **Bidding:**
    - Each new bid must have greater scoring value.
    - A player who changes suit lets everyone re-enter, "even if they have previously passed".
    - With two or more bidders, the player left with the bid "may change suit once or increase the bid", after which others may re-enter.
    - A sole first-round bidder may increase once, without changing suit.
  - **Kitty:** the maker alone may view it; nothing may be changed after the first lead; penalties apply.
  - **Joker in NT:**
    - It "is a suit by itself, but is also the highest card of any suit".
    - It may not be played if the holder can follow suit.
    - When led, the leader names the suit, and may do so "even if the player is holding cards still in their hand of that suit".
    - The holder "may lay off a led suit and use the joker later on to trump in on that same suit".
    - No pre-play nomination is mentioned.
  - **Scoring and game:**
    - "No more 10-point tricks can be taken beyond 490" (460 in the 2024 and earlier rules).
    - "You must make and win the bid to go out, unless being awarded 100 penalty points takes you to 500 or above."
    - Games last 10 minutes or 4 hands.
  - **Weak hands:** "A player dealt no picture cards cannot throw in the cards."
  - **Not stated:** who leads first in a normal contract. Assume the contractor, as in Australia.

### 4.3 USA

- **Original USPCC rules**, 1904 copyright, Avondale table 1906 [pagat] [wiki]. Compiled from Hoyle editions [hsg] [hgm] [foster1914] and pagat's American section [pagat]:
  - Originally a game "primarily and specifically" for three players [hgm]; partnerships for 4, 5 and 6 hands.
  - Deal 3, widow, 4, 3. One round of bidding: "Only one bid is allowed each player". A player cannot raise their own bid if the others pass [hsg].
  - Nullo (optional): 250, ranked between 8♠ and 8♣; the bidder leads and plays alone.
    - A Nullo bidder holding the Joker must discard it.
    - The opponents score 10 for each trick the bidder takes [hsg] [pagat].
  - Joker in NT [pagat] [hsg]:
    - It cannot be given a suit, and "is a suit by itself, and is a trump".
    - It may be played only when void, and is not compulsory then.
    - It may be led to any trick with a named suit. pagat's American summary forbids naming "a suit in which you have previously shown void"; Hoyle 1908–24 allows naming the suit "regardless of his previous play".
  - Scoring and game:
    - Defenders score 10 per trick and **can win on trick points**.
    - The bidder counts first; play stops once someone has scored out and the bidder cannot win.
    - A negative score is "in the hole"; the partnership spread rule applies [hsg].
  - All pass: "the cards are bunched and the deal passes to the left". In some localities the hand is played at NT for 10 per trick [hsg].
- **St Paul, Minnesota (West Seventh Street)** [pagat] [denexa]:
  - 45-card pack, 5-card middle.
  - Ace–no face redeal; inkles (first two bidders only); "No" bids.
  - Nullo by agreement; Grand Nullo 510; Grand Slam call (McQuston).
  - If no one bids above 6, the same dealer redeals.
  - Midway tournament form: no Nullo; rounds of four hands; no 500 target; the highest individual total wins.
- **Youngstown, Ohio** [pagat]: ten rounds of four hands, no 500 target; +100 for all ten tricks on a bid under ten; no Nullo; description bids.
- **Distribution:** Wikipedia says the game is still played in Ohio and Pennsylvania [wiki].

### 4.4 Canada and Quebec

- pagat: played as *Cinq Cents* in French Canada, especially Montreal, with 46 cards (two distinguishable Jokers as the highest trumps) and a six-card kitty [pagat].
- **FADOQ tournament rules** (Quebec seniors' federation, *Jeux en tête*) [fadoq]:
  - 46 cards; lowest card deals; the player on the dealer's right cuts.
  - Deal 3, 2 widow, 2, 2 widow, 3, 2 widow, 2.
  - One call per player, starting from the dealer's left and ending with the dealer. Minimum 6; a player holding both Jokers must bid at least 8. If all pass, the deal is replayed ("la brasse est reprise").
  - Taker: picks up the 6-card widow and discards 6. Must show a Joker found in the widow when already holding the other, for 6- and 7-level contracts only. Leads first.
  - Joker lead: a player who leads a Joker and calls a suit may do so only at the second card of that suit ("qu'à la 2e carte de cette couleur"), so on the first round of a suit they must play another card of it first.
  - All cards are played. Made contracts add the value; failed contracts subtract it; the defenders score 10 per trick.
  - 8 deals per game; scores summed across the tournament.
- **Traditional Quebec rules (French-language account)** [zpag]:
  - White Joker > coloured Joker > bowers.
  - One bid each; all pass, the deal is buried.
  - In NT a Joker may be led only to call a suit the contractor has already led.
  - A failed contract credits its value to the opponents; no negative scores; game is 1000.
  - "Prendre la partie": a 10-trick call worth the game. "Chicago": winning 1000 to 0.
  - Quebec Nullo: 500, equal to 10♥; the partner passes one card; no widow.
- **Wikipedia (unsourced)** describes a French-Canadian *petite misère* (500) and *la grosse* open misère (1000), a 1000-point target, failed contracts added to the other team, and "points are never subtracted" [wiki].
- **NeuralPlay "Canadian" preset** [neuralplay]:
  - Single-round bidding starting at 7; 46 cards, 2 Jokers, 6-card kitty.
  - No nullo, but open nullo allowed.
  - No ordinary defender trick points; defenders who set a contract receive its value.
  - Game ends at ±1000.
- **Ontario:** Wikipedia names Ontario as a place the game is played [wiki]. No written Ontario rules were found.

### 4.5 Other house rules (all Rare; mostly unsourced on Wikipedia)

- **Set rule:** a contractor who falls short by a set margin may not bid next hand [wiki].
- **Breach rule:** a defender may call "breach" against a 7+ bid before play [wiki].
- **Ralphing:** a bidder set by more than 3 tricks sits out the next auction [wiki].
- **Point-spread end:** the game ends when one team is negative and the other positive, 500 or more apart [wiki]. Compare the US Hoyle spread rule [hsg].
- **Walker Ultimate 500:** the winner must score exactly 500; reaching 1000 loses [wiki].
- **Card-point games of 1000 and 1500** [hsg] [zpag].

---

## 5. Variant toggles

Defaults are AU-Standard (section 2). "(decide)" marks defaults the user should confirm (section 7).

### 5.1 Toggle table

| Toggle | Options | Default | Prevalence | Sources |
|---|---|---|---|---|
| **Table and pack** | | | | |
| `player_count` | 3 / 4 / 5 / 6 (2 optional) | 4 | 4 is standard | [pagat] |
| `pack` | 43 (kitty 3) / 42, no Joker (kitty 2) / 45 (kitty 5) / 46 with two Jokers (kitty 6) | 43 | 43 standard AU and US; 45 St Paul; 46 Quebec | [pagat] [hsg] [fadoq] [neuralplay] [trickster] |
| `two_joker_order` | white (or black) Joker high / red high | white high | Quebec / apps | [zpag] [neuralplay] [trickster] |
| `six_player_teams` | 2 × 3 alternating / 3 × 2 | 2 × 3 | AU / US | [pagat] [hsg] [zpag] |
| `six_player_extra_rank` | J > 13 > 12 > 11 > 10 / Q > 13 > 12 > 11 > J | first | Standard / rare | [pagat] [hsg] [zpag] |
| `five_player_partner` | secret called card / called card announced at once / name a player / by bid level (1 partner at 6–7, 2 at 8–10) / always alone | secret called card | AU / some / US / US | [pagat] [hsg] [zpag] |
| `five_player_callable` | any card except Joker and bowers / no trumps / any except Joker | except Joker and bowers | AU standard / some / some | [pagat] |
| `five_player_secret_solo` | off / on (call a card you hold or discarded) | off | Some | [pagat] |
| `five_player_shares` | half each with a partner, full alone / full each | half | AU / variant | [pagat] [wiki] |
| **Deal** | | | | |
| `first_dealer` | random / cut, low deals | random | AU / US, Quebec | [pagat] [hsg] [fadoq] |
| `deal_pattern` (cosmetic) | 3-1-4-1-3-1 / 3-K3-4-3 / NZ 3-4-3 single kitty cards / FADOQ / St Paul | 3-1-4-1-3-1 | per region | [pagat] [hsg] [nz2026] [fadoq] |
| `weak_hand_redeal` | off / no face cards / no ace, face card or Joker / Ace–no face (partner consents; team loses Nullo) | off | NZ forbids; rare | [nz2026] [play500] [wiki] [neuralplay] [pagat] |
| **Auction** | | | | |
| `auction_style` | multi-round, pass final / single round / multi-round with re-entry always / re-entry when the high bidder changes suit (NZ) | multi-round, pass final | AU / US, Quebec / some / NZ | [pagat] [hsg] [fadoq] [nz2026] [trickster-rules] |
| `raise_own_bid` | no / yes if someone else bid (a suit change reopens bidding) / NZ rules | no | Standard / some / NZ | [pagat] [nz2026] [hsg] |
| `minimum_bid` | 6 / inkle (first two bidders may bid 6 to signal; cannot win) / 7 | 6 | Standard / US regional / app | [pagat] [trickster] [neuralplay] [bga] |
| `double_joker_min_bid` | off / holder of both Jokers must bid 8+ | off | Quebec | [fadoq] |
| `all_pass` | throw in, next dealer / throw in, same dealer / play NT, no kitty, eldest leads, 10 per trick / dealer must bid | throw in, next dealer | AU / St Paul / NZ / rare | [pagat] [catsatcards] [hsg] [nz2026] [wiki] [neuralplay] |
| `misere_enabled` | on / off | on | NZ clubs off; some AU off | [pagat] [nz2026] |
| `misere_entry` | only over a 7-level bid / any time | only over a 7-level bid | Standard / some, common in 3-handed | [pagat] [wiki] [bga] [trickster-rules] |
| `misere_rank` | between 7NT and 8♠ / by value (between 8♠ and 8♣) / custom slot | between 7NT and 8♠ | AU / US | [pagat] [hsg] [trickster-rules] [neuralplay] |
| `misere_value` | 250 / 230 / 210 / 150 / 130 | 250 | Standard / app options / Knights / BGA | [pagat] [trickster-rules] [bga] |
| `open_misere_enabled` | on / off | on | NZ clubs off | [pagat] [nz2026] |
| `open_misere_entry` | any time / only after a 7-bid / only after an 8-bid | any time | Standard / QLD local / some | [pagat] [qld500] [trickster-rules] |
| `open_misere_rank` | between 10♦ and 10♥ / between 10♥ and 10NT / above 10NT / equal to 10♥ / equal to 10NT / between 9NT and 10♠ / between 8NT and 9♠ | between 10♦ and 10♥ (decide) | Standard / Trickster default / some / some / Tasmania / QLD / BGA | [pagat] [trickster-rules] [britannica] [qld500] [bga] |
| `open_misere_value` | 500 / 520 / 430 / 330 / 250 | 500 | Standard / some / options | [pagat] [trickster-rules] [bga] |
| `special_bids` | none / blind misère / Grand Nullo / Grand Slam call / Hi-Lo / misère without kitty / prendre la partie | none | Rare | [wiki] [pagat] [zpag] |
| **Kitty** | | | | |
| `misere_takes_kitty` | yes / no / not for open misère | yes | Standard / options | [pagat] [trickster-rules] |
| `misere_must_discard_joker` | off / on | off | Standard / historical US Nullo, QLD | [hsg] [qld500] |
| `show_found_joker` | off / on (Quebec, 6- and 7-level contracts) | off | Quebec | [fadoq] |
| **Joker in NT and misère** | | | | |
| `joker_nomination` | NT and misère / misère only / never | NT and misère | Standard / some / US | [pagat] [catsatcards] |
| `joker_follow` | only when void / any time / first or last card of a suit (Gadd) | only when void | Standard / rare / rare | [pagat] [neuralplay] [wiki] |
| `joker_on_reneged_suit` | allowed / forbidden / allowed but cannot win | allowed (decide) | Standard (NZ explicit) / "many" AU / Hoffmann | [pagat] [nz2026] [hgm] |
| `joker_must_play_when_void` | in misère only / never | in misère only | Standard / US, Quebec (NeuralPlay offers it as an option) | [pagat] [zpag] [neuralplay] |
| `joker_lead_suit` | not yet led / not shown void by the leader / any suit / already led (Quebec) | not yet led (decide) | Standard / US, QLD / NZ, Hoyle / Quebec | [pagat] [hsg] [nz2026] [zpag] |
| `joker_lead_timing` | until all four suits led, then last trick only / any trick / last trick only (McNabb) / first or last trick only | until all four suits led | Standard / US, NZ / rare / rare | [pagat] [nz2026] [gamerules] |
| `joker_low_in_misere` | off / on | off | Standard / app options | [trickster-rules] [neuralplay] |
| `misere_defender_joker` | must play when void / must be the last card played | must play when void | Standard / QLD local | [pagat] [qld500] |
| **Play** | | | | |
| `opening_leader` | contractor / eldest | contractor | Standard / some US localities | [pagat] [hsg] [neuralplay] |
| `open_misere_exposure` | after trick 1 / before the opening lead | after trick 1 | Standard / Britannica | [pagat] [britannica] |
| `misere_partner` | sits out face down / passes one card, then sits out | sits out | Standard / Quebec | [pagat] [zpag] |
| `early_finish` | play all tricks / end misère at the contractor's first trick / end when all scores are determined / US: stop once a player has scored out and the bidder cannot win | end misère at first trick; play all suit and NT tricks (decide) | none stated in AU / Quebec, St Paul / app option / historical US | [neuralplay] [hsg] [fadoq] [denexa] |
| `claims` | off / solver-verified claims | off | NZ penalises laydowns | [nz2026] [neuralplay] |
| **Scoring** | | | | |
| `schedule` | Avondale / Original 1904 / Inverted / Perfect | Avondale | Standard / historical / rare / rare | [pagat] [hsg] [foster1914] [wiki] [neuralplay] |
| `slam_bonus` | 250 if the bid is under 250 / none / +100 on any sub-10 bid / value of tricks taken | 250 if under 250 | Standard / some / Youngstown / rare | [pagat] [zpag] |
| `defender_trick_points` | always / only if the contract fails / never | always | Standard / some / some | [pagat] |
| `misere_defender_points` | none / 10 per trick the contractor takes / 10 per own trick | none | AU / US / rare | [pagat] [hsg] [zpag] |
| `failed_contract` | subtract from the contractors / award to the opponents, no negatives | subtract | Standard / Tasmania, Quebec | [pagat] [zpag] [neuralplay] |
| `defender_cap` (must bid to win) | 490 / no cap but cannot win by defending / 460 / 400 / defenders may win | 490 (decide) | Common, NZ / pagat main / NZ to 2024 / rare / US | [nz2026] [pagat] [nz2024] [hsg] |
| **Game end** | | | | |
| `target` | 500 / 1000 / fixed number of hands / timed | 500 | Standard / Tasmania, Quebec / tournaments | [pagat] [zpag] [nz2026] [fadoq] |
| `back_door` | lose at −500 / off / only if the opponents are not negative / spread rule (500 ahead) | lose at −500 | Standard / no-negatives variants / rare / historical US | [pagat] [wiki] [hsg] |
| `simultaneous_500` | contractor wins / highest score wins / US trick-order rule | contractor wins | implied AU, historical US / some | [pagat] [hsg] [trickster-rules] |

### 5.2 Presets

| Preset | Key settings | Sources |
|---|---|---|
| **AU-Standard** (default) | Everything as in section 2 | [pagat] |
| **NZ-Clubs (2025–26)** | `misere_enabled` and `open_misere_enabled` off; `auction_style` NZ re-entry; `raise_own_bid` NZ; `all_pass` play NT for 10 per trick; `joker_nomination` never (none stated); `joker_lead_suit` any; `joker_lead_timing` any; `joker_on_reneged_suit` allowed; `defender_cap` 490; `target` 4 hands / 10 minutes / 500; `claims` off | [nz2026] |
| **US-USPCC classic** | `auction_style` single round; Nullo at 250 ranked by value; `misere_must_discard_joker` on; `misere_defender_points` 10 per contractor trick; `joker_nomination` never; `joker_must_play_when_void` never; `joker_lead_suit` any (Hoyle) or not shown void (pagat summary); `defender_cap` defenders may win; `simultaneous_500` contractor counts first, then trick order; `early_finish` stop when someone has scored out; `all_pass` throw in, next dealer | [hsg] [pagat] [hgm] [foster1914] |
| **US-St Paul** | 45 cards / kitty 5; single round; inkle; Ace–no face; `all_pass` (or nobody above 6) same dealer redeals; optional Nullo, Grand Nullo and Grand Slam call | [pagat] [denexa] |
| **Quebec-FADOQ** | 46 cards / two Jokers / kitty 6; single round; `double_joker_min_bid` on; `show_found_joker` on; `joker_lead_suit` already led; `all_pass` redeal; `target` 8 deals, cumulative | [fadoq] |
| **Quebec-traditional** | As FADOQ, but `failed_contract` award to opponents; game 1000; Nullo 500 equal to 10♥ with a passed card | [zpag] |
| **Tasmanian police** | `failed_contract` award to opponents; `defender_trick_points` never; `target` 1000; open misère 520, equal in rank to 10NT | [pagat] |

---

## 6. Edge cases a digital implementation must decide

Each item gives the recommended resolution. "Unsourced" means no source found settles it.

**Joker and suits**

- **E1. Does a suit named on a Joker lead count as "led" for R29 and R30?** Unsourced. Recommend yes: it was led to a trick, and the others had to follow it.
- **E2. A Joker holder on lead when all four suits have been led.** Unless it is the last trick, they must lead another card. They always have one, because hands are equal in size and only the last trick leaves one card. The UI should grey out the Joker with a reason.
- **E3. Joker led to the last trick.** Naming a suit cannot change anything. Skip the prompt, or record the nomination as a formality.
- **E4. A nominated Joker** (R25) is announced to all players before the opening lead.
  - It is then a card of that suit for every rule: it must be used to follow that suit when it is the holder's only card of it; it cannot be played on another suit unless the holder is void; it does not win other suits' tricks.
  - Recommend allowing nomination of a suit the contractor does not hold, as pagat sets no restriction.
  - Nomination happens after the kitty exchange.
- **E5. Contractor nominating the Joker as an NT trump suit.** No such thing exists: NT has no trumps, so nomination only makes the Joker the top card of a suit.
- **E6. The misère contractor keeping an un-nominated Joker.** Legal under AU rules, but the contract is lost (R31). The engine should not block it. The coach should warn at the discard and nomination decision.
- **E7. "Must play the Joker when void" in misère (R28)** applies to every active player, including a contractor holding an un-nominated Joker. Define "void" by effective suit.
- **E8. The Joker in the misère partner's dead hand.** That hand takes no part, so the Joker never appears. Bots and hints must treat the partner's 10 cards as unseen by everyone, the contractor included, and as permanently out of play.
- **E9. Tracking "shown void" by player.** This is needed if `joker_lead_suit` is "not shown void" or `joker_on_reneged_suit` is "forbidden". A player shows void in suit S by failing to follow when S is led, including when S was named on a Joker lead.

**Trumps**

- **E10. The left bower's suit (R7).** With hearts trumps, a player whose only "diamond" is J♦ is void in diamonds and must follow a heart lead with J♦ if it is their only trump. Every legality check and void inference must use the effective suit.
- **E11. Trick-winner rule.** In order: an un-nominated Joker wins; otherwise the highest trump wins (suit contracts); otherwise the highest card of the suit led. A nominated Joker ranks above that suit's ace.

**Auction**

- **E12. Bid comparison.** Compare bids by position in an explicit ordered list (table 3.5a), never by value. This handles 8♠ against misère and 10♥ against open misère. If a value-ranked variant is chosen and two bids are equal in value, the earlier bid stands [neuralplay].
- **E13. Legal-bid generator (AU).**
  - Misère is legal only if the high bid is one of 7♠ to 7NT.
  - Open misère is legal if there is no bid yet, or the high bid ranks below it (10♦ or lower).
  - After open misère, only 10♥ and 10NT are legal.
- **E14. Ending the auction.** A sole bidder whose opponents and partner all pass gets no further turn: the auction ends at once. If the first three players pass and the dealer bids, the auction also ends at once.
- **E15. All pass (R19).** No score is recorded and no kitty is shown. Recommend the deal rotates. Under the NZ all-pass NT hand, the 490 cap and must-bid-to-win apply to the 10-per-trick points too. This is unsourced but consistent with NZ rule 1(c).

**Kitty**

- **E16. Kitty timing and discards.**
  - The kitty is revealed to the contractor only after the auction closes; the contract cannot change afterwards.
  - Exactly 3 cards must be discarded. Bowers and the Joker may be discarded.
  - The discards stay hidden from all other players for the hand. A trainer may show them in the post-hand review (a UI decision).
- **E17. Misère contractor discarding the Joker.** Allowed under AU rules (R20: "any three cards"). No source forbids it; US Nullo and the QLD local rule go further and require it [hsg] [qld500].

**Misère and open misère**

- **E18. Exposure in open misère.**
  - The contractor's remaining 9 cards are revealed once trick 1 is complete.
  - If the contractor won trick 1, the contract has already failed. Under R33 the hand ends before any exposure.
  - The exposure becomes public information for bots and hints.
- **E19. Early termination (R32, R33) and statistics.** If misère hands stop at the first trick taken, "tricks taken" statistics for misère are capped at 1. Under US-style misère scoring (defenders score per contractor trick), play must continue to the end.

**Scoring**

- **E20. The slam.** It requires the contracting side to win all 10 tricks in a suit or NT contract, and replaces the bid value only when that value is below 250. Misère has no slam. Under the original schedule the same "<250" threshold applies, to different bids [foster1914].
- **E21. Cap arithmetic (R39).**
  - Partial: 470 + 50 → 490.
  - Negative scores are unaffected (−100 + 50 → −50).
  - The cap applies only to points won as defenders, never to contract points.
  - Display capped tricks clearly ("+0, capped at 490").
- **E22. No-cap mode.** A team at 500 or more from defending has not won (3.9). Its next made contract wins even if it is a 6♠. A failed contract may leave it above 500 and still not winning. Document this in the UI.
- **E23. End-of-hand order.** Apply contract points and defender points (with cap), then check for a win (contractors at 500+ having made their contract), then check −500. Under AU rules these cannot conflict.
- **E24. US mode.**
  - Defenders can win on trick points, so scores must be tracked trick by trick.
  - The bidder's points count first.
  - With 3 or 5 players, the first defender whose trick takes them over 500 wins if the bidder does not reach 500.
  - The hand may stop as soon as someone has won and the bidder cannot.

**Five players**

- **E25. The called card.** After the discard, every card is either in a hand or among the discards.
  - Default: forbid calling a card in the contractor's own hand or discards; the contractor must declare "alone" instead.
  - With `five_player_secret_solo` on, allow it and score the contractor as alone. pagat leaves the scoring unstated; recommend full value.
- **E26. The partner's identity.** It stays hidden until the called card is played, and the partner may not reveal it. Bots must not use the true identity. They need a belief model, including the partner's own knowledge: the partner knows from the start.
- **E27. Two winners.** If the contractor and partner both reach 500 on the same hand, both win [pagat].
- **E28. Half shares of a slam.** A five-player half share of a 250 slam is 125, the only non-multiple of 10 in the game. Unsourced. Recommend allowing it (store scores as integers, not tens), or rounding up to 130 as a house option.

**Six players and three players**

- **E29. Six players.** Generate 63 unique cards; duplicate-card rules are not needed. In misère both partners sit out, so tricks have 4 cards.
- **E30. Three players.** Defenders score individually, by who won each trick. Misère has no partner to remove.

**Fairness and interface**

- **E31. Hidden information.** The kitty, the discards, the misère dead hand and the five-player partner identity must never reach a bot or hint generator that would not legally know them. Per-seat views must be enforced in the engine.
- **E32. Claims.** Off by default. NZ treats a laydown by the makers as a forfeit [nz2026]. If claims are added, a solver must verify them, and they should never be required.
- **E33. Timed and fixed-hand games** (NZ: 4 hands or 10 minutes). Highest score wins and ties are possible. The cap still applies; the NZ remit shows why the cap value matters in timed play [nz-remit].
- **E34. Misclicked or insufficient bids.** NZ forces an insufficient bid to be corrected upward in the same suit [nz2026]. Software should simply not offer illegal bids. Whether to confirm high-value bids is a UI choice.

---

## 7. Conflicts the user should decide

| # | Question | Options and sources | Recommendation |
|---|---|---|---|
| C1 | How is "must bid to win" enforced? | (a) Defending points capped at 490: NZ clubs 2025–26, Trickster AU default, NeuralPlay default, BGA, catsatcards, Bryce Francis [nz2026] [trickster-rules] [neuralplay] [bga] [catsatcards] [play500]. (b) No cap, points accrue but cannot win: pagat's main text [pagat]. (c) Cap 460 [nz2024] [pagat]. (d) Defenders can win: historical US [hsg] | (a) 490 |
| C2 | Does 10♥ beat open misère? | Yes, open misère sits between 10♦ and 10♥ [pagat] [bga] [gamerules] [catsatcards]. No, only 10NT beats it [trickster-rules]. Open misère is the highest bid [britannica] | Yes (pagat) |
| C3 | Which suit may a Joker leader name? | Not previously led [pagat]; not one the leader has shown void in [pagat] [hgm] [qld500]; any suit [nz2026] [hsg]; only one already led [zpag] [fadoq] | Not previously led (pagat), with a toggle |
| C4 | May the Joker be used on a suit its holder has already discarded on? | Allowed: pagat main text, NZ explicit [pagat] [nz2026]. Forbidden: "many players" [pagat]. Allowed but cannot win [hgm] | Allowed |
| C5 | Is misère allowed, and where does it rank? | Only over a 7-bid, between 7NT and 8♠ [pagat]; by value between 8♠ and 8♣ [hsg]; not allowed [nz2026] | pagat |
| C6 | What happens when all four pass? | Throw in [pagat]; play the hand at NT for 10 per trick [nz2026] | Throw in, next dealer |
| C7 | Bidding after a pass, and raising your own bid | Pass is final, no raising [pagat]; NZ re-entry and raising rules [nz2026]; other variants [pagat] [play500] | Pass is final, no raising |
| C8 | May the contractor give the Joker a suit in NT? | Yes [pagat]; misère only [pagat]; never [pagat] (American section) | Yes |
| C9 | When is the open-misère hand exposed? | After trick 1 [pagat]; before the opening lead [britannica] | After trick 1 |
| C10 | Does a misère hand stop at the contractor's first trick? | No source in AU [pagat]; play through when defenders score per trick [britannica] [hsg] | Stop (no scoring effect under AU rules) |
| C11 | Do defenders score when the contract is made? | Always [pagat]; only on a failure [pagat]; never [pagat] | Always |
| C12 | Does the misère contractor take the kitty? | Yes [pagat]; option of no, or not for open misère [trickster-rules] | Yes |
| C13 | Five-player scoring | Half shares each [pagat]; full each [wiki] | Half shares (pagat) |

---

## 8. Sources

Accessed 2026-10-17. Notes give the publisher, what each source was used for, and any reliability caveat.

**Primary and near-primary rules**
- [pagat] – John McLeod, "Five Hundred", pagat.com. Main description of the Australian game and its variations, plus sections on the American (USPCC), St Paul, Youngstown, Tasmanian, three-, five-, six- and two-player games. Section anchors used by Wikipedia: [pagat-bid], [pagat-end].
- [nz2026] – Clubs New Zealand, "All Clubs NZ Singles 500 Rules of Play (2026)" (PDF, South Otago subdomain). Identical in substance to [nz2025].
- [nz2025] – Clubs New Zealand, "All Clubs NZ Singles 500 Rules of Play (2025)" (PDF).
- [nz2024] – Clubs New Zealand, "All Clubs NZ Singles 500 Rules of Play (2024)" (PDF). 460 cap; also allows a 3-3-4 deal.
- [nz-remit] – Tauranga Citizens Club 500 Adjunct, "2024 Rules Remit … Additional notes for 2025 Delegates Meeting" (PDF). The rationale for changing the cap from 460 to 490.
- [hsg] – *Hoyle's Standard Games* (Laird & Lee, Chicago; ©1908, 1914, 1920, 1924; "courtesy of the United States Playing Card Company"), "Five Hundred", pp. 141–147, archive.org full text. Packs, deal, one-bid rule, the Joker, schedules (Avondale, Original, Inverted), Nullo, game end, the five-hand partner methods.
- [hgm] – *Hoyle's Games Modernized*, "Five Hundred" (Wikisource; three-handed game, Prof. Hoffmann's rules). Avondale described, the Joker in NT, all-pass NT play, revoke rules.
- [foster1914] – R. F. Foster, *Foster's Complete Hoyle* (1914), "500, or Bid Euchre", pp. 287–288 (Project Gutenberg #53881). The original schedule, slam 250, the bidder leads, the bidder counts first.
- [fadoq] – FADOQ (Fédération de l'Âge d'Or du Québec), *Jeux en tête* official rules, "Le 500" (PDF). Quebec tournament rules.
- [zpag] – "Le Cinq-cents, Jeu de Cartes" (PDF, zpag.net). Traditional Quebec rules, Quebec Nullo, and a French account of the American game. Author and date unknown.
- [qld500] – 500 Challenge (Queensland), "Format: 500 for 2" local rules. Two-player tournament rules that otherwise follow four-player 500.

**Encyclopedic and secondary**
- [wiki] – Wikipedia, "500 (card game)". Many variant sections are tagged as unsourced; cited with that caveat.
- [britannica] – Encyclopaedia Britannica, "five hundred". The page blocked direct retrieval; the quoted sentences were read in search-engine excerpts (misère beaten by any 8-bid; open misère the highest bid; open-misère hand spread before the opening lead; misère played through because opponents score per declarer trick).
- [catsatcards] – CatsAtCards.com, "How To Play Five Hundred" (2015). Secondary; covers 3-, 4- and 5-player play, 490 cap, Joker rules, misère placement, original and inverted schedules.
- [gamerules] – GameRules.com, "Five Hundred". Secondary, closely follows pagat; variant list and reader comments.
- [play500] – Bryce Francis, "The card game of 500 according to Francis", play500.com (1996–2011). Personal system; trump counts, 490 cap, re-entry on suit change, no redeal for weak hands.
- [denexa] – Denexa Games, "Five Hundred" (2014). St Paul-style American partnership rules (45 cards, inkle, ace–no face, Nullo, Grand Nullo).
- [regles] – Regles.com, "Règle du 500". French summary based on FADOQ; used only to locate [fadoq].

**Digital implementations** (evidence of common defaults and option sets)
- [neuralplay] – NeuralPlay, "500 Rules & Help". The most complete option list; presets for Australian, American, Canadian and three-player.
- [trickster] – Trickster Cards, "500 Basics".
- [trickster-rules] – Trickster Cards, "500 Rules" (option definitions and defaults).
- [trickster-more] – Trickster Cards, "How to Play 500" (option summary).
- [bga] – Board Game Arena, "500" game panel (rules summary and options).

**Sources tried but unavailable or unhelpful.** Bicycle's current site has only 500 Rum; its 2016 Five Hundred page survives only on the Wayback Machine, which could not be fetched. australiancardgames.com.au and cardgames.net.au returned errors (HTTP 522). officialgamerules.org returned HTTP 403. cardrulesplus.com, card-games.nz, lodige.com and newtsgames.com were read but are internally inconsistent or derived from Wikipedia, so they are not relied on.

[pagat]: https://www.pagat.com/euchre/500.html
[pagat-bid]: https://www.pagat.com/euchre/500.html#bidding
[pagat-end]: https://www.pagat.com/euchre/500.html#end
[wiki]: https://en.wikipedia.org/wiki/500_(card_game)
[nz2026]: https://southotago.clubsnz.org.nz/assets/Tournament-Documents/500-Tournaments/CLUBS-NZ-SINGLES-RULES-OF-PLAY-updated-2026.pdf
[nz2025]: https://www.clubsnz.org.nz/assets/Tournament-Documents/500-Tournaments/CLUBS-NZ-SINGLES-RULES-OF-PLAY-updated-2025.pdf
[nz2024]: https://www.clubsnz.org.nz/assets/Tournament-Documents/500-Tournaments/500-RULES-OF-PLAY-Updated-March-2024.pdf
[nz-remit]: https://clubsnz.org.nz/assets/Tournament-Documents/500-Tournaments/2025/Nationals/500-Kitty-Remit-2025-Supporting-Notes.pdf
[hsg]: https://archive.org/stream/hoylesstandardga00unse/hoylesstandardga00unse_djvu.txt
[hgm]: https://en.wikisource.org/wiki/Hoyle%27s_Games_Modernized/Five_Hundred
[foster1914]: https://www.gutenberg.org/cache/epub/53881/pg53881-images.html
[fadoq]: https://www.fadoq.ca/wp-content/uploads/2018/09/500.pdf
[zpag]: https://www.zpag.net/Jeux/Cartes/Le_Cinq.pdf
[qld500]: https://www.500challenge.net/format
[britannica]: https://www.britannica.com/topic/five-hundred
[catsatcards]: https://catsatcards.com/Games/FiveHundred.html
[gamerules]: https://gamerules.com/rules/five-hundred-card-game/
[play500]: https://www.play500.com/gameof500.html
[denexa]: https://www.denexa.com/blog/five-hundred/
[regles]: https://www.regles.com/jeux-cartes/500.html
[neuralplay]: https://www.neuralplay.com/five_hundred_help.html
[trickster]: https://www.trickstercards.com/help/500/
[trickster-rules]: https://www.trickstercards.com/help/500-rules/
[trickster-more]: https://trickstercards.com/home/help/HowToPlay.aspx?game=500
[bga]: https://en.boardgamearena.com/gamepanel?game=fivehundred
