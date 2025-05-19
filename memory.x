/* Memory layout for Teensy 4.0 */
MEMORY
{
    /* 
     * ITCM: Instruction Tightly Coupled Memory
     * Fast access for frequently executed code
     */
    ITCM (rwx): ORIGIN = 0x00000000, LENGTH = 512K
    
    /*
     * DTCM: Data Tightly Coupled Memory
     * Fast access for frequently accessed data
     */
    DTCM (rwx): ORIGIN = 0x20000000, LENGTH = 512K
    
    /*
     * OCRAM: On-Chip RAM
     * General purpose RAM
     */
    OCRAM (rwx): ORIGIN = 0x20200000, LENGTH = 512K
    
    /*
     * FLASH: Main program storage
     * The Teensy 4.0 has 2MB of Flash
     */
    FLASH (rx): ORIGIN = 0x60000000, LENGTH = 2M
}

/* Sections definitions */
SECTIONS
{
    /* The stack starts at the end of DTCM with a reserved section for extra safety */
    _stack_size = 48K;  /* Increased stack size for USB operations and error handling */
    _stack_start = ORIGIN(DTCM) + LENGTH(DTCM);
    _stack_end = _stack_start - _stack_size;
    
    /* Use ITCM for fast access to vector table and core code */
    .text :
    {
        *(.text.entry)
        *(.text)
        *(.text*)
        . = ALIGN(4);
    } > ITCM AT > FLASH
    
    /* Constants should be in ITCM for fast access */
    .rodata :
    {
        *(.rodata)
        *(.rodata*)
        . = ALIGN(4);
    } > ITCM AT > FLASH
    
    /* Initialized data in DTCM for fast access */
    .data :
    {
        *(.data)
        *(.data*)
        . = ALIGN(4);
    } > DTCM AT > FLASH
    
    /* BSS in DTCM */
    .bss :
    {
        *(.bss)
        *(.bss*)
        *(COMMON)
        . = ALIGN(4);
    } > DTCM
    
    /* USB DMA Buffer Section - Place in DTCM for fastest access */
    .usb_buffer (NOLOAD) :
    {
        . = ALIGN(4);
        _usb_buffer_start = .;
        /* Reserve 4KB for USB DMA buffers */
        . += 4K;
        _usb_buffer_end = .;
        . = ALIGN(4);
    } > DTCM
}

/* Entry point */
ENTRY(reset_handler);