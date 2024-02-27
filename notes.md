# Notes

* Unit testing catching panics is probably not possible. Look into defmt-test or embedded-test instead,
 since it uses a debugger to halt and go back to run the next test. Hooking this up to QEMU will be "FUN"
* Maybe enable teh panic_handler feature?
* Might be possible to use std::panic::set_hook: https://doc.rust-lang.org/std/panic/fn.set_hook.html
* Another idea would be to figure out how to spawn a separate QEMU session for each individual test. Possibly
  using the `nexttest` package.



Helpful tip:

#![cfg_attr(not(test), no_std)]

This way, when writing the tests, you still get the std prelude.


### Tests not running

For reasons man was not meant to know, tests for crates other than the base one are not run, currently asking about it on matrix



ivmarkov
Gabe R.
Hmm. That's interesting. As an experiment, I dropped the test binary for one of the library crates into Ghidra, and it seems that the app_main function is empty, whereas for the main crate, there's a lot of instructions in app_main
app_main is here, and it is generated using the binstart or libstart feature of esp-idf-sys. If neither of these is enabled, user has to provide app_main.
esp-idf-sys/src/start.rs at master · esp-rs/esp-idf-sys - GitHub
Bindings for ESP-IDF (Espressif's IoT Development Framework) - esp-rs/esp-idf-sys
Basically the fact that we have and can operate a regular STD main function is because of the binstart feature of esp-idf-sys. ESP IDF expects app_main with a fixed signature, as the entry point. So for cargo test I guess you have to generate the app_main in a similar fashion as I do there.

ivmarkov
No idea. You understand the test framework of cargo much more than me now.
Gabe R.
Hmm. That's a bit scary.
But thanks, at least I know understand what the issue is (the lack of main.rs and a main function means that the test main function -- whatever that is -- is not being called)
ivmarkov
Hmmm, it works the other way around. The main.rs and the Rust STD main function inside is being called from (the) C runtime, not the other way around. It is just that what the Rust compiler generates as a callable C "main" function is not really what ESP IDF expects. I'm in fact surprised that cargo test does not result in linkage errors with ESP IDF, as who is generating app_main in that case?