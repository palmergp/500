# House Rules (the engine's default ruleset)

This is the authoritative spec for the rules engine. Each rule has an ID; the tests in
`crates/core/tests/` cite these IDs. Background and other variants: `docs/research/01-rules-and-variants.md`.

## Table and cards

- **H1. Players.** Four players in two partnerships; partners sit opposite. Seats in play order (clockwise):
  South → West → North → East. North–South is one team, East–West the other.
- **H2. Pack.** 43 cards: ♥ and ♦ 4–A, ♠ and ♣ 5–A, plus one Joker.
- **H3. Deal.** 10 cards to each player and 3 to the kitty. The first dealer is chosen at random; after each
  completed hand the deal passes to the left.

## Card ranking

- **H4. Trumps.** In a suit contract the trumps rank: Joker > right bower (jack of trumps) > left bower (other jack
  of the same color) > A > K > Q > 10 > 9 > … down to the suit's lowest card. A red trump suit has 13 trumps and a
  black one 12.
- **H5. The left bower is a trump.** It is not a card of its printed suit. Example with hearts trumps: J♦ follows a
  heart lead, does not follow a diamond lead, and a player whose only "diamond" is J♦ is void in diamonds. Leading
  J♦ is leading a trump. The Joker likewise belongs to the trump suit.
- **H6. Non-trump suits** rank A > K > Q > J > 10 > … (a suit that lost its jack to the left bower simply has no jack).
- **H7. No trumps.** There are no bowers; all four jacks are ordinary cards. The Joker is a suit of its own and the
  only trump.

## Bidding

- **H8. Once around.** Calling starts left of the dealer; each player makes exactly one call; the dealer calls last.
- **H9. Calls.** Pass, *mention* a strain, *6 of a strain* (including 6NT), or a *bid* of 7–10 in a strain.
  Strains rank ♠ < ♣ < ♦ < ♥ < NT.
- **H10. Mentions** say "this is my strongest suit (typically I hold a bower), but I can't bid." A mention must
  outrank any earlier mention, so a suit can't be mentioned twice. Mentioning NT is allowed.
- **H11. 6 of a strain** says "I hold the Joker, and this is my best suit" (6NT: "no trumps suits me"). It may
  name any strain, including NT, whatever has been mentioned. It never becomes the contract and never scores.
- **H12. Signals close.** Mentions and 6-calls are allowed only while nobody has called 6 or bid 7+. After that a
  player may only pass or bid.
- **H13. Bids** are 7–10 tricks in a strain and must beat the previous bid by value (table in H19). The highest bid
  becomes the contract and its bidder the *declarer*.
- **H14. No bid.** If no one bids 7+, the cards are thrown in and the same dealer redeals. This may happen twice.
  On the third deal, if no one has bid by the dealer's turn, the dealer must bid (pass, mentions and 6 are not
  allowed).
- **H15. No nullo** (misère).

## Play

- **H16. Kitty.** The declarer adds the kitty to their hand and discards any 3 cards face down. Nobody else sees
  the kitty or the discards.
- **H17. Leads.** The declarer leads to the first trick; the winner of each trick leads to the next.
- **H18. Following suit.** A player must follow the suit led if they can, where suit means *effective* suit (H5).
  Otherwise they may play any card.
- **H18a. The Joker in no trumps** may be played only when its holder can't follow the suit led, and is never
  compulsory then. It may be led; the leader names any suit, and the others must follow that suit. It wins every
  trick it is played to.
- **H18b. Winning a trick.** The Joker wins in no trumps. Otherwise the highest trump wins; with no trump played,
  the highest card of the suit led wins.

## Scoring and winning

- **H19. Contract values.**

  | Tricks | ♠ | ♣ | ♦ | ♥ | NT |
  |---|---|---|---|---|---|
  | 7 | 140 | 160 | 180 | 200 | 220 |
  | 8 | 240 | 260 | 280 | 300 | 320 |
  | 9 | 340 | 360 | 380 | 400 | 420 |
  | 10 | 440 | 460 | 480 | 500 | 520 |

- **H20. Declarers** who take at least the tricks bid score the contract's value (no points for extra tricks); if
  they fall short they lose its value.
- **H21. Defenders** score 10 points for every trick they take, whether or not the contract is made.
- **H22. Slam.** Declarers who take all 10 tricks score 250, or the contract's value if that is higher.
- **H23. Winning.** A team wins only on a hand it bids: declarers who make their bid and have 500 or more at the
  end of the hand win. Defensive points count toward the score but never win the game: a team can pass 500 on
  defense and play on (it then wins by making any bid). There is no cap on defensive points.
- **H24. Out the back door.** A team at −500 or worse at the end of a hand loses. Only declarers lose points, so
  only the declarers can go out this way, and at most one team meets a win or loss condition on any hand.
- **H25. Short game (optional).** Instead of playing to 500, play a set number of hands (four at our table). Deals
  that are passed out and redealt don't count. After the last hand the higher score wins, defensive points
  included; if the scores are tied, play one more hand at a time until they aren't. H23 and H24 don't apply:
  reaching 500 or −500 doesn't end a short game.

## Assumptions to confirm

- **A1.** The engine allows any legal call, but every AI makes mentions and 6-calls honestly.
- **A2.** If every call on a deal is a pass, mention or 6, the deal counts as passed out (H14).
