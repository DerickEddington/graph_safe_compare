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

     Running unittests (target/bench-max-optim/deps/equiv-ed6e12b16ac8ffe5)

running 66 tests
test basic::degenerate_dag::equiv                        ... bench:   5,427,188 ns/iter (+/- 242,759)
test basic::degenerate_dag::limited_equiv                ... bench:   5,535,640 ns/iter (+/- 198,584)
test basic::inverted_list::equiv                         ... bench:     214,180 ns/iter (+/- 23,338)
test basic::inverted_list::limited_equiv                 ... bench:     210,207 ns/iter (+/- 20,432)
test basic::list::equiv                                  ... bench:     214,885 ns/iter (+/- 23,034)
test basic::list::limited_equiv                          ... bench:     208,403 ns/iter (+/- 22,628)
test basic::short_degenerate_dag::equiv                  ... bench:       2,696 ns/iter (+/- 79)
test basic::short_degenerate_dag::limited_equiv          ... bench:       2,646 ns/iter (+/- 72)
test basic::short_inverted_list::equiv                   ... bench:       2,147 ns/iter (+/- 87)
test basic::short_inverted_list::limited_equiv           ... bench:       2,233 ns/iter (+/- 68)
test basic::short_list::equiv                            ... bench:       2,196 ns/iter (+/- 86)
test basic::short_list::limited_equiv                    ... bench:       2,121 ns/iter (+/- 58)
test cycle_safe::degenerate_cyclic::equiv                ... bench:       3,209 ns/iter (+/- 146)
test cycle_safe::degenerate_cyclic::precheck_equiv       ... bench:       7,862 ns/iter (+/- 424)
test cycle_safe::degenerate_dag::equiv                   ... bench:       3,143 ns/iter (+/- 123)
test cycle_safe::degenerate_dag::precheck_equiv          ... bench:       7,440 ns/iter (+/- 173)
test cycle_safe::inverted_list::equiv                    ... bench:     284,767 ns/iter (+/- 37,622)
test cycle_safe::inverted_list::precheck_equiv           ... bench:     300,184 ns/iter (+/- 37,902)
test cycle_safe::list::equiv                             ... bench:     275,883 ns/iter (+/- 31,452)
test cycle_safe::list::precheck_equiv                    ... bench:     285,216 ns/iter (+/- 33,419)
test cycle_safe::short_degenerate_cyclic::equiv          ... bench:       1,439 ns/iter (+/- 45)
test cycle_safe::short_degenerate_cyclic::precheck_equiv ... bench:       6,138 ns/iter (+/- 70)
test cycle_safe::short_degenerate_dag::equiv             ... bench:       1,371 ns/iter (+/- 134)
test cycle_safe::short_degenerate_dag::precheck_equiv    ... bench:       2,680 ns/iter (+/- 118)
test cycle_safe::short_inverted_list::equiv              ... bench:       7,011 ns/iter (+/- 317)
test cycle_safe::short_inverted_list::precheck_equiv     ... bench:       2,287 ns/iter (+/- 112)
test cycle_safe::short_list::equiv                       ... bench:       6,984 ns/iter (+/- 240)
test cycle_safe::short_list::precheck_equiv              ... bench:       2,211 ns/iter (+/- 55)
test deep_safe::degenerate_dag::equiv                    ... bench:   5,076,106 ns/iter (+/- 226,654)
test deep_safe::inverted_list::equiv                     ... bench:     158,422 ns/iter (+/- 8,352)
test deep_safe::list::equiv                              ... bench:     163,809 ns/iter (+/- 10,813)
test deep_safe::long_inverted_list::equiv                ... bench:   8,153,440 ns/iter (+/- 408,277)
test deep_safe::long_list::equiv                         ... bench:   5,500,291 ns/iter (+/- 478,701)
test deep_safe::short_degenerate_dag::equiv              ... bench:       2,669 ns/iter (+/- 117)
test deep_safe::short_inverted_list::equiv               ... bench:       2,020 ns/iter (+/- 53)
test deep_safe::short_list::equiv                        ... bench:       2,002 ns/iter (+/- 44)
test derived_eq::degenerate_dag::eq                      ... bench:   1,242,302 ns/iter (+/- 57,628)
test derived_eq::inverted_list::eq                       ... bench:      83,803 ns/iter (+/- 9,585)
test derived_eq::list::eq                                ... bench:      67,077 ns/iter (+/- 4,211)
test derived_eq::short_degenerate_dag::eq                ... bench:         589 ns/iter (+/- 19)
test derived_eq::short_inverted_list::eq                 ... bench:         677 ns/iter (+/- 28)
test derived_eq::short_list::eq                          ... bench:         516 ns/iter (+/- 25)
test robust::degenerate_cyclic::equiv                    ... bench:       3,270 ns/iter (+/- 125)
test robust::degenerate_cyclic::precheck_equiv           ... bench:       7,235 ns/iter (+/- 235)
test robust::degenerate_dag::equiv                       ... bench:       3,231 ns/iter (+/- 158)
test robust::degenerate_dag::precheck_equiv              ... bench:       7,326 ns/iter (+/- 238)
test robust::inverted_list::equiv                        ... bench:     225,543 ns/iter (+/- 24,250)
test robust::inverted_list::precheck_equiv               ... bench:     237,060 ns/iter (+/- 24,718)
test robust::list::equiv                                 ... bench:     220,299 ns/iter (+/- 22,309)
test robust::list::precheck_equiv                        ... bench:     227,002 ns/iter (+/- 25,566)
test robust::long_degenerate_cyclic::equiv               ... bench: 123,864,966 ns/iter (+/- 8,535,391)
test robust::long_degenerate_cyclic::precheck_equiv      ... bench: 125,340,287 ns/iter (+/- 8,739,280)
test robust::long_degenerate_dag::equiv                  ... bench: 122,161,738 ns/iter (+/- 7,930,290)
test robust::long_degenerate_dag::precheck_equiv         ... bench: 123,992,887 ns/iter (+/- 8,042,880)
test robust::long_inverted_list::equiv                   ... bench:  14,045,557 ns/iter (+/- 1,250,550)
test robust::long_inverted_list::precheck_equiv          ... bench:  15,422,656 ns/iter (+/- 1,451,033)
test robust::long_list::equiv                            ... bench:  11,077,399 ns/iter (+/- 1,462,595)
test robust::long_list::precheck_equiv                   ... bench:  12,071,313 ns/iter (+/- 1,366,031)
test robust::short_degenerate_cyclic::equiv              ... bench:       1,463 ns/iter (+/- 41)
test robust::short_degenerate_cyclic::precheck_equiv     ... bench:       5,462 ns/iter (+/- 295)
test robust::short_degenerate_dag::equiv                 ... bench:       1,390 ns/iter (+/- 24)
test robust::short_degenerate_dag::precheck_equiv        ... bench:       2,527 ns/iter (+/- 84)
test robust::short_inverted_list::equiv                  ... bench:       6,787 ns/iter (+/- 213)
test robust::short_inverted_list::precheck_equiv         ... bench:       1,909 ns/iter (+/- 53)
test robust::short_list::equiv                           ... bench:       6,690 ns/iter (+/- 277)
test robust::short_list::precheck_equiv                  ... bench:       1,957 ns/iter (+/- 94)

