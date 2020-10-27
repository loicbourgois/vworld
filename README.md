# VWorld

An artificial life simulation

## Quickstart

```bash
git clone git@github.com:loicbourgois/vworld.git
export vworld_root_folder=$(pwd)/vworld
$vworld_root_folder/scripts/demo.sh
```

## Other demos

```bash
$vworld_root_folder/scripts/fish.sh
```

## Deploy on [Scaleway Instances](https://www.scaleway.com/en/virtual-instances/)

```bash
export scaleway_secret_key=$(cat $HOME/.scaleway-vworld-secret-key)
export scaleway_organization_id=$(cat $HOME/.scaleway-vworld-organization-id)
$vworld_root_folder/scripts/scaleway-instance-deploy.sh
```


thread '<unnamed>' panicked at 'called `Option::unwrap()` on a `None` value', src/particle.rs:305:55
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread '<unnamed>' panicked at 'internal error: entered unreachable code', /rustc/04488afe34512aa4c33566eb16d8c912a3ae04f9/src/libstd/macros.rs:13:23
thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: RecvError', src/main.rs:130:63
stack backtrace:
thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: "PoisonError { inner: .. }"', src/main.rs:164:64
   0:        0x10a6357fe - <std::sys_common::backtrace::_print::DisplayBacktrace as core::fmt::Display>::fmt::h24bb64d98a7e25d6
   1:        0x10a65687c - core::fmt::write::h8fdc9cddb01cd8b2
   2:        0x10a6319e9 - std::io::Write::write_fmt::hcc3030013983bab6
   3:        0x10a6376a5 - std::panicking::default_hook::{{closure}}::h95817712c5ff0736
   4:        0x10a6373e2 - std::panicking::default_hook::h34e085f4e0b1062d
   5:        0x10a637c05 - std::panicking::rust_panic_with_hook::haf571858f996ac45
   6:        0x10a66317d - std::panicking::begin_panic::h4cc6d922f85a4422
   7:        0x10a5ed392 - std::sync::mpsc::oneshot::Packet<T>::drop_port::h221c76d9b0af3c92
   8:        0x10a5ed4c3 - core::ptr::drop_in_place::h0f5c04f03b60f79c
   9:        0x10a5d8c55 - core::ptr::drop_in_place::hb7e514f295852d72
  10:        0x10a5dfcd8 - vworld_server::main::{{closure}}::h5efee060ac2ddef0
  11:        0x10a5d7698 - std::sys_common::backtrace::__rust_begin_short_backtrace::h53a0e101435d1b9c
  12:        0x10a5f6409 - core::ops::function::FnOnce::call_once{{vtable.shim}}::h908eebbdbc0fed06
  13:        0x10a639cbd - std::sys::unix::thread::Thread::new::thread_start::hf8a0ec6cfa81ff3d
  14:     0x7fff7187e109 - _ZL12preoptimized
thread panicked while panicking. aborting.
/Users/lbourgois/github/vworld/scripts/vworld-server-start-singlechunk.sh: line 14: 60112 Illegal instruction: 4  vworld_port="10001" vworld_address="127.0.0.1" vworld_chunk_configuration=$chunk_configuration cargo run $release


error: Connection closed normally
error writer socket: IO error: Protocol wrong type for socket (os error 41)
thread '<unnamed>' panicked at 'called `Option::unwrap()` on a `None` value', src/particle.rs:305:55
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread '<unnamed>' panicked at 'internal error: entered unreachable code', /rustc/04488afe34512aa4c33566eb16d8c912a3ae04f9/src/libstd/macros.rs:13:23
thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: RecvError', src/main.rs:130:63
stack backtrace:
thread '<unnamed>' panicked at 'called `Result::unwrap()` on an `Err` value: "PoisonError { inner: .. }"', src/main.rs:164:64
   0:        0x107b6e7fe - <std::sys_common::backtrace::_print::DisplayBacktrace as core::fmt::Display>::fmt::h24bb64d98a7e25d6
   1:        0x107b8f87c - core::fmt::write::h8fdc9cddb01cd8b2
   2:        0x107b6a9e9 - std::io::Write::write_fmt::hcc3030013983bab6
   3:        0x107b706a5 - std::panicking::default_hook::{{closure}}::h95817712c5ff0736
   4:        0x107b703e2 - std::panicking::default_hook::h34e085f4e0b1062d
   5:        0x107b70c05 - std::panicking::rust_panic_with_hook::haf571858f996ac45
   6:        0x107b9c17d - std::panicking::begin_panic::h4cc6d922f85a4422
   7:        0x107b26392 - std::sync::mpsc::oneshot::Packet<T>::drop_port::h221c76d9b0af3c92
   8:        0x107b264c3 - core::ptr::drop_in_place::h0f5c04f03b60f79c
   9:        0x107b11c55 - core::ptr::drop_in_place::hb7e514f295852d72
  10:        0x107b18cd8 - vworld_server::main::{{closure}}::h5efee060ac2ddef0
  11:        0x107b10698 - std::sys_common::backtrace::__rust_begin_short_backtrace::h53a0e101435d1b9c
  12:        0x107b2f409 - core::ops::function::FnOnce::call_once{{vtable.shim}}::h908eebbdbc0fed06
  13:        0x107b72cbd - std::sys::unix::thread::Thread::new::thread_start::hf8a0ec6cfa81ff3d
  14:     0x7fff7187e109 - _ZL12preoptimized
thread panicked while panicking. aborting.
/Users/lbourgois/github/vworld/scripts/vworld-server-start-singlechunk.sh: line 14: 61439 Illegal instruction: 4  vworld_port="10001" vworld_address="127.0.0.1" vworld_chunk_configuration=$chunk_configuration cargo run $release
lbourgois@scw-loic-mac vworld % $vworld_root_folder/scripts/demo.sh
