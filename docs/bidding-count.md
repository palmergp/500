# Counting your hand for a bid

This is every number in the coach's "rule-of-thumb count", the line you see in a bidding hint such as
*"Rule-of-thumb count for ♥: about 8¼ (partner and kitty 3, Joker 1½, …)"*.

The count estimates how many tricks your side will take if you declare. Add up your cards, add the allowance for
your partner and the kitty, adjust for what the others have called, and compare the total with the bid.

*Where the numbers come from:* 64,000 bidding decisions, each solved with every card visible, fitted to the
tricks the bidder's side actually took (`train/rules_of_thumb.py`; `docs/rules-of-thumb.md` rule 1).
They're rounded to easy fractions. The count is a guide, not a guarantee: two hands with the same count
can differ by a trick or more. The coach's own recommendation comes from a neural network trained on the same
solved deals, so now and then it disagrees with the count.

---

## Bidding a suit

Count these with the suit you're thinking of making trumps.

### Trumps

| Card | Tricks |
|---|---|
| **Joker** | **1½** |
| **Right bower** (the jack of trumps) | **1¼** |
| **Left bower** (the other jack of the same colour) | **1** |
| Ace of trumps | ¾ |
| King of trumps | ⅔ |
| Queen of trumps | ½ |
| Each other trump (10 and below) | ½ |

### Side suits

| Holding | Tricks |
|---|---|
| Each side ace | ⅔ |
| Each side king **beside its ace** (A-K in the same suit), on top of the ace | **⅓** |
| Each side king with **two or more guards** (other cards in that suit) | **⅕** |
| Each side king with just one guard (K-x) | ⅛ |
| Each side void (no cards in a suit) | ⅕ |
| Each side singleton (one card in a suit) | ⅛ |