test result: ok. 0 passed; 0 failed; 0 ignored; 66 measured; 0 filtered out; finished in 270.47s
```

## Interpretation

TODO: Update the following, for latest different results, before publishing.

---

```
basic::degenerate_dag::equiv                        ... bench:   3,847,750 ns/iter (+/- 70,649)
basic::degenerate_dag::limited_equiv                ... bench:   3,974,424 ns/iter (+/- 68,153)
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
basic::inverted_list::equiv                         ... bench:     175,439 ns/iter (+/- 12,300)
basic::inverted_list::limited_equiv                 ... bench:     188,871 ns/iter (+/- 8,864)
basic::list::equiv                                  ... bench:     161,609 ns/iter (+/- 36,843)
basic::list::limited_equiv                          ... bench:     166,787 ns/iter (+/- 24,798)
```

All variants do `2*length` recursions for lists.

The basic variant uses the normal call-stack, which seems to be nearly as fast for `inverted_list`
(left edges: list tail, right edges: leaf elements) as it is for `list` (left edges: leaf
elements, right edges: list tail).

These cases, with a length of `8,000`, do `16,000` recursions.

---

```
basic::short_degenerate_dag::equiv                  ... bench:       1,942 ns/iter (+/- 82)
basic::short_degenerate_dag::limited_equiv          ... bench:       2,033 ns/iter (+/- 82)
```

These cases, with a depth of `7`, do `254` (`2^8 - 2`) recursions.

---

```
basic::short_inverted_list::equiv                   ... bench:       1,745 ns/iter (+/- 69)
basic::short_inverted_list::limited_equiv           ... bench:       1,927 ns/iter (+/- 69)
basic::short_list::equiv                            ... bench:       1,723 ns/iter (+/- 80)
basic::short_list::limited_equiv                    ... bench:       1,807 ns/iter (+/- 72)
```

These cases, with a length of `100`, do `200` recursions.

---

```
cycle_safe::degenerate_cyclic::equiv                ... bench:       2,988 ns/iter (+/- 53)
cycle_safe::degenerate_cyclic::precheck_equiv       ... bench:       6,820 ns/iter (+/- 297)
cycle_safe::degenerate_dag::equiv                   ... bench:       2,931 ns/iter (+/- 86)
cycle_safe::degenerate_dag::precheck_equiv          ... bench:       6,152 ns/iter (+/- 419)
```

The cycle-safe variants do detect shared structure and so do only `2*depth` recursions for the
`degenerate_dag` and `degenerate_cyclic` graph shapes.

These cases, with a depth of `18`, do `36` recursions for the `equiv` cases.  Unlike the basic
variant, each recursion involves hash-table operations, because the "interleave" mode stays in its
"slow" phase for all recursions due to continously detecting shared structure.

For the `equiv` cases, which only do the "interleave" mode, while the `recursion/ns` speed is
around `10%` as fast, there are only around `0.007%` as many recursions, and so it handles the
same `degenerate_dag` shape around `130,000%` as fast and handles the `degenerate_cyclic` shape at
that speed which the basic and only-deep-safe variants cannot handle at any speed.

For the `precheck_equiv` cases, the "precheck" mode, which is like the limited basic variant,
would need to do `2^19 - 2` recursions but reaches its limit and aborts (due to, either, the
exponential complexity of the basic way of traversing the `degenerate_dag` shape, or, due to
infinitely cycling while traversing the `degenerate_cyclic` shape) and so this effort is wasted,
before doing the "interleave" mode which succeeds quickly because it is unlimited and it does
shared-structure detection.

---

```
cycle_safe::inverted_list::equiv                    ... bench:     238,830 ns/iter (+/- 40,060)
cycle_safe::inverted_list::precheck_equiv           ... bench:     242,658 ns/iter (+/- 40,944)
cycle_safe::list::equiv                             ... bench:     246,091 ns/iter (+/- 41,619)
cycle_safe::list::precheck_equiv                    ... bench:     237,945 ns/iter (+/- 63,633)
```

Like the basic variant, the cycle-safe variants do `2*length` recursions for lists.  Unlike the
basic variant, the "interleave" mode is used which interleaves a shared-structure-detecting "slow"
phase with a basic "fast" phase.

These cases, with a length of `8,000`, do `16,000` recursions.

For these lists without shared structure, the "slow" phase only does about `10%` of recursions and
the "fast" phase does about `90%`.  These cases are around `69%` as fast as the basic variant,
which is not too bad of a trade-off for the ability to also handle cyclic and degenerate graphs
efficiently.

---

```
cycle_safe::short_degenerate_cyclic::equiv          ... bench:       1,323 ns/iter (+/- 40)
cycle_safe::short_degenerate_cyclic::precheck_equiv ... bench:       5,109 ns/iter (+/- 195)
cycle_safe::short_degenerate_dag::equiv             ... bench:       1,263 ns/iter (+/- 33)
cycle_safe::short_degenerate_dag::precheck_equiv    ... bench:       1,955 ns/iter (+/- 35)
```

These cases, with a depth of `7`, do only `14` recursions for the `equiv` cases, unlike the basic
variant.

The "interleave" mode stays in its "slow" phase for all recursions, but the
`short_degenerate_dag::equiv` case is still faster than `basic` and `deep_safe` due to less
recursions, and the `short_degenerate_cyclic::equiv` is also fast and can be handled.

The `short_degenerate_cyclic::precheck_equiv` case wastes the effort of the "precheck" mode on
this shape that has more basic-traversed edges (infinite) than the precheck limit.

The `short_degenerate_dag::precheck_equiv` case, which does `2^8 - 2` recursions, is able to
complete the "precheck" mode on this small short shape, without doing "interleave", and so is as
fast as the `limited_equiv` of the basic variant

---

```
cycle_safe::short_inverted_list::equiv              ... bench:       6,635 ns/iter (+/- 241)
cycle_safe::short_inverted_list::precheck_equiv     ... bench:       1,820 ns/iter (+/- 57)
cycle_safe::short_list::equiv                       ... bench:       4,459 ns/iter (+/- 143)
cycle_safe::short_list::precheck_equiv              ... bench:       1,758 ns/iter (+/- 27)
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
deep_safe::degenerate_dag::equiv                    ... bench:   2,822,701 ns/iter (+/- 140,410)
```

The deep-safe variants do not use the normal call-stack and instead use a vector as the stack of
which nodes to continue recurring on.

The `deep_safe` cases, like the basic variant, do not detect shared structure, and so must do
`2^(depth+1)-2` recursions for the `degenerate_dag` graph shape.  With a depth of `18`, `2^19 - 2`
recursions are done.

The vector stack is `136%` as fast as the call-stack, comparing this case to
`basic::degenerate_dag::equiv`, presumably due to a vector being faster at pushing and popping due
to those operations being simpler and smaller than they are for the call-stack.

---

```
deep_safe::inverted_list::equiv                     ... bench:      91,485 ns/iter (+/- 24,764)
deep_safe::list::equiv                              ... bench:      88,588 ns/iter (+/- 7,122)
```

Like the basic variant, the deep-safe variants do `2*length` recursions for lists.

These cases, with a length of `8,000`, do `16,000` recursions, on a vector stack.

The vector stack is `182%` and `192%` as fast as the call-stack, comparing these cases to the
`basic::list::equiv` and `basic::inverted_list::equiv` cases.

---

```
deep_safe::long_inverted_list::equiv                ... bench:   6,459,989 ns/iter (+/- 290,648)
deep_safe::long_list::equiv                         ... bench:   3,812,624 ns/iter (+/- 395,841)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector stack.

