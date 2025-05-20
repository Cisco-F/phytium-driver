pub const FGIC_RSGI_AFF1_OFFSET: u32 = 16;
pub const FGIC_RSGI_AFF2_OFFSET: u32 = 32;
pub const FGIC_RSGI_AFF3_OFFSET: u32 = 48;

pub const FGIC_LPI_START_ID: u32 = 8192;
pub const FGIC_DEVICE_MAX_ID: u32 = 256;
pub const FGIC_ITS_DEVICEID_MAX_EVENT: u32 = 32;
pub const LPI_INTR_MAX_NUM: u32 = 0xFFFF;

/// cpu interfaces
pub const GICC_SRE_SRE: u32 = 1 << 0;
pub const GICC_SRE_DFB: u32 = 1 << 1;
pub const GICC_SRE_DIB: u32 = 1 << 2;
pub const GICC_SRE_ENABLE: u32 = 1 << 3;

pub const GICC_CTLR_CBPR: u32 = 1 << 0;
pub const GICC_CTLR_CBPR_EL1S: u32 = 1 << 0;
pub const GICC_CTLR_EOIMODE: u32 = 1 << 1;
pub const GICC_CTLR_CBPR_EL1NS: u32 = 1 << 1;
pub const GICC_CTLR_EOIMODE_EL3: u32 = 1 << 2;
pub const GICC_CTLR_EOIMODE_EL1S: u32 = 1 << 3;
pub const GICC_CTLR_EOIMODE_EL1NS: u32 = 1 << 4; /* GICC_EOIR 和 GICC_AEOIR 提供优先级下降功能。GICC_DIR 提供中断停用功能。 */
pub const GICC_CTLR_RM: u32 = 1 << 5;
pub const GICC_CTLR_PMHE: u32 = 1 << 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpiRoutingMode {
    SpiRoutingToSpecific = 0,       // spi interrupts routed to the PE specified by affinity level
    SpiRoutingToAny = (1 << 31),    // spi interrupts routed to any PE defined as a participating node
}
pub type GicdIrpiterMode = SpiRoutingMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaitRwqMode {
    GicdCtlrRwpWait = 0,
    GicrCtlrRwpWait,
}

pub const FGIC_SPI_START_ID: i32 = 32;
pub const FGIC_PPI_END_ID: i32 = 31;
pub const FGIC_SGI_END_ID: i32 = 15;

pub const FGIC_GICD_32_PER_REG: u32 = 32;
pub const FGIC_GICD_16_PER_REG: u32 = 16;
pub const FGIC_GICD_4_PER_REG: u32 = 4;

pub const FGIC_INT_DEFAULT_PRI_X4: u32 = 0xA0A0_A0A0;
pub const FGIC_CPU_INTERFACE_DEFAULT_FILTER: u32 = 0xFF;

pub const INTERRUPT_CPU_ALL_SELECT: u64 = 0xffff_ffff_ffff_ffff;
pub const INTERRUPT_CPU_TARGET_ALL_SET: u32 = 0xFFFF_FFFF;

pub const IRQ_MODE_TRIG_LEVEL: u8 = 0x00; // Trigger: level triggered interrupt
pub const IRQ_MODE_TRIG_EDGE: u8 = 0x01;  // Trigger: edge triggered interrupt

pub const IRQ_GROUP_PRIORITY_3: u8 = 3; // group priority valid mask is bit[7:3], subpriority valid mask is bit[2:0]
pub const IRQ_GROUP_PRIORITY_4: u8 = 4; // group priority valid mask is bit[7:4], subpriority valid mask is bit[3:0]
pub const IRQ_GROUP_PRIORITY_5: u8 = 5; // group priority valid mask is bit[7:5], subpriority valid mask is bit[4:0]
pub const IRQ_GROUP_PRIORITY_6: u8 = 6; // group priority valid mask is bit[7:6], subpriority valid mask is bit[5:0]
pub const IRQ_GROUP_PRIORITY_7: u8 = 7; // group priority valid mask is bit[7],   subpriority valid mask is bit[6:0]

pub const IRQ_PRIORITY_OFFSET: u8 = 4; // implemented priority bit offset

pub const IRQ_PRIORITY_VALUE: [u8; 16] = [
    0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0xa, 0xb, 0xc, 0xd, 0xe, 0xf,
];

pub const IRQ_PRIORITY_MASK: [u8; 16] = [
    0x00, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80, 0x90, 0xa0, 0xb0, 0xc0, 0xd0, 0xe0, 0xf0,
];

