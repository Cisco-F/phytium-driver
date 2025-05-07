pub enum SpiRoutingMode {
    SpiRoutingToSpecific = 0,       // spi interrupts routed to the PE specified by affinity level
    SpiRoutingToAny = (1 << 31),    // spi interrupts routed to any PE defined as a participating node
}
pub type GicdIrpiterMode = SpiRoutingMode;

pub const GIC_PPI_END_ID: isize = 31;

pub const INTERRUPT_CPU_TARGET_ALL_SET: u32 = 0xFFFF_FFFF;

/* FGIC_GICD_IROUTER_OFFSET --- SPI Routing */
pub const FGIC_GICD_IROUTER_AFFINITY_MASK: usize = 0xFF << 32 | 0xFF << 16 | 0xFF << 8 | 0xFF; /* affinity mask */
pub const FGIC_GICD_IROUTER_RANGE_LIMIT: u32 = 1023; /* GICD_IROUTER<n>, Interrupt Routing Registers, n = 0 - 1019 */
pub const FGIC_GICD_IROUTER_BYTE_WIDTH: usize = 8;

// 寄存器偏移量
pub const FGIC_GICD_IROUTER_OFFSET: u32 = 0x6000;