The same amount of recursions is done as the `degenerate_dag` cases, but with much deeper depth.
While the `recursion/ns` speed is, at worst, around `43%` as fast, or, at best, around `74%` as
fast, (comparing `deep_safe::long_inverted_list::equiv` or `deep_safe::long_list::equiv` to
`deep_safe::degenerate_dag::equiv`), the deep-safe variants can handle very-deep graphs which the
basic and only-cycle-safe variants cannot handle at any speed.

While a vector stack is faster than the call stack for the cases with shallower shapes, it is
slower for these cases.  For the `long_inverted_list` shape, this is expected, but for the
`long_list` shape, it is unexpected.

The `long_list` benefits from a kind of "tail-call elimination" because it descends its list
elements, which are leaf nodes, before descending its list tails, and so the maximum amount of
items on its vector stack should be only `2`.  Whereas, `long_inverted_list` descends its list
tails before its list elements, and so the maximum amount of items on its vector stack should be
the same as its length of `2^18`.

With `long_inverted_list` using so much of a vector there are factors that explain why it is
slower than `long_list`.  Linux's demand paging of larger allocations is suspected to be at play,
which will cause some slow-down since the cost is not amortized since the vector memory is
allocated and used only once for each iteration of this case.  Further, twice the initial capacity
of a vector is used, causing a reallocation for resizing it, for each iteration, which will cause
more slow-down.

