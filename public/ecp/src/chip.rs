use std::{collections::BTreeMap, fmt::Display};

use bincode::{Decode, Encode};
use prjcombine_entity::{EntityId, EntityRangeVec, EntityVec};
use prjcombine_interconnect::{
    db::IntDb,
    dir::{Dir, DirH, DirHV, DirV},
    grid::{
        BelCoord, CellCoord, ColId, ColIdExt, DieId, DieIdExt, EdgeIoCoord, RowId, RowIdExt,
        TileIobId,
    },
};
use prjcombine_types::enumerable::{EnumMap, EnumPartMap};

use crate::defs::bslots;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum ChipKind {
    Scm,
    Ecp,
    Xp,
    MachXo,
    Ecp2,
    Ecp2M,
    Xp2,
    Ecp3(Ecp3Kind),
    MachXo2(MachXo2Kind),
    Ecp4,
    Ecp5,
    Crosslink,
    Nexus(NexusKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum MachXo2Kind {
    MachXo2,
    MachXo3L,
    MachXo3Lfp,
    MachXo3D,
    MachNx,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum Ecp3Kind {
    Ecp3,
    Ecp3A,
}

/// Specifies the top-level clocking structure of a Nexus device.
///
/// Small devices have 2 PCLK regions (W, E), 1 center DCS, 2 EBR/DSP rows used for clock interconnect.
/// Large devices have 4 PCLK regions (like ECP5), 2 center DCS, 4 EBR/DSP rows used for clock interconnect.
///
/// The variant determines the exact set of `CLK_[WESN]*` tiles to use.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum NexusKind {
    /// LIFCL-17
    ///
    /// Clocks:
    ///
    /// - W: 2× CDR, 2× PLL, 3+3× IO, 10×CIB
    /// - E: 1× PLL (SE), 3+3× IO, 10×CIB
    /// - S: 2× PLL, 4+4+4× IO+ECLKDIV, 6×CIB
    /// - N: TCK+OSC, 1×PLL (NW; 4 inputs only), 2×MIPI, PCIEX1, 2×IO, 6×CIB
    ///
    /// IO banks:
    ///
    /// - 60, 61: N, MIPI
    /// - 0: N, active-mode config I/O
    /// - 1: E, passive-mode config I/O
    /// - 70: E, ADC
    /// - 3, 4, 5: S, HP IO; has 2× CDR in bank 5
    SmallMipi,
    /// LIFCL-40
    ///
    /// Clocks: like SmallMipi
    ///
    /// IO banks: superset of SmallMipi
    ///
    /// - 60, 61: N, MIPI
    /// - 80: N, PCIE ×1
    /// - 0: N, active-mode config I/O
    /// - 1: E, passive-mode config I/O
    /// - 2: E
    /// - 70: E, ADC
    /// - 3, 4, 5: S, HP IO; has 2× CDR in bank 5
    /// - 6, 7: W
    SmallMipiPcie,
    /// LFCPNX-100
    ///
    /// Clocks:
    ///
    /// - W: 2× CDR, 2× PLL, 3+3× IO, 9×CIB
    /// - E: OSC, 2× PLL, 3+3× IO, 9×CIB
    /// - S: 2× PLL, 4+4+4× IO+ECLKDIV, 6×CIB (like SmallMipi)
    /// - N: TCK+OSC, 2×PLL (4 inputs only), 8×PCS, PCIELL, 2×IO, 6×CIB
    ///
    /// IO banks: similar to SmallMipi numbering
    ///
    /// - 80, 81: N, PCS×8
    /// - 83: N, PCS REFCLK×2
    /// - 0: N, active-mode config I/O
    /// - 1: E, passive-mode config I/O
    /// - 2: E
    /// - 70: E, ADC
    /// - 3, 4, 5: S, HP IO; has 2× CDR in bank 5
    /// - 6, 7: W
    LargePcs,
    /// LIFCL-33
    ///
    /// Clocks:
    ///
    /// - W, E: absent
    /// - S: 1×PLL (SW), 4+4+1× IO, 4+4+4× ECLKDIV, 6×CIB
    /// - N: TCK+OSC, 4×IO, 6×CIB
    ///
    /// IO banks:
    ///
    /// - 5: N, extra passive-mode config data pins
    /// - 0: N, active-mode config pins
    /// - 1: N, passive-mode config pins
    /// - 2, 3, 4: S, HP IO
    SmallHalfIo,
    /// LFMXO5-25
    ///
    /// Clocks:
    ///
    /// - W: 2× CDR, 1× PLL (NW), 2+2+3× IO, 9×CIB
    /// - E: OSC, 1×PLL (SE), 2+2+3× IO, 9×CIB
    /// - S: 1×PLL (SE), 4+4× IO+ECLKDIV, 6×CIB
    /// - N: TCK+OSC, 1×PLL (NW), 2+2× IO, 6×CIB
    ///
    /// IO banks:
    ///
    /// - 0: N
    /// - 1: N, active-mode config pins
    /// - 2: E, passive-mode config pins
    /// - 3, 4: E
    /// - 70: E, ADC
    /// - 5, 6: S, HP IO; has 2× CDR in bank 6
    /// - 7, 8, 9: W
    SmallDenseIo,
}

impl NexusKind {
    pub fn is_large(self) -> bool {
        match self {
            NexusKind::SmallMipi => false,
            NexusKind::SmallMipiPcie => false,
            NexusKind::LargePcs => true,
            NexusKind::SmallHalfIo => false,
            NexusKind::SmallDenseIo => false,
        }
    }
}

impl ChipKind {
    pub fn has_out_f_branch(self) -> bool {
        matches!(
            self,
            ChipKind::Ecp | ChipKind::Xp | ChipKind::MachXo | ChipKind::MachXo2(_)
        )
    }

    pub fn has_out_ofx_branch(self) -> bool {
        matches!(
            self,
            ChipKind::Scm
                | ChipKind::Ecp
                | ChipKind::Xp
                | ChipKind::MachXo
                | ChipKind::MachXo2(_)
                | ChipKind::Ecp4
                | ChipKind::Ecp5
                | ChipKind::Crosslink
        )
    }

    pub fn has_ecp_plc(self) -> bool {
        matches!(
            self,
            ChipKind::Scm | ChipKind::Ecp | ChipKind::Xp | ChipKind::MachXo
        )
    }

    pub fn has_ecp2_plc(self) -> bool {
        matches!(
            self,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_)
        )
    }

    pub fn has_x1_bi(self) -> bool {
        matches!(
            self,
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_)
        )
    }

    pub fn has_distributed_sclk(self) -> bool {
        matches!(
            self,
            ChipKind::Ecp2
                | ChipKind::Ecp2M
                | ChipKind::Xp2
                | ChipKind::Ecp3(_)
                | ChipKind::MachXo2(_)
        )
    }

    pub fn has_distributed_sclk_ecp3(self) -> bool {
        matches!(self, ChipKind::Ecp3(_) | ChipKind::MachXo2(_))
    }
}

