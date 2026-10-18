# 500 (Five Hundred) Strategy Playbook

Research document 02 · Australian four-handed partnership game · compiled 2026-10-17

---

## How to read this document

**Purpose.** This is source material for three uses: AI players of graded skill and temperament, hint and lesson text, and experiments for the simulation lab. Each item gives a rule of thumb, the reasoning behind it, a confidence label, and its source.

**Rules baseline** (pagat's Australian four-handed rules [PG], unless noted):
- **Deck and deal.** 43 cards and 10 each, with a 3-card kitty. In trumps the order is Joker > right bower (RB) > left bower (LB) > A K Q 10 9 …
- **No trumps (NT).** Jacks rank normally. The Joker belongs to no suit. It may be played only when the holder is void in the suit led, or led with a nominated suit that has not yet been led. It cannot be led once all four suits have been led, except to the last trick. The contractor may nominate the Joker's suit before the first lead.
- **Auction.** Clockwise from the dealer's left, minimum bid six. A player who passes is out of the auction, which ends when three players have passed. If all four pass, the cards are thrown in.
- **Misère and open misère.** Misère (250) may be bid only after someone has bid seven. It outranks every 7-bid and loses to every 8-bid. Open misère (500) may be bid at any time and ranks between 10♦ and 10♥.
- **Play.** The contractor takes the kitty, discards three cards, and leads to the first trick. In misère the contractor's partner takes no part. In open misère the contractor's hand is exposed after the first trick.
- **Scoring.** Avondale table. Defenders score 10 per trick whether or not the contract succeeds, but nothing in misère. A slam scores 250 if the bid is worth less than 250.
- **Winning and losing.** A side wins by reaching 500 *as a result of a successful contract*. A side at −500 or worse loses ("back door").

**Labels**
- **(A)** Widely agreed. Several independent sources agree, or the advice follows directly from the rules.
- **(B)** Commonly advised but contested. Sources disagree, or the advice depends on conventions or rule variants.
- **(C)** Single-source or folk. One source or a forum anecdote. **C·analysis** marks my own reasoning with no external source; treat it as a hypothesis.
- **(M)** A mathematical fact derived in §9. It is certain under the stated assumptions; it is not advice.

**Source keys.** [PG] pagat · [WP] Wikipedia · [BF-…] Bryce Francis, play500.com (gs = "Before getting started" checklist, numbered items cited as #n; g5 = "The game of 500"; c = "Calling four handed"; c1 = its continuation; old = "Calling Traditional"; t = terminology; home = home page) · [NM] Neil Maloney's glossary of 500 terms (Special K Software, Australia) · [FH] Foster's Complete Hoyle, 1914 (Solo Whist and Euchre sections, used as analogues) · [HY24] Hoyle's Standard Games, 1924 · [WF] Whirlpool forum threads · [BGG] BoardGameGeek threads · [GR] gamerules.com · [RP] rarepike.com · [CRP] cardrulesplus.com · [FGS] familygamesquad.com · [AK] an older encyclopedia text on academickids.com · [OC] US regional rules page on oocities.org · [TC] Trickster Cards · [CGC] cardgames.club · [NP] NeuralPlay app listing · [WPE] Wikipedia, "Endplay". Full URLs are in **Sources**.

**Evidence caveat.** Little has been published on 500 strategy. An older encyclopedia text notes that 500 has not attracted the kind of deep tactical analysis bridge has [AK]. The most substantial 500-specific writing found is Bryce Francis's personal system [BF]. Neil Maloney's glossary [NM] describes a different system. Andrew Blucher's strategy page, linked from pagat, survives only as a web-archive copy that could not be retrieved. BGG and Whirlpool threads could be read only through search-engine snippets. Treat everything not marked (A) or (M) as a hypothesis for the lab.

### 0. Rule facts and variant switches that change strategy

Strategic consequences of the baseline rules:
- **The declarer leads to trick 1 in every contract** [PG]. Defenders never choose an opening lead. Their first decisions are how to follow and what to discard. **(M)**
- **The left bower changes suit** [PG]. When hearts are trumps, J♦ is a heart, so it cannot follow a diamond lead. Beginners often find this confusing [CGC]. **(A)**
- **A pass is final** [PG]. A player who wants to keep exchanging information must keep bidding. **(A)**
- **Misère can be bid only after a 7-bid and is killed by any 8-bid** [PG]. It competes with 7-level contracts. **(A)**
- **Defenders score in suit and NT contracts but not in misère** [PG]. A misère is a pure ±250 bet, and an open misère a pure ±500 bet. **(M)**
- **Slam bonus.** Any contract worth less than 250 scores 250 if all ten tricks are taken [PG]. The 10th trick is a big swing on cheap contracts (§7.1). **(M)**
- **You can only win by making a contract** [PG]. This drives end-game sacrifice bidding (§7). **(A)**

| Variant switch (all seen in sources) | Strategic effect |
|---|---|
| Misère banned, allowed only after a 7-bid, or allowed any time [PG, WF] | Changes how often weak, flat hands can compete. Without misère, very weak hands should defend. |
| Defensive scoring capped at 490, 460 or 400, or not scored at all [PG, NP] | Near the cap, defending earns nothing, so both sides bid more aggressively. [BF-home]: "if both teams are plus 490 … one side is going to go down next hand." |
| Some variants score nothing for defenders when the contract makes [OC] | Low contracts become far more attractive (see §7.1). |
| Joker nomination in NT allowed or forbidden; restrictions on the Joker in a suit already shown void [PG] | Decides whether the NT Joker can be turned into a fixed stopper (§4.5). |
| Bidding after passing; contractor may raise after the others pass; re-entry when the suit changes [PG, BF-g5] | Enables underbid-then-raise tactics (§2.4). Under the baseline rules these tactics do not exist. |
| Whether the misère contractor may discard the Joker into the kitty (unstated in [PG]; forum posters [WF, BGG] say there is no official answer) | Decides whether holding the Joker kills a misère hand. |

---

## 1. Hand evaluation for bidding

### 1.1 Structural facts
- **Trumps number 13 when a red suit is trumps and 12 when a black suit is.** Side suits have 10 cards each under red trumps. Under black trumps they have 9 (the same-colour black suit) or 11 (each red suit). **(M)**
- **The Joker is exactly one guaranteed trick for its holder in every contract except misère.** Proof in §9.11. It follows that any 10-trick contract requires the Joker on your side, as [BF-old] states. **(M)**
- **Trump honours need guards.** Counting down from the top, the RB is safe with one card below it, the LB with two, and the trump ace with three. [BF-c1] applies the same idea to side suits: keep one guard with a king and two with a queen. **(A, analogue of the bridge guard rule)**
- **Bower position matters.** An RB sitting in front of the Joker holder can still be caught: the Joker holder leads small trumps until the guard is gone, then leads the Joker. An RB with at least one guard that plays *after* the Joker holder is effectively safe. **(C·analysis)**
- **The top three trumps are worth three tricks.** [CRP] counts the Joker plus both bowers as three certain winners, "enough for a 6-bid on most hands." **(C)**
- **More trumps beat higher trumps.** [FGS] says Joker + LB + several trumps beats Joker + RB + one. Reason: once the defenders' trumps are drawn, every remaining trump is a trick. **(C; test H9)**

### 1.2 Counting trump tricks
- **Count sure top tricks first, then length tricks.** After the opponents are exhausted, each remaining trump wins. To estimate length tricks, ask how many rounds the draw will take. With 7 red (or 6 black) trumps after the kitty, the longest opponent holds at most 3 trumps 85% of the time (§9.6). Three rounds usually finish the job, but they also pull partner's trumps. **(M)**
- **"Two rounds of trumps usually clear the defenders" is wrong for this deck.** [CRP] and [FGS] claim it (FGS plays a 5-card-kitty variant). With 6 red trumps after the kitty, at least one defender still has a trump after two rounds 76% of the time. With 8 red trumps it is still 38%. **(M; the folk claim is refuted, see H4)**
- **Bare high trumps are fragile.** The Joker never fails. A bare RB or LB in front of the Joker holder can be captured. Credit unguarded bowers at less than one trick. **(C·analysis)**

### 1.3 Side-suit tricks
- **A side ace is about one trick.** Keep aces [WP]; save them for late tricks when opponents cannot trump [RP]. Before trumps are drawn, the risk of a ruff on the first round is only about 4–13% with a 2–4-card holding (§9.8). The risk rises when you are long in the suit and on the second round. **(A for keeping aces; M for the numbers)**
- **Guarded kings are worth roughly half a trick, depending on where the ace sits.** [BF-c1] says a lone king is usually wasted. Keep king plus a low card [WP, BF-c1]. **(B: this conflicts with voiding, see §3)**
- **Long side suits rarely run before trumps are drawn.** In a 10-card side suit where you hold 3, both opponents follow to a second round only 56% of the time, and to a third round 10% (§9.8). **(M)**
- **Voids and singletons have ruffing value only when your trumps exceed what the draw needs.** Otherwise each ruff spends a trump you needed for drawing. After the kitty you can almost always create at least one void (§9.11). **(C·analysis; M for the void fact)**

### 1.4 Crediting partner and the kitty
- **Bid the partnership total, not just your hand.** This includes the cards you expect from the kitty and from partner [WP]. Bryce Francis likewise tells bidders to consider the potential of partner's hand as well as their own [BF-c1]. **(A)**
- **Partner has already counted on average support from you.** So a partner's 6♥ does not justify your jumping to 8♥ just because you hold two hearts [WP]. **(C)**
- **A weak hand usually means a strong partner** [BF-gs #23]. This is card conservation. If you hold no ace, partner's expected number of aces is 10×4/33 = 1.21, and partner holds at least one ace 78% of the time (§9.3). **(A; M)**
- **Partner's expected trump support** is 2.1–3.0 cards in your suit, depending on your own length (§9.9). **(M)**
- **Starting prior for partner credit.** Credit a silent partner with about one trick. Credit more after partner has shown aces or the Joker, and less after an informative pass. **(C·analysis; calibrate with H7)**
- **Kitty credit is contested.** [RP] says to add 1–2 tricks for the kitty, which conflicts with [GR]'s "Don't simply rely on the kitty!" [BF-c] says the kitty may add a trick but your bids should stay within your "bargaining power" (§7) unless the opponents threaten to go out. **(B)**
- **What the math says about the kitty.** Expected trumps gained are only 0.45–0.91. The kitty contains no trump at all 32–60% of the time (§9.5). The reliable gain is getting to discard your three worst cards. A reasonable prior is +0.5 to +1 trick, more for hands with short losing side suits. **(M; C·analysis for the prior; test H6)**

### 1.5 Requirements for no trumps
- **Don't bid NT without the Joker or three aces, plus support such as kings** [GR]. **(C)**
- **9NT without the Joker** is achievable with all four aces and good support or a long strong suit [BF-gs #27]. [BF-old] calls 9NT "quite easy" even with the Joker held against you. **(C)**
- **NT outscores every suit at the same level, so check every hand for NT** [NM]. Maloney observes that players avoid NT because they have no method for estimating it. **(C)**
- **The NT Joker is a sure trick but a slow stopper.** Un-nominated, it can be played only when you are void in the suit led, so it does not stop a suit the opponents are running until you are out of that suit. Nominated, it becomes the top card of that suit, an immediate stopper there [PG for the rules]. **(M for the rules; C·analysis for the use)**
- **NT has no ruffing protection.** Each suit needs a stopper, or the partnership needs enough fast tricks to win the race. Without the Joker, plan to lose one trick to it. If partner's bidding system would have shown the Joker and partner did not, it is with an opponent 87% of the time (§9.2). **(C·analysis; M)**

### 1.6 Requirements for misère and open misère (details in §6)
- **No Joker** [PG, WF, HY24]. **(A)**
- **Low cards, voids and short suits** [CRP: "only low cards and one short suit"; GR]. **(B)**
- **Hold the lowest cards:** the 4 in red suits and the 5 in black suits. **(B; analogue of Foster's "deuce" rule in Solo Whist [FH])**
- **Ladder test.** A suit cannot be beaten if, for each of your cards, the number of unknown cards in the suit ranking below it is no more than the number of your cards below it. Equivalently, your k-th lowest card has at most k−1 unknown cards below it. **(M; proof in §9.11)**
- **Open misère** needs every suit to be ladder-safe or void, and a safe first lead (§9.11). **(C·analysis, following Foster's rule for the spread misère)**

### 1.7 Typical minimum holdings
| Level | Sourced guidance | Starting point for AI (C·analysis; calibrate in sim) |
|---|---|---|
| 6 | Joker + both bowers is enough "on most hands" [CRP]. In conventional systems a 6-bid is a message, not a trick estimate: an ace [BF-c], the Joker as 6NT [BF-old, WP] or as 6♠ [NM]. | About 4 playing tricks in hand: 4+ trumps including one of the top three, plus a side ace. |
| 7 | No ace or Joker but strength in one suit justifies an immediate seven call [BF-c1]. Five good trumps can justify a jump to 7 [BF-c]. | About 5 tricks: 5 trumps with two of the top three plus a side ace or void, or 6 trumps with one of the top three plus a side ace. |
| 8 | Bid high only with 6+ cards in the suit [GR]. 7–8 trumps plus an ace is a good hand even without the Joker or a bower [FGS, 5-card-kitty variant]. Don't jump straight to 8 [NM]. | About 6 tricks: 6+ trumps with two or three of the top three and a side ace or void. Prefer partner's support to have been shown. |
| 9–10 | A nine or ten call is not hard to make even with all four aces against you [BF-gs #26]. Ten tricks need the Joker [BF-old; M]. | 7+ tricks in hand plus shown support. 10-level bids need the Joker on your side. |

**Sources for §1:** https://www.pagat.com/euchre/500.html · https://en.wikipedia.org/wiki/500_(card_game) · https://play500.com/gettingstarted.html · https://play500.com/calling.html · https://play500.com/calling1.html · https://play500.com/oldway.html · https://www.specialksoftware.com/terms.html · https://cardrulesplus.com/games/five-hundred-500/ · https://familygamesquad.com/how-to-play-500-bid/ · https://gamerules.com/rules/five-hundred-card-game/ · https://rarepike.com/how-to-play-500/ · https://www.cardgames.club/five-hundred/ · https://gutenberg.org/cache/epub/53881/pg53881-images.html

---

## 2. Bidding tactics and partnership communication

### 2.1 What an opening bid shows: competing schools
No national standard convention was found; meanings are local. So the trainer should let users pick a system and show each bid's meaning to all players. That is bridge's full-disclosure principle: no partnership understanding is hidden from the opponents. **(C·analysis)**

| School | 6NT | 6♠ | Other 6-bids | Later bids | Label |
|---|---|---|---|---|---|
| Traditional / natural [BF-old, WP] | The Joker, whatever else is held | Natural | A good suit; 7 or 8 shows more. Some propose a 6-bid should show an ace or king [WP]. | Raise by trick expectation | (B) |
| Francis "aces first" [BF-c] | Joker and no aces | Ace of spades only | 6♣ = A♣ without A♠; 6♦ = A♦ without black aces; 6♥ = A♥ only | 7 in partner's suit = strength there without the Joker or another ace. With nothing to bid, bid 7 in partner's suit to keep partner in the auction [BF-gs #24]; the site's pages differ on how this is told apart from the strength bid. A jump bid = suit strength without aces or the Joker. Kings, then queens, at 7–10. | (C) |
| Maloney [NM] | *Denies* the Joker (wants NT) | The Joker | 6♣ "rainbow" = one red jack, one black jack and one sure trick | Establish NT, then show aces, kings and queens by rebids. A complementary-suit (same-colour) rebid shows support or preference. | (C) |
| Youngstown, Ohio (US) [PG, OC] | The Joker | — | A bower in that suit; "6 no" = two aces | "7 no" = better than two aces but does not want the contract | (C, American) |
| St Paul, Minnesota (US) [PG, OC] | — | — | Inkle (6-bid) allowed only for the first two bidders and cannot win the auction [TC] | "7 no" = Joker or "split bowers" (opposite-colour jacks) | (C, American) |
| One Australian school's custom [WF] | — | — | All 6-calls showed aces or the Joker | — | (C) |

Design notes:
- **6NT as the Joker bid wastes bidding space.** It uses up the whole 6-level, forces partner to bid 7 whatever partner holds, and prevents a genuine 6NT [NM]. A system should show the most useful facts with its cheapest bids. **(C, plus C·analysis)**
- **Conventional bids tell the opponents too.** Bryce Francis treats pitching your calls to lock the opponents out of information as "THE GREATEST SKILL IN 500" [BF-c1]. **(C)**

### 2.2 Supporting and raising partner
- **Under natural bidding, raise only with extra values,** because partner has already assumed average support [WP]. **(C)**
- **Use the complementary (same-colour) suit.** The bowers serve both suits of a colour, so your J♦ helps a hearts contract. Rebidding partner's complementary suit at the same level, or one higher, shows support; at a higher level it shows preference [NM]. Try to bid both colours when strong in both [NM]. **(C)**
- **Rebids must reflect genuine strength.** A suit rebid needs at least one of the top three trumps [NM]. **(C)**
- **Don't leave partner alone on a first ace bid.** With nothing to bid, bid seven in partner's suit to keep partner in (Francis system) [BF-gs #24]. **(C)**
- **Weak hands can still support.** A hand with no court cards but 5+ cards in partner's suit is useful support [BF-g5]. **(C)**

### 2.3 Showing suits and reaching NT
- **With two roughly equal suits, bid the higher-scoring one** [WP]. **(C)**
- **Maloney's route to NT** [NM]: one partner shows the Joker, then NT is "established" and the partners bid aces, then kings by rebidding the suit, then queens. Show aces at the lowest available level. **(C)**
- **Francis's jump bids** show suit strength without aces. With five strong cards in each of a black and a red suit, jump in the black suit first, so the red suit can still be shown without leaving the 7-level [BF-c]. **(C)**

### 2.4 Competitive bidding
- **Listen to every bid and work out where the four aces and the Joker are** [BF-gs #4, #17]. **(A)**
- **"If one team is bidding red, you and your partner usually are better off calling black"** [FGS]. **(C; test H13)**
- **Don't overcall a contract you can't afford** unless the opponents' call would win them the game [BF-g5]. **(C)**
- **Be wary of opponents who bid less than they can make** and then raise in the same suit to lock you out [BF-g5]. This only works where the rules let the final bidder raise. **(B, variant-dependent)**
- **Call one level below your intended bid to hide strength,** then move to your real suit if left on it [BF-c]. This also requires a rule letting the final bidder change their bid. **(C, variant-dependent)**
- **Pitch calls to block the opponents' communication** [BF-c, BF-c1]. **(C)**

### 2.5 When to pass
- **In Francis's system a pass denies a lot:** no aces, no Joker, no suit strength and no NT potential [BF-c]. Passing as first caller while holding an ace is risky [BF-c1]. **(C)**
- **Consider defending instead.** [FGS] advises asking what shape you would be in "if I didn't get the bid … a setting hand is better to pass with." Hands with spread aces and guarded kings but short trumps often score more by defending. **(C; C·analysis)**
- **Don't chase a contract when the opponents' bid can't win them the game.** [BF-gs #18] says it is better to miss a strong hand, and risk losing the game, than to risk the hand in that case. **(C)**
- **Low contracts are worth almost nothing on their own.** Because defenders score 10 per trick, a 6♠ made exactly changes the score difference by 0 (§7.1). Bid low contracts to compete, to block, or to win the game, not for their points. **(M)**

### 2.6 Sacrifice, lock-out and defensive bidding near the end
- **When the opponents are bidding to win the game, don't pass** unless you're sure you can set them, or you know you can't make your own bid and would lose the game by staying in [BF-gs #16]. **(B; [CRP] also advises bidding to deny the opponents the contract)**
- **"Never be afraid to make a call to go down"** [BF-c1]. Choose a level that leaves the opponents only dangerous calls above it; [BF-c1] adds, without giving a reason, that "your best lockout call is often in your weakest suit." The score *difference* matters more than the absolute scores [BF-c1]. **(C)**
- **Small losses beat big opposing gains.** Going down on a small bid is better than letting the opponents make a large one [BF-gs #29]. **(C; quantify in H24)**
- **Control bids.** When scores are high, the Joker holder bids well above a normal jump so that any further opposing bid must expect to fail. The control bidder's partner should not re-enter [BF-c]. For a lock-out, Francis mentions 9NT as the minimum and says 10NT was considered safe in his school [BF-c]. **(C)**
- **Stay in when a same-suit raise would win it for them.** If the opponents' bid doesn't take them out but a higher bid in the same suit would, stay in the bidding: in his house rules you can't re-enter unless the suit changes [BF-gs #28]. **(C, variant-dependent)**
- **Why sacrifice works:** defenders cannot win the game on defensive points [PG]. Your failed sacrifice costs points but does not end the game, unless it puts you through the back door. **(M)**

### 2.7 Adjusting to the score
- **Check the exact score before every bid** [BF-gs #3; BF-c]. **(A)**
- **Francis's jump-bid guidance by score** [BF-c]. **(C)**
  - Jump freely from −100 up to any plus score.
  - Be cautious at low minus scores.
  - Never jump if losing the bid would lose the game, unless the opponents are near 500.
  - When both sides are high, jumps must show real strength.
- **Tell partner about the Joker early at critical scores.** When both sides are at 250 or more, a 10-bid can win the game [BF-c]. **(C)**
- **Use your "bargaining power":** your score plus 490, which is how far you can fall before the back door [BF-t, BF-gs #20]. **(C)**

### 2.8 Communication ethics
- **Use only legal channels: bids and card play.** One encyclopedia text describes "surreptitious signals (by the tone of voice, exact words used …)" as part of the game [AK]. In any serious game they are cheating. Etiquette from [BF-gs] points the same way: #21 no comment on your hand until it has been played, #25 talk only before or after play, and #37 never reveal your discards. **(A)**

**Sources for §2:** https://play500.com/calling.html · https://play500.com/calling1.html · https://play500.com/oldway.html · https://play500.com/gettingstarted.html · https://play500.com/gameof500.html · https://play500.com/terminology.html · https://www.specialksoftware.com/terms.html · https://en.wikipedia.org/wiki/500_(card_game) · https://www.pagat.com/euchre/500.html · https://www.oocities.org/timessquare/2767/fivehun.html · https://www.trickstercards.com/help/500/ · https://familygamesquad.com/how-to-play-500-bid/ · https://cardrulesplus.com/games/five-hundred-500/ · https://forums.whirlpool.net.au/archive/2036783 · https://academickids.com/encyclopedia/index.php/500_(game)

---

## 3. Kitty and discards

### 3.1 Suit contracts
- **Mechanics and etiquette.** Pick the kitty up in one motion, discard exactly three cards, don't reveal them, and don't comment [BF-gs #5, #37; BF-c1]. You may discard cards you just picked up [PG]. **(A)**
- **Never discard a trump** [BF-c1, in capitals]. The only exception is when you hold 11 trumps among your 13 cards. [WP] agrees in spirit: discard side suits to make the most of your trumps. **(A)**
- **Short-suit by default.** Void or shorten side suits [WP; BF-gs #5; FGS: "Short suiting … is the best practice"]. But [BF-home] stresses that "you do not have to short-suit yourself." **(B)** The likely optimum depends on trump length (C·analysis; H8):
  - With 6+ trumps, voids create ruffs you can afford.
  - With 5 or fewer, guards and aces matter more.
- **Keep side aces** [WP]. **(A)**
- **Keep guards.** Keep at least one card with a king and two with a queen [BF-c1]. Keep king plus a low card if you can't void the suit, and lead the low card first to lose it to the ace [WP]. **(B)**
- **Keep a way to reach partner.** Keep one card in each suit partner has bid, especially partner's aces, so you can lead to them [BF-c1, BF-gs #5]. A partner's ace in your void suit is hard to use, because you can't lead the suit. **(C; C·analysis)**
- **Suggested priority order:** never trumps; then losers in your shortest side suit (create a void); keep side aces; keep king plus one if you can't void; keep a low card in partner's ace suit. **(C·analysis)**

### 3.2 No trumps
- **Keep long suits intact,** since length is where NT tricks come from, and lead the long suit first [BF-gs #10]. **(B)**
- **NT guards are stricter.** A queen needs three cards, not two [BF-c1]. **(C)**
- **Decide which suit you can afford to leave unstopped,** and discard from it. A void has no ruffing value in NT. It only lets you play an un-nominated Joker on that suit. **(C·analysis)**
- **If you hold the Joker, decide whether to nominate it as part of the same plan** (§4.5). **(C·analysis)**

### 3.3 Misère
- **Discard the Joker if the house rules allow it.** An un-nominated Joker kept in a misère means the misère "automatically fails" [PG]. The 1924 American Hoyle's "Nullo" section says the bidder "must lay it away in his discard" [HY24]. If you can't discard it, nominate a suit for it and shed it at the first lead of a suit you're void in [PG]. **(A)**
- **Discard from the top of dangerous suits:** high singletons and high cards in short suits. Never discard your lowest cards, because they are your guards. Removing a suit's top card can never break the ladder condition; removing a low card can (§9.11). **(M; agrees with Foster's Solo Whist advice to shed high cards in short suits [FH])**
- **Create voids.** Foster treats a missing suit as a great advantage, since every lead of it lets you discard a dangerous card [FH]. **(B; Solo Whist analogue)**

### 3.4 What defenders can infer from the discard
- **Three dead cards are unknown to the defenders.** The declarer will usually have voided their shortest side suit. Expect declarer to show out early in one side suit. The "missing" cards of that suit may be in the discard rather than in partner's hand. **(C·analysis; test H28)**

**Sources for §3:** https://play500.com/gettingstarted.html · https://play500.com/calling1.html · https://www.play500.com/ · https://en.wikipedia.org/wiki/500_(card_game) · https://familygamesquad.com/how-to-play-500-bid/ · https://www.pagat.com/euchre/500.html · https://archive.org/stream/hoylesstandardga00unse/hoylesstandardga00unse_djvu.txt · https://gutenberg.org/cache/epub/53881/pg53881-images.html

---

## 4. Declarer play

### 4.1 Drawing trumps: whether, when, and with which card
- **Draw trumps early.** [BF-gs #6a, #38], [RP], [CRP] and [FGS] say so; [WP] adds "often (but isn't always)." **(A)**
- **Lead the Joker first, then the bowers** [BF-gs #38; FGS: "Leading with the joker is usually a good idea"]. **(B)** Refinements:
  - With the Joker and the RB, the order doesn't change who wins. Leading a bower first hides who holds the Joker. **(C·analysis)**
  - A Joker lead rarely catches a bare RB: only 3–14% of the time when you hold 5–8 trumps (§9.7). Its real value is pulling up to two opposing trumps at no risk. **(M)**
  - **Holding the RB but not the Joker:** still lead the RB, to draw out the Joker [BF-gs #39]. **(C; H2)**
  - **Holding the Joker and both bowers when partner holds the trump ace:** lead toward partner's ace first, because partner may hold it alone [BF-gs #40]. Leading the Joker would crush a bare ace. **(C; H3)**
- **Stop drawing once only your side holds trumps** [BF-gs #6]. **(A)**
- **Don't lead into a lone higher trump.** If the only trump out against you outranks yours, don't lead trumps to remove it. Keep yours to regain the lead after it has been used [BF-gs #6]. **(B; the same as bridge's "don't draw the master trump")**
- **Trade one for two.** If partner is out of trumps and both opponents may still have some, a losing trump lead can draw two of theirs for one of yours [BF-gs #6]. **(C)**
- **Keep one or two trumps when you switch to side suits.** If you lead out all your trumps and lose the lead, you may not get it back [BF-gs #41]. **(B)**
- **Count the trumps** [BF-gs #6; NM "Counting"; FGS]. **(A)**
  - Red trumps (13): Joker, RB, LB, A K Q 10 9 8 7 6 5 4.
  - Black trumps (12): the same down to the 5.

### 4.2 Side suits and ruffing
- **After the draw, run your long side suit.** Lose any necessary losers while you still hold trump control. **(C·analysis, extending [BF-c1]'s NT advice to lose your losers first)**
- **Early aces are cheap; second rounds are not.** Cashing a short side ace before drawing trumps risks a ruff only 4–13% of the time. Second rounds and long suits are much riskier (§9.8). **(M)**
- **Cross-ruffing** (both partners leading suits the other is void in) and **"laying off"** (not ruffing so that partner can win and you can discard a loser) are recognised techniques [NM]. **(C)**

### 4.3 Using partner's hand
- **Lead low cards to partner's announced aces** [BF-gs #30, #5]. #30 covers holding the Joker and both bowers, and Francis adds that it applies even without the Joker. **(C)**
- **Don't trump partner's winning trick** unless you need the lead to clear the opponents' trumps [BF-gs #31]. Whirlpool players describe "second player plays low" as the convention that avoids it [WF]. Foster gives the same advice for euchre [FH]. **(A)**
- **Don't lead trumps to partner once the opponents' trumps are drawn,** unless you have no other suit or are squeezing [NM]. **(C)**
- **The stronger hand should usually be the one that picks up the kitty** [NM]. **(C; H26)**

### 4.4 Playing NT as declarer
- **Lead your long, strong suit first,** to make the opponents' discards awkward [BF-gs #10]. **(B)**
- **Without a long suit, lose your losers first.** Lead small cards early to clear the middle cards, and keep your aces as re-entries [BF-gs #12; BF-c1: "ALWAYS LOSE YOUR NO TRUMP LOSERS FIRST"]. **(B)**
- **King with one guard:** don't lead the small card. Wait for the ace to be led [BF-c1]. **(C)**
- **King and queen:** lead the king, never the queen first [BF-gs #8; BF-c1]. **(A, also standard in bridge)**
- **Protect partner's aces.** Don't force partner to discard aces you'll need [BF-gs #11]. If partner leads to your ace and you have nothing to run, return partner's suit [BF-gs #13]. **(C)**

### 4.5 Managing the Joker in NT
- **The un-nominated Joker's rules** [PG]: you may play it only when void in the suit led. You may lead it only by nominating a suit not yet led. Once all four suits have been led, you can lead it only to the last trick. Plan any lead of it before the fourth suit opens. **(M from the rules)**
- **Nominating it before the first lead makes it the top card of one suit.** That gives an immediate stopper, for example in your unguarded suit, but costs its flexibility. One Whirlpool poster warns that nominating it can reveal your long suit [WF]. **(C·analysis; C; H15)**
- **When an opponent holds the Joker,** expect them to take one trick with it as soon as they are void in your long suit. Budget for that interruption. **(M; C·analysis)**

### 4.6 Endplays and squeezes
- **Throw-in.** Strip a defender's safe exits, then give them the lead so they must lead into your tenace (for example K-x) or give you a ruff-and-discard. The bridge definition: a defender "put on lead at a strategic moment" [WPE]. **(C·analysis, bridge analogue)**
- **Long trumps are 500's natural squeeze engine.** Running trumps forces defenders to discard. A defender guarding two suits must give one up [NM: "Squeezing"]. **(C)**

### 4.7 Slam awareness
- **Ten tricks always score at least 250** [PG]. Be alert to this, whether by design or by accident, near the end of a game [BF-gs #35]. On contracts worth less than 250, the 10th trick adds +220 (6♠), +120 (7♠) and +20 (8♠) to the score difference (§7.1). **(A for the rule; M for the numbers)**
- **When a slam would win the game, consider riskier lines** (H25). A slam needs the Joker on your side. **(C·analysis; M)**

**Sources for §4:** https://play500.com/gettingstarted.html · https://play500.com/calling1.html · https://en.wikipedia.org/wiki/500_(card_game) · https://rarepike.com/how-to-play-500/ · https://cardrulesplus.com/games/five-hundred-500/ · https://familygamesquad.com/how-to-play-500-bid/ · https://www.specialksoftware.com/terms.html · https://www.pagat.com/euchre/500.html · https://forums.whirlpool.net.au/archive/2553605 · https://gutenberg.org/cache/epub/53881/pg53881-images.html · https://en.wikipedia.org/wiki/Endplay

---

## 5. Defence

### 5.1 Following suit
- **Second hand low, third hand high** [BF-gs #9; WF]. **(A, also standard in bridge)**
- **Partner leads to your ace and you hold the king:** play the king, so partner knows you still hold the ace [BF-gs #7]. This is the bridge habit of winning with the lower of touching honours. **(A, bridge analogue)**
- **From K-Q, lead the king.** Leading the queen can tempt partner to waste the ace [BF-gs #8]. **(A)**

### 5.2 What to lead after winning a trick
(Defenders never make the opening lead, §0.)
- **Don't lead trumps to the bidding side** unless you can run them out [BF-gs #36]. Foster says the same for euchre: against a lone hand, lead plain-suit cards, never trumps [FH]. **(B)**
- **Lead long side suits early,** so the declaring side must ruff [CRP]. This is the forcing defence. It is most promising when one defender has 4+ trumps (29% when declarer has 6 red trumps, §9.6). **(C; H23)**
- **Don't automatically lead a singleton as your first side-suit lead** [NM]. **(C)**
- **Low-high:** lead your lowest cards toward partner's high cards [NM]. **(C)**
- **Finesse conventions** [NM]:
  - Play as if the missing king is on your right, and have partner lead through it.
  - Never finesse yourself. Wait for partner to lead the suit, and ask for it by discarding a small card of it.

  **(C)**
- **Lead through declarer's strength, up to weakness.** The defender on declarer's right leads, so declarer must play second and your partner sees declarer's card before playing. **(C·analysis, bridge analogue)**

### 5.3 Signals and discards
- **Discard a small card from your strongest side suit first,** asking partner to lead it [NM]. **(C)**
- **Keep low cards behind partner's aces.** When forced to discard, throw whole suits rather than bits of several [BF-c1]. **(C)**
- **Euchre analogues** [FH]:
  - Discard singletons unless they are aces.
  - Discard an ace rather than a king to show partner a winner.
  - Play the higher of two cards before the lower to signal a sure trick.

  **(C)**
- **No standard Australian signalling method was found,** and the sources above differ. AI partners must agree a scheme, and the lab should compare schemes. **(B)**

### 5.4 Ruffing, overruffing and partner's winners
- **Don't trump partner's winners.** The exception is when you must take the lead [BF-gs #31; WF; FH]. **(A)**
- **Ruff declarer's winners when you can.** Overruff to promote partner's trumps (the bridge "uppercut") only when the overruff costs nothing you need. **(C·analysis)**

### 5.5 Preserving bowers
- **A guarded RB that plays after the Joker holder is a trick.** One that plays in front can be caught by a small-trump lead followed by the Joker. When declarer leads a small trump and the Joker holder has already played to the trick (or is declarer and has just led), you can safely win with the RB. **(C·analysis; H29)**
- **Don't spend a bower ruffing a trick partner is already winning.** **(A, follows from 5.4)**

### 5.6 Defending NT
- **Holding the Joker, delay playing it** unless playing it now ensures enough tricks to set the contract [BF-gs #33]. It is your guaranteed trick and entry, and its timing is the asset. **(C; H16)**
- **Set up your long suit and count declarer's tricks.** The Joker lets the side holding it break into declarer's long suit once void. **(C·analysis)**

**Sources for §5:** https://play500.com/gettingstarted.html · https://play500.com/calling1.html · https://www.specialksoftware.com/terms.html · https://cardrulesplus.com/games/five-hundred-500/ · https://forums.whirlpool.net.au/archive/2553605 · https://gutenberg.org/cache/epub/53881/pg53881-images.html

---

## 6. Misère and open misère

### 6.1 Rules recap [PG]
- **Bidding:** misère (250) only after a 7-bid; beaten by any 8-bid. Open misère (500) at any time.
- **Play:** the partner sits out with cards face down. The contractor takes the kitty, discards three, and leads to trick 1. Open misère is exposed after trick 1.
- **Scoring:** the opponents score nothing either way.
- **The Joker:** it must be played when void in the suit led. An un-nominated Joker kept by the contractor means automatic failure. A nominated Joker is the highest card of its suit.

### 6.2 Declarer technique
- **Leading is a disadvantage.** In a successful misère the contractor leads only once, to trick 1. After that a defender always wins and leads (§9.11). Foster calls the lead a disadvantage in Solo Whist [FH]. **(M; C)**
- **The first lead** should be a card with no live card below it, so any defender who follows beats it. Choose it from a singleton, or from a suit with one card of slack: after this card, your k-th card has at most k−2 unknown cards below it. Otherwise the defenders can follow high, keep their low cards, and break the ladder in that suit. If both defenders are void in the led suit, your card wins. **(M; §9.11)**
- **Play the highest card that still loses.** Foster advises playing the card just below the highest card already played [FH]. When a defender has led, this never breaks the ladder condition (§9.11). **(C [FH]; M)**
- **When void, discard the top cards of your most dangerous suit.** Never discard from long, safe suits [FH]. Never discard a suit's lowest card (§9.11). **(B; M)**
- **Danger patterns** (Foster's "weak spots" [FH]):
  - a singleton middle card;
  - a short suit with one high card;
  - a medium or long suit missing the bottom card or cards.

  **(B)**
- **The partner's dead hand helps you.** From the contractor's seat, each unknown card is out of play with probability 1/3. Both of two "gap" cards are dead 10% of the time (§9.10). So many hands that fail the ladder test still succeed. **(M; H17)**
- **When to bid it.** [GR] says "call misère when you can," especially on weak hands. Francis notes misère hands are usually, but not necessarily, weak [BF-c1]. **(C)**

### 6.3 Defensive technique
- **Get the right defender on lead.** The player on the contractor's right must win the first trick and lead a fairly low card through the contractor. Ideally it comes from partner's short suit. One defender then runs a suit where the contractor looks safe, while the other discards into the suit where the contractor can be beaten [NM]. **(C)**
- **Why that seat:** the contractor then plays second, and the defender on the contractor's left plays last, able to duck under the contractor's card or discard. **(C·analysis)**
- **Find the weak spot.** "Every misère can be defeated, if the weak spot in it can be found" [FH]. Lead your shortest suits first, to cut the contractor's discards. Don't lead a single ace [FH]. **(C, Solo Whist analogue)**
- **Discard a whole suit early,** so later discards are free choices [FH]. **(C)**
- **A defender's Joker is a liability.** It is forced to win a trick, and on that trick the contractor can throw a dangerous card [WF: the side holding it "is guaranteed to 'win' that particular trick"]. **(C; M)**

### 6.4 Open misère specifics
- **Bid it only with a hand you can lay down safely:**
  - every suit ladder-safe or void;
  - a safe first lead;
  - no Joker.

  This follows Foster's Solo Whist spread rule: bid it only when "every suit contains the deuce" and nothing short of extraordinary circumstances can beat you [FH]. **(C·analysis)**
- **After trick 1 the defenders see your whole hand and will attack every gap.** Count all unknown cards as live. **(C·analysis)**
- **"Open misere can be the easiest hand to win if you know how"** [WF]. **(C)**
- **At any score of 0 or more, a successful open misère wins the game** (500 points). It is the classic come-from-behind bid. **(M)**

**Sources for §6:** https://www.pagat.com/euchre/500.html · https://www.specialksoftware.com/terms.html · https://play500.com/calling1.html · https://gamerules.com/rules/five-hundred-card-game/ · https://cardrulesplus.com/games/five-hundred-500/ · https://forums.whirlpool.net.au/archive/2553605 · https://forums.whirlpool.net.au/archive/2036783 · https://archive.org/stream/hoylesstandardga00unse/hoylesstandardga00unse_djvu.txt · https://gutenberg.org/cache/epub/53881/pg53881-images.html

---

## 7. Game and score strategy

### 7.1 The point economics of Avondale scoring **(M)**
"Net" means the change in (our score − their score) for the hand. Each entry assumes the contract is made exactly or set by exactly one trick. The break-even is against a hand worth zero.

| Contract | Made exactly | Set by one | Break-even P(make) |
|---|---|---|---|
| 6♠ (40) | 0 | −90 | 100% |
| 6♣ / 6♦ / 6♥ / 6NT | +20 / +40 / +60 / +80 | −110 / −130 / −150 / −170 | 85% / 76% / 71% / 68% |
| 7♠ … 7NT | +110 … +190 | −180 … −260 | 62% … 58% |
| 8♠ … 8NT | +220 … +300 | −270 … −350 | 55% … 54% |
| 9♠ … 9NT | +330 … +410 | −360 … −440 | 52% |
| 10♠ … 10NT | +440 … +520 | −450 … −530 | 51% … 50% |
| Misère / open misère | +250 / +500 | −250 / −500 | 50% |

Lessons:
- **Low contracts are almost pointless on their own account.** Their value lies in blocking the opponents or winning the game. **(M)**
- **High contracts need little more than a 50% chance of making.** **(M)**
- **Don't overbid without need.** If you make 7♥ 90% of the time, 8♥ is better only if you would make 8 tricks at least about 77% of the time. The calculation includes overtricks. **(M)**
- **The 10th trick is huge on contracts worth less than 250.** On 7♠, nine tricks nets +130, but ten tricks scores 250 and nets +250. **(M)**
- **The real baseline is not zero.** If you pass, the opponents may play and score. So the lab must measure thresholds in game-win probability, not points (H10, H11).

### 7.2 Consequences of "you can't win on defence"
- **Only a made contract wins** [PG]. Defensive points near 500 are either worthless (under a 490/460/400 cap) or insufficient to win [PG, BF-g5]. **(A)**
- **The side near 500 wins with any made contract.** At 460 or more, even a 6♠ (40) wins. The other side must outbid, or set them. **(M)**
- **A failed sacrifice is not a lost game.** It can't lose you the game unless it takes you to −500 [PG]. This is why lock-out bidding pays (§2.6). **(M)**
- **With the defence capped at 490, two sides at 490 each gain nothing from defending.** Someone will bid, and "one side is going to go down" [BF-home]. **(C, with M reasoning)**

### 7.3 Approaching 500
- **At 250 or more, a slam (250) or misère (250) wins.** At 0 or more, open misère wins, as does 10♥ (500) or 10NT (520). At 460 or more, anything wins. Re-check which contracts end the game before every auction. **(M)**
- **When you need only a small contract, bid it.** Expect the opponents to sacrifice. Decide in advance how far to push. **(C·analysis)**
- **When the opponents need only a small contract,** apply Francis's lock-out and control-bid advice and the "don't pass" rule (§2.6). **(B)**

### 7.4 The back door and bargaining power
- **Bargaining power is your score + 490** [BF-t]. Never let a single failure take you to −500 unless the opponents would otherwise win the game. **(C)**
- **Maximize the score difference.** The gap between the sides matters more than the absolute scores [BF-c1]. **(C)**
- **[CRP]'s example:** at −350, facing a possible 200-point set, consider bidding to deny the opponents even if it costs a modest underbid. **(C)**

### 7.5 Risk table by score (starting policy for AI temperament; C·analysis)
| State | Cautious style | Aggressive style |
|---|---|---|
| Both 0–300 | Bid at break-even + 10% | Bid at break-even |
| We ≥ 460 | Bid any contract likely to make | Same, plus contest every auction |
| They ≥ 460 | Sacrifice when P(they make) > 60% and our failure stays above −500 | Sacrifice when P(they make) > 40% |
| We ≤ −300 | Avoid failures of 200 or more | Seek a 500-point open misère or 10-level bid |
| Both ≥ 490 (capped variant) | Bid only with ≥ 50% to make | Bid any plausible contract |

**Sources for §7:** https://www.pagat.com/euchre/500.html · https://en.wikipedia.org/wiki/500_(card_game) · https://play500.com/gameof500.html · https://play500.com/terminology.html · https://play500.com/calling1.html · https://www.play500.com/ · https://cardrulesplus.com/games/five-hundred-500/ · https://apps.apple.com/us/app/-/id1559298243

---

## 8. Card counting and inference

### 8.1 What to track
- **Trumps:** how many are out of 13 or 12, and which of Joker, RB, LB, A, K have fallen [BF-gs #6; NM "Counting"; FGS]. **(A)**
- **Voids and short suits,** revealed by discards and ruffs [FGS: watch what the others throw; NM "Counting"]. **(A)**
- **The four aces and the Joker,** placed from the auction [BF-gs #17]. **(A)**
- **Dead cards.** Defenders face 3 unknown discards. In misère, 10 more cards (the partner's) are out of play. **(M)**

### 8.2 Inferences from bids
- **The kitty is the residual.** When the opponents' calls deny certain aces, those aces are probably in the kitty [BF-c]. More generally, any key card denied by three players' bids is in the fourth hand or the kitty. **(C; M)**
- **System-specific denials.** Under Francis's system a pass denies aces and the Joker [BF-c]. Under Maloney's, 6NT denies the Joker [NM]. Under traditional bidding, 6NT shows the Joker [BF-old]. **(B)**
- **Read the declarer's shape.** A declarer who passed up NT and bid a suit at 8+ usually has long trumps. Expect a void or two after the kitty. **(C·analysis)**

### 8.3 Inferences from play
- **Plays at "lowest of equals" carry information.** If partner wins with the king, partner has the ace [BF-gs #7]. A queen lead from K-Q would mislead partner [BF-gs #8]. **(A)**
- **The order in which declarer leads trumps carries information.** If declarer usually leads the Joker first [BF-gs #38], an opening bower lead suggests the Joker is elsewhere. Expect deception from strong players. **(C·analysis)**
- **Watch for tells.** Bryce Francis warns that an opponent who sees you move a jack after picking up the kitty may work out your strength [BF-g5]. Train AIs and players to behave the same whatever they hold. **(C)**

### 8.4 Bayesian updating examples (§9 shows the arithmetic)
- **Prior:** a card you don't hold is with partner 30.3% of the time.
- **If partner's system would have shown the Joker and didn't,** the Joker is with an opponent 87% of the time and in the kitty 13%. **(M)**
- **Illustration with assumed behaviour,** not data. Say a Joker holder passes 20% of the time and a non-holder 60%. Then P(Joker | partner passed) = 0.2·0.303 / (0.2·0.303 + 0.6·0.697) = 12.7%. The lab can estimate the real pass rates of each AI style. **(M, assumed inputs)**

**Sources for §8:** https://play500.com/gettingstarted.html · https://play500.com/calling.html · https://play500.com/gameof500.html · https://play500.com/oldway.html · https://www.specialksoftware.com/terms.html · https://familygamesquad.com/how-to-play-500-bid/

---

## 9. Probability facts (computed here; no sourced numbers)

All values are exact hypergeometric results, or exact enumerations over suit-length vectors, assuming a uniformly random deal. "Unknown" cards exclude your own hand, and as declarer also your known discards. Percentages are rounded.

### 9.1 Deck and suit sizes
- Red suits have 11 cards (4–A), black suits 10 (5–A), plus the Joker: 2×11 + 2×10 + 1 = 43.
- **Hearts trumps:** 11 hearts + J♦ + Joker = **13**. Diamonds 10, clubs 10, spades 10. **Spades trumps:** 10 + J♣ + Joker = **12**. Clubs 9, hearts 11, diamonds 11.
- **NT:** suits of 11, 11, 10, 10, plus the Joker.
- **Scale:** C(43,10) = 1,917,334,783 possible hands. 43!/(10!⁴·3!) ≈ 5.81 × 10²⁵ distinct deals.

### 9.2 Where is a missing card?
- **Before looking:** each card is in a given hand with probability 10/43 = 23.3%, and in the kitty 3/43 = 7.0%. Your partnership is dealt the Joker 20/43 = 46.5% of the time.
- **After seeing your hand** (33 unseen cards): partner 10/33 = **30.3%**, each opponent 30.3%, kitty 3/33 = **9.1%**.
- **As declarer after the kitty** (30 unknown): each other hand 10/30 = 33.3%.
- **At least one of k specific cards** is with partner with probability 1 − C(33−k,10)/C(33,10), and in the kitty with 1 − C(33−k,3)/C(33,3):

| k | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|
| partner has ≥1 | 30.3% | 52.1% | 67.5% | 78.4% | 85.8% |
| kitty has ≥1 | 9.1% | 17.6% | 25.6% | 33.0% | 40.0% |

- **If partner is known not to hold a card** (for example, a system that always shows the Joker): opponents 20/23 = **87.0%**, kitty 3/23 = 13.0%.

### 9.3 Aces, the Joker and bower pairs
- **Aces in your hand:** P(a) = C(4,a)·C(39,10−a)/C(43,10). That gives 0 aces 33.2%, 1 ace 44.2%, 2 aces 19.3%, 3 aces 3.2%, 4 aces 0.2%. For example, P(0) = 635,745,396 / 1,917,334,783.
- **Aces plus the Joker** ("key cards"): 0 → 24.7%, 1 → 42.5%, 2 → 25.5%, 3 → 6.6%, 4 → 0.7%.
- **Partner's aces when you hold none:** P(a) = C(4,a)·C(29,10−a)/C(33,10). That gives 0 → 21.6%, 1 → 43.3%, 2 → 27.8%, 3 → 6.7%, 4 → 0.5%. In general E[partner's aces] = 10·(4 − your aces)/33, or 1.21 when you hold none.
- **Kitty holds at least one ace when you hold none:** 1 − C(29,3)/C(33,3) = 1 − 3654/5456 = **33.0%**.
- **Bower pairs:**
  - Both jacks of a given colour: C(41,8)/C(43,10) = 4.98%. Both jacks of some colour: 9.80%. All four jacks: 0.17%.
  - Joker plus both jacks of a given colour: 0.97%. Of either colour: 1.92%.
  - The Joker or at least one jack: 75.3%.

### 9.4 Trump length
- **For a named suit:** P(k) = C(T,k)·C(43−T,10−k)/C(43,10).

| trumps held (10 cards) | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | mean | P(≥5) |
|---|---|---|---|---|---|---|---|---|---|---|
| red suit (T = 13) | 1.6% | 9.7% | 23.8% | 30.4% | 22.1% | 9.6% | 2.5% | 0.4% | 3.02 | 12.4% |
| black suit (T = 12) | 2.3% | 12.6% | 27.2% | 30.2% | 19.0% | 7.0% | 1.5% | 0.2% | 2.79 | 8.7% |

- **Your longest potential trump suit**, counting the Joker in every suit and each jack in both suits of its colour, computed by exact enumeration:

| longest | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|
| 10 dealt cards | 12.2% | 48.0% | 30.8% | 7.9% | 1.1% | 0.1% |
| 13 cards (hand + kitty, unconditional) | — | 15.0% | 44.9% | 29.8% | 8.7% | 1.5% |

  So P(longest ≥ 5) = 39.8% and P(≥ 6) = 9.1% for a dealt hand. The minimum is 3 by the pigeonhole principle.
- **A dealt hand has a natural void** (counting jacks in their own suit) **16.2%** of the time.

### 9.5 The kitty
Hold t trumps, so u = T − t are unseen. Then P(j trumps in the kitty) = C(u,j)·C(33−u,3−j)/C(33,3), and the mean is 3u/33.

| you hold (red trumps) | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|
| P(no trump in kitty) | 32.5% | 37.1% | 42.2% | 47.7% | 53.6% | 60.0% |
| P(exactly 1) | 46.4% | 45.5% | 44.0% | 41.7% | 38.6% | 34.6% |
| P(2 or 3) | 21.2% | 17.3% | 13.8% | 10.6% | 7.8% | 5.3% |
| expected trumps gained | 0.91 | 0.82 | 0.73 | 0.64 | 0.55 | 0.45 |

Black trumps: holding t black trumps gives the same row as holding t+1 red trumps. Worked example: holding 5 red trumps leaves 8 unseen, so P(no trump in the kitty) = C(25,3)/C(33,3) = 2300/5456 = 42.2%.

### 9.6 Outstanding trumps after the kitty (declarer holds k)
P(partner, LHO, RHO hold p, l, r) = [C(u,p)·C(30−u,10−p)]·[C(u−p,l)·C(20−(u−p),10−l)] / [C(30,10)·C(20,10)], where u = T − k. The number of trump rounds needed to strip both opponents equals the larger opponent holding.

| declarer's trumps (red) | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|
| outstanding | 8 | 7 | 6 | 5 | 4 |
| E[trumps held by the two opponents] | 5.33 | 4.67 | 4.00 | 3.33 | 2.67 |
| P(partner holds ≥1) | 97.8% | 96.2% | 93.5% | 89.1% | 82.3% |
| P(longest opponent ≤2): two rounds suffice | 11.9% | 23.7% | 41.1% | 61.8% | 81.0% |
| P(longest opponent ≤3): three rounds suffice | 54.8% | 71.3% | 84.8% | 93.8% | 98.5% |
| P(some opponent holds ≥4) | 45.2% | 28.7% | 15.2% | 6.2% | 1.5% |

Black trumps: k black trumps gives the same column as k+1 red. With 9 black trumps (3 outstanding), two rounds suffice 94.1% of the time.

### 9.7 Dropping a bare right bower with the Joker
The chance that the RB is with an opponent and is that opponent's only trump is (20/30)·C(29−(u−1),9)/C(29,9). For u = 6: (2/3)·C(24,9)/C(29,9) = (2/3)·1,307,504/10,015,005 = **8.7%**.

| declarer's trumps (red) | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|
| P(Joker lead drops a bare RB) | 3.3% | 5.4% | 8.7% | 13.6% | 20.8% |

The RB is with partner 33.3% of the time. Black trumps shift by one column; 9 black trumps gives 31.2%.

### 9.8 Side-suit breaks and ruff risk (declarer's view after the kitty)
Use the same formula as 9.6 with the side suit in place of trumps.

| holding in a 10-card side suit | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| both opponents follow to round 1 | 98% | 96% | 92% | 87% | 78% | 65% |
| … to round 2 | 80% | 70% | 56% | 39% | 22% | 7% |
| … to round 3 | 36% | 21% | 10% | 2% | 0% | 0% |
| some opponent void | 2% | 4% | 8% | 13% | 22% | 35% |

For an 11-card suit, holding s+1 gives the 10-card column for s. For a 9-card suit, holding s−1 does. The risk that an opponent is both void and holds a trump, so can ruff the first round (exact three-category enumeration):

| case | P(first-round ruff possible) |
|---|---|
| red trumps, declarer has 6, side holding 2 / 3 / 4 | 4.3% / 7.6% / 12.9% |
| red trumps, declarer has 7, side holding 3 | 7.5% |
| black trumps, declarer has 6, holding 3 in an 11-card red suit / in the 9-card black suit | 4.3% / 12.7% |

### 9.9 Partner's trump support (before the kitty; you hold t)
E = 10(T − t)/33.

| | t = 3 | 4 | 5 | 6 |
|---|---|---|---|---|
| red: E / P(≥2) / P(≥3) | 3.03 / 89.9% / 66.1% | 2.73 / 85.2% / 56.6% | 2.42 / 78.8% / 46.1% | 2.12 / 70.6% / 35.2% |
| black: E / P(≥2) / P(≥3) | 2.73 / 85.2% / 56.6% | 2.42 / 78.8% / 46.1% | 2.12 / 70.6% / 35.2% | 1.82 / 60.5% / 24.5% |

### 9.10 Misère: the partner's dead hand
From the contractor's seat after discarding, 30 cards are unknown: 10 dead in the partner's hand and 20 live with the defenders.

| g specific "gap" cards | 1 | 2 | 3 | 4 |
|---|---|---|---|---|
| all g dead: C(10,g)/C(30,g) | 33.3% | 10.3% | 3.0% | 0.8% |
| none dead: C(20,g)/C(30,g) | 66.7% | 43.7% | 28.1% | 17.7% |

Both defenders are void in a suit with 4 cards outstanding only when all four are dead: C(10,4)/C(30,4) = 0.8%. So a lead of your lowest card is almost always safe.

### 9.11 Certainties (proof sketches)
1. **The Joker is always one trick for its holder (all contracts except misère).** It beats every card. Its holder can always keep it until the last trick, when it is the only card left. Leading to the last trick is legal, and following with it is legal because the holder has no card of the suit led. So it wins at least one trick, and it can win only one.
2. **Ten tricks require the Joker on your side.** This follows from 1.
3. **After the kitty, the declarer can always create a side-suit void** (with at least 2 trumps among the 13 cards). The three side suits share at most 11 cards, so the shortest has at most 3, and three discards can clear it.
4. **The defenders never make the opening lead,** and **a successful misère contractor leads only at trick 1.** Any later lead by the contractor would mean they had won a trick.
5. **Ladder lemma (misère).** Sort your cards in a suit c₁ < c₂ < …. If, for every k, at most k−1 live outstanding cards of that suit rank below c_k, you can never be forced to win a trick in that suit. Whenever the suit is led at x, play your highest card below x, or below the current winning card. That card exists because c₁ has no live card below it. The condition survives every trick **led by a defender**. Your cards above the one you played lose one index and at least one lower live card (the led card). Your cards below are not made worse. It also survives discarding a suit's **top** card, but not its bottom card. **Caveat 1:** your own lead of the suit (only possible at trick 1) can break it, because the defenders may follow high and keep their low cards. A safe trick-1 lead is a card with no live card below it, from a singleton or from a suit where, for k ≥ 2, at most k−2 live cards lie below c_k. **Caveat 2:** if you lead the suit and both defenders are void in it, you win the trick. That happens only when you hold nearly all of the suit (§9.10). For a guarantee (open misère), treat every unknown card as live.

**Sources for §9:** none. All values were computed for this document. The rules facts used are from https://www.pagat.com/euchre/500.html.

---

## 10. Common mistakes and how expert thinking differs

| Common mistake | Expert alternative and why | Label / source |
|---|---|---|
| Trumping partner's winner | Ruff only when you need the lead, for example to draw trumps; otherwise discard a loser. | (A) [BF-home, BF-gs #31, WF, FH] |
| Discarding a trump from the kitty | Keep every trump unless forced; discard side-suit losers. | (A) [BF-c1, WP] |
| Bidding without checking the score | Recompute which contracts end the game, and your bargaining power, before every auction. | (A) [BF-gs #3] |
| Passing while the opponents bid a game-winning contract | Sacrifice or lock out; your failure can't make them win. | (B) [BF-gs #16, BF-c1, CRP] |
| Treating 6-level contracts as safe, cheap points | A made 6♠ nets about 0 once defenders' tricks are counted; bid low only to compete, block or win. | (M) §7.1 |
| Counting on the kitty for 1–2 tricks | Expect about 0.5–1 trick; the reliable value is the discard. | (B) [RP vs GR]; (M) §9.5 |
| Assuming two trump leads clear the defenders | Count the trumps out. With 6–7 trumps, three rounds are typically needed. | (M) §9.6 vs [CRP, FGS] |
| Leading the Joker by reflex | Consider partner's possibly bare trump ace, concealment, and whether the Joker is better kept as control. | (B/C) [BF-gs #38–40]; (M) §9.7 |
| Leading trumps when only your side has them | Stop drawing; use the remaining trumps for control and ruffs. | (A) [BF-gs #6] |
| Leading the queen from K-Q | Lead the king. | (A) [BF-gs #8] |
| Playing the Joker early when defending NT | Hold it as your guaranteed trick and entry until it beats the contract. | (C) [BF-gs #33] |
| Keeping the Joker in a misère | Discard it, or nominate it and shed it on a void. | (A) [PG, HY24] |
| Bidding misère on a hand with gaps and no voids | Apply the ladder test; count the gaps and the chance those cards are dead. | (M) §9.11; (C) [FH] |
| Misère defenders leading from either seat | The contractor's right-hand opponent should win and lead through the contractor. | (C) [NM] |
| Forgetting the left bower changes suit | Re-sort your hand by suit before bidding. Francis suggests placing the jacks with your strongest suit. | (A) [CGC, BF-g5] |
| Raising partner on modest support | Raise only with extra values; partner already assumed average support. | (C) [WP] |
| Table talk, tells and "signals" | Use only bids and plays; keep your manner neutral. | (A) [BF-gs #21, #25; AK] |
| Ignoring the slam bonus | On cheap contracts the 10th trick is worth up to +220 (on 6♠; +120 on 7♠); near game it can win outright. | (A/M) [BF-gs #35, PG] |

**Skill ladder for AI design** (C·analysis; each rung adds to the one before):

| Skill area | Beginner | Intermediate | Expert |
|---|---|---|---|
| Evaluation | High cards in the longest suit; may forget the left bower | Top trumps + aces + length | Rounds-to-draw, bower position, partner and kitty priors, ladder test |
| Bidding | Natural; ignores the score | One convention; knows front and back door | Reads every bid for aces and the Joker; break-even thresholds; sacrifice and lock-out; considers NT |
| Kitty | Discards its lowest cards | Voids one suit | Trades voids against guards against partner entries |
| Declarer | Cashes aces first | Joker first, draws trumps | Counts trumps, stops the draw at the right time, unblocks, endplays, slam lines |
| Defence | Plays its highest card | Second hand low, third hand high | Bower position, agreed signals, forcing defence, inference from the discard |
| Misère | Bids any weak hand | Avoids high cards and the Joker | Full ladder analysis; the right-hand-opponent defence |

Temperament knobs (cautious ↔ aggressive): the bidding threshold relative to the §7.1 break-evens, willingness to sacrifice, how often misère is bid, whether to play for slams, and how deceptive the trump-lead order is.

**Sources for §10:** as cited in the table, plus https://www.play500.com/ · https://play500.com/gettingstarted.html · https://www.cardgames.club/five-hundred/

---

## 11. Hypotheses to test in simulation

**Method notes.**
- **Pair the deals.** Use duplicate deals, with each policy pair playing the same deals from both sides, and seat rotation. Report paired differences with 95% confidence intervals.
- **Separate card play from the auction.** Measure card-play claims with a fixed bidding policy, and auction claims with a fixed play engine.
- **Two play engines.** Use a double-dummy solver for upper bounds and an imperfect-information player (for example, determinized Monte Carlo) for realistic results.
- **Correct for multiple comparisons** across the list.

| # | Claim | Suggested measurement | Origin |
|---|---|---|---|
| H1 | With ≥5 trumps including the Joker, leading the Joker at trick 1 makes the contract at least as often as leading a bower or a small trump. | Three lead policies on identical deals, everything else fixed. Make rate and mean tricks, by trump count and bowers held. | [BF-gs #38], [FGS] |
| H2 | Holding the RB (± LB) without the Joker, leading the RB at the first trump lead beats leading low trumps. | Filter deals with that holding; paired make rate; check where the Joker sat. | [BF-gs #39] |
| H3 | With Joker + RB + LB and partner holding the trump ace, a low trump first gains tricks over Joker first when partner's ace is bare, and costs little otherwise. | Net trick difference, split by whether the ace is bare; repeat with partner's ace shown in the auction. | [BF-gs #40] |
| H4 | "Two rounds of trumps usually clear the defenders" is false for 5–8 trumps. | Count defender trumps left after two declarer-led rounds; compare with §9.6 (also validates the simulator). | [CRP], [FGS] vs (M) |
| H5 | Never discarding a trump beats a policy that discards the lowest trump (when holding ≥7) to keep a guarded side king. | Paired make rate on deals where both discards are possible. | [BF-c1] |
| H6 | The kitty is worth about 0.5–1.0 trick on average, not 1–2. | Double-dummy tricks with the best 10 of 13 cards versus the original 10; distribution by hand type. | [RP] vs [GR] |
| H7 | A silent partner contributes about 1–1.5 tricks to the declarer's contract. | Regress declarer-side tricks on declarer hand features; estimate the partner term; condition on partner's bid. | [WP] |
| H8 | With ≥6 trumps after the kitty, voiding a side suit beats keeping K-x; with ≤5, keeping K-x wins. | Two discard policies × trump-count strata; make rate. | [WP], [BF-home], [BF-c1] |
| H9 | Joker + LB + 4 small trumps out-performs Joker + RB + 1 small at the 7–8 level. | Matched hand pairs with equal side cards; make rate by level. | [FGS] |
| H10 | At neutral scores, made 6♠/6♣ contracts change the score difference by about 0 to +20, and sets cost about −90 to −110. | Realized score-difference distributions by contract and outcome. | (M) §7.1 |
| H11 | The win-maximizing bidding threshold at neutral scores is close to the §7.1 break-even for each level, and falls toward 50% at higher levels. | Policy search over P(make) thresholds by level; game-win rate. | (M) |
| H12 | Bidding systems differ in win rate: natural, Joker-6NT, Francis aces-first, Maloney 6♠-Joker/rainbow. | Round-robin duplicate matches; win rate; how often the partnership reaches its best makeable contract. | [BF-c], [NM], [WP] |
| H13 | After an opponent's red bid, your best suit is more often black, and your black contracts make more often than baseline. | Conditional suit-colour frequencies; make rates in competitive auctions. | [FGS] |
| H14 | 7NT/8NT make rates rank: Joker + long suit > Joker + 2 aces > 3 aces without the Joker. 9NT with four aces and no Joker makes more than 50% of the time. | Filtered deal sets; make rates by holding. | [GR], [BF-gs #27], [BF-old] |
| H15 | Nominating the NT Joker to an otherwise unstopped suit beats leaving it un-nominated, which beats nominating it to the long suit. | Three nomination policies; make rate. | [PG] rules, [WF], C·analysis |
| H16 | Defending NT, holding the Joker until it sets the contract beats using it at the first void. | Set rate, paired. | [BF-gs #33] |
| H17 | Misère success is predicted by ladder deficit, voids and highest card held; ladder-safe hands never fail. | Logistic model of success on misère hands; any failure of a ladder-safe hand is an engine bug. | (M) §9.11, [FH] |
| H18 | Maloney's misère defence (the right-hand opponent wins trick 1 and leads low through; one partner runs a safe suit while the other discards) beats naive defence. | Defeat rate, paired. | [NM] |
| H19 | Misère discards "from the top of the most dangerous suit" beat "highest cards overall" and random discards. | Misère success rate by discard policy. | (M), [FH] |
| H20 | Exposure costs open misère X percentage points against closed misère on the same hands. Some hands have higher expected value as open misère (500). | Same hands played both ways; success rates and expected values. | [PG], [WF] |
| H21 | Second hand low / third hand high beats "second hand high with bowers" as a default. | Defensive tricks, paired. | [BF-gs #9], [WF] |
| H22 | "Never lead trumps to the bidder unless you can run them out" beats leading trumps when declarer's partner is likely short. | Defensive tricks; set rate. | [BF-gs #36] |
| H23 | Forcing defence (leading declarer's void suit) raises the set rate when a defender has ≥4 trumps. | Set rate conditional on the defenders' trump lengths. | [CRP] |
| H24 | When the opponents need ≤140 to win and would hold the contract, a sacrifice bid with P(make) ≤ 30% raises our game-win probability, unless failing would take us to −500. | Win probability from sampled score states; sacrifice versus defend. | [BF-c], [BF-c1], [CRP] |
| H25 | When a slam (250) would win but the contract's value would not, max-tricks play raises the game-win rate despite more failed contracts. | Paired games from such states. | [BF-gs #35], [PG] |
| H26 | When both partners could declare the same strain, the stronger hand declaring makes contracts more often. | Forced-declarer experiments on identical deals. | [NM] |
| H27 | Full card counting gives a large win-rate edge over counting trumps only, which beats no counting. | Agent ablation; win rate (helps decide lesson priority). | [BF-gs #6, #17], [NM] |
| H28 | Defenders who assume declarer has voided his shortest side suit are right more than 70% of the time; a heuristic built on this improves defence. | Accuracy of the inference; defensive tricks with and without the heuristic. | C·analysis |
| H29 | A guarded RB held until the Joker appears gains more defensive tricks than winning at first chance, especially when sitting in front of the Joker holder. | Defensive tricks by policy × seat relative to the Joker. | C·analysis |
| H30 | Under a 490 cap, sacrifices are more frequent and games are longer than with no cap. With both sides at 490, P(the contract fails) is high. | Compare rule variants; failure rate in both-at-490 states. | [BF-home], [PG] |

---

## Sources

**500-specific, read directly**
- [PG] John McLeod, "Five Hundred," pagat.com: https://www.pagat.com/euchre/500.html
- [WP] "500 (card game)," Wikipedia: https://en.wikipedia.org/wiki/500_(card_game)
- [BF-home] Bryce Francis, play500.com home: https://www.play500.com/
- [BF-gs] Bryce Francis, "Before getting started" (numbered checklist): https://play500.com/gettingstarted.html
- [BF-g5] Bryce Francis, "The game of 500": https://play500.com/gameof500.html
- [BF-c] Bryce Francis, "Calling four handed": https://play500.com/calling.html
- [BF-c1] Bryce Francis, "Calling four handed" (continued): https://play500.com/calling1.html
- [BF-old] Bryce Francis, "Calling Traditional": https://play500.com/oldway.html
- [BF-t] Bryce Francis, "Terminology": https://play500.com/terminology.html
- Bryce Francis, "Other forms of 500" (consulted; three-handed advice not used): https://play500.com/other.html
- [NM] Neil J. Maloney, "Explanation of 500 terms," Special K Software: https://www.specialksoftware.com/terms.html
- [CRP] cardrulesplus.com: https://cardrulesplus.com/games/five-hundred-500/
- [GR] gamerules.com: https://gamerules.com/rules/five-hundred-card-game/
- [RP] rarepike.com: https://rarepike.com/how-to-play-500/
- [FGS] familygamesquad.com (5-card-kitty variant): https://familygamesquad.com/how-to-play-500-bid/
- [AK] academickids.com (older encyclopedia text): https://academickids.com/encyclopedia/index.php/500_(game)
- [OC] US regional rules (Youngstown, St Paul, Aus/NZ notes): https://www.oocities.org/timessquare/2767/fivehun.html
- [TC] Trickster Cards, "500 Basics": https://www.trickstercards.com/help/500/
- [CGC] cardgames.club: https://www.cardgames.club/five-hundred/
- [NP] NeuralPlay "Five Hundred (500) – Expert AI" app listing (defenders'-points cap option): https://apps.apple.com/us/app/-/id1559298243

**Forums (read only through search-engine snippets; direct fetch was blocked)**
- [WF] Whirlpool: https://forums.whirlpool.net.au/archive/2553605 · https://forums.whirlpool.net.au/archive/2036783 · https://forums.whirlpool.net.au/archive/2668538
- [BGG] BoardGameGeek: https://boardgamegeek.com/thread/104185/500-review · https://boardgamegeek.com/thread/2702105/rules-for-playing-joker-in-no-trump-and-misere-bid

**Analogues and background**
- [FH] R. F. Foster, *Foster's Complete Hoyle* (1914 revision), Solo Whist misère and Euchre strategy: https://gutenberg.org/cache/epub/53881/pg53881-images.html
- [HY24] *Hoyle's Standard Games* (1924), Five Hundred and Nullo: https://archive.org/stream/hoylesstandardga00unse/hoylesstandardga00unse_djvu.txt
- [WPE] "Endplay," Wikipedia: https://en.wikipedia.org/wiki/Endplay

**Identified but not used for claims**
- Andrew Blucher, "How to Play 500" (archive copy; could not be fetched): https://web.archive.org/web/20080501144119/www.csse.monash.edu.au/%7Eandyb/500.html
- Clubs New Zealand 500 tournament pages. The rules PDFs were not readable as text, so no tournament conventions are claimed: https://www.clubsnz.org.nz/tournaments/500/tournament/clubs-new-zealand-2025-south-island-500-tournament
- *500 Card Game Deep Dive* (D. Thayer, 2025). Bookseller listing only; contents not consulted.
- "Hay's Rule Book: 500" (rec.games.playing-cards). The fetch was rate-limited; it is known only from a search snippet confirming the "6NT = Joker" convention.