It is currently unexplained why the speed of the `long_list` case is not closer to that of
`deep_safe::degenerate_dag::equiv`.  It is assumed to not be due to cache-locality differences
regarding their vector stacks, since they both access only the very beginning of a vector.

(Note about achieving the TCE outside these benchmarks: Users can control the order that edges are
descended for their types, and so can achieve TCE for their shapes regardless of whether they are
"left-handed" or "right-handed".  Unlike with traditional TCE of fixed equivalence predicates.)

---

```
deep_safe::short_degenerate_dag::equiv              ... bench:       1,371 ns/iter (+/- 57)
```

This case, with a depth of `7`, does `254` (`2^8 - 2`) recursions, like the basic variant, but on
a vector stack, unlike the basic variant.

The vector stack is `142%` as fast as the call-stack, comparing this case to
`basic::short_degenerate_dag::equiv`.

---

```
deep_safe::short_inverted_list::equiv               ... bench:       1,076 ns/iter (+/- 39)
deep_safe::short_list::equiv                        ... bench:       1,064 ns/iter (+/- 46)
```

These cases, with a length of `100`, do `200` recursions, like the basic variant, but on a vector
stack, unlike the basic variant.

The vector stack is `162%` as fast as the call-stack, comparing these cases to
`basic::short_list::equiv` and `basic::short_inverted_list::equiv`.