// GICR
pub const FGIC_GICR_CTLR_OFFSET: u32 = 0x0;    /* See the register description Redistributor Control Register ,RW */
pub const FGIC_GICR_IIDR_OFFSET: u32 = 0x4;    /* Implementer Identification Register ,RO */
pub const FGIC_GICR_TYPER_L_OFFSET: u32 = 0x8;   /* Redistributor Type Register ,RO */
pub const FGIC_GICR_TYPER_H_OFFSET: u32 = 0xC;   /* Redistributor Type Register ,RO */
pub const FGIC_GICR_STATUSR_OFFSET: u32 = 0x10; /* Error Reporting Status Register, optional ,RW */
pub const FGIC_GICR_WAKER_OFFSET: u32 = 0x14;   /* See the register description Redistributor Wake Register ,RW */
pub const FGIC_GICR_MPAMIDR_OFFSET: u32 = 0x18; /* Report maximum PARTID and PMG Register ,RO */
pub const FGIC_GICR_PARTIDR_OFFSET: u32 = 0x1C; /* Set PARTID and PMG Register ,RW */

pub const FGIC_GICR_CTLR_RWP_MASK: u32 = 1 << 3;

/************************************* distributor ****************************************/
// GICD
pub const FGIC_GICD_CTLR_OFFSET: u32 = 0x0;       /* Distributor Control Register ,RW */
pub const FGIC_GICD_TYPER_OFFSET: u32 = 0x4;      /* Interrupt Controller Type Register ,RO */
pub const FGIC_GICD_IIDR_OFFSET: u32 = 0x8;       /* Distributor Implementer Identification Register ,RO */
pub const FGIC_GICD_TYPER2_OFFSET: u32 = 0xC;     /* Interrupt controller Type Register 2,RO */
pub const FGIC_GICD_STATUSR_OFFSET: u32 = 0x10;    /* Error Reporting Status Register, optional ,RW */
pub const FGIC_GICD_SETSPI_NSR_OFFSET: u32 = 0x40; /* Set SPI Register ,WO */
pub const FGIC_GICD_CLRSPI_NSR_OFFSET: u32 = 0x48; /* Clear SPI Register ,WO */
pub const FGIC_GICD_SETSPI_SR_OFFSET: u32 = 0x50;  /* Set SPI, Secure Register ,WO */
pub const FGIC_GICD_CLRSPI_SR_OFFSET: u32 = 0x58;  /* Clear SPI, Secure Register ,WO */
pub const FGIC_GICD_IGROUPR_OFFSET: u32 = 0x80;    /* Interrupt Group Registers ,RW */

pub const FGIC_GICD_ISENABLER_OFFSET: u32 = 0x100;  /* Interrupt Set-Enable Registers ,RW */
pub const FGIC_GICD_ICENABLER_OFFSET: u32 = 0x180;  /* Interrupt Clear-Enable Registers ,RW */
pub const FGIC_GICD_ISPENDR_OFFSET: u32 = 0x200;    /* Interrupt Set-Pending Registers ,RW */
pub const FGIC_GICD_ICPENDR_OFFSET: u32 = 0x280;    /* Interrupt Clear-Pending Registers ,RW */
pub const FGIC_GICD_ISACTIVER_OFFSET: u32 = 0x300;    // Interrupt Set-Active Registers, RW
pub const FGIC_GICD_ICACTIVER_OFFSET: u32 = 0x380;    // Interrupt Clear-Active Registers, RW
pub const FGIC_GICD_IPRIORITYR_OFFSET: u32 = 0x400;   // Interrupt Priority Registers, RW
pub const FGIC_GICD_ITARGETSR_OFFSET: u32 = 0x800;    // Interrupt Processor Targets Registers, RO

pub const FGIC_GICD_ICFGR_OFFSET: u32 = 0xC00;        // Interrupt Configuration Registers, RW
pub const FGIC_GICD_IGRPMODR_OFFSET: u32 = 0xD00;     // Interrupt Group Modifier Registers

pub const FGIC_GICD_NSACR_OFFSET: u32 = 0xE00;        // Non-secure Access Control, RW
pub const FGIC_GICD_SGIR_OFFSET: u32 = 0xF00;         // Software Generated Interrupt Register, WO
pub const FGIC_GICD_CPENDSGIR_OFFSET: u32 = 0xF10;    // SGI Clear-Pending Registers, RW
pub const FGIC_GICD_SPENDSGIR_OFFSET: u32 = 0xF20;    // SGI Set-Pending Registers, RW

