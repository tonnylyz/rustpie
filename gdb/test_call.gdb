target remote 127.0.0.1:1234
add-symbol-file trusted/target/aarch64/release/trusted
set confirm off
set pagination off
set logging on
set $last = 0
break microcall::message::Message::call
command
set $last = $PMCCNTR_EL0
end
add-symbol-file target/aarch64/release/rustpi
break *(&pop_context+92) if $last != 0
command
printf "DELTA:%d\n", $PMCCNTR_EL0 - $last
set $last = 0
end