---

```
robust::degenerate_cyclic::equiv                    ... bench:       3,055 ns/iter (+/- 156)
robust::degenerate_cyclic::precheck_equiv           ... bench:       5,099 ns/iter (+/- 38)
robust::degenerate_dag::equiv                       ... bench:       2,995 ns/iter (+/- 49)
robust::degenerate_dag::precheck_equiv              ... bench:       5,287 ns/iter (+/- 145)
```

The robust variant is like a combination of `cycle_safe` and `deep_safe`, in that it does detect
shared structure and so is cycle-safe and that it uses a vector stack and so is deep-safe.  Like
`cycle_safe`, and unlike `deep_safe` and `basic`, it does only `2*depth` recursions for the
`degenerate_dag` and `degenerate_cyclic` graph shapes.

These cases, with a depth of `18`, do `36` recursions, involving hash-table operations, like
`cycle_safe`.

The `equiv` cases have the same speed as the `cycle_safe` cases, as expected, since they do the
"interleave" mode staying in "slow" phase the same, but a vector stack is used instead of the call
stack.

The `precheck_equiv` cases waste the effort of the "precheck" mode for these large (as traversed
basically) or cyclic shapes, like the `cycle_safe` cases, as expected, but the cost is a little
less due to the "precheck" mode using a vector stack, which improves the attractiveness of the
trade-off for the precheck (which benefits different shapes: those that are small and acyclic).

---

```
robust::inverted_list::equiv                        ... bench:     161,694 ns/iter (+/- 9,852)
robust::inverted_list::precheck_equiv               ... bench:     163,608 ns/iter (+/- 20,359)
robust::list::equiv                                 ... bench:     156,986 ns/iter (+/- 13,962)
robust::list::precheck_equiv                        ... bench:     173,661 ns/iter (+/- 22,156)
```

Like `basic`, the robust variant does `2*length` recursions for lists.  Like `deep_safe`, a vector
stack is used.  Like `cycle_safe`, the "interleave" mode is used with about `10%` "slow" phase and
about `90%` "fast" phase.

These cases, with a length of `8,000`, do `16,000` recursions.

The speed is as fast as `basic`, is significantly slower than `deep_safe` due to the involvement
of the "slow" phase of "interleave" mode, and is significantly faster than `cycle_safe` due to the
use of a vector stack, which improves the attractiveness of the trade-off for the cycle-safety,
and it also has the deep-safety.

---

```
robust::long_degenerate_cyclic::equiv               ... bench: 112,770,332 ns/iter (+/- 6,191,108)
robust::long_degenerate_cyclic::precheck_equiv      ... bench: 109,561,112 ns/iter (+/- 6,810,804)
robust::long_degenerate_dag::equiv                  ... bench: 112,442,771 ns/iter (+/- 6,530,739)
robust::long_degenerate_dag::precheck_equiv         ... bench: 108,724,493 ns/iter (+/- 5,582,998)
```

These shapes are degenerate pair-chains but their depth is `2^18` which is the same as the length
of the long-list shapes.

