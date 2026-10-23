# Console 0.8.4 manual acceptance — fresh v7 deal

Recorded: 19 October 2026, Australia/Brisbane.

Charlie followed the Rust-generated route on the freshly acquired matching guest
board and reported: “I followed through the sequence and it worked perfectly!”
This is operator-reported live acceptance of M05–M09. The complete sequence he
supplied is retained below; no new guest observation or input was performed while
recording this acceptance.

## Correlated inputs and result

- Deal: [pyramid-recon-20261019-v7-01.json](../Evidence/pyramid-recon-20261019-v7-01.json),
  4,743 bytes, SHA-256
  `7c18883152eb6ad05d39cdb7f82523dfce039497e07591832fd72f0cb79b50de`.
- Acquisition producer: `qmp-qemu-socket/2.4.101`, native-v7 profile SHA-256
  `9517ce85382ec0c977e735c79871c1e2f8110b7265343dc1140ac45d506e3e08`.
  The previously audited diagnostics record 31 captures, 24 acknowledged draws,
  acknowledged Undo All/confirmation, complete inventory, restoration and
  publication, with no error or uncertain input delivery.
- Solver: `basic-route-planning 0.8.4`, release SHA-256
  `ebafa907ffe1936a71be36823d1db7ddedeeba1cc8787d5c65cc26d4fd6431cf`.
- Reported route: **49 logical actions: 24 MATCH, 20 MOVE, 4 KING, 1 RECYCLE**.
  The final row has tableau zero and both DL and DR empty. These removals account
  for all 52 cards.
- The successful manual trial exercises the printed selection order, stock/waste
  matching, pile updates, King removal and one recycle for this deal. It does not
  independently exercise the maximum two-recycle boundary, establish a three-board
  GAME WIN or verify automated QMP execution. No new minimum-cost search or timing
  measurement was performed for this documentation update.

M01–M09 are Complete and accepted. M10/M11 remain Planned. The next implementation
approval is M10's guarded independent guest execution contract and scope; this
acceptance does not authorise that work, commits, pushes or deployment.

Versions remain unchanged: console 0.8.4, acquisition application 2.4.101 and xSAR
0.4.100. This documentation/live-acceptance update earns no build increment and
does not reapply the five minor credits already recorded for M05–M09.

## User-supplied successful sequence

The following is the route Charlie supplied with his successful-play confirmation.
It belongs to this v7 deal, separately from the earlier v6 deal's 48-action trace.

```text
Sequence Action  CARD1              CARD2              DL after   DR after   Tableau
1 MATCH   7|6=AS(1)          7|7=QC(12)         4S(4)      -               26
2 MATCH   6|6=7S(7)          7|4=6C(6)          4S(4)      -               24
3 MOVE    DL=4S(4)           -                  4H(4)      4S(4)           24
4 MOVE    DL=4H(4)           -                  5C(5)      4H(4)           24
5 MOVE    DL=5C(5)           -                  8S(8)      5C(5)           24
6 MATCH   DL=8S(8)           DR=5C(5)           5D(5)      4H(4)           24
7 MOVE    DL=5D(5)           -                  6H(6)      5D(5)           24
8 MOVE    DL=6H(6)           -                  3D(3)      6H(6)           24
9 MOVE    DL=3D(3)           -                  8H(8)      3D(3)           24
10 MATCH   7|3=5S(5)          DL=8H(8)           KD(13)     3D(3)           23
11 KING    DL=KD(13)          -                  JC(11)     3D(3)           23
12 MOVE    DL=JC(11)          -                  JH(11)     JC(11)          23
13 MOVE    DL=JH(11)          -                  KS(13)     JH(11)          23
14 KING    DL=KS(13)          -                  3H(3)      JH(11)          23
15 MOVE    DL=3H(3)           -                  9D(9)      3H(3)           23
16 MOVE    DL=9D(9)           -                  3C(3)      9D(9)           23
17 MOVE    DL=3C(3)           -                  10D(10)    3C(3)           23
18 MATCH   DL=10D(10)         DR=3C(3)           2D(2)      9D(9)           23
19 MOVE    DL=2D(2)           -                  4C(4)      2D(2)           23
20 MOVE    DL=4C(4)           -                  10S(10)    4C(4)           23
21 MATCH   7|5=3S(3)          DL=10S(10)         AC(1)      4C(4)           22
22 MATCH   6|4=QS(12)         DL=AC(1)           9H(9)      4C(4)           21
23 MATCH   DL=9H(9)           DR=4C(4)           10H(10)    2D(2)           21
24 MATCH   6|5=JD(11)         DR=2D(2)           10H(10)    9D(9)           20
25 MATCH   5|5=6D(6)          6|3=7H(7)          10H(10)    9D(9)           18
26 MATCH   5|3=AD(1)          7|1=QH(12)         10H(10)    9D(9)           16
27 MOVE    DL=10H(10)         -                  10C(10)    10H(10)         16
28 MOVE    DL=10C(10)         -                  5H(5)      10C(10)         16
29 MOVE    DL=5H(5)           -                  -          5H(5)           16
30 RECYCLE -                  -                  4S(4)      -               16
31 MATCH   5|4=9S(9)          DL=4S(4)           4H(4)      -               15
32 MATCH   4|4=6S(6)          7|2=7D(7)          4H(4)      -               13
33 MOVE    DL=4H(4)           -                  5D(5)      4H(4)           13
34 MATCH   6|2=8D(8)          DL=5D(5)           6H(6)      4H(4)           12
35 KING    5|2=KH(13)         -                  6H(6)      4H(4)           11
36 MOVE    DL=6H(6)           -                  3D(3)      6H(6)           11
37 MOVE    DL=3D(3)           -                  JC(11)     3D(3)           11
38 MATCH   4|2=2S(2)          DL=JC(11)          JH(11)     3D(3)           10
39 MATCH   6|1=2H(2)          DL=JH(11)          3H(3)      3D(3)            9
40 MATCH   4|3=QD(12)         5|1=AH(1)          3H(3)      3D(3)            7
41 MOVE    DL=3H(3)           -                  9D(9)      3H(3)            7
42 MATCH   3|3=4D(4)          DL=9D(9)           10H(10)    3H(3)            6
43 MATCH   DL=10H(10)         DR=3H(3)           10C(10)    3D(3)            6
44 MATCH   DL=10C(10)         DR=3D(3)           5H(5)      6H(6)            6
45 MATCH   3|2=8C(8)          DL=5H(5)           -          6H(6)            5
46 MATCH   2|2=JS(11)         4|1=2C(2)          -          6H(6)            3
47 MATCH   3|1=7C(7)          DR=6H(6)           -          4H(4)            2
48 KING    2|1=KC(13)         -                  -          4H(4)            1
49 MATCH   1|1=9C(9)          DR=4H(4)           -          -                0
```

The retained [host verification](Verification-0.8.4.md) and this human acceptance
are complementary evidence. The master milestone/version ledger remains in the
local sibling checkout at
`../qmp-qemu-socket/docs/Pyramid-Implementation-Schedule.md`.