pub const FGIC_GICD_IGROUPR_E_OFFSET: u32 = 0x1000;   // Interrupt Group Registers for extended SPI, RW

pub const FGIC_GICD_ISENABLER_E_OFFSET: u32 = 0x1200; // Interrupt Set-Enable for extended SPI, RW

pub const FGIC_GICD_ICENABLER_E_OFFSET: u32 = 0x1400; // Interrupt Clear-Enable for extended SPI, RW

pub const FGIC_GICD_ISPENDR_E_OFFSET: u32 = 0x1600;   // Interrupt Set-Pend for extended SPI range, RW
pub const FGIC_GICD_ICPENDR_E_OFFSET: u32 = 0x1800;   // Interrupt Clear-Pend for extended SPI, RW

pub const FGIC_GICD_ISACTIVER_E_OFFSET: u32 = 0x1A00; // Interrupt Set-Active for extended SPI, RW

pub const FGIC_GICD_ICACTIVER_E_OFFSET: u32 = 0x1C00; // Interrupt Clear-Active for extended SPI, RW

pub const FGIC_GICD_IPRIORITYR_E_OFFSET: u32 = 0x2000; // Interrupt Priority for extended SPI range, RW
pub const FGIC_GICD_ICFGR_E_OFFSET: u32 = 0x3000;      // Extended SPI Configuration Register, RW
pub const FGIC_GICD_IGRPMODR_E_OFFSET: u32 = 0x3400;   // Interrupt Group Modifier for extended SPI, RW
pub const FGIC_GICD_NSACR_E_OFFSET: u32 = 0x3600;      // Non-secure Access Control Registers for extended SPI range, RW

pub const FGIC_GICD_IROUTER_OFFSET: u32 = 0x6000;      // Interrupt Routing Registers, RW
pub const FGIC_GICD_IROUTER_E_OFFSET: u32 = 0x8000;    // Interrupt Routing Registers for extended SPI range, RW

/* FGIC_GICD_IPRIORITYR_OFFSET --- SPI priority */
pub const fn fgic_gicd_ipriorityr_value_mask(int_id: u32) -> u32 {
    0xFF << ((int_id % 4) << 3)
}

/* FGIC_GICD_IROUTER_OFFSET --- SPI Routing */
pub const FGIC_GICD_IROUTER_AFFINITY_MASK: usize = 0xFF << 32 | 0xFF << 16 | 0xFF << 8 | 0xFF; /* affinity mask */
pub const FGIC_GICD_IROUTER_RANGE_LIMIT: u32 = 1023; /* GICD_IROUTER<n>, Interrupt Routing Registers, n = 0 - 1019 */
pub const FGIC_GICD_IROUTER_BYTE_WIDTH: usize = 8;




// SGI
pub const FGIC_GICR_SGI_BASE_OFFSET: u32 = 0x10000;
pub const FGIC_GICR_IGROUPR0_OFFSET: u32 = 0x80;       /* Interrupt Group Register 0 ,RW */
pub const FGIC_GICR_IGROUPR_E_OFFSET: u32 = 0x84;      /* Interrupt Group Registers for extended PPI range ,RW */
pub const FGIC_GICR_ISENABLER0_OFFSET: u32 = 0x100;    /* Interrupt Set-Enable Register 0 ,RW */
pub const FGIC_GICR_ISENABLER_E_OFFSET: u32 = 0x104;   /* Interrupt Set-Enable for extended PPI range ,RW */
pub const FGIC_GICR_ICENABLER0_OFFSET: u32 = 0x180;    /* Interrupt Clear-Enable Register 0 ,RW */
pub const FGIC_GICR_ICENABLER_E_OFFSET: u32 = 0x184;   /* Interrupt Clear-Enable for extended PPI range ,RW */
pub const FGIC_GICR_ISPENDR0_OFFSET: u32 = 0x200;      /* Interrupt Set-Pend Register 0 ,RW */
pub const FGIC_GICR_ISPENDR_E_OFFSET: u32 = 0x204;     /* Interrupt Set-Pend for extended PPI range ,RW */
pub const FGIC_GICR_ICPENDR0_OFFSET: u32 = 0x280;      /* Interrupt Clear-Pend Register 0 ,RW */
pub const FGIC_GICR_ICPENDR_E_OFFSET: u32 = 0x284;     /* Interrupt Clear-Pend for extended PPI range ,RW */
pub const FGIC_GICR_ISACTIVER0_OFFSET: u32 = 0x300;    /* Interrupt Set-Active Register 0 ,RW */
pub const FGIC_GICR_ISACTIVER_E_OFFSET: u32 = 0x304;   /* Interrupt Set-Active for extended PPI range ,RW */
pub const FGIC_GICR_ICACTIVER0_OFFSET: u32 = 0x380;    /* Interrupt Clear-Active Register 0 ,RW */
pub const FGIC_GICR_ICACTIVER_E_OFFSET: u32 = 0x384;   /* Interrupt Clear-Active for extended PPI range ,RW */
pub const FGIC_GICR_IPRIORITYR_OFFSET: u32 = 0x400;    /* Interrupt Priority Registers ,RW */
pub const FGIC_GICR_IPRIORITYR_E_OFFSET: u32 = 0x420;  /* Interrupt Priority for extended PPI range ,RW */
pub const FGIC_GICR_ICFGR0_OFFSET: u32 = 0xC00;        /* SGI Configuration Register ,RW */
pub const FGIC_GICR_ICFGR1_OFFSET: u32 = 0xC04;        /* PPI Configuration Register ,RW */
pub const FGIC_GICR_ICFGR_E_OFFSET: u32 = 0xC08;       /* Extended PPI Configuration Register ,RW */
pub const FGIC_GICR_IGRPMODR0_OFFSET: u32 = 0xD00;     /* Interrupt Group Modifier Register 0 ,RW */
pub const FGIC_GICR_IGRPMODR_E_OFFSET: u32 = 0xD04;    /* Interrupt Group Modifier for extended PPI range ,RW */