For the "interleave" mode used by this robust variant, `2^19` (`2*depth`) recursions are done.
For the basic variant and the "precheck" mode, an infeasible `2^(2^18+1)-2` recursions would be
required.  For `cycle_safe`, the depth would cause stack-overflow crash.

While the amount of recursions is the same as `basic::degenerate_dag` and `deep_safe::long_list`,
the "interleave" mode stays in its "slow" phase for all recursions, like `cycle_safe`.  This is
why the `recursion/ns` speed is `3%` as fast as `basic` and `deep_safe`.  That is the trade-off
for the ability to handle these very-deep degenerate shapes which all other variants cannot.

It is currently unexplained why the speed of the `precheck_equiv` cases was slightly faster than
the `equiv` cases, when the additional effort of the "precheck" mode is always wasted for these
shapes.

---

```
robust::long_inverted_list::equiv                   ... bench:  12,539,428 ns/iter (+/- 1,318,153)
robust::long_inverted_list::precheck_equiv          ... bench:  11,193,337 ns/iter (+/- 972,868)
robust::long_list::equiv                            ... bench:   8,377,566 ns/iter (+/- 628,553)
robust::long_list::precheck_equiv                   ... bench:  12,078,208 ns/iter (+/- 996,303)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector stack, like
`deep_safe::long`.

The "interleave" mode is used with about `10%` "slow" phase and about `90%` "fast" phase, like
`cycle_safe`, but unlike `deep_safe`.  This is why these cases are about half as fast as those of
`deep_safe::long`, but are still much faster than the `robust::long_degenerate` cases, which all
have the same amount of recursions.

The benefit from the kind of "tail-call elimination" with a vector stack is why the
`long_list::equiv` case is faster than the `long_inverted_list::equiv`, like the `deep_safe::long`
cases.

---

```
robust::short_degenerate_cyclic::equiv              ... bench:       1,371 ns/iter (+/- 32)
robust::short_degenerate_cyclic::precheck_equiv     ... bench:       3,463 ns/iter (+/- 147)
robust::short_degenerate_dag::equiv                 ... bench:       1,303 ns/iter (+/- 35)
robust::short_degenerate_dag::precheck_equiv        ... bench:       1,514 ns/iter (+/- 22)
```

These cases, with a depth of `7`, do only `14` recursions for the `equiv` cases, like
`cycle_safe`, and unlike `basic` and `deep_safe` (which do `254`).

These cases are as fast as `cycle_safe::short_degenerate`, as expected, since they involve the
same aspects other than `robust` using a vector stack (which happens to reduce the cost of the
wasted prechecks by a little).

---

```
robust::short_inverted_list::equiv                  ... bench:       3,704 ns/iter (+/- 208)
robust::short_inverted_list::precheck_equiv         ... bench:       1,205 ns/iter (+/- 50)
robust::short_list::equiv                           ... bench:       3,647 ns/iter (+/- 283)
robust::short_list::precheck_equiv                  ... bench:       1,157 ns/iter (+/- 29)
```

These cases, with a length of `100`, do `200` recursions, like the other variants.

The `equiv` cases use the "interleave" mode and so involve the "slow" phase along with the "fast"
phase, and so are slower, as expected, like the `cycle_safe::short` cases.

The `precheck_equiv` cases are faster because they are able to complete the "precheck" mode on
these small short shapes.  They are also faster than `basic::short` and `cycle_safe::short` due to
using a vector stack.  This shows the purpose of the "precheck" mode: to be fast for small acyclic
inputs while still being able to handle cyclic, degenerate, large, and deep inputs while not
wasting too much effort.

---

```
derived_eq::degenerate_dag::eq                      ... bench:   1,178,313 ns/iter (+/- 75,761)
derived_eq::inverted_list::eq                       ... bench:      88,380 ns/iter (+/- 29,681)
derived_eq::list::eq                                ... bench:      74,408 ns/iter (+/- 7,253)
derived_eq::short_degenerate_dag::eq                ... bench:         564 ns/iter (+/- 11)
derived_eq::short_inverted_list::eq                 ... bench:         752 ns/iter (+/- 114)
derived_eq::short_list::eq                          ... bench:         448 ns/iter (+/- 11)
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