impl Display for ChipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipKind::Scm => write!(f, "scm"),
            ChipKind::Ecp => write!(f, "ecp"),
            ChipKind::Xp => write!(f, "xp"),
            ChipKind::MachXo => write!(f, "machxo"),
            ChipKind::Ecp2 => write!(f, "ecp2"),
            ChipKind::Ecp2M => write!(f, "ecp2m"),
            ChipKind::Xp2 => write!(f, "xp2"),
            ChipKind::Ecp3(Ecp3Kind::Ecp3) => write!(f, "ecp3"),
            ChipKind::Ecp3(Ecp3Kind::Ecp3A) => write!(f, "ecp3a"),
            ChipKind::MachXo2(MachXo2Kind::MachXo2) => write!(f, "machxo2"),
            ChipKind::MachXo2(MachXo2Kind::MachXo3L) => write!(f, "machxo3l"),
            ChipKind::MachXo2(MachXo2Kind::MachXo3Lfp) => write!(f, "machxo3lfp"),
            ChipKind::MachXo2(MachXo2Kind::MachXo3D) => write!(f, "machxo3d"),
            ChipKind::MachXo2(MachXo2Kind::MachNx) => write!(f, "machnx"),
            ChipKind::Ecp4 => write!(f, "ecp4"),
            ChipKind::Ecp5 => write!(f, "ecp5"),
            ChipKind::Crosslink => write!(f, "crosslink"),
            ChipKind::Nexus(NexusKind::SmallMipi) => write!(f, "nexus_small_mipi"),
            ChipKind::Nexus(NexusKind::SmallMipiPcie) => write!(f, "nexus_small_mipi_pcie"),
            ChipKind::Nexus(NexusKind::LargePcs) => write!(f, "nexus_large_pcs"),
            ChipKind::Nexus(NexusKind::SmallHalfIo) => write!(f, "nexus_small_half_io"),
            ChipKind::Nexus(NexusKind::SmallDenseIo) => write!(f, "nexus_small_dense_io"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Chip {
    pub kind: ChipKind,
    pub columns: EntityVec<ColId, Column>,
    pub rows: EntityVec<RowId, Row>,
    pub col_clk: ColId,
    pub row_clk: RowId,
    // column -> PCLK driver column
    // valid on: scm, ecp2, xp2, ecp5, crosslink
    // on ecp2/xp2, only the ranges are valid, the values carry no information and are set to range start/end
    pub cols_pclk: EntityRangeVec<ColId, ColId>,
    // row -> PCLK driver row
    // valid on: ecp3, machxo2, ecp4
    pub rows_pclk: EntityRangeVec<RowId, RowId>,
    pub special_loc: BTreeMap<SpecialLocKey, CellCoord>,
    pub special_io: BTreeMap<SpecialIoKey, BelCoord>,
    pub io_direct_plc: BTreeMap<BelCoord, (CellCoord, usize)>,
    pub extra_frames: EnumMap<DirH, usize>,
    pub double_frames: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum RowKind {
    Plc,
    Fplc,
    Io,
    Ebr,
    Dsp,
}

impl Display for RowKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RowKind::Plc => write!(f, "plc"),
            RowKind::Fplc => write!(f, "fplc"),
            RowKind::Io => write!(f, "io"),
            RowKind::Ebr => write!(f, "ebr"),
            RowKind::Dsp => write!(f, "dsp"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum IoGroupKind {
    None,
    Single,
    Double,
    DoubleA,
    DoubleB,
    DoubleC,
    DoubleDqs,
    DoubleDummy,
    Quad,
    QuadReverse,
    QuadDqs,
    QuadDqsDummy,
    QuadEbrDqs,
    EbrDqs,
    QuadI3c,
    Hex,
    HexReverse,
    Octal,
    Dozen,
    Serdes,
    Ebr,
    Mipi,
}

impl Display for IoGroupKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoGroupKind::None => write!(f, "none"),
            IoGroupKind::Single => write!(f, "single"),
            IoGroupKind::Double => write!(f, "double"),
            IoGroupKind::DoubleA => write!(f, "double_a"),
            IoGroupKind::DoubleB => write!(f, "double_b"),
            IoGroupKind::DoubleC => write!(f, "double_c"),
            IoGroupKind::DoubleDummy => write!(f, "double_dummy"),
            IoGroupKind::DoubleDqs => write!(f, "double_dqs"),
            IoGroupKind::Quad => write!(f, "quad"),
            IoGroupKind::QuadReverse => write!(f, "quad_reverse"),
            IoGroupKind::QuadDqs => write!(f, "quad_dqs"),
            IoGroupKind::QuadDqsDummy => write!(f, "quad_dqs_dummy"),
            IoGroupKind::QuadEbrDqs => write!(f, "quad_ebr_dqs"),
            IoGroupKind::EbrDqs => write!(f, "ebr_dqs"),
            IoGroupKind::QuadI3c => write!(f, "quad_i3c"),
            IoGroupKind::Hex => write!(f, "hex"),
            IoGroupKind::HexReverse => write!(f, "hex_reverse"),
            IoGroupKind::Octal => write!(f, "octal"),
            IoGroupKind::Dozen => write!(f, "dozen"),
            IoGroupKind::Serdes => write!(f, "serdes"),
            IoGroupKind::Ebr => write!(f, "ebr"),
            IoGroupKind::Mipi => write!(f, "mipi"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum IoKind {
    Dummy,
    Io,
    IoPll,
    Sio,
    Xsio,
    Dqs,
    SDqs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Column {
    pub io: EnumMap<DirV, IoGroupKind>,
    pub bank: EnumPartMap<DirV, u32>,
    pub eclk_tap: EnumMap<DirV, bool>,
    pub sclk_break: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Row {
    pub kind: RowKind,
    pub io: EnumMap<DirH, IoGroupKind>,
    pub bank: EnumPartMap<DirH, u32>,
    pub sclk_break: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub struct PllLoc {
    pub quad: DirHV,
    pub idx: u8,
}

impl PllLoc {
    pub fn new(quad: DirHV, idx: u8) -> Self {
        Self { quad, idx }
    }
}

impl Display for PllLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.quad, self.idx)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum PllPad {
    PllIn0,
    PllIn1,
    PllFb,
    DllIn0,
    DllIn1,
    DllIn2,
    DllIn3,
    DllFb,
}

impl Display for PllPad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PllPad::PllIn0 => write!(f, "PLL_IN0"),
            PllPad::PllIn1 => write!(f, "PLL_IN1"),
            PllPad::PllFb => write!(f, "PLL_FB"),
            PllPad::DllIn0 => write!(f, "DLL_IN0"),
            PllPad::DllIn1 => write!(f, "DLL_IN1"),
            PllPad::DllIn2 => write!(f, "DLL_IN2"),
            PllPad::DllIn3 => write!(f, "DLL_IN3"),
            PllPad::DllFb => write!(f, "DLL_FB"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum SpecialIoKey {
    Clock(Dir, u8),
    Pll(PllPad, PllLoc),
    // MachXO2
    DqsE(u8),
    Vref1(u32),
    Vref2(u32),
    DiffR(u32),
    Gsr,
    TsAll,
    WriteN,
    ReadN,
    CsN,
    Cs1N,
    Cs1,
    D(u8),
    DP(u8),
    A(u8),
    Dout,
    Qout,
    Di,
    Busy,
    MClk,
    SleepN,
    MpiClk,
    MpiAckN,
    MpiRetryN,
    MpiTeaN,
    Hdc,
    Ldc,
    ExtDoneI,
    ExtDoneO,
    ExtClkI(u8),
    ExtClkO(u8),
    // XP2 stuff
    InitB,
    Cclk,
    SpiCCsB,
    SpiPCsB,
    M1,
    Done,
    ProgB,
    // MachXO2
    Tck,
    Tms,
    Tdi,
    Tdo,
    JtagEn,
    // Crosslink
    PmuWakeupN,
    MipiClk(DirH),
    // Nexus
    SpiCD(u8),
    CdrRx(u8),
    AdcCh(u8),
    AdcCompI(u8),
}

impl Display for SpecialIoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialIoKey::Clock(dir, i) => write!(f, "CLOCK_{dir}{i}"),
            SpecialIoKey::Pll(pad, loc) => write!(f, "{pad}_{loc}"),
            SpecialIoKey::DqsE(i) => write!(f, "DQS_E{i}"),
            SpecialIoKey::Vref1(bank) => write!(f, "VREF1_{bank}"),
            SpecialIoKey::Vref2(bank) => write!(f, "VREF2_{bank}"),
            SpecialIoKey::DiffR(bank) => write!(f, "DIFFR_{bank}"),
            SpecialIoKey::Gsr => write!(f, "GSR"),
            SpecialIoKey::TsAll => write!(f, "TSALL"),
            SpecialIoKey::WriteN => write!(f, "WRITE_N"),
            SpecialIoKey::ReadN => write!(f, "READ_N"),
            SpecialIoKey::CsN => write!(f, "CS_N"),
            SpecialIoKey::Cs1N => write!(f, "CS1_N"),
            SpecialIoKey::Cs1 => write!(f, "CS1"),
            SpecialIoKey::D(i) => write!(f, "D{i}"),
            SpecialIoKey::DP(i) => write!(f, "DP{i}"),
            SpecialIoKey::A(i) => write!(f, "A{i}"),
            SpecialIoKey::Dout => write!(f, "DOUT"),
            SpecialIoKey::Qout => write!(f, "QOUT"),
            SpecialIoKey::Di => write!(f, "DI"),
            SpecialIoKey::Busy => write!(f, "BUSY"),
            SpecialIoKey::MClk => write!(f, "MCLK"),
            SpecialIoKey::SleepN => write!(f, "SLEEP_N"),
            SpecialIoKey::MpiClk => write!(f, "MPI_CLK"),
            SpecialIoKey::MpiAckN => write!(f, "MPI_ACK_N"),
            SpecialIoKey::MpiRetryN => write!(f, "MPI_RETRY_N"),
            SpecialIoKey::MpiTeaN => write!(f, "MPI_TEA_N"),
            SpecialIoKey::Hdc => write!(f, "HDC"),
            SpecialIoKey::Ldc => write!(f, "LDC"),
            SpecialIoKey::ExtDoneI => write!(f, "EXT_DONE_I"),
            SpecialIoKey::ExtDoneO => write!(f, "EXT_DONE_O"),
            SpecialIoKey::ExtClkI(idx) => write!(f, "EXT_CLK_I{idx}"),
            SpecialIoKey::ExtClkO(idx) => write!(f, "EXT_CLK_O{idx}"),
            SpecialIoKey::InitB => write!(f, "INIT_B"),
            SpecialIoKey::Cclk => write!(f, "CCLK"),
            SpecialIoKey::SpiCCsB => write!(f, "SPI_C_CS_B"),
            SpecialIoKey::SpiCD(idx) => write!(f, "SPI_C_D{idx}"),
            SpecialIoKey::SpiPCsB => write!(f, "SPI_P_CS_B"),
            SpecialIoKey::M1 => write!(f, "M1"),
            SpecialIoKey::Done => write!(f, "DONE"),
            SpecialIoKey::ProgB => write!(f, "PROG_B"),
            SpecialIoKey::Tck => write!(f, "TCK"),
            SpecialIoKey::Tms => write!(f, "TMS"),
            SpecialIoKey::Tdi => write!(f, "TDI"),
            SpecialIoKey::Tdo => write!(f, "TDO"),
            SpecialIoKey::JtagEn => write!(f, "JTAG_EN"),
            SpecialIoKey::PmuWakeupN => write!(f, "PMU_WAKEUP_N"),
            SpecialIoKey::MipiClk(side) => write!(f, "MIPI_CLK_{side}"),
            SpecialIoKey::CdrRx(i) => write!(f, "CDR{i}_RX"),
            SpecialIoKey::AdcCh(i) => write!(f, "ADC_CH{i}"),
            SpecialIoKey::AdcCompI(i) => write!(f, "ADC_COMP{i}I"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum SpecialLocKey {
    Pll(PllLoc),
    Ebr(u8),
    PclkIn(Dir, u8),
    PclkInMid(u8),
    SclkIn(Dir, u8),
    Config,
    ConfigBits,
    Bc(u32),
    Osc,
    DqsDll(Dir),
    SerdesSingle,
    SerdesDouble,
    SerdesTriple,
    DdrDll(DirHV),
    ClkQuarter(DirV),
    Pmu,
    Cdr,
    DllDel(Dir, u8),
    Adc,
    Mipi,
    Pcs(u8),
    PcsRefmux,
    PcsRefclk,
    Pcie,
    Lram(Dir, u8),
    Alu,
    IVref(u32),
    PvtCal33,
    PvtCal18,
}

impl Display for SpecialLocKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpecialLocKey::Pll(dir) => write!(f, "PLL_{dir}"),
            SpecialLocKey::Ebr(idx) => write!(f, "EBR{idx}"),
            SpecialLocKey::Config => write!(f, "CONFIG"),
            SpecialLocKey::ConfigBits => write!(f, "CONFIG_BITS"),
            SpecialLocKey::Bc(bank) => write!(f, "BC{bank}"),
            SpecialLocKey::Osc => write!(f, "OSC"),
            SpecialLocKey::PclkIn(dir, idx) => write!(f, "PCLK_IN_{dir}{idx}"),
            SpecialLocKey::PclkInMid(idx) => write!(f, "PCLK_IN_M{idx}"),
            SpecialLocKey::SclkIn(dir, idx) => write!(f, "SCLK_IN_{dir}{idx}"),
            SpecialLocKey::DqsDll(dir) => write!(f, "DQSDLL_{dir}"),
            SpecialLocKey::SerdesSingle => write!(f, "SERDES_SINGLE"),
            SpecialLocKey::SerdesDouble => write!(f, "SERDES_DOUBLE"),
            SpecialLocKey::SerdesTriple => write!(f, "SERDES_TRIPLE"),
            SpecialLocKey::DdrDll(dir) => write!(f, "DDRDLL_{dir}"),
            SpecialLocKey::ClkQuarter(dir) => write!(f, "CLK_QUARTER_{dir}"),
            SpecialLocKey::Pmu => write!(f, "PMU"),
            SpecialLocKey::Cdr => write!(f, "CDR"),
            SpecialLocKey::DllDel(dir, idx) => write!(f, "DLLDEL_{dir}{idx}"),
            SpecialLocKey::Adc => write!(f, "ADC"),
            SpecialLocKey::Mipi => write!(f, "MIPI"),
            SpecialLocKey::Pcs(idx) => write!(f, "PCS{idx}"),
            SpecialLocKey::PcsRefmux => write!(f, "PCS_REFMUX"),
            SpecialLocKey::PcsRefclk => write!(f, "PCS_REFCLK"),
            SpecialLocKey::Pcie => write!(f, "PCIE"),
            SpecialLocKey::Lram(dir, idx) => write!(f, "LRAM_{dir}{idx}"),
            SpecialLocKey::Alu => write!(f, "ALU"),
            SpecialLocKey::IVref(bank) => write!(f, "IVREF{bank}"),
            SpecialLocKey::PvtCal33 => write!(f, "PVTCAL33"),
            SpecialLocKey::PvtCal18 => write!(f, "PVTCAL18"),
        }
    }
}

impl Chip {
    // always single die
    pub const DIE: DieId = DieId::from_idx_const(0);

    pub fn col_w(&self) -> ColId {
        self.columns.first_id().unwrap()
    }

    pub fn col_e(&self) -> ColId {
        self.columns.last_id().unwrap()
    }

    pub fn col_edge(&self, edge: DirH) -> ColId {
        match edge {
            DirH::W => self.col_w(),
            DirH::E => self.col_e(),
        }
    }

    pub fn is_col_io(&self, col: ColId) -> bool {
        col == self.col_w() || col == self.col_e()
    }

    pub fn row_s(&self) -> RowId {
        self.rows.first_id().unwrap()
    }

    pub fn row_n(&self) -> RowId {
        self.rows.last_id().unwrap()
    }

    pub fn row_edge(&self, edge: DirV) -> RowId {
        match edge {
            DirV::S => self.row_s(),
            DirV::N => self.row_n(),
        }
    }

    pub fn is_row_io(&self, row: RowId) -> bool {
        row == self.row_s() || row == self.row_n()
    }

    pub fn try_get_io_cell(&self, key: SpecialIoKey) -> Option<CellCoord> {
        Some(self.special_io.get(&key)?.cell)
    }

    pub fn get_io_cell(&self, key: SpecialIoKey) -> CellCoord {
        self.special_io[&key].cell
    }

    pub fn try_get_spec_cell(&self, key: SpecialLocKey) -> Option<CellCoord> {
        self.special_loc.get(&key).copied()
    }

    pub fn get_io_loc(&self, io: EdgeIoCoord) -> BelCoord {
        if matches!(self.kind, ChipKind::Ecp3(_) | ChipKind::Ecp4) && io.iob().to_idx() >= 4 {
            let (col, row, iob) = match io {
                EdgeIoCoord::W(row, iob) => (self.col_w() + 1, row, iob),
                EdgeIoCoord::E(row, iob) => (self.col_e() - 1, row, iob),
                _ => unreachable!(),
            };
            let slot = bslots::IO[iob.to_idx() - 4];
            Self::DIE.cell(col, row).bel(slot)
        } else {
            let (col, row, iob) = match io {
                EdgeIoCoord::N(col, iob) => (col, self.row_n(), iob),
                EdgeIoCoord::E(row, iob) => (self.col_e(), row, iob),
                EdgeIoCoord::S(col, iob) => (col, self.row_s(), iob),
                EdgeIoCoord::W(row, iob) => (self.col_w(), row, iob),
            };
            let slot = bslots::IO[iob.to_idx()];
            Self::DIE.cell(col, row).bel(slot)
        }
    }

    pub fn get_io_edge(&self, cell: CellCoord) -> Dir {
        if cell.row == self.row_s() && matches!(self.kind, ChipKind::Nexus(_)) {
            Dir::S
        } else if cell.row == self.row_n() && matches!(self.kind, ChipKind::Nexus(_)) {
            Dir::N
        } else if cell.col == self.col_w() && self.kind != ChipKind::Crosslink {
            Dir::W
        } else if cell.col == self.col_e() && self.kind != ChipKind::Crosslink {
            Dir::E
        } else if cell.row == self.row_s() {
            Dir::S
        } else if cell.row == self.row_n() {
            Dir::N
        } else if matches!(self.kind, ChipKind::Ecp3(_) | ChipKind::Ecp4) {
            if cell.col == self.col_w() + 1 {
                Dir::W
            } else if cell.col == self.col_e() - 1 {
                Dir::E
            } else {
                unreachable!()
            }
        } else {
            unreachable!()
        }
    }

    pub fn get_io_crd(&self, bcrd: BelCoord) -> EdgeIoCoord {
        let iob = TileIobId::from_idx(bslots::IO.index_of(bcrd.slot).unwrap());
        match self.get_io_edge(bcrd.cell) {
            Dir::W => {
                if bcrd.col == self.col_w() + 1 {
                    let iob = TileIobId::from_idx(bslots::IO.index_of(bcrd.slot).unwrap() + 4);
                    EdgeIoCoord::W(bcrd.row, iob)
                } else {
                    EdgeIoCoord::W(bcrd.row, iob)
                }
            }
            Dir::E => {
                if bcrd.col == self.col_e() - 1 {
                    let iob = TileIobId::from_idx(bslots::IO.index_of(bcrd.slot).unwrap() + 4);
                    EdgeIoCoord::E(bcrd.row, iob)
                } else {
                    EdgeIoCoord::E(bcrd.row, iob)
                }
            }
            Dir::S => EdgeIoCoord::S(bcrd.col, iob),
            Dir::N => EdgeIoCoord::N(bcrd.col, iob),
        }
    }

    pub fn get_io_bank(&self, cell: CellCoord) -> u32 {
        match self.get_io_edge(cell) {
            Dir::H(h) => self.rows[cell.row].bank[h],
            Dir::V(v) => self.columns[cell.col].bank[v],
        }
    }

    pub fn get_io_kind(&self, bcrd: BelCoord) -> IoKind {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        match self.kind {
            ChipKind::Scm => IoKind::Io,
            ChipKind::MachXo => IoKind::Io,
            ChipKind::Ecp | ChipKind::Xp | ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                match self.get_io_edge(bcrd.cell) {
                    Dir::H(h) => match (self.rows[bcrd.row].io[h], idx) {
                        (IoGroupKind::DoubleDqs, 0) => IoKind::Dqs,
                        (IoGroupKind::DoubleA, 1) => IoKind::Dummy,
                        (IoGroupKind::DoubleB, 0) => IoKind::Dummy,
                        (IoGroupKind::DoubleDummy, _) => IoKind::Dummy,
                        _ => IoKind::Io,
                    },
                    Dir::V(v) => match (self.columns[bcrd.col].io[v], idx) {
                        (IoGroupKind::DoubleDqs, 0) => IoKind::Dqs,
                        (IoGroupKind::DoubleA, 1) => IoKind::Dummy,
                        (IoGroupKind::DoubleB, 0) => IoKind::Dummy,
                        (IoGroupKind::DoubleDummy, _) => IoKind::Dummy,
                        _ => IoKind::Io,
                    },
                }
            }
            ChipKind::Ecp3(_) => match self.get_io_edge(bcrd.cell) {
                Dir::W => {
                    if bcrd.col == self.col_w() {
                        match (self.rows[bcrd.row].io[DirH::W], idx) {
                            (IoGroupKind::QuadDqs, 2) => IoKind::Dqs,
                            (IoGroupKind::QuadDqs, 3) => IoKind::Xsio,
                            _ => IoKind::Io,
                        }
                    } else {
                        IoKind::IoPll
                    }
                }
                Dir::E => {
                    if bcrd.col == self.col_e() {
                        match (
                            self.rows[bcrd.row].io[DirH::E],
                            self.rows[bcrd.row].bank[DirH::E],
                            idx,
                        ) {
                            (_, 8, _) => IoKind::Xsio,
                            (IoGroupKind::QuadDqs, _, 2) => IoKind::Dqs,
                            (IoGroupKind::QuadDqs, _, 3) => IoKind::Xsio,
                            _ => IoKind::Io,
                        }
                    } else {
                        IoKind::IoPll
                    }
                }

                Dir::S => IoKind::Xsio,
                Dir::N => match (
                    self.columns[bcrd.col].io[DirV::N],
                    self.columns[bcrd.col].bank[DirV::N],
                    idx,
                ) {
                    (_, 8, _) => IoKind::Xsio,
                    (IoGroupKind::QuadDqs, _, 2) => IoKind::SDqs,
                    (IoGroupKind::QuadDqs, _, 3) => IoKind::Xsio,
                    _ => IoKind::Sio,
                },
            },
            ChipKind::MachXo2(_) => IoKind::Io,
            ChipKind::Ecp4 => IoKind::Io,
            ChipKind::Ecp5 => match self.get_io_edge(bcrd.cell) {
                Dir::H(_) => IoKind::Io,
                Dir::V(_) => IoKind::Sio,
            },
            ChipKind::Crosslink => {
                if self.columns[bcrd.col].bank[DirV::S] == 0 {
                    IoKind::Sio
                } else {
                    IoKind::Io
                }
            }
            ChipKind::Nexus(_) => IoKind::Io,
        }
    }

    pub fn col_sclk_idx(&self, col: ColId) -> usize {
        assert!(self.kind.has_distributed_sclk());
        if self.kind.has_distributed_sclk_ecp3() {
            (col - self.col_clk).rem_euclid(4) as usize
        } else {
            match col.to_idx() % 4 {
                0 => 0,
                1 => 3,
                2 => 2,
                3 => 1,
                _ => unreachable!(),
            }
        }
    }

    pub fn clk_to_rclk(&self, col: ColId, idx: usize) -> usize {
        let offset = self.col_sclk_idx(col);
        assert!(idx < 8);
        (idx + 4 - offset) % 4 + idx / 4 * 4
    }

    pub fn rclk_to_clk(&self, col: ColId, idx: usize) -> usize {
        let offset = self.col_sclk_idx(col);
        assert!(idx < 8);
        (idx + offset) % 4 + idx / 4 * 4
    }

    pub fn try_bel_pll(&self, quad: DirHV, idx: usize) -> Option<BelCoord> {
        if matches!(self.kind, ChipKind::Scm | ChipKind::Ecp4) {
            if idx < 2 {
                let cell = self.try_get_spec_cell(SpecialLocKey::Pll(PllLoc::new(quad, 0)))?;
                Some(cell.bel(bslots::PLL[idx]))
            } else {
                None
            }
        } else {
            let cell = self.try_get_spec_cell(SpecialLocKey::Pll(PllLoc::new(quad, idx as u8)))?;
            Some(cell.bel(bslots::PLL[0]))
        }
    }

    pub fn bel_pll(&self, quad: DirHV, idx: usize) -> BelCoord {
        self.try_bel_pll(quad, idx).unwrap()
    }

    pub fn plls(&self) -> Vec<(PllLoc, BelCoord)> {
        let mut res = vec![];
        for (&loc, &cell) in &self.special_loc {
            let SpecialLocKey::Pll(loc) = loc else {
                continue;
            };
            res.push((loc, cell.bel(bslots::PLL[0])));
            if matches!(self.kind, ChipKind::Scm | ChipKind::Ecp4) {
                res.push((PllLoc { idx: 1, ..loc }, cell.bel(bslots::PLL[1])));
            }
        }
        res
    }

    pub fn bel_cibtest_sel(&self) -> BelCoord {
        assert_eq!(self.kind, ChipKind::MachXo);
        assert!(self.special_loc.contains_key(&SpecialLocKey::Ebr(0)));
        Self::DIE
            .cell(self.col_w(), self.row_n() - 1)
            .bel(bslots::CIBTEST_SEL)
    }

    pub fn clk_root(&self) -> CellCoord {
        Self::DIE.cell(self.col_clk, self.row_clk)
    }

    pub fn bel_clk_root(&self) -> BelCoord {
        self.clk_root().bel(bslots::CLK_INT)
    }

    pub fn clk_emid(&self, edge: impl Into<Dir>) -> CellCoord {
        let edge = edge.into();
        if matches!(self.kind, ChipKind::Ecp3(_)) && edge == Dir::S {
            self.clk_root().with_row(self.row_serdes(DirV::S))
        } else {
            match edge {
                Dir::H(h) => self.clk_root().with_col(self.col_edge(h)),
                Dir::V(v) => self.clk_root().with_row(self.row_edge(v)),
            }
        }
    }

    pub fn bel_dqsdll(&self, io: CellCoord) -> BelCoord {
        match self.kind {
            ChipKind::Ecp | ChipKind::Xp => self.bel_dqsdll_ecp(io.row.side_of(self.row_clk)),
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 | ChipKind::Ecp3(_) => {
                self.bel_dqsdll_ecp2(io.col.side_of(self.col_clk))
            }
            ChipKind::MachXo => unreachable!(),
            ChipKind::MachXo2(_) => unreachable!(),
            ChipKind::Ecp4 => unreachable!(),
            ChipKind::Ecp5 => unreachable!(),
            ChipKind::Crosslink => unreachable!(),
            ChipKind::Scm => unreachable!(),
            ChipKind::Nexus(_) => unreachable!(),
        }
    }

    pub fn bel_dqsdll_ecp(&self, edge: DirV) -> BelCoord {
        assert!(matches!(self.kind, ChipKind::Ecp | ChipKind::Xp));
        match edge {
            DirV::S => Self::DIE
                .cell(self.col_clk - 2, self.row_s())
                .bel(bslots::DQSDLL),
            DirV::N => Self::DIE
                .cell(self.col_clk + 1, self.row_n())
                .bel(bslots::DQSDLL),
        }
    }

    pub fn bel_dqsdll_ecp2(&self, edge: DirH) -> BelCoord {
        match self.kind {
            ChipKind::Ecp2 | ChipKind::Ecp2M => self.bel_pll(edge & DirV::S, 0).bel(bslots::DQSDLL),
            ChipKind::Xp2 => self.clk_emid(edge).bel(bslots::DQSDLL),
            ChipKind::Ecp3(_) => self.clk_emid(edge).step(!edge, 1).bel(bslots::DQSDLL),
            _ => unreachable!(),
        }
    }

    pub fn bel_eclksync(&self, edge: Dir, idx: usize) -> BelCoord {
        match self.kind {
            ChipKind::Ecp3(_) => match edge {
                Dir::H(_) => self.clk_emid(edge).bel(bslots::ECLKSYNC[idx]),
                Dir::S => unreachable!(),
                Dir::N => self
                    .clk_emid(edge)
                    .step(DirH::W, 1)
                    .bel(bslots::ECLKSYNC[idx]),
            },
            ChipKind::MachXo2(_) => match edge {
                Dir::H(_) => unreachable!(),
                Dir::V(_) => self
                    .clk_emid(edge)
                    .step(DirH::W, 1)
                    .bel(bslots::ECLKSYNC[idx]),
            },
            _ => unreachable!(),
        }
    }

    pub fn bel_eclksync_bank(&self, bank: u32, idx: usize) -> BelCoord {
        match self.kind {
            ChipKind::Ecp4 => match bank {
                0..4 => self
                    .clk_emid(Dir::N)
                    .bel(bslots::ECLKSYNC[(bank as usize) * 4 + idx]),
                4..6 => self
                    .clk_emid(Dir::E)
                    .bel(bslots::ECLKSYNC[(5 - bank as usize) * 4 + idx]),
                6..8 => self
                    .clk_emid(Dir::W)
                    .bel(bslots::ECLKSYNC[(bank as usize - 6) * 4 + idx]),
                _ => unreachable!(),
            },
            ChipKind::Ecp5 => match bank {
                2..4 => self
                    .clk_emid(Dir::E)
                    .bel(bslots::ECLKSYNC[(3 - bank as usize) * 2 + idx]),
                6..8 => self
                    .clk_emid(Dir::W)
                    .bel(bslots::ECLKSYNC[(bank as usize - 6) * 2 + idx]),
                _ => unreachable!(),
            },
            ChipKind::Crosslink => self
                .clk_emid(Dir::S)
                .bel(bslots::ECLKSYNC[(2 - bank as usize) * 2 + idx]),
            _ => unreachable!(),
        }
    }

    pub fn bel_clock_dlldel(&self, edge: Dir, idx: usize) -> BelCoord {
        assert_eq!(self.kind, ChipKind::Ecp4);
        self.clk_emid(edge).bel(bslots::DLLDEL[idx])
    }

    pub fn row_serdes(&self, edge: DirV) -> RowId {
        match (self.kind, edge) {
            (ChipKind::Scm, DirV::N) => self.row_n() - 12,
            (ChipKind::Ecp2M, DirV::S) => self.row_s() + 7,
            (ChipKind::Ecp2M, DirV::N) => self.row_n() - 7,
            (ChipKind::Ecp3(_), DirV::S) => self.row_s() + 9,
            (ChipKind::Ecp4 | ChipKind::Ecp5, DirV::S) => self.row_s(),
            _ => unreachable!(),
        }
    }

    pub fn try_bel_serdes_ecp2(&self, hv: DirHV) -> Option<BelCoord> {
        let col = match hv.h {
            DirH::W => self.col_w() + 1,
            DirH::E => self.col_e() - 27,
        };
        let row = self.row_serdes(hv.v);
        let cell = Self::DIE.cell(col, row);
        if self.columns[col].io[hv.v] != IoGroupKind::Serdes {
            None
        } else {
            Some(cell.bel(bslots::SERDES))
        }
    }

    pub fn bel_serdes(&self, edge: DirV, col: ColId) -> BelCoord {
        let row = self.row_serdes(edge);
        Self::DIE.cell(col, row).bel(bslots::SERDES)
    }

    pub fn bel_mipi(&self, col: ColId) -> BelCoord {
        match self.kind {
            ChipKind::Crosslink => Self::DIE.cell(col, self.row_n()).bel(bslots::MIPI[0]),
            ChipKind::Nexus(_) => {
                let cell = self.special_loc[&SpecialLocKey::Mipi];
                let idx = col - cell.col;
                cell.bel(bslots::MIPI[idx as usize])
            }
            _ => unreachable!(),
        }
    }

    pub fn btile_term_width(&self, _col: ColId) -> usize {
        match self.kind {
            ChipKind::Scm => todo!(),
            ChipKind::Ecp | ChipKind::Xp => 2,
            ChipKind::MachXo => 0,
            ChipKind::Ecp2 => todo!(),
            ChipKind::Ecp2M => todo!(),
            ChipKind::Xp2 => todo!(),
            ChipKind::Ecp3(_) => todo!(),
            ChipKind::MachXo2(_) => todo!(),
            ChipKind::Ecp4 => todo!(),
            ChipKind::Ecp5 => todo!(),
            ChipKind::Crosslink => todo!(),
            ChipKind::Nexus(_) => todo!(),
        }
    }

    pub fn btile_clk_width(&self) -> usize {
        match self.kind {
            ChipKind::Scm => todo!(),
            ChipKind::Ecp | ChipKind::Xp => 6,
            ChipKind::MachXo => 1,
            ChipKind::Ecp2 => todo!(),
            ChipKind::Ecp2M => todo!(),
            ChipKind::Xp2 => todo!(),
            ChipKind::Ecp3(_) => todo!(),
            ChipKind::MachXo2(_) => todo!(),
            ChipKind::Ecp4 => todo!(),
            ChipKind::Ecp5 => todo!(),
            ChipKind::Crosslink => todo!(),
            ChipKind::Nexus(_) => todo!(),
        }
    }

    pub fn btile_width(&self, col: ColId) -> usize {
        match self.kind {
            ChipKind::Scm => todo!(),
            ChipKind::Ecp | ChipKind::Xp => 64,
            ChipKind::MachXo => {
                let has_ebr = self.special_loc.contains_key(&SpecialLocKey::Ebr(0));
                if col == self.col_w() {
                    if has_ebr { 135 } else { 19 }
                } else if col == self.col_e() {
                    if has_ebr { 20 } else { 19 }
                } else {
                    64
                }
            }
            ChipKind::Ecp2 => todo!(),
            ChipKind::Ecp2M => todo!(),
            ChipKind::Xp2 => todo!(),
            ChipKind::Ecp3(_) => todo!(),
            ChipKind::MachXo2(_) => todo!(),
            ChipKind::Ecp4 => todo!(),
            ChipKind::Ecp5 => todo!(),
            ChipKind::Crosslink => todo!(),
            ChipKind::Nexus(_) => todo!(),
        }
    }

    pub fn btile_height(&self, row: RowId) -> usize {
        let rd = &self.rows[row];
        match self.kind {
            ChipKind::Scm => todo!(),
            ChipKind::Ecp | ChipKind::Xp => match rd.kind {
                RowKind::Plc | RowKind::Fplc => 22,
                RowKind::Io => 21,
                RowKind::Dsp => 20,
                RowKind::Ebr => 20,
            },
            ChipKind::MachXo => {
                let has_ebr = self.special_loc.contains_key(&SpecialLocKey::Ebr(0));
                match rd.kind {
                    RowKind::Plc | RowKind::Fplc => 22,
                    RowKind::Io => {
                        if has_ebr {
                            9
                        } else {
                            8
                        }
                    }
                    _ => unreachable!(),
                }
            }
            ChipKind::Ecp2 => todo!(),
            ChipKind::Ecp2M => todo!(),
            ChipKind::Xp2 => todo!(),
            ChipKind::Ecp3(_) => todo!(),
            ChipKind::MachXo2(_) => todo!(),
            ChipKind::Ecp4 => todo!(),
            ChipKind::Ecp5 => todo!(),
            ChipKind::Crosslink => todo!(),
            ChipKind::Nexus(_) => todo!(),
        }
    }
}

impl Chip {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        writeln!(o, "\tkind {};", self.kind)?;
        writeln!(o, "\tcolumns {{")?;
        for (col, cd) in &self.columns {
            if self.col_clk == col {
                writeln!(o, "\t\t// clock")?;
            }
            if !self.cols_pclk.is_empty()
                && self.cols_pclk.range(col).start() == col
                && col != self.col_w()
            {
                writeln!(o, "\t\t// PCLK break")?;
            }
            write!(o, "\t\t")?;
            let mut first = true;
            if cd.io[DirV::S] != IoGroupKind::None {
                first = false;
                write!(
                    o,
                    "io_s[{bank_s}, {io_s}]",
                    bank_s = cd.bank[DirV::S],
                    io_s = cd.io[DirV::S]
                )?;
            }
            if cd.io[DirV::N] != IoGroupKind::None {
                if first {
                    first = false;
                } else {
                    write!(o, " + ")?;
                }
                write!(
                    o,
                    "io_n[{bank_n}, {io_n}]",
                    bank_n = cd.bank[DirV::N],
                    io_n = cd.io[DirV::N]
                )?;
            }
            if cd.eclk_tap[DirV::S] {
                if first {
                    first = false;
                } else {
                    write!(o, " + ")?;
                }
                write!(o, "eclk_tap_s")?;
            }
            if cd.eclk_tap[DirV::N] {
                if first {
                    first = false;
                } else {
                    write!(o, " + ")?;
                }
                write!(o, "eclk_tap_n")?;
            }
            if cd.sclk_break {
                if first {
                    first = false;
                } else {
                    write!(o, " + ")?;
                }
                write!(o, "sclk_break")?;
            }
            if first {
                write!(o, "null")?;
            }
            write!(o, ", // {col}")?;
            if !self.cols_pclk.is_empty()
                && (self.cols_pclk[col] == col
                    || (col != self.col_w() && self.cols_pclk[col - 1] == col))
            {
                write!(o, " PCLK_DRIVE")?;
            }
            writeln!(o)?;
        }
        writeln!(o, "\t}}")?;
        writeln!(o, "\tcol_clk {};", self.col_clk)?;
        for (cols, &col_pclk) in &self.cols_pclk {
            writeln!(o, "\tcol_pclk {cols} = {col_pclk};")?;
        }
        writeln!(o, "\trows {{")?;
        for (row, rd) in &self.rows {
            if self.row_clk == row {
                writeln!(o, "\t\t// clock")?;
            }
            if !self.rows_pclk.is_empty()
                && self.rows_pclk.range(row).start() == row
                && row != self.row_s()
            {
                writeln!(o, "\t\t// PCLK break")?;
            }
            write!(o, "\t\t{kind}", kind = rd.kind)?;
            if rd.io[DirH::W] != IoGroupKind::None {
                write!(
                    o,
                    " + io_w[{bank_w}, {io_w}]",
                    bank_w = rd.bank[DirH::W],
                    io_w = rd.io[DirH::W]
                )?;
            }
            if rd.io[DirH::E] != IoGroupKind::None {
                write!(
                    o,
                    " + io_e[{bank_e}, {io_e}]",
                    bank_e = rd.bank[DirH::E],
                    io_e = rd.io[DirH::E]
                )?;
            }
            if rd.sclk_break {
                write!(o, " + sclk_break")?;
            }
            write!(o, ", // {row}")?;
            if !self.rows_pclk.is_empty()
                && (self.rows_pclk[row] == row
                    || (row != self.row_s() && self.rows_pclk[row - 1] == row))
            {
                write!(o, " PCLK_DRIVE")?;
            }
            writeln!(o)?;
        }
        writeln!(o, "\t}}")?;
        writeln!(o, "\trow_clk {};", self.row_clk)?;
        for (rows, &row_pclk) in &self.rows_pclk {
            writeln!(o, "\trow_pclk {rows} = {row_pclk};")?;
        }

        for (k, v) in &self.special_loc {
            writeln!(o, "\tspecial_loc {k} = {v};")?;
        }
        for (k, io) in &self.special_io {
            writeln!(o, "\tspecial_io {k} = {io};", io = io.to_string(db))?;
        }
        for (io, (cell, lut)) in &self.io_direct_plc {
            writeln!(o, "\tio_direct {io} = {cell}_{lut};", io = io.to_string(db))?;
        }
        if self.extra_frames.values().any(|&v| v != 0) {
            writeln!(
                o,
                "\textra_frames {}, {};",
                self.extra_frames[DirH::W],
                self.extra_frames[DirH::E]
            )?;
        }
        if self.double_frames {
            writeln!(o, "\tdouble_frames;")?;
        }
        Ok(())
    }
}