/* FGIC_GICR_IGROUPR0_OFFSET --- Controls whether the corresponding SGI or PPI is in Group 0 or Group 1. */
pub const FGIC_GICR_IGROUPR0_DEFAULT_MASK: u32 = 0xFFFF_FFFF;


/* FGIC_GICR_IGRPMODR0_OFFSET --- controls whether the corresponding interrupt is in: • Secure Group 0.• Non-secure Group 1.• When System register access is enabled, Secure Group 1. */
pub const FGIC_GICR_IGRPMODR0_DEFAULT_MASK: u32 = 0xFFFF_FFFF;


/* FGIC_GICD_TYPER_OFFSET --- Provides information about what features the GIC implementation supports. */
pub const FGIC_GICD_TYPER_ITLINESNUMBER_MASK: u32 = 0x1F;
pub const FGIC_GICD_TYPER_MBIS: u32 = 1 << 16;


/* FGIC_GICR_WAKER_OFFSET ---  Permits software to control the behavior of the WakeRequest power management signal corresponding to the Redistributor */
pub const FGIC_GICR_WAKER_PROCESSOR_SLEEP_MASK: u32 = 1 << 1;
pub const FGIC_GICR_WAKER_CHILDREN_ASLEEP_MASK: u32 = 1 << 2;

/* FGIC_GICR_IPRIORITYR_OFFSET --- Enables forwarding of the corresponding SGI or PPI to the CPU interfaces*/
#[inline(always)]
pub const fn fgic_gicr_ipriorityr_value_mask(int_id: u32) -> u32 {
    0xFF << ((int_id % 4) << 3)
}

/* FGIC_GICD_CTLR_OFFSET --- Distributor switch */
pub const FGIC_GICD_CTLR_RWP_MASK: u32 = 1 << 31;
pub const FGIC_GICD_CTLR_DS_MASK: u32 = 1 << 6;


/* FGIC_GICR_ISPENDR0_OFFSET --- Adds the pending state to the corresponding SGI or PPI. */
pub const FGIC_GICR_ICPENDR0_DEFAULT_MASK: u32 = 0xFFFF_FFFF;

/* FGIC_GICR_ICENABLER0_OFFSET --- Disables forwarding of the corresponding interrupt to the CPU interfaces. */
pub const FGIC_GICR_ICENABLER0_DEFAULT_MASK: u32 = 0xFFFF_FFFF;


pub const GICV3_BASE_ADDR: u32 = 0x29A0_0000;
/// Gic distributor base address
pub const GICV3_DISTRIBUTOR_BASE_ADDR: u32 = GICV3_BASE_ADDR + 0;
pub const GICV3_RD_BASE_ADDR: u32 = GICV3_BASE_ADDR + 0x100000;
pub const GICV3_RD_OFFSET: u32 = 2 << 16;
pub const GICV3_RD_SIZE: u32 = 16 << 16;
/// ITS base address
pub const GICV3_ITS_BASE_ADDR: u32 = GICV3_BASE_ADDR + 0x20000;