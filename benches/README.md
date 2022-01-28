# Overview of benchmark results

## Results from my computer

Executed on my Ryzen 7 5800H CPU in "maximum performance" mode (45 W, low-power states disabled,
fans on max, but GHz can still be varied a little), on my 3200 MHz DDR4 RAM (2x32 G, dual
channel), with very little other load, on NixOS 21.11 with its 5.15.13 kernel, compiled by rustc
1.60.0-nightly, with LTO, with each benchmark running serially (one at a time) using only 1 core
(leaving 7 other cores for the other tiny load).

The following results are interpreted farther [below](#interpretation).

```
$ cargo +nightly bench --profile bench-max-optim

     Running unittests (target/bench-max-optim/deps/equiv-9e888ac63a576109)

running 74 tests
test basic::degenerate_dag::equiv                        ... bench:   1,433,309 ns/iter (+/- 60,091)
test basic::degenerate_dag::limited_equiv                ... bench:   1,520,915 ns/iter (+/- 69,427)
test basic::inverted_list::equiv                         ... bench:      47,667 ns/iter (+/- 6,044)
test basic::inverted_list::limited_equiv                 ... bench:      49,089 ns/iter (+/- 6,787)
test basic::list::equiv                                  ... bench:      52,502 ns/iter (+/- 8,509)
test basic::list::limited_equiv                          ... bench:      57,698 ns/iter (+/- 6,841)
test basic::short_degenerate_dag::equiv                  ... bench:         659 ns/iter (+/- 29)
test basic::short_degenerate_dag::limited_equiv          ... bench:         729 ns/iter (+/- 26)
test basic::short_inverted_list::equiv                   ... bench:         512 ns/iter (+/- 21)
test basic::short_inverted_list::limited_equiv           ... bench:         578 ns/iter (+/- 32)
test basic::short_list::equiv                            ... bench:         512 ns/iter (+/- 21)
test basic::short_list::limited_equiv                    ... bench:         559 ns/iter (+/- 30)
test cycle_safe::degenerate_cyclic::equiv                ... bench:       2,933 ns/iter (+/- 159)
test cycle_safe::degenerate_cyclic::precheck_equiv       ... bench:       4,174 ns/iter (+/- 434)
test cycle_safe::degenerate_dag::equiv                   ... bench:       2,917 ns/iter (+/- 145)
test cycle_safe::degenerate_dag::precheck_equiv          ... bench:       4,002 ns/iter (+/- 320)
test cycle_safe::inverted_list::equiv                    ... bench:     117,080 ns/iter (+/- 21,993)
test cycle_safe::inverted_list::precheck_equiv           ... bench:     118,758 ns/iter (+/- 24,359)
test cycle_safe::list::equiv                             ... bench:     120,221 ns/iter (+/- 24,705)
test cycle_safe::list::precheck_equiv                    ... bench:     121,167 ns/iter (+/- 26,692)
test cycle_safe::short_degenerate_cyclic::equiv          ... bench:       1,279 ns/iter (+/- 50)
test cycle_safe::short_degenerate_cyclic::precheck_equiv ... bench:       2,496 ns/iter (+/- 125)
test cycle_safe::short_degenerate_dag::equiv             ... bench:       1,232 ns/iter (+/- 55)
test cycle_safe::short_degenerate_dag::precheck_equiv    ... bench:         728 ns/iter (+/- 52)
test cycle_safe::short_inverted_list::equiv              ... bench:       5,387 ns/iter (+/- 448)
test cycle_safe::short_inverted_list::precheck_equiv     ... bench:         560 ns/iter (+/- 9)
test cycle_safe::short_list::equiv                       ... bench:       5,186 ns/iter (+/- 678)
test cycle_safe::short_list::precheck_equiv              ... bench:         559 ns/iter (+/- 23)
test deep_safe::degenerate_dag::equiv                    ... bench:   1,517,375 ns/iter (+/- 271,506)
test deep_safe::inverted_list::equiv                     ... bench:      41,912 ns/iter (+/- 1,982)
test deep_safe::list::equiv                              ... bench:      41,612 ns/iter (+/- 2,126)
test deep_safe::long_inverted_list::equiv                ... bench:   1,408,319 ns/iter (+/- 75,921)
test deep_safe::long_list::equiv                         ... bench:   1,380,287 ns/iter (+/- 98,359)
test deep_safe::short_degenerate_dag::equiv              ... bench:         685 ns/iter (+/- 39)
test deep_safe::short_inverted_list::equiv               ... bench:         529 ns/iter (+/- 32)
test deep_safe::short_list::equiv                        ... bench:         526 ns/iter (+/- 20)
test derived_eq::degenerate_dag::eq                      ... bench:     524,942 ns/iter (+/- 27,098)
test derived_eq::inverted_list::eq                       ... bench:      19,226 ns/iter (+/- 1,382)
test derived_eq::list::eq                                ... bench:      14,950 ns/iter (+/- 1,000)
test derived_eq::short_degenerate_dag::eq                ... bench:         239 ns/iter (+/- 12)
test derived_eq::short_inverted_list::eq                 ... bench:         193 ns/iter (+/- 2)
test derived_eq::short_list::eq                          ... bench:         189 ns/iter (+/- 3)
test robust::degenerate_cyclic::equiv                    ... bench:       2,948 ns/iter (+/- 137)
test robust::degenerate_cyclic::precheck_equiv           ... bench:       4,381 ns/iter (+/- 186)
test robust::degenerate_dag::equiv                       ... bench:       2,879 ns/iter (+/- 91)
test robust::degenerate_dag::precheck_equiv              ... bench:       4,330 ns/iter (+/- 166)
test robust::inverted_list::equiv                        ... bench:     103,460 ns/iter (+/- 6,442)
test robust::inverted_list::precheck_equiv               ... bench:     105,235 ns/iter (+/- 9,100)
test robust::list::equiv                                 ... bench:     103,715 ns/iter (+/- 8,440)
test robust::list::precheck_equiv                        ... bench:     110,508 ns/iter (+/- 1,681)
test robust::long_degenerate_cyclic::equiv               ... bench:  85,777,426 ns/iter (+/- 9,432,243)
test robust::long_degenerate_cyclic::precheck_equiv      ... bench:  86,969,685 ns/iter (+/- 10,483,774)
test robust::long_degenerate_dag::equiv                  ... bench:  85,901,175 ns/iter (+/- 11,394,270)
test robust::long_degenerate_dag::precheck_equiv         ... bench:  86,267,721 ns/iter (+/- 8,334,810)
test robust::long_inverted_list::equiv                   ... bench:   5,027,901 ns/iter (+/- 707,067)
test robust::long_inverted_list::precheck_equiv          ... bench:   5,072,338 ns/iter (+/- 894,887)
test robust::long_list::equiv                            ... bench:   5,085,048 ns/iter (+/- 1,121,376)
test robust::long_list::precheck_equiv                   ... bench:   5,243,630 ns/iter (+/- 1,437,008)
test robust::short_degenerate_cyclic::equiv              ... bench:       1,256 ns/iter (+/- 55)
test robust::short_degenerate_cyclic::precheck_equiv     ... bench:       2,660 ns/iter (+/- 119)
test robust::short_degenerate_dag::equiv                 ... bench:       1,193 ns/iter (+/- 49)
test robust::short_degenerate_dag::precheck_equiv        ... bench:         779 ns/iter (+/- 29)
test robust::short_inverted_list::equiv                  ... bench:       5,058 ns/iter (+/- 103)
test robust::short_inverted_list::precheck_equiv         ... bench:         548 ns/iter (+/- 29)
test robust::short_list::equiv                           ... bench:       5,277 ns/iter (+/- 227)
test robust::short_list::precheck_equiv                  ... bench:         541 ns/iter (+/- 8)
test wide_safe::degenerate_dag::equiv                    ... bench:   1,202,406 ns/iter (+/- 46,288)
test wide_safe::inverted_list::equiv                     ... bench:      36,674 ns/iter (+/- 2,574)
test wide_safe::list::equiv                              ... bench:      40,830 ns/iter (+/- 1,895)
test wide_safe::long_inverted_list::equiv                ... bench:   1,579,051 ns/iter (+/- 261,512)
test wide_safe::long_list::equiv                         ... bench:   1,392,088 ns/iter (+/- 166,487)
test wide_safe::short_degenerate_dag::equiv              ... bench:         644 ns/iter (+/- 24)
test wide_safe::short_inverted_list::equiv               ... bench:         542 ns/iter (+/- 18)
test wide_safe::short_list::equiv                        ... bench:         445 ns/iter (+/- 20)

test result: ok. 0 passed; 0 failed; 0 ignored; 74 measured; 0 filtered out; finished in 231.77s
```

## Interpretation

---

```
basic::degenerate_dag::equiv                        ... bench:   1,433,309 ns/iter (+/- 60,091)
basic::degenerate_dag::limited_equiv                ... bench:   1,520,915 ns/iter (+/- 69,427)
```

The basic variant, which is similar to the common `derive`d `PartialEq`, does not detect shared
structure and so must do `2^(depth+1)-2` recursions for the `degenerate_dag` graph shape (a chain
of pairs (2-tuples) with both left and right edges linking to the same next tails of the chain,
which, without shared-structure detection, is traversed like a perfect binary tree).

The `limited_equiv` decrements and checks an integer for each recursion (to be able to abort early
if a limit is reached, which does not occur for any of the benchmarks), but `equiv` does not.

These cases do the same amount of recursions as the `long_list` and `long_degenerate` cases (but
with much shallower depth).  With a depth of `18`, `2^19 - 2` recursions are done.

---

```
basic::inverted_list::equiv                         ... bench:      47,667 ns/iter (+/- 6,044)
basic::inverted_list::limited_equiv                 ... bench:      49,089 ns/iter (+/- 6,787)
basic::list::equiv                                  ... bench:      52,502 ns/iter (+/- 8,509)
basic::list::limited_equiv                          ... bench:      57,698 ns/iter (+/- 6,841)
```

All variants do `2*length` recursions for lists.

The basic variant uses the normal call-stack, which seems to be approx. as fast for `inverted_list`
(left edges: list tail, right edges: leaf elements) as it is for `list` (left edges: leaf
elements, right edges: list tail).

These cases, with a length of `8,000`, do `16,000` recursions.

---

```
basic::short_degenerate_dag::equiv                  ... bench:         659 ns/iter (+/- 29)
basic::short_degenerate_dag::limited_equiv          ... bench:         729 ns/iter (+/- 26)
```

These cases, with a depth of `7`, do `254` (`2^8 - 2`) recursions.

---

```
basic::short_inverted_list::equiv                   ... bench:         512 ns/iter (+/- 21)
basic::short_inverted_list::limited_equiv           ... bench:         578 ns/iter (+/- 32)
basic::short_list::equiv                            ... bench:         512 ns/iter (+/- 21)
basic::short_list::limited_equiv                    ... bench:         559 ns/iter (+/- 30)
```

These cases, with a length of `100`, do `200` recursions.

---

```
cycle_safe::degenerate_cyclic::equiv                ... bench:       2,933 ns/iter (+/- 159)
cycle_safe::degenerate_cyclic::precheck_equiv       ... bench:       4,174 ns/iter (+/- 434)
cycle_safe::degenerate_dag::equiv                   ... bench:       2,917 ns/iter (+/- 145)
cycle_safe::degenerate_dag::precheck_equiv          ... bench:       4,002 ns/iter (+/- 320)
```

The cycle-safe variants do detect shared structure and so do only `2*depth` recursions for the
`degenerate_dag` and `degenerate_cyclic` graph shapes.

These cases, with a depth of `18`, do `36` recursions for the `equiv` cases.  Unlike the basic
variant, each recursion involves hash-table operations, because the "interleave" mode stays in its
"slow" phase for all recursions due to continously detecting shared structure.

For the `equiv` cases, which only do the "interleave" mode, while the `recursion/ns` speed is
around `3%` as fast, there are only around `0.007%` as many recursions, and so it handles the same
`degenerate_dag` shape around `49,100%` as fast and handles the `degenerate_cyclic` shape at that
speed which the basic and only-deep-safe variants cannot handle at any speed.

For the `precheck_equiv` cases, the "precheck" mode, which is like the limited basic variant,
would need to do `2^19 - 2` recursions but reaches its limit and aborts (due to, either, the
exponential complexity of the basic way of traversing the `degenerate_dag` shape, or, due to
infinitely cycling while traversing the `degenerate_cyclic` shape) and so this effort is wasted,
before doing the "interleave" mode which succeeds quickly because it is unlimited and it does
shared-structure detection.

---

```
cycle_safe::inverted_list::equiv                    ... bench:     117,080 ns/iter (+/- 21,993)
cycle_safe::inverted_list::precheck_equiv           ... bench:     118,758 ns/iter (+/- 24,359)
cycle_safe::list::equiv                             ... bench:     120,221 ns/iter (+/- 24,705)
cycle_safe::list::precheck_equiv                    ... bench:     121,167 ns/iter (+/- 26,692)
```

Like the basic variant, the cycle-safe variants do `2*length` recursions for lists.  Unlike the
basic variant, the "interleave" mode is used which interleaves a shared-structure-detecting "slow"
phase with a basic "fast" phase.

These cases, with a length of `8,000`, do `16,000` recursions.

For these lists without shared structure, the "slow" phase only does about `10%` of recursions and
the "fast" phase does about `90%`.  These cases are around `44%` as fast as the basic variant,
which is not too bad of a trade-off for the ability to also handle cyclic and degenerate graphs
efficiently.

---

```
cycle_safe::short_degenerate_cyclic::equiv          ... bench:       1,279 ns/iter (+/- 50)
cycle_safe::short_degenerate_cyclic::precheck_equiv ... bench:       2,496 ns/iter (+/- 125)
cycle_safe::short_degenerate_dag::equiv             ... bench:       1,232 ns/iter (+/- 55)
cycle_safe::short_degenerate_dag::precheck_equiv    ... bench:         728 ns/iter (+/- 52)
```

These cases, with a depth of `7`, do only `14` recursions for the `equiv` cases, unlike the basic
variant.

The "interleave" mode stays in its "slow" phase for all recursions, and so the
`short_degenerate_dag::equiv` case is slower than `basic`, but at least the
`short_degenerate_cyclic::equiv` can also be handled at that speed which is acceptable for having
cycle-safety which `basic` does not.

The `short_degenerate_cyclic::precheck_equiv` case wastes the effort of the "precheck" mode on
this shape that has more basic-traversed edges (infinite) than the precheck limit.

The `short_degenerate_dag::precheck_equiv` case, which does `2^8 - 2` recursions, is able to
complete the "precheck" mode on this small short shape, without doing "interleave", and so is as
fast as the `limited_equiv` of the basic variant

---

```
cycle_safe::short_inverted_list::equiv              ... bench:       5,387 ns/iter (+/- 448)
cycle_safe::short_inverted_list::precheck_equiv     ... bench:         560 ns/iter (+/- 9)
cycle_safe::short_list::equiv                       ... bench:       5,186 ns/iter (+/- 678)
cycle_safe::short_list::precheck_equiv              ... bench:         559 ns/iter (+/- 23)
```

These cases, with a length of `100`, do `200` recursions, like the basic variant.

The `equiv` cases use the "interleave" mode and so involve the "slow" phase along with the "fast"
phase, and so are slower.

The `precheck_equiv` cases are able to complete the "precheck" mode on these small short shapes,
and so are as fast as the `limited_equiv` of the basic variant.  This shows the purpose of the
"precheck" mode: to be as fast for small acyclic inputs while still being able to handle cyclic
and degenerate inputs while not wasting too much effort.

---

```
deep_safe::degenerate_dag::equiv                    ... bench:   1,517,375 ns/iter (+/- 271,506)
```

The deep-safe variants do not use the normal call-stack and instead use a vector as a queue of
which nodes to continue recurring on.

The `deep_safe` cases, like the basic variant, do not detect shared structure, and so must do
`2^(depth+1)-2` recursions for the `degenerate_dag` graph shape.  With a depth of `18`, `2^19 - 2`
recursions are done.

The vector queue is `94%` as fast as the call-stack, comparing this case to
`basic::degenerate_dag::equiv`.

---

```
deep_safe::inverted_list::equiv                     ... bench:      41,912 ns/iter (+/- 1,982)
deep_safe::list::equiv                              ... bench:      41,612 ns/iter (+/- 2,126)
```

Like the basic variant, the deep-safe variants do `2*length` recursions for lists.

These cases, with a length of `8,000`, do `16,000` recursions, on a vector queue.

The vector queue is `126%` and `114%` as fast as the call-stack, comparing these cases to the
`basic::list::equiv` and `basic::inverted_list::equiv` cases.

---

```
deep_safe::long_inverted_list::equiv                ... bench:   1,408,319 ns/iter (+/- 75,921)
deep_safe::long_list::equiv                         ... bench:   1,380,287 ns/iter (+/- 98,359)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector queue.

The same amount of recursions is done as the `degenerate_dag` cases, and the `recursion/ns` speed
is approx. the same as `deep_safe::degenerate_dag::equiv` and `basic::degenerate_dag::equiv`.

---

```
deep_safe::short_degenerate_dag::equiv              ... bench:         685 ns/iter (+/- 39)
```

This case, with a depth of `7`, does `254` (`2^8 - 2`) recursions, like the basic variant, but on
a vector queue, unlike the basic variant.

The vector queue is approx. as fast as the call-stack, comparing this case to
`basic::short_degenerate_dag::equiv`.

---

```
deep_safe::short_inverted_list::equiv               ... bench:         529 ns/iter (+/- 32)
deep_safe::short_list::equiv                        ... bench:         526 ns/iter (+/- 20)
```

These cases, with a length of `100`, do `200` recursions, like the basic variant, but on a vector
queue, unlike the basic variant.

The vector queue is approx. as fast as the call-stack, comparing these cases to
`basic::short_list::equiv` and `basic::short_inverted_list::equiv`.

---

```
robust::degenerate_cyclic::equiv                    ... bench:       2,948 ns/iter (+/- 137)
robust::degenerate_cyclic::precheck_equiv           ... bench:       4,381 ns/iter (+/- 186)
robust::degenerate_dag::equiv                       ... bench:       2,879 ns/iter (+/- 91)
robust::degenerate_dag::precheck_equiv              ... bench:       4,330 ns/iter (+/- 166)
```

The robust variant is like a combination of `cycle_safe` and `deep_safe`, in that it does detect
shared structure and so is cycle-safe and that it uses a vector queue and so is deep-safe.  Like
`cycle_safe`, and unlike `deep_safe` and `basic`, it does only `2*depth` recursions for the
`degenerate_dag` and `degenerate_cyclic` graph shapes.

These cases, with a depth of `18`, do `36` recursions, involving hash-table operations, like
`cycle_safe`.

The `equiv` cases have the same speed as the `cycle_safe` cases, as expected, since they do the
"interleave" mode staying in "slow" phase the same, but a vector queue is used instead of the call
stack.

The `precheck_equiv` cases waste the effort of the "precheck" mode for these large (as traversed
basically) or cyclic shapes, like the `cycle_safe` cases, as expected.

---

```
robust::inverted_list::equiv                        ... bench:     103,460 ns/iter (+/- 6,442)
robust::inverted_list::precheck_equiv               ... bench:     105,235 ns/iter (+/- 9,100)
robust::list::equiv                                 ... bench:     103,715 ns/iter (+/- 8,440)
robust::list::precheck_equiv                        ... bench:     110,508 ns/iter (+/- 1,681)
```

Like `basic`, the robust variant does `2*length` recursions for lists.  Like `deep_safe`, a vector
queue is used.  Like `cycle_safe`, the "interleave" mode is used with about `10%` "slow" phase and
about `90%` "fast" phase.

These cases, with a length of `8,000`, do `16,000` recursions.

The speed is significantly slower than `basic` and `deep_safe` due to the involvement of the
"slow" phase of "interleave" mode, and is a little faster than `cycle_safe` due to the use of a
vector queue, which improves the attractiveness of the trade-off for the cycle-safety, and it also
has the deep-safety.

---

```
robust::long_degenerate_cyclic::equiv               ... bench:  85,777,426 ns/iter (+/- 9,432,243)
robust::long_degenerate_cyclic::precheck_equiv      ... bench:  86,969,685 ns/iter (+/- 10,483,774)
robust::long_degenerate_dag::equiv                  ... bench:  85,901,175 ns/iter (+/- 11,394,270)
robust::long_degenerate_dag::precheck_equiv         ... bench:  86,267,721 ns/iter (+/- 8,334,810)
```

These shapes are degenerate pair-chains but their depth is `2^18` which is the same as the length
of the long-list shapes.

For the "interleave" mode used by this robust variant, `2^19` (`2*depth`) recursions are done.
For the basic variant and the "precheck" mode, an infeasible `2^(2^18+1)-2` recursions would be
required.  For `cycle_safe`, the depth would cause stack-overflow crash.

While the amount of recursions is the same as `basic::degenerate_dag` and `deep_safe::long_list`,
the "interleave" mode stays in its "slow" phase for all recursions, like `cycle_safe`.  This is
why the `recursion/ns` speed is `2%` as fast as `basic` and `deep_safe`.  That is the trade-off
for the ability to handle these very-deep degenerate shapes which all other variants cannot.

---

```
robust::long_inverted_list::equiv                   ... bench:   5,027,901 ns/iter (+/- 707,067)
robust::long_inverted_list::precheck_equiv          ... bench:   5,072,338 ns/iter (+/- 894,887)
robust::long_list::equiv                            ... bench:   5,085,048 ns/iter (+/- 1,121,376)
robust::long_list::precheck_equiv                   ... bench:   5,243,630 ns/iter (+/- 1,437,008)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector queue, like
`deep_safe::long`.

The "interleave" mode is used with about `10%` "slow" phase and about `90%` "fast" phase, like
`cycle_safe`, but unlike `deep_safe`.  This is why these cases are much slower than the
`deep_safe::long` cases, but are still much faster than the `robust::long_degenerate` cases, which
all have the same amount of recursions.

---

```
robust::short_degenerate_cyclic::equiv              ... bench:       1,256 ns/iter (+/- 55)
robust::short_degenerate_cyclic::precheck_equiv     ... bench:       2,660 ns/iter (+/- 119)
robust::short_degenerate_dag::equiv                 ... bench:       1,193 ns/iter (+/- 49)
robust::short_degenerate_dag::precheck_equiv        ... bench:         779 ns/iter (+/- 29)
```

These cases, with a depth of `7`, do only `14` recursions for the `equiv` cases, like
`cycle_safe`, and unlike `basic` and `deep_safe` (which do `254`).

These cases are as fast as `cycle_safe::short_degenerate`, as expected, since they involve the
same aspects other than `robust` using a vector queue.

---

```
robust::short_inverted_list::equiv                  ... bench:       5,058 ns/iter (+/- 103)
robust::short_inverted_list::precheck_equiv         ... bench:         548 ns/iter (+/- 29)
robust::short_list::equiv                           ... bench:       5,277 ns/iter (+/- 227)
robust::short_list::precheck_equiv                  ... bench:         541 ns/iter (+/- 8)
```

These cases, with a length of `100`, do `200` recursions, like the other variants.

The `equiv` cases use the "interleave" mode and so involve the "slow" phase along with the "fast"
phase, and so are slower, as expected, like the `cycle_safe::short` cases.

The `precheck_equiv` cases are faster because they are able to complete the "precheck" mode on
these small short shapes, and are as fast as `basic::short` and `cycle_safe::short`, as expected.
This shows the purpose of the "precheck" mode: to be fast for small acyclic inputs while still
being able to handle cyclic, degenerate, large, and deep inputs while not wasting too much effort.

---

```
wide_safe::degenerate_dag::equiv                    ... bench:   1,202,406 ns/iter (+/- 46,288)
```

The wide-safe variants do not use the normal call-stack and instead use a vector as a stack of
which nodes to continue recurring on.

The `wide_safe` cases, like the basic variant, do not detect shared structure, and so must do
`2^(depth+1)-2` recursions for the `degenerate_dag` graph shape.  With a depth of `18`, `2^19 - 2`
recursions are done.

The vector stack is `119%` as fast as the call-stack, comparing this case to
`basic::degenerate_dag::equiv`.

---

```
wide_safe::inverted_list::equiv                     ... bench:      36,674 ns/iter (+/- 2,574)
wide_safe::list::equiv                              ... bench:      40,830 ns/iter (+/- 1,895)
```

Like the basic variant, the wide-safe variants do `2*length` recursions for lists.

These cases, with a length of `8,000`, do `16,000` recursions, on a vector stack.

The vector stack is `129%` as fast as the call-stack, comparing these cases to the
`basic::list::equiv` and `basic::inverted_list::equiv` cases.

---

```
wide_safe::long_inverted_list::equiv                ... bench:   1,579,051 ns/iter (+/- 261,512)
wide_safe::long_list::equiv                         ... bench:   1,392,088 ns/iter (+/- 166,487)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector stack.

The same amount of recursions is done as the `degenerate_dag` cases, and the `recursion/ns` speed
is approx. the same as `wide_safe::degenerate_dag::equiv` and `basic::degenerate_dag::equiv`.

The `long_list` benefits from a kind of "tail-call elimination" because it descends its list
elements, which are leaf nodes, before descending its list tails, and so the maximum amount of
items on its vector stack should be only `1`.  Whereas, `long_inverted_list` descends its list
tails before its list elements, and so the maximum amount of items on its vector stack should be
the same as its length of `2^18`, which is why it is slower.

---

```
wide_safe::short_degenerate_dag::equiv              ... bench:         644 ns/iter (+/- 24)
```

This case, with a depth of `7`, does `254` (`2^8 - 2`) recursions, like the basic variant, but on
a vector stack, unlike the basic variant.

The vector stack is approx. as fast as the call-stack, comparing this case to
`basic::short_degenerate_dag::equiv`.

---

```
wide_safe::short_inverted_list::equiv               ... bench:         542 ns/iter (+/- 18)
wide_safe::short_list::equiv                        ... bench:         445 ns/iter (+/- 20)
```

These cases, with a length of `100`, do `200` recursions, like the basic variant, but on a vector
stack, unlike the basic variant.

The vector stack is approx. as fast as the call-stack, comparing these cases to
`basic::short_list::equiv` and `basic::short_inverted_list::equiv`.

---

```
derived_eq::degenerate_dag::eq                      ... bench:     524,942 ns/iter (+/- 27,098)
derived_eq::inverted_list::eq                       ... bench:      19,226 ns/iter (+/- 1,382)
derived_eq::list::eq                                ... bench:      14,950 ns/iter (+/- 1,000)
derived_eq::short_degenerate_dag::eq                ... bench:         239 ns/iter (+/- 12)
derived_eq::short_inverted_list::eq                 ... bench:         193 ns/iter (+/- 2)
derived_eq::short_list::eq                          ... bench:         189 ns/iter (+/- 3)
```

The common `derive`d `PartialEq` is faster than all of the other variants, for the limited shapes
that it can handle, except for the `degenerate_dag` shape where it is much slower than the
`cycle_safe` and `robust` variants that detect shared structure.

---

## Conclusion

The robust variant has the best all-around performance, when all possible shapes could be given as
inputs, and is the only variant that can handle all possible shapes.

When the possible shapes of inputs are more limited, the best variant is the one that supports the
possible shapes but excludes support for the impossible shapes.

The basic variant is slower than `derive`d `PartialEq` and so should not be used when `derive`d
`PartialEq` will suffice.  The basic variant should only be used when implementing the `Node`
trait in peculiar ways for a type is useful for having different behavior than `derive`d
`PartialEq`.
