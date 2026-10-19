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
| Each **guarded side king** (a king with at least one other card in that suit) | **⅕** |
| Each side void (no cards in a suit) | ⅕ |
| Each side singleton (one card in a suit) | ⅛ |

A lone side king (no other card in the suit) counts nothing: it's usually caught by the ace.

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
| Each **guarded king** (a king with at least one other card in the suit) | **½** |
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
| **Your partner called 6** (they hold the Joker) | **+¾** | **+1** |
| Your partner mentioned this suit (or NT) | +0.6 | +0.7 |
| Your partner bid this suit (or NT) | +2¼ | +2¼ |
| **Your partner passed** | **−0.9** | **−1.3** |
| An opponent called 6 | −½ | −0.7 |
| An opponent mentioned this suit (or NT) | −⅔ | −0.7 |
| An opponent bid this suit (or NT) | −2 | −2 |
| You speak last (you're the dealer) and no opponent has shown strength | +½ | +0.6 |

---

## What the count says to bid

### With no one bidding against you

| Your count | Bid | How often it's made |
|---|---|---|
| Under about 7 (6¾ in clubs, diamonds, hearts or no trumps) | Pass, mention or call 6 | |
| About 7 | **7** | about 60–65% |
| About 8½ | **8** | about 78% |
| About 10 | **9** | about 80–85% |

**Why 8 waits for 8½:** each extra level adds only 100 points when you make it, but one trick short turns a plus
into a minus. With 8 tricks, 8♦ scores +280 and 9♦ scores −380. So go up a level only when you'd make the
higher bid about 80% of the time. Bidding higher to stop players still to call doesn't change this: across
20,000 deals played out, a 9 bid never paid below a count of 10, from any seat.

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
