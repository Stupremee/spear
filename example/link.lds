ENTRY(_entry)

SECTIONS
{
  . = 0x80000000;

  .text : {
    *(.text)
    *(.text.*)
  }

  .data : {
    *(.data)
    *(.data.*)
  }
}
