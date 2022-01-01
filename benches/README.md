# Overview of benchmark results

## Results from my computer

Executed on my Ryzen 7 5800H CPU in "maximum performance" mode (45 W, low-power states disabled,
fans on max, but GHz can still be varied a little), on my 3200 MHz DDR4 RAM (2x32 G, dual
channel), with very little other load, on NixOS 21.11 with its 5.15.11 kernel, compiled by rustc
1.59.0-nightly, with LTO, with each benchmark running serially (one at a time) using only 1 core
(leaving 7 other cores for the other tiny load).

The following results are interpreted farther [below](#interpretation).

```
$ cargo +nightly bench --profile bench-max-optim

     Running unittests (target/bench-max-optim/deps/equiv-3d2a7a15b064f126)

running 66 tests
test basic::degenerate_dag::equiv                        ... bench:   3,709,111 ns/iter (+/- 136,433)
test basic::degenerate_dag::limited_equiv                ... bench:   3,937,272 ns/iter (+/- 152,814)
test basic::inverted_list::equiv                         ... bench:     179,331 ns/iter (+/- 37,946)
test basic::inverted_list::limited_equiv                 ... bench:     176,260 ns/iter (+/- 27,371)
test basic::list::equiv                                  ... bench:     152,842 ns/iter (+/- 15,247)
test basic::list::limited_equiv                          ... bench:     169,783 ns/iter (+/- 21,178)
test basic::short_degenerate_dag::equiv                  ... bench:       1,818 ns/iter (+/- 64)
test basic::short_degenerate_dag::limited_equiv          ... bench:       1,970 ns/iter (+/- 68)
test basic::short_inverted_list::equiv                   ... bench:       1,681 ns/iter (+/- 53)
test basic::short_inverted_list::limited_equiv           ... bench:       1,898 ns/iter (+/- 49)
test basic::short_list::equiv                            ... bench:       1,782 ns/iter (+/- 42)
test basic::short_list::limited_equiv                    ... bench:       1,700 ns/iter (+/- 67)
test cycle_safe::degenerate_cyclic::equiv                ... bench:       3,090 ns/iter (+/- 58)
test cycle_safe::degenerate_cyclic::precheck_equiv       ... bench:       7,013 ns/iter (+/- 262)
test cycle_safe::degenerate_dag::equiv                   ... bench:       2,958 ns/iter (+/- 117)
test cycle_safe::degenerate_dag::precheck_equiv          ... bench:       6,040 ns/iter (+/- 228)
test cycle_safe::inverted_list::equiv                    ... bench:     271,584 ns/iter (+/- 25,081)
test cycle_safe::inverted_list::precheck_equiv           ... bench:     267,445 ns/iter (+/- 21,876)
test cycle_safe::list::equiv                             ... bench:     248,335 ns/iter (+/- 27,933)
test cycle_safe::list::precheck_equiv                    ... bench:     258,195 ns/iter (+/- 37,146)
test cycle_safe::short_degenerate_cyclic::equiv          ... bench:       1,348 ns/iter (+/- 54)
test cycle_safe::short_degenerate_cyclic::precheck_equiv ... bench:       5,649 ns/iter (+/- 164)
test cycle_safe::short_degenerate_dag::equiv             ... bench:       1,278 ns/iter (+/- 20)
test cycle_safe::short_degenerate_dag::precheck_equiv    ... bench:       1,914 ns/iter (+/- 100)
test cycle_safe::short_inverted_list::equiv              ... bench:       6,882 ns/iter (+/- 275)
test cycle_safe::short_inverted_list::precheck_equiv     ... bench:       1,862 ns/iter (+/- 89)
test cycle_safe::short_list::equiv                       ... bench:       4,620 ns/iter (+/- 130)
test cycle_safe::short_list::precheck_equiv              ... bench:       1,750 ns/iter (+/- 75)
test deep_safe::degenerate_dag::equiv                    ... bench:   3,395,443 ns/iter (+/- 153,561)
test deep_safe::inverted_list::equiv                     ... bench:     111,597 ns/iter (+/- 11,637)
test deep_safe::list::equiv                              ... bench:     112,183 ns/iter (+/- 7,879)
test deep_safe::long_inverted_list::equiv                ... bench:   4,403,571 ns/iter (+/- 629,291)
test deep_safe::long_list::equiv                         ... bench:   6,693,426 ns/iter (+/- 422,358)
test deep_safe::short_degenerate_dag::equiv              ... bench:       1,727 ns/iter (+/- 76)
test deep_safe::short_inverted_list::equiv               ... bench:       1,298 ns/iter (+/- 54)
test deep_safe::short_list::equiv                        ... bench:       1,303 ns/iter (+/- 70)
test derived_eq::degenerate_dag::eq                      ... bench:   1,158,603 ns/iter (+/- 49,270)
test derived_eq::inverted_list::eq                       ... bench:      75,044 ns/iter (+/- 15,041)
test derived_eq::list::eq                                ... bench:      70,195 ns/iter (+/- 11,495)
test derived_eq::short_degenerate_dag::eq                ... bench:         544 ns/iter (+/- 24)
test derived_eq::short_inverted_list::eq                 ... bench:         607 ns/iter (+/- 75)
test derived_eq::short_list::eq                          ... bench:         422 ns/iter (+/- 20)
test robust::degenerate_cyclic::equiv                    ... bench:       3,011 ns/iter (+/- 110)
test robust::degenerate_cyclic::precheck_equiv           ... bench:       5,676 ns/iter (+/- 201)
test robust::degenerate_dag::equiv                       ... bench:       2,952 ns/iter (+/- 76)
test robust::degenerate_dag::precheck_equiv              ... bench:       5,894 ns/iter (+/- 227)
test robust::inverted_list::equiv                        ... bench:     187,335 ns/iter (+/- 22,390)
test robust::inverted_list::precheck_equiv               ... bench:     185,101 ns/iter (+/- 27,250)
test robust::list::equiv                                 ... bench:     193,172 ns/iter (+/- 29,298)
test robust::list::precheck_equiv                        ... bench:     194,018 ns/iter (+/- 15,518)
test robust::long_degenerate_cyclic::equiv               ... bench: 115,256,800 ns/iter (+/- 11,149,944)
test robust::long_degenerate_cyclic::precheck_equiv      ... bench: 113,632,758 ns/iter (+/- 11,256,653)
test robust::long_degenerate_dag::equiv                  ... bench: 114,925,357 ns/iter (+/- 10,666,182)
test robust::long_degenerate_dag::precheck_equiv         ... bench: 113,278,800 ns/iter (+/- 9,800,128)
test robust::long_inverted_list::equiv                   ... bench:   8,724,244 ns/iter (+/- 1,839,951)
test robust::long_inverted_list::precheck_equiv          ... bench:   8,907,767 ns/iter (+/- 1,634,104)
test robust::long_list::equiv                            ... bench:  12,276,208 ns/iter (+/- 1,079,487)
test robust::long_list::precheck_equiv                   ... bench:  14,883,785 ns/iter (+/- 2,294,494)
test robust::short_degenerate_cyclic::equiv              ... bench:       1,378 ns/iter (+/- 131)
test robust::short_degenerate_cyclic::precheck_equiv     ... bench:       4,055 ns/iter (+/- 160)
test robust::short_degenerate_dag::equiv                 ... bench:       1,339 ns/iter (+/- 48)
test robust::short_degenerate_dag::precheck_equiv        ... bench:       1,895 ns/iter (+/- 107)
test robust::short_inverted_list::equiv                  ... bench:       4,389 ns/iter (+/- 217)
test robust::short_inverted_list::precheck_equiv         ... bench:       1,473 ns/iter (+/- 73)
test robust::short_list::equiv                           ... bench:       4,424 ns/iter (+/- 295)
test robust::short_list::precheck_equiv                  ... bench:       1,563 ns/iter (+/- 78)

test result: ok. 0 passed; 0 failed; 0 ignored; 66 measured; 0 filtered out; finished in 319.48s
```

## Interpretation

---

```
basic::degenerate_dag::equiv                        ... bench:   3,709,111 ns/iter (+/- 136,433)
basic::degenerate_dag::limited_equiv                ... bench:   3,937,272 ns/iter (+/- 152,814)
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
basic::inverted_list::equiv                         ... bench:     179,331 ns/iter (+/- 37,946)
basic::inverted_list::limited_equiv                 ... bench:     176,260 ns/iter (+/- 27,371)
basic::list::equiv                                  ... bench:     152,842 ns/iter (+/- 15,247)
basic::list::limited_equiv                          ... bench:     169,783 ns/iter (+/- 21,178)
```

All variants do `2*length` recursions for lists.

The basic variant uses the normal call-stack, which seems to be nearly as fast for `inverted_list`
(left edges: list tail, right edges: leaf elements) as it is for `list` (left edges: leaf
elements, right edges: list tail).

These cases, with a length of `8,000`, do `16,000` recursions.

---

```
basic::short_degenerate_dag::equiv                  ... bench:       1,818 ns/iter (+/- 64)
basic::short_degenerate_dag::limited_equiv          ... bench:       1,970 ns/iter (+/- 68)
```

These cases, with a depth of `7`, do `254` (`2^8 - 2`) recursions.

---

```
basic::short_inverted_list::equiv                   ... bench:       1,681 ns/iter (+/- 53)
basic::short_inverted_list::limited_equiv           ... bench:       1,898 ns/iter (+/- 49)
basic::short_list::equiv                            ... bench:       1,782 ns/iter (+/- 42)
basic::short_list::limited_equiv                    ... bench:       1,700 ns/iter (+/- 67)
```

These cases, with a length of `100`, do `200` recursions.

---

```
cycle_safe::degenerate_cyclic::equiv                ... bench:       3,090 ns/iter (+/- 58)
cycle_safe::degenerate_cyclic::precheck_equiv       ... bench:       7,013 ns/iter (+/- 262)
cycle_safe::degenerate_dag::equiv                   ... bench:       2,958 ns/iter (+/- 117)
cycle_safe::degenerate_dag::precheck_equiv          ... bench:       6,040 ns/iter (+/- 228)
```

The cycle-safe variants do detect shared structure and so do only `2*depth` recursions for the
`degenerate_dag` and `degenerate_cyclic` graph shapes.

These cases, with a depth of `18`, do `36` recursions for the `equiv` cases.  Unlike the basic
variant, each recursion involves hash-table operations, because the "interleave" mode stays in its
"slow" phase for all recursions due to continously detecting shared structure.

For the `equiv` cases, which only do the "interleave" mode, while the `recursion/ns` speed is
around `10%` as fast, there are only around `0.007%` as many recursions, and so it handles the
same `degenerate_dag` shape around `125,000%` as fast and handles the `degenerate_cyclic` shape at
that speed which the basic and only-deep-safe variants cannot handle at any speed.

For the `precheck_equiv` cases, the "precheck" mode, which is like the limited basic variant,
would need to do `2^19 - 2` recursions but reaches its limit and aborts (due to, either, the
exponential complexity of the basic way of traversing the `degenerate_dag` shape, or, due to
infinitely cycling while traversing the `degenerate_cyclic` shape) and so this effort is wasted,
before doing the "interleave" mode which succeeds quickly because it is unlimited and it does
shared-structure detection.

---

```
cycle_safe::inverted_list::equiv                    ... bench:     271,584 ns/iter (+/- 25,081)
cycle_safe::inverted_list::precheck_equiv           ... bench:     267,445 ns/iter (+/- 21,876)
cycle_safe::list::equiv                             ... bench:     248,335 ns/iter (+/- 27,933)
cycle_safe::list::precheck_equiv                    ... bench:     258,195 ns/iter (+/- 37,146)
```

Like the basic variant, the cycle-safe variants do `2*length` recursions for lists.  Unlike the
basic variant, the "interleave" mode is used which interleaves a shared-structure-detecting "slow"
phase with a basic "fast" phase.

These cases, with a length of `8,000`, do `16,000` recursions.

For these lists without shared structure, the "slow" phase only does about `10%` of recursions and
the "fast" phase does about `90%`.  These cases are around `62%` as fast as the basic variant,
which is not too bad of a trade-off for the ability to also handle cyclic and degenerate graphs
efficiently.

---

```
cycle_safe::short_degenerate_cyclic::equiv          ... bench:       1,348 ns/iter (+/- 54)
cycle_safe::short_degenerate_cyclic::precheck_equiv ... bench:       5,649 ns/iter (+/- 164)
cycle_safe::short_degenerate_dag::equiv             ... bench:       1,278 ns/iter (+/- 20)
cycle_safe::short_degenerate_dag::precheck_equiv    ... bench:       1,914 ns/iter (+/- 100)
```

These cases, with a depth of `7`, do only `14` recursions for the `equiv` cases, unlike the basic
variant.

The "interleave" mode stays in its "slow" phase for all recursions, but the
`short_degenerate_dag::equiv` case is still faster than `basic` and `deep_safe`, and the
`short_degenerate_cyclic::equiv` is also fast and can be handled.

The `short_degenerate_cyclic::precheck_equiv` case wastes the effort of the "precheck" mode on
this shape that has more basic-traversed edges (infinite) than the precheck limit.

The `short_degenerate_dag::precheck_equiv` case, which does `2^8 - 2` recursions, is able to
complete the "precheck" mode on this small short shape, without doing "interleave", and so is as
fast as the `limited_equiv` of the basic variant

---

```
cycle_safe::short_inverted_list::equiv              ... bench:       6,882 ns/iter (+/- 275)
cycle_safe::short_inverted_list::precheck_equiv     ... bench:       1,862 ns/iter (+/- 89)
cycle_safe::short_list::equiv                       ... bench:       4,620 ns/iter (+/- 130)
cycle_safe::short_list::precheck_equiv              ... bench:       1,750 ns/iter (+/- 75)
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
deep_safe::degenerate_dag::equiv                    ... bench:   3,395,443 ns/iter (+/- 153,561)
```

The deep-safe variants do not use the normal call-stack and instead use a vector as the stack of
which nodes to continue recurring on.

The `deep_safe` cases, like the basic variant, do not detect shared structure, and so must do
`2^(depth+1)-2` recursions for the `degenerate_dag` graph shape.  With a depth of `18`, `2^19 - 2`
recursions are done.

The vector stack is `109%` as fast as the call-stack, comparing this case to
`basic::degenerate_dag::equiv`.

---

```
deep_safe::inverted_list::equiv                     ... bench:     111,597 ns/iter (+/- 11,637)
deep_safe::list::equiv                              ... bench:     112,183 ns/iter (+/- 7,879)
```

Like the basic variant, the deep-safe variants do `2*length` recursions for lists.

These cases, with a length of `8,000`, do `16,000` recursions, on a vector stack.

The vector stack is `136%` and `161%` as fast as the call-stack, comparing these cases to the
`basic::list::equiv` and `basic::inverted_list::equiv` cases.

---

```
deep_safe::long_inverted_list::equiv                ... bench:   4,403,571 ns/iter (+/- 629,291)
deep_safe::long_list::equiv                         ... bench:   6,693,426 ns/iter (+/- 422,358)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector stack.

The same amount of recursions is done as the `degenerate_dag` cases, but with much deeper depth.
While the `recursion/ns` speed is, at worst, around `50%` as fast, or, at best, around `77%` as
fast, (comparing `deep_safe::long_list::equiv` or `deep_safe::long_inverted_list::equiv` to
`deep_safe::degenerate_dag::equiv`), the deep-safe variants can handle very-deep graphs which the
basic and only-cycle-safe variants cannot handle at any speed.

While a vector stack is faster than the call stack for the cases with shallower shapes, it is
slower for these cases.  For the `long_list` shape, this is expected, but for the
`long_inverted_list` shape, it is unexpected.

The `long_inverted_list` benefits from a kind of "tail-call elimination" because it descends its
list elements, which are leaf nodes, before descending its list tails, and so the maximum amount
of items on its vector stack should be only `2`.  Whereas, `long_list` descends its list tails
before its list elements, and so the maximum amount of items on its vector stack should be the
same as its length of `2^18`.

With `long_list` using so much of a vector there are factors that explain why it is slower than
`long_inverted_list`.  Linux's demand paging of larger allocations is suspected to be at play,
which will cause some slow-down since the cost is not amortized since the vector memory is
allocated and used only once for each iteration of this case.  Further, twice the initial capacity
of a vector is used, causing a reallocation for resizing it, for each iteration, which will cause
more slow-down.

It is currently unexplained why the speed of `deep_safe::long_inverted_list::equiv` is not closer
to that of `deep_safe::degenerate_dag::equiv`.

(Note about achieving the TCE:  Users control the order that edges are descended for their types,
and so can achieve TCE for their shapes regardless of whether they are "left-handed" or
"right-handed".  Unlike with traditional TCE of fixed equivalence predicates.)

---

```
deep_safe::short_degenerate_dag::equiv              ... bench:       1,727 ns/iter (+/- 76)
```

This case, with a depth of `7`, does `254` (`2^8 - 2`) recursions, like the basic variant, but on
a vector stack, unlike the basic variant.

The vector stack is `105%` as fast as the call-stack, comparing this case to
`basic::short_degenerate_dag::equiv`.

---

```
deep_safe::short_inverted_list::equiv               ... bench:       1,298 ns/iter (+/- 54)
deep_safe::short_list::equiv                        ... bench:       1,303 ns/iter (+/- 70)
```

These cases, with a length of `100`, do `200` recursions, like the basic variant, but on a vector
stack, unlike the basic variant.

The vector stack is `%137` and `130%` as fast as the call-stack, comparing these cases to
`basic::short_list::equiv` and `basic::short_inverted_list::equiv`.

---

```
robust::degenerate_cyclic::equiv                    ... bench:       3,011 ns/iter (+/- 110)
robust::degenerate_cyclic::precheck_equiv           ... bench:       5,676 ns/iter (+/- 201)
robust::degenerate_dag::equiv                       ... bench:       2,952 ns/iter (+/- 76)
robust::degenerate_dag::precheck_equiv              ... bench:       5,894 ns/iter (+/- 227)
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
robust::inverted_list::equiv                        ... bench:     187,335 ns/iter (+/- 22,390)
robust::inverted_list::precheck_equiv               ... bench:     185,101 ns/iter (+/- 27,250)
robust::list::equiv                                 ... bench:     193,172 ns/iter (+/- 29,298)
robust::list::precheck_equiv                        ... bench:     194,018 ns/iter (+/- 15,518)
```

Like `basic`, the robust variant does `2*length` recursions for lists.  Like `deep_safe`, a vector
stack is used.  Like `cycle_safe`, the "interleave" mode is used with about `10%` "slow" phase and
about `90%` "fast" phase.

These cases, with a length of `8,000`, do `16,000` recursions.

The speed is around `79%`, at worst, as fast as `basic`, is significantly slower than `deep_safe`
due to the involvement of the "slow" phase of "interleave" mode, and is significantly faster than
`cycle_safe` due to the use of a vector stack, which improves the attractiveness of the trade-off
for the cycle-safety, and it also has the deep-safety.

---

```
robust::long_degenerate_cyclic::equiv               ... bench: 115,256,800 ns/iter (+/- 11,149,944)
robust::long_degenerate_cyclic::precheck_equiv      ... bench: 113,632,758 ns/iter (+/- 11,256,653)
robust::long_degenerate_dag::equiv                  ... bench: 114,925,357 ns/iter (+/- 10,666,182)
robust::long_degenerate_dag::precheck_equiv         ... bench: 113,278,800 ns/iter (+/- 9,800,128)
```

These shapes are degenerate pair-chains but their depth is `2^18` which is the same as the length
of the long-list shapes.

For the "interleave" mode used by this robust variant, `2^19` (`2*depth`) recursions are done.
For the basic variant and the "precheck" mode, an infeasible `2^(2^18+1)-2` recursions would be
required.  For `cycle_safe`, the depth would cause stack-overflow crash.

While the amount of recursions is the same as `basic::degenerate_dag` and `deep_safe::long_list`,
the "interleave" mode stays in its "slow" phase for all recursions, like `cycle_safe`.  This is
why the `recursion/ns` speed is `3%` as fast as `basic`.  That is the trade-off for the ability to
handle these very-deep degenerate shapes which all other variants cannot.

It is currently unexplained why the speed of the `precheck_equiv` cases was slightly faster than
the `equiv` cases, when the additional effort of the "precheck" mode is always wasted for these
shapes.

---

```
robust::long_inverted_list::equiv                   ... bench:   8,724,244 ns/iter (+/- 1,839,951)
robust::long_inverted_list::precheck_equiv          ... bench:   8,907,767 ns/iter (+/- 1,634,104)
robust::long_list::equiv                            ... bench:  12,276,208 ns/iter (+/- 1,079,487)
robust::long_list::precheck_equiv                   ... bench:  14,883,785 ns/iter (+/- 2,294,494)
```

These cases, with a length of `2^18`, do `2^19` recursions, on a vector stack, like
`deep_safe::long`.

The "interleave" mode is used with about `10%` "slow" phase and about `90%` "fast" phase, like
`cycle_safe`, but unlike `deep_safe`.  This is why these cases are about half as fast as those of
`deep_safe::long`, but are still much faster than the `robust::long_degenerate` cases, which all
have the same amount of recursions.

The benefit from the kind of "tail-call elimination" with a vector stack is why the
`long_inverted_list` cases are faster than the `long_list`, like the `deep_safe::long` cases.

---

```
robust::short_degenerate_cyclic::equiv              ... bench:       1,378 ns/iter (+/- 131)
robust::short_degenerate_cyclic::precheck_equiv     ... bench:       4,055 ns/iter (+/- 160)
robust::short_degenerate_dag::equiv                 ... bench:       1,339 ns/iter (+/- 48)
robust::short_degenerate_dag::precheck_equiv        ... bench:       1,895 ns/iter (+/- 107)
```

These cases, with a depth of `7`, do only `14` recursions for the `equiv` cases, like
`cycle_safe`, and unlike `basic` and `deep_safe` (which do `254`).

These cases are as fast as `cycle_safe::short_degenerate`, as expected, since they involve the
same aspects other than `robust` using a vector stack (which happens to reduce the cost of the
wasted prechecks by a little).

---

```
robust::short_inverted_list::equiv                  ... bench:       4,389 ns/iter (+/- 217)
robust::short_inverted_list::precheck_equiv         ... bench:       1,473 ns/iter (+/- 73)
robust::short_list::equiv                           ... bench:       4,424 ns/iter (+/- 295)
robust::short_list::precheck_equiv                  ... bench:       1,563 ns/iter (+/- 78)
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
derived_eq::degenerate_dag::eq                      ... bench:   1,158,603 ns/iter (+/- 49,270)
derived_eq::inverted_list::eq                       ... bench:      75,044 ns/iter (+/- 15,041)
derived_eq::list::eq                                ... bench:      70,195 ns/iter (+/- 11,495)
derived_eq::short_degenerate_dag::eq                ... bench:         544 ns/iter (+/- 24)
derived_eq::short_inverted_list::eq                 ... bench:         607 ns/iter (+/- 75)
derived_eq::short_list::eq                          ... bench:         422 ns/iter (+/- 20)
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
