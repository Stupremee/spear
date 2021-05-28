_entry:
  li t1, 255
  la t0, tohost

exit:
  sw t1, 0(t0)
  j exit

# The `tohost` symbol is a special symbol that is used to communicate with the emulator.
# It allows to perform "syscalls". Currently, any non-zero write to the `tohost` symbol
# will result in stopping execution
tohost:
  # The `nop` is currently needed, because spear will not map the `.data` section if it
  # has no contents, so we add a dummy instruction
  nop
