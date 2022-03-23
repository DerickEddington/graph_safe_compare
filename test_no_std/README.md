This package is only an integration test of using the package in the parent
directory as `#![no_std]`.  It is not a "test target", "example target", nor
"workspace member" of the parent package because that would be too dirty to do
as `no_std`.  (The dirtiness would be either:  Only working with the nightly
compiler so that the `eh_personality` "lang item" could be provided;  Or,
requiring the parent package's profile to have `panic="abort"` just for an
"example target" (which would not work with a "test target" since those are
always `panic="unwind"`);  Or, only working when remembering to export
`CARGO_BUILD_RUSTFLAGS="-C panic=abort"` when building an "example target".)

Note that, due to what seems to be a bug in nightly rustc, currently this
package fails to build with nightly unless --release is given.
