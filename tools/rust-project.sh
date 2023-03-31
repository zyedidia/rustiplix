#!/bin/bash

cat << EOM
{
    "sysroot_src": "$(rustc --print sysroot)/lib/rustlib/src/rust/library",
    "crates": [
        {
            "display_name": "kernel",
            "root_module": "kernel/lib.rs",
            "edition": "2021",
            "deps": []
        },
        {
            "display_name": "bootloader",
            "root_module": "bootloader/lib.rs",
            "edition": "2021",
            "deps": [
                {
                    "crate": 0,
                    "name": "kernel"
                }
            ]
        }
        {
            "display_name": "kmain",
            "root_module": "kmain/lib.rs",
            "edition": "2021",
            "deps": [
                {
                    "crate": 0,
                    "name": "kernel"
                }
            ]
        }
    ]
}
EOM
