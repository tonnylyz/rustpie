#!/usr/bin/env bash

for value in putc \
            getc \
            set_exception_handler \
            get_tid \
            thread_yield \
            thread_destroy \
            thread_alloc \
            thread_set_status \
            mem_alloc \
            mem_map \
            mem_unmap \
            get_asid \
            address_space_alloc \
            address_space_destroy \
            event_wait \
            itc_receive \
            itc_send \
            itc_call \
            server_register \
            server_tid
do
    echo "injecting $value"
    touch src/syscall/mod.rs
    trap '' 2
    make emu FI=$value
    trap 2
done