A lone side king (no other card in the suit) counts nothing: it's usually caught by the ace. Side queens count
nothing either, however they're guarded (under 0.1 of a trick each). See [how often side kings and queens win a
trick](#side-kings-and-queens-how-often-they-win-a-trick) below.

### Then add

| | Tricks |
|---|---|
| **Your partner and the kitty** | **3** |

### Example

You hold Joker, J♥ 9♥ 7♥ 5♥, A♠ 8♠ 6♠, 9♦ and 4♣, and you're counting hearts:

| | |
|---|---|
| Joker | 1½ |
| Right bower (J♥) | 1¼ |
| Three other trumps (9♥ 7♥ 5♥), ½ each | 1½ |
| Side ace (A♠) | ⅔ |
| Two singletons (9♦, 4♣), ⅛ each | ¼ |
| Partner and kitty | 3 |
| **Count** | **about 8: bid 7♥** (8♥ waits for about 8½) |

---

## Bidding no trumps

In no trumps there are no trump suits and no bowers: both jacks are ordinary jacks. The Joker is the only trump.

| Card or holding | Tricks |
|---|---|
| **Joker** | **2¼** (worth more than in a suit: it's the only trump, and it stops any suit) |
| Each ace | 1 |
| Each king with **two or more guards**, or beside its ace | **½** |
| Each king with just one guard (K-x) | ⅓ |
| Each guarded queen (a queen with at least two other cards in the suit) | ¼ |
| Each long card in a suit headed by its ace (each card beyond the third) | ⅛ |
| **Your partner and the kitty** | **3⅓** |

Long suits without the ace count nothing extra: in no trumps the opponents take their tricks first.

**Taking the kitty in no trumps:** keep your aces, your guarded kings and the small cards guarding them. Put back
low cards from suits with no honours. A void doesn't help: there's nothing to trump with (`docs/rules-of-thumb.md`,
the kitty).

---

## Adjusting for the auction

Add or subtract these for what the other three players have called. (Your own calls don't change your count.)

| Call | Suit bid | No trumps |
|---|---|---|
| **Partner called 6 in this suit (they hold the Joker)** | **+1⅓** | — |
| Partner called 6 in the left bower's suit (same color) | +0.6 | — |
| Partner called 6 in another suit | +¼ | +0.8 |
| Partner mentioned this strain | +0.4 | +0.4 |
| **Partner mentioned another suit (their strength is elsewhere)** | **−⅔** | −1⅛ |
| Partner mentioned NT | −¼ | (this strain: above) |
| Partner bid this strain | +2¼ | +2¼ |
| **Partner passed** | **−1** | −1.6 |
| **An opponent called 6 in this suit** | **−1** | — |
| An opponent called 6 in the left bower's suit (same color) | −0.4 | — |
| An opponent called 6 in another suit | −⅛ | −0.4 |
| An opponent mentioned this strain | −½ | −0.4 |
| An opponent mentioned another suit | +¼ | +½ |
| An opponent mentioned NT | 0 | (this strain: above) |
| An opponent bid this strain | −2 | −2 |
| You speak last (dealer), with no opposing bid, mention or 6 in this strain | +½ | +0.6 |

---

## What the count says to bid

### With no one bidding against you

| Your count | Bid | How often it's made |
|---|---|---|
| Under about 7 (6¾ in clubs, diamonds, hearts or no trumps) | Pass, mention or call 6 | |
| About 7 | **7** | about 60–65% |
| About 8½ or more | **8** | about 78% |
| (9 almost never: not even at 11) | | |

**Why 8 waits for 8½:** each extra level adds only 100 points when you make it, but one trick short turns a plus
into a minus. With 8 tricks, 8♦ scores +280 and 9♦ scores −380. So go up a level only when you'd make the
higher bid about 80% of the time. Bidding higher to stop players still to call doesn't change this: across
20,000 deals played out, a 9 bid never paid below a count of 10, from any seat, and checked against the coach a 9
almost never paid even at 10 to 12 (lab finding 32). At 8½, 7 and 8 score about the same on average, so the coach
may prefer either for a particular hand.

**After an opponent has bid:** pass unless your count is about 8½ or more.

### Near the end of the game

| Score | Bid |
|---|---|
| They're near 500 (say 460) and you're not | Bid even a light hand (a count of 6½), at the cheapest level that takes you to 500 if you make it. Passing lets them win on defense with four tricks. |
| You're near 500 and they're not | 7 is enough: any made bid wins. Over an opponent's bid you can pass, since four defensive tricks win too. |
| Both near 500 | Bid 7 even with a light hand: any made bid wins, and the declarers count first. |

---

## Choosing between a suit and no trumps

Count both. Some measured guides (`docs/rules-of-thumb.md`, rules 5–5b):

- **Without the Joker,** no trumps beats your best suit only about 30% of the time.
- **With the Joker, two aces and no suit longer than four,** no trumps scores better about 79% of the time.
- **In a suit, top trumps beat length:** the Joker or a bower is worth about ¾ of a trick more than a small trump,
  at any length.

---

*Kept in step with `crates/ai/src/thumb.rs` (the coach's count) and `docs/rules-of-thumb.md`. If either changes,
update this page.*

## Side kings and queens: how often they win a trick

*Measured by `fh-lab honors`: the Oracle played 4,000 deals in all four seats, and every side-suit king and queen
was followed to see whether that card itself won a trick (lab finding 25).* A **guard** is another card of the
same suit in the same hand.

| Holding | Suit bids: won a trick | …when you declare | No trumps: won a trick | …when you declare |
|---|---|---|---|---|
| King alone | 20% | 40% | 32% | 57% |
| King with 1 guard | 36% | 46% | 51% | 57% |
| King with 2 guards | 28% | 53% | 57% | 68% |
| King with 3+ guards | 15% | 46% | 51% | 77% |
| King with the ace (A-K) | 53% | 84% | 83% | 96% |
| Queen alone | 8% | 22% | 11% | 21% |
| Queen with 1 guard | 13% | 25% | 18% | 34% |
| Queen with 2 guards | 16% | 33% | 32% | 47% |
| Queen with 3+ guards | 10% | 41% | 33% | 64% |
| Queen with the king (K-Q) | 21% | 52% | 42% | 65% |

- **In a suit bid a long side suit gets trumped.** A king with three guards wins less often than one with a
  single guard: by the time the suit comes round a third time, someone is void.
- **Where the ace sits matters.** A guarded king without its ace, in a suit bid / no trumps, won 19% / 43% with
  the ace on your left (it plays after you), 26% / 54% on your right (before you), 31% / 60% with your partner,
  and 41% / 69% in the kitty.
- **What it's worth when you bid** is less than how often it wins: a king that wins a trick often takes one you'd
  have won anyway, with a trump or another card. Held equal on everything else in the hand, the extra tricks
  for your side as declarer were:

| Holding | Suit bid | No trumps |
|---|---|---|
| King alone | +0.08 | +0.20 |
| King with 1 guard | +0.13 | +0.32 |
| King with 2+ guards | +0.20 | +0.51 |
| King with the ace (on top of the ace) | +0.30 | +0.50 |
| Queen alone | +0.03 | +0.01 |
| Queen with 1 guard | +0.08 | +0.12 |
| Queen with 2+ guards | +0.07 | +0.23 |
| Queen with the king | +0.08 | +0.20 |

These are the count's values above, rounded: in a suit bid only kings count (⅛ to ⅓), and in no trumps a
guarded king is worth ½ and a queen with two guards ¼.
