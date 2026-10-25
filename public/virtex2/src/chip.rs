use bincode::{Decode, Encode};
use prjcombine_entity::{EntityId, EntityRange, EntityRangeVec, EntityVec};
use prjcombine_interconnect::db::{BelSlotId, CellSlotId, IntDb};
use prjcombine_interconnect::dir::{Dir, DirH, DirHV, DirV};
use prjcombine_interconnect::grid::{
    BelCoord, BelPadCoord, CellCoord, ColId, DieId, DieIdExt, RowId, TileCoord,
};
use prjcombine_types::Polarity;
use prjcombine_types::enumerable::EnumMap;
use std::collections::BTreeMap;

use crate::defs::{bcls, bslots, tslots};
use crate::iob::{
    IobKind, IobTileData, get_iob_data_e, get_iob_data_n, get_iob_data_s, get_iob_data_w,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum ChipKind {
    Virtex2,
    Virtex2P,
    Virtex2PX,
    Spartan3,
    FpgaCore,
    Spartan3E,
    Spartan3A,
    Spartan3ADsp,
}

impl std::fmt::Display for ChipKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChipKind::Virtex2 => write!(f, "virtex2"),
            ChipKind::Virtex2P => write!(f, "virtex2p"),
            ChipKind::Virtex2PX => write!(f, "virtex2px"),
            ChipKind::Spartan3 => write!(f, "spartan3"),
            ChipKind::Spartan3E => write!(f, "spartan3e"),
            ChipKind::Spartan3A => write!(f, "spartan3a"),
            ChipKind::Spartan3ADsp => write!(f, "spartan3adsp"),
            ChipKind::FpgaCore => write!(f, "fpgacore"),
        }
    }
}

impl ChipKind {
    pub fn is_virtex2(self) -> bool {
        matches!(self, Self::Virtex2 | Self::Virtex2P | Self::Virtex2PX)
    }
    pub fn is_virtex2p(self) -> bool {
        matches!(self, Self::Virtex2P | Self::Virtex2PX)
    }
    pub fn is_spartan3ea(self) -> bool {
        matches!(self, Self::Spartan3E | Self::Spartan3A | Self::Spartan3ADsp)
    }
    pub fn is_spartan3a(self) -> bool {
        matches!(self, Self::Spartan3A | Self::Spartan3ADsp)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Chip {
    pub kind: ChipKind,
    pub columns: EntityVec<ColId, Column>,
    pub col_clk: ColId,
    // For Spartan 3* other than 3s50a
    pub cols_clkv: Option<EnumMap<DirH, ColId>>,
    // column -> GT banks
    pub cols_gt: BTreeMap<ColId, EnumMap<DirV, u32>>,
    pub rows: EntityVec<RowId, RowIoKind>,
    // For Spartan 3E: range of rows containing RAMs
    pub rows_ram: Option<EntityRange<RowId>>,
    // row -> hclk row
    pub rows_hclk: EntityRangeVec<RowId, RowId>,
    // For Virtex 2
    pub row_pci: Option<RowId>,
    pub holes_ppc: Vec<(ColId, RowId)>,
    // For Spartan 3E, 3A*
    pub dcms: Option<Dcms>,
    pub has_ll: bool,
    pub has_small_int: bool,
    pub cfg_io: BTreeMap<SharedCfgPad, BelPadCoord>,
    pub dci_io: BTreeMap<u32, EnumMap<Polarity, BelPadCoord>>,
    pub dci_io_alt: BTreeMap<u32, EnumMap<Polarity, BelPadCoord>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Column {
    pub kind: ColumnKind,
    pub io: ColumnIoKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum ColumnKind {
    Io,
    Clb,
    Bram,
    BramCont(u8),
    Dsp,
}

impl std::fmt::Display for ColumnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ColumnKind::Io => write!(f, "IO"),
            ColumnKind::Clb => write!(f, "CLB"),
            ColumnKind::Bram => write!(f, "BRAM"),
            ColumnKind::BramCont(i) => write!(f, "BRAM_CONT:{i}"),
            ColumnKind::Dsp => write!(f, "DSP"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum ColumnIoKind {
    None,
    Single,
    Double(u8),
    Triple(u8),
    Quad(u8),
    SingleW,
    SingleE,
    SingleWAlt,
    SingleEAlt,
    DoubleW(u8),
    DoubleE(u8),
    DoubleEClk(u8),
}

impl std::fmt::Display for ColumnIoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ColumnIoKind::None => write!(f, "NONE"),
            ColumnIoKind::Single => write!(f, "SINGLE"),
            ColumnIoKind::Double(i) => write!(f, "DOUBLE:{i}"),
            ColumnIoKind::Triple(i) => write!(f, "TRIPLE:{i}"),
            ColumnIoKind::Quad(i) => write!(f, "QUAD:{i}"),
            ColumnIoKind::SingleW => write!(f, "SINGLE_W"),
            ColumnIoKind::SingleE => write!(f, "SINGLE_E"),
            ColumnIoKind::SingleWAlt => write!(f, "SINGLE_W_ALT"),
            ColumnIoKind::SingleEAlt => write!(f, "SINGLE_E_ALT"),
            ColumnIoKind::DoubleW(i) => write!(f, "DOUBLE_W:{i}"),
            ColumnIoKind::DoubleE(i) => write!(f, "DOUBLE_E:{i}"),
            ColumnIoKind::DoubleEClk(i) => write!(f, "DOUBLE_E_CLK:{i}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum RowIoKind {
    None,
    Single,
    Double(u8),
    Triple(u8),
    Quad(u8),
    DoubleS(u8),
    DoubleN(u8),
}

impl std::fmt::Display for RowIoKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RowIoKind::None => write!(f, "NONE"),
            RowIoKind::Single => write!(f, "SINGLE"),
            RowIoKind::Double(i) => write!(f, "DOUBLE:{i}"),
            RowIoKind::Triple(i) => write!(f, "TRIPLE:{i}"),
            RowIoKind::Quad(i) => write!(f, "QUAD:{i}"),
            RowIoKind::DoubleS(i) => write!(f, "DOUBLE_S:{i}"),
            RowIoKind::DoubleN(i) => write!(f, "DOUBLE_N:{i}"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum Dcms {
    Two,
    Four,
    Eight,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Encode, Decode)]
pub enum SharedCfgPad {
    Data(u8), // ×8
    CsiB,     // Called CS_B on Virtex 2 and Spartan 3.  Doubles as COPI on Spartan-3E and up.
    InitB,
    RdWrB,
    Dout,
    // Shared on Spartan 3E, Spartan 3A only; dedicated on Virtex 2, Spartan 3.
    M0,
    M1,
    M2,
    Cclk,
    HswapEn,
    // Spartan 3E, Spartan 3A only.
    CsoB, // Daisy chain output and SPI CS_B.
    Ldc0,
    Ldc1,
    Ldc2,
    Hdc,
    Addr(u8), // ×20 on 3s100e, ×24 on other Spartan 3E, ×26 on Spartan 3A
    Vs(u8),   // Spartan-3A only; Spartan-3E has them shared with A17-A19 instead
    // Spartan 3A only.
    Awake,
}

impl std::fmt::Display for SharedCfgPad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SharedCfgPad::Data(i) => write!(f, "D{i}"),
            SharedCfgPad::Addr(i) => write!(f, "A{i}"),
            SharedCfgPad::Vs(i) => write!(f, "VS{i}"),
            SharedCfgPad::CsiB => write!(f, "CSI_B"),
            SharedCfgPad::CsoB => write!(f, "CSO_B"),
            SharedCfgPad::RdWrB => write!(f, "RDWR_B"),
            SharedCfgPad::Dout => write!(f, "DOUT"),
            SharedCfgPad::InitB => write!(f, "INIT_B"),
            SharedCfgPad::Cclk => write!(f, "CCLK"),
            SharedCfgPad::M0 => write!(f, "M0"),
            SharedCfgPad::M1 => write!(f, "M1"),
            SharedCfgPad::M2 => write!(f, "M2"),
            SharedCfgPad::Ldc0 => write!(f, "LDC0"),
            SharedCfgPad::Ldc1 => write!(f, "LDC1"),
            SharedCfgPad::Ldc2 => write!(f, "LDC2"),
            SharedCfgPad::Hdc => write!(f, "HDC"),
            SharedCfgPad::HswapEn => write!(f, "HSWAP_EN"),
            SharedCfgPad::Awake => write!(f, "AWAKE"),
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Encode, Decode)]
pub enum DcmPairKind {
    S,
    SingleS,
    N,
    SingleN,
    // S3E
    W,
    E,
    // S3A
    Bram,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, Encode, Decode)]
pub struct DcmPair {
    pub kind: DcmPairKind,
    pub cell: CellCoord,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Encode, Decode)]
pub enum IoDiffKind {
    P(BelPadCoord),
    N(BelPadCoord),
    None,
}

#[derive(Clone, Copy, Debug)]
pub struct IoInfo {
    pub coord: BelPadCoord,
    pub bank: u32,
    pub vcco: Option<BelPadCoord>,
    pub diff: IoDiffKind,
    pub pad_kind: Option<IobKind>,
}

impl Chip {
    // always single chip
    pub const DIE: DieId = DieId::from_idx_const(0);

    pub fn col_edge(&self, dir: DirH) -> ColId {
        match dir {
            DirH::W => self.col_w(),
            DirH::E => self.col_e(),
        }
    }

    pub fn row_edge(&self, dir: DirV) -> RowId {
        match dir {
            DirV::S => self.row_s(),
            DirV::N => self.row_n(),
        }
    }

    pub fn col_w(&self) -> ColId {
        self.columns.first_id().unwrap()
    }

    pub fn col_e(&self) -> ColId {
        self.columns.last_id().unwrap()
    }

    pub fn row_s(&self) -> RowId {
        self.rows.first_id().unwrap()
    }

    pub fn row_n(&self) -> RowId {
        self.rows.last_id().unwrap()
    }

    pub fn is_row_io(&self, row: RowId) -> bool {
        row == self.row_s() || row == self.row_n()
    }

    pub fn is_col_io(&self, col: ColId) -> bool {
        col == self.col_w() || col == self.col_e()
    }

    pub fn corner(&self, dir: DirHV) -> TileCoord {
        Self::DIE
            .cell(self.col_edge(dir.h), self.row_edge(dir.v))
            .tile(tslots::BEL)
    }

    pub fn row_mid(&self) -> RowId {
        RowId::from_idx(self.rows.len() / 2)
    }

    pub fn bram_row(&self, row: RowId) -> Option<usize> {
        if let Some(rows_ram) = self.rows_ram {
            rows_ram.index_of(row).map(|idx| idx % 4)
        } else {
            if row == self.row_s() || row == self.row_n() {
                None
            } else {
                Some((row.to_idx() - 1) % 4)
            }
        }
    }

    pub fn get_dcm_pairs(&self) -> Vec<DcmPair> {
        let mut res = vec![];
        if let Some(dcms) = self.dcms {
            if dcms == Dcms::Two {
                if self.kind == ChipKind::Spartan3E {
                    res.extend([
                        DcmPair {
                            kind: DcmPairKind::SingleS,
                            cell: Self::DIE.cell(self.col_clk, self.row_s() + 1),
                        },
                        DcmPair {
                            kind: DcmPairKind::SingleN,
                            cell: Self::DIE.cell(self.col_clk, self.row_n() - 1),
                        },
                    ]);
                } else {
                    res.extend([DcmPair {
                        kind: DcmPairKind::N,
                        cell: Self::DIE.cell(self.col_clk, self.row_n() - 1),
                    }]);
                }
            } else {
                res.extend([
                    DcmPair {
                        kind: DcmPairKind::S,
                        cell: Self::DIE.cell(self.col_clk, self.row_s() + 1),
                    },
                    DcmPair {
                        kind: DcmPairKind::N,
                        cell: Self::DIE.cell(self.col_clk, self.row_n() - 1),
                    },
                ]);
            }
            if dcms == Dcms::Eight {
                if self.kind == ChipKind::Spartan3E {
                    res.extend([
                        DcmPair {
                            kind: DcmPairKind::W,
                            cell: Self::DIE.cell(self.col_w() + 9, self.row_mid()),
                        },
                        DcmPair {
                            kind: DcmPairKind::E,
                            cell: Self::DIE.cell(self.col_e() - 9, self.row_mid()),
                        },
                    ]);
                } else {
                    res.extend([
                        DcmPair {
                            kind: DcmPairKind::Bram,
                            cell: Self::DIE.cell(self.col_w() + 3, self.row_mid()),
                        },
                        DcmPair {
                            kind: DcmPairKind::Bram,
                            cell: Self::DIE.cell(self.col_e() - 6, self.row_mid()),
                        },
                    ]);
                }
            }
        }
        res
    }

    pub fn bel_corner_misc(&self, side: DirHV) -> BelCoord {
        self.corner(side).bel(match side {
            DirHV::SW => bslots::MISC_SW,
            DirHV::SE => bslots::MISC_SE,
            DirHV::NW => bslots::MISC_NW,
            DirHV::NE => bslots::MISC_NE,
        })
    }

    pub fn pad_io_h(&self, edge: DirH, row: RowId, slot: BelSlotId) -> BelPadCoord {
        let pad = if bslots::IOI.contains(slot) {
            bcls::IOI::PAD
        } else if bslots::IREG.contains(slot) {
            bcls::IREG::PAD
        } else if bslots::OREG.contains(slot) {
            bcls::OREG::PAD
        } else {
            unreachable!()
        };
        Self::DIE.cell(self.col_edge(edge), row).bel(slot).pad(pad)
    }

    pub fn pad_io_v(&self, edge: DirV, col: ColId, slot: BelSlotId) -> BelPadCoord {
        let pad = if bslots::IOI.contains(slot) {
            bcls::IOI::PAD
        } else if bslots::IREG.contains(slot) {
            bcls::IREG::PAD
        } else if bslots::OREG.contains(slot) {
            bcls::OREG::PAD
        } else {
            unreachable!()
        };
        Self::DIE.cell(col, self.row_edge(edge)).bel(slot).pad(pad)
    }

    pub fn bel_gt(&self, edge: DirV, col: ColId) -> BelCoord {
        let slot = match self.kind {
            ChipKind::Virtex2P => bslots::GT,
            ChipKind::Virtex2PX => bslots::GT10,
            _ => unreachable!(),
        };
        Self::DIE.cell(col, self.row_edge(edge)).bel(slot)
    }

    pub fn pad_gt_rx(&self, bel: BelCoord, pol: Polarity) -> BelPadCoord {
        let pad = match (bel.slot, pol) {
            (bslots::GT, Polarity::P) => bcls::GT::RXP,
            (bslots::GT, Polarity::N) => bcls::GT::RXN,
            (bslots::GT10, Polarity::P) => bcls::GT10::RXP,
            (bslots::GT10, Polarity::N) => bcls::GT10::RXN,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pad_gt_tx(&self, bel: BelCoord, pol: Polarity) -> BelPadCoord {
        let pad = match (bel.slot, pol) {
            (bslots::GT, Polarity::P) => bcls::GT::TXP,
            (bslots::GT, Polarity::N) => bcls::GT::TXN,
            (bslots::GT10, Polarity::P) => bcls::GT10::TXP,
            (bslots::GT10, Polarity::N) => bcls::GT10::TXN,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pad_gt_gnda(&self, bel: BelCoord) -> BelPadCoord {
        let pad = match bel.slot {
            bslots::GT => bcls::GT::GNDA,
            bslots::GT10 => bcls::GT10::GNDA,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pad_gt_vtrx(&self, bel: BelCoord) -> BelPadCoord {
        let pad = match bel.slot {
            bslots::GT => bcls::GT::VTRX,
            bslots::GT10 => bcls::GT10::VTRX,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pad_gt_vttx(&self, bel: BelCoord) -> BelPadCoord {
        let pad = match bel.slot {
            bslots::GT => bcls::GT::VTTX,
            bslots::GT10 => bcls::GT10::VTTX,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pad_gt_avccauxrx(&self, bel: BelCoord) -> BelPadCoord {
        let pad = match bel.slot {
            bslots::GT => bcls::GT::AVCCAUXRX,
            bslots::GT10 => bcls::GT10::AVCCAUXRX,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pad_gt_avccauxtx(&self, bel: BelCoord) -> BelPadCoord {
        let pad = match bel.slot {
            bslots::GT => bcls::GT::AVCCAUXTX,
            bslots::GT10 => bcls::GT10::AVCCAUXTX,
            _ => unreachable!(),
        };
        bel.pad(pad)
    }

    pub fn pads_mode(&self) -> [BelPadCoord; 3] {
        if self.kind.is_spartan3ea() {
            [
                self.cfg_io[&SharedCfgPad::M0],
                self.cfg_io[&SharedCfgPad::M1],
                self.cfg_io[&SharedCfgPad::M2],
            ]
        } else {
            let bel = self.bel_corner_misc(DirHV::SW);
            [
                bel.pad(bcls::MISC_SW::M0),
                bel.pad(bcls::MISC_SW::M1),
                bel.pad(bcls::MISC_SW::M2),
            ]
        }
    }

    pub fn pad_cclk(&self) -> BelPadCoord {
        if self.kind.is_spartan3ea() {
            self.cfg_io[&SharedCfgPad::Cclk]
        } else {
            self.bel_corner_misc(DirHV::SE).pad(bcls::MISC_SE::CCLK)
        }
    }

    pub fn pad_hswap_en(&self) -> BelPadCoord {
        assert_ne!(self.kind, ChipKind::FpgaCore);
        if self.kind.is_spartan3ea() {
            self.cfg_io[&SharedCfgPad::HswapEn]
        } else {
            self.bel_corner_misc(DirHV::NW).pad(bcls::MISC_NW::HSWAPEN)
        }
    }

    pub fn pad_done(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::SE).pad(bcls::MISC_SE::DONE)
    }

    pub fn pad_prog_b(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NW).pad(bcls::MISC_NW::PROG_B)
    }

    pub fn pad_suspend(&self) -> BelPadCoord {
        assert!(self.kind.is_spartan3a());
        self.bel_corner_misc(DirHV::SE).pad(bcls::MISC_SE::SUSPEND)
    }

    pub fn pad_tdi(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NW).pad(bcls::MISC_NW::TDI)
    }

    pub fn pad_tms(&self) -> BelPadCoord {
        if self.kind.is_spartan3a() {
            self.bel_corner_misc(DirHV::NW).pad(bcls::MISC_NW::TMS)
        } else {
            self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::TMS)
        }
    }

    pub fn pad_tdo(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::TDO)
    }

    pub fn pad_tck(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::TCK)
    }

    pub fn pad_powerdown_b(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::SE)
            .pad(bcls::MISC_SE::POWERDOWN_B)
    }

    pub fn pad_vccbatt(&self) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NE).pad(bcls::MISC_NE::VCCBATT)
    }

    pub fn pad_dx(&self, pol: Polarity) -> BelPadCoord {
        self.bel_corner_misc(DirHV::NW).pad(match pol {
            Polarity::P => bcls::MISC_NW::DXP,
            Polarity::N => bcls::MISC_NW::DXN,
        })
    }

    pub fn pad_gnd(&self) -> BelPadCoord {
        self.bel_global().pad(bcls::GLOBAL::GND)
    }

    pub fn pad_vccint(&self) -> BelPadCoord {
        self.bel_global().pad(bcls::GLOBAL::VCCINT)
    }

    pub fn pad_vccaux(&self) -> BelPadCoord {
        self.bel_global().pad(bcls::GLOBAL::VCCAUX)
    }

    pub fn pad_bank_vcco(&self, bank: u32) -> Option<BelPadCoord> {
        match self.kind {
            ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX | ChipKind::Spartan3 => {
                let bel = match bank {
                    0 => self.corner(DirHV::NW).bel(bslots::DCI_SN),
                    1 => self.corner(DirHV::NE).bel(bslots::DCI_SN),
                    2 => self.corner(DirHV::NE).bel(bslots::DCI_WE),
                    3 => self.corner(DirHV::SE).bel(bslots::DCI_WE),
                    4 => self.corner(DirHV::SE).bel(bslots::DCI_SN),
                    5 => self.corner(DirHV::SW).bel(bslots::DCI_SN),
                    6 => self.corner(DirHV::SW).bel(bslots::DCI_WE),
                    7 => self.corner(DirHV::NW).bel(bslots::DCI_WE),
                    _ => unreachable!(),
                };
                Some(bel.pad(bcls::DCI::VCCO))
            }
            ChipKind::FpgaCore => None,
            ChipKind::Spartan3E => {
                let bel = match bank {
                    0 => self.corner(DirHV::NW).bel(bslots::BANK_SN),
                    1 => self.corner(DirHV::NE).bel(bslots::BANK_WE),
                    2 => self.corner(DirHV::SE).bel(bslots::BANK_SN),
                    3 => self.corner(DirHV::SW).bel(bslots::BANK_WE),
                    _ => unreachable!(),
                };
                Some(bel.pad(bcls::BANK::VCCO))
            }
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                let bel = match bank {
                    0 => self.corner(DirHV::NW).bel(bslots::BANK_SN),
                    1 => self.corner(DirHV::SE).bel(bslots::BANK_WE),
                    2 => self.corner(DirHV::SW).bel(bslots::BANK_SN),
                    3 => self.corner(DirHV::SW).bel(bslots::BANK_WE),
                    _ => unreachable!(),
                };
                Some(bel.pad(bcls::BANK::VCCO))
            }
        }
    }

    pub fn get_io_edge(&self, io: BelPadCoord) -> Dir {
        if io.row == self.row_n() {
            Dir::N
        } else if io.row == self.row_s() {
            Dir::S
        } else if io.col == self.col_w() {
            Dir::W
        } else if io.col == self.col_e() {
            Dir::E
        } else {
            unreachable!()
        }
    }

    pub fn get_io_info(&self, io: BelPadCoord) -> IoInfo {
        let bank = match self.kind {
            ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX | ChipKind::Spartan3 => {
                match self.get_io_edge(io) {
                    Dir::N => {
                        if io.col < self.col_clk {
                            0
                        } else {
                            1
                        }
                    }
                    Dir::E => {
                        if io.row < self.row_mid() {
                            3
                        } else {
                            2
                        }
                    }
                    Dir::S => {
                        if io.col < self.col_clk {
                            5
                        } else {
                            4
                        }
                    }
                    Dir::W => {
                        if io.row < self.row_mid() {
                            6
                        } else {
                            7
                        }
                    }
                }
            }
            ChipKind::FpgaCore => 0,
            ChipKind::Spartan3E | ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                match self.get_io_edge(io) {
                    Dir::N => 0,
                    Dir::E => 1,
                    Dir::S => 2,
                    Dir::W => 3,
                }
            }
        };
        let diff = match self.kind {
            ChipKind::Virtex2 | ChipKind::Virtex2P | ChipKind::Virtex2PX => {
                let idx = bslots::IOI.index_of(io.slot).unwrap();
                let is_single = if io.row == self.row_s() || io.row == self.row_n() {
                    matches!(
                        self.columns[io.col].io,
                        ColumnIoKind::SingleWAlt | ColumnIoKind::SingleEAlt
                    )
                } else {
                    false
                };
                if is_single {
                    match idx {
                        0 => IoDiffKind::None,
                        1 => IoDiffKind::P(io.bel(bslots::IOI[2]).pad(bcls::IOI::PAD)),
                        2 => IoDiffKind::N(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                        3 => IoDiffKind::None,
                        _ => unreachable!(),
                    }
                } else {
                    match idx {
                        0 => IoDiffKind::P(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                        1 => IoDiffKind::N(io.bel(bslots::IOI[0]).pad(bcls::IOI::PAD)),
                        2 => IoDiffKind::P(io.bel(bslots::IOI[3]).pad(bcls::IOI::PAD)),
                        3 => IoDiffKind::N(io.bel(bslots::IOI[2]).pad(bcls::IOI::PAD)),
                        _ => unreachable!(),
                    }
                }
            }
            ChipKind::Spartan3 => {
                let idx = bslots::IOI.index_of(io.slot).unwrap();
                if io.col == self.col_w() {
                    match idx {
                        0 => IoDiffKind::N(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                        1 => IoDiffKind::P(io.bel(bslots::IOI[0]).pad(bcls::IOI::PAD)),
                        2 => IoDiffKind::None,
                        _ => unreachable!(),
                    }
                } else {
                    match idx {
                        0 => IoDiffKind::P(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                        1 => IoDiffKind::N(io.bel(bslots::IOI[0]).pad(bcls::IOI::PAD)),
                        2 => IoDiffKind::None,
                        _ => unreachable!(),
                    }
                }
            }
            ChipKind::FpgaCore => IoDiffKind::None,
            ChipKind::Spartan3E => {
                let idx = bslots::IOI.index_of(io.slot).unwrap();
                match idx {
                    0 => IoDiffKind::P(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                    1 => IoDiffKind::N(io.bel(bslots::IOI[0]).pad(bcls::IOI::PAD)),
                    2 => IoDiffKind::None,
                    _ => unreachable!(),
                }
            }
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp => {
                let idx = bslots::IOI.index_of(io.slot).unwrap();
                if io.col == self.col_w() || io.row == self.row_n() {
                    match idx {
                        0 => IoDiffKind::N(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                        1 => IoDiffKind::P(io.bel(bslots::IOI[0]).pad(bcls::IOI::PAD)),
                        2 => IoDiffKind::None,
                        _ => unreachable!(),
                    }
                } else {
                    match idx {
                        0 => IoDiffKind::P(io.bel(bslots::IOI[1]).pad(bcls::IOI::PAD)),
                        1 => IoDiffKind::N(io.bel(bslots::IOI[0]).pad(bcls::IOI::PAD)),
                        2 => IoDiffKind::None,
                        _ => unreachable!(),
                    }
                }
            }
        };
        let mut pad_kind = None;
        if let Some((data, tidx)) = self.get_iob_tile_data(io.cell) {
            for &iob_data in &data.iobs {
                if iob_data.cell == tidx && iob_data.ioi == io.slot {
                    pad_kind = Some(iob_data.kind);
                }
            }
        }
        IoInfo {
            coord: io,
            vcco: self.pad_bank_vcco(bank),
            bank,
            diff,
            pad_kind,
        }
    }

    pub fn get_bonded_ios(&self) -> Vec<BelPadCoord> {
        let mut res = vec![];
        for col in self.columns.ids() {
            let row = self.row_n();
            if let Some((data, tidx)) = self.get_iob_tile_data(Self::DIE.cell(col, row)) {
                for &iob in &data.iobs {
                    if iob.cell == tidx {
                        res.push(self.pad_io_v(DirV::N, col, iob.ioi));
                    }
                }
            }
        }
        for row in self.rows.ids().rev() {
            let col = self.col_e();
            if let Some((data, tidx)) = self.get_iob_tile_data(Self::DIE.cell(col, row)) {
                for &iob in &data.iobs {
                    if iob.cell == tidx {
                        res.push(self.pad_io_h(DirH::E, row, iob.ioi));
                    }
                }
            }
        }
        for col in self.columns.ids().rev() {
            let row = self.row_s();
            if let Some((data, tidx)) = self.get_iob_tile_data(Self::DIE.cell(col, row)) {
                for &iob in &data.iobs {
                    if iob.cell == tidx {
                        res.push(self.pad_io_v(DirV::S, col, iob.ioi));
                    }
                }
            }
        }
        for row in self.rows.ids() {
            let col = self.col_w();
            if let Some((data, tidx)) = self.get_iob_tile_data(Self::DIE.cell(col, row)) {
                for &iob in &data.iobs {
                    if iob.cell == tidx {
                        res.push(self.pad_io_h(DirH::W, row, iob.ioi));
                    }
                }
            }
        }
        res
    }

    pub fn get_clk_io(&self, edge: Dir, idx: usize) -> Option<BelPadCoord> {
        if self.kind.is_virtex2() {
            match edge {
                Dir::S => {
                    if self.kind == ChipKind::Virtex2PX && matches!(idx, 6 | 7) {
                        return None;
                    }
                    if idx < 4 {
                        Some(self.pad_io_v(DirV::S, self.col_clk, bslots::IOI[idx]))
                    } else if idx < 8 {
                        Some(self.pad_io_v(DirV::S, self.col_clk - 1, bslots::IOI[idx - 4]))
                    } else {
                        None
                    }
                }
                Dir::N => {
                    if self.kind == ChipKind::Virtex2PX && matches!(idx, 4 | 5) {
                        return None;
                    }
                    if idx < 4 {
                        Some(self.pad_io_v(DirV::N, self.col_clk, bslots::IOI[idx]))
                    } else if idx < 8 {
                        Some(self.pad_io_v(DirV::N, self.col_clk - 1, bslots::IOI[idx - 4]))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else if self.kind == ChipKind::Spartan3 {
            match edge {
                Dir::S => {
                    if idx < 2 {
                        Some(self.pad_io_v(DirV::S, self.col_clk, bslots::IOI[idx]))
                    } else if idx < 4 {
                        Some(self.pad_io_v(DirV::S, self.col_clk - 1, bslots::IOI[idx - 2]))
                    } else {
                        None
                    }
                }
                Dir::N => {
                    if idx < 2 {
                        Some(self.pad_io_v(DirV::N, self.col_clk, bslots::IOI[idx]))
                    } else if idx < 4 {
                        Some(self.pad_io_v(DirV::N, self.col_clk - 1, bslots::IOI[idx - 2]))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else if self.kind == ChipKind::FpgaCore {
            match edge {
                Dir::S => {
                    if idx < 2 {
                        Some(self.pad_io_v(DirV::S, self.col_clk, bslots::IREG[idx]))
                    } else if idx < 4 {
                        Some(self.pad_io_v(DirV::S, self.col_clk - 1, bslots::IREG[idx - 2]))
                    } else {
                        None
                    }
                }
                Dir::N => {
                    if idx < 2 {
                        Some(self.pad_io_v(DirV::N, self.col_clk, bslots::IREG[idx]))
                    } else if idx < 4 {
                        Some(self.pad_io_v(DirV::N, self.col_clk - 1, bslots::IREG[idx - 2]))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else if self.kind == ChipKind::Spartan3E {
            match (edge, idx) {
                (Dir::S, 0 | 1) => Some(self.pad_io_v(DirV::S, self.col_clk, bslots::IOI[idx % 2])),
                (Dir::S, 2 | 3) => {
                    Some(self.pad_io_v(DirV::S, self.col_clk + 1, bslots::IOI[idx % 2]))
                }
                (Dir::S, 4 | 5) => {
                    Some(self.pad_io_v(DirV::S, self.col_clk - 3, bslots::IOI[idx % 2]))
                }
                (Dir::S, 6 | 7) => {
                    Some(self.pad_io_v(DirV::S, self.col_clk - 1, bslots::IOI[idx % 2]))
                }
                (Dir::N, 0 | 1) => {
                    Some(self.pad_io_v(DirV::N, self.col_clk + 2, bslots::IOI[idx % 2]))
                }
                (Dir::N, 2 | 3) => Some(self.pad_io_v(DirV::N, self.col_clk, bslots::IOI[idx % 2])),
                (Dir::N, 4 | 5) => {
                    Some(self.pad_io_v(DirV::N, self.col_clk - 1, bslots::IOI[idx % 2]))
                }
                (Dir::N, 6 | 7) => {
                    Some(self.pad_io_v(DirV::N, self.col_clk - 2, bslots::IOI[idx % 2]))
                }
                (Dir::W, 0 | 1) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() + 3, bslots::IOI[idx % 2]))
                }
                (Dir::W, 2 | 3) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() + 1, bslots::IOI[idx % 2]))
                }
                (Dir::W, 4 | 5) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() - 1, bslots::IOI[idx % 2]))
                }
                (Dir::W, 6 | 7) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() - 3, bslots::IOI[idx % 2]))
                }
                (Dir::E, 0 | 1) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid(), bslots::IOI[idx % 2]))
                }
                (Dir::E, 2 | 3) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid() + 2, bslots::IOI[idx % 2]))
                }
                (Dir::E, 4 | 5) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid() - 4, bslots::IOI[idx % 2]))
                }
                (Dir::E, 6 | 7) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid() - 2, bslots::IOI[idx % 2]))
                }
                _ => None,
            }
        } else {
            match (edge, idx) {
                (Dir::S, 0 | 1) => Some(self.pad_io_v(DirV::S, self.col_clk, bslots::IOI[idx % 2])),
                (Dir::S, 2 | 3) => {
                    Some(self.pad_io_v(DirV::S, self.col_clk + 1, bslots::IOI[idx % 2]))
                }
                (Dir::S, 4 | 5) => {
                    Some(self.pad_io_v(DirV::S, self.col_clk - 2, bslots::IOI[idx % 2]))
                }
                (Dir::S, 6 | 7) => {
                    Some(self.pad_io_v(DirV::S, self.col_clk - 1, bslots::IOI[idx % 2]))
                }
                (Dir::N, 0 | 1) => {
                    Some(self.pad_io_v(DirV::N, self.col_clk + 1, bslots::IOI[idx % 2]))
                }
                (Dir::N, 2 | 3) => Some(self.pad_io_v(DirV::N, self.col_clk, bslots::IOI[idx % 2])),
                (Dir::N, 4 | 5) => {
                    Some(self.pad_io_v(DirV::N, self.col_clk - 1, bslots::IOI[idx % 2]))
                }
                (Dir::N, 6 | 7) => {
                    Some(self.pad_io_v(DirV::N, self.col_clk - 2, bslots::IOI[idx % 2]))
                }
                (Dir::W, 0 | 1) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() + 2, bslots::IOI[(idx % 2) ^ 1]))
                }
                (Dir::W, 2 | 3) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() + 1, bslots::IOI[(idx % 2) ^ 1]))
                }
                (Dir::W, 4 | 5) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() - 1, bslots::IOI[(idx % 2) ^ 1]))
                }
                (Dir::W, 6 | 7) => {
                    Some(self.pad_io_h(DirH::W, self.row_mid() - 2, bslots::IOI[(idx % 2) ^ 1]))
                }
                (Dir::E, 0 | 1) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid(), bslots::IOI[idx % 2]))
                }
                (Dir::E, 2 | 3) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid() + 1, bslots::IOI[idx % 2]))
                }
                (Dir::E, 4 | 5) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid() - 3, bslots::IOI[idx % 2]))
                }
                (Dir::E, 6 | 7) => {
                    Some(self.pad_io_h(DirH::E, self.row_mid() - 2, bslots::IOI[idx % 2]))
                }
                _ => None,
            }
        }
    }

    pub fn get_brefclk_io(&self, edge: DirV) -> [BelPadCoord; 2] {
        assert_eq!(self.kind, ChipKind::Virtex2P);
        match edge {
            DirV::S => [
                self.pad_io_v(DirV::S, self.col_clk - 1, bslots::IOI[2]),
                self.pad_io_v(DirV::S, self.col_clk, bslots::IOI[0]),
            ],
            DirV::N => [
                self.pad_io_v(DirV::N, self.col_clk - 1, bslots::IOI[0]),
                self.pad_io_v(DirV::N, self.col_clk, bslots::IOI[2]),
            ],
        }
    }

    pub fn get_pci_io(&self, edge: DirH) -> [BelPadCoord; 2] {
        match self.kind {
            ChipKind::Spartan3E => match edge {
                DirH::W => [
                    self.pad_io_h(DirH::W, self.row_mid() + 1, bslots::IOI[1]),
                    self.pad_io_h(DirH::W, self.row_mid() - 1, bslots::IOI[0]),
                ],
                DirH::E => [
                    self.pad_io_h(DirH::E, self.row_mid(), bslots::IOI[0]),
                    self.pad_io_h(DirH::E, self.row_mid() - 2, bslots::IOI[1]),
                ],
            },
            ChipKind::Spartan3A | ChipKind::Spartan3ADsp => match edge {
                DirH::W => [
                    self.pad_io_h(DirH::W, self.row_mid() + 1, bslots::IOI[0]),
                    self.pad_io_h(DirH::W, self.row_mid() - 2, bslots::IOI[1]),
                ],
                DirH::E => [
                    self.pad_io_h(DirH::E, self.row_mid() + 1, bslots::IOI[0]),
                    self.pad_io_h(DirH::E, self.row_mid() - 2, bslots::IOI[1]),
                ],
            },
            _ => unreachable!(),
        }
    }

    pub fn get_iob_tile_data(&self, cell: CellCoord) -> Option<(IobTileData, CellSlotId)> {
        if cell.col == self.col_w() {
            let kind = self.rows[cell.row];
            if kind == RowIoKind::None {
                None
            } else {
                Some(get_iob_data_w(self.kind, kind))
            }
        } else if cell.col == self.col_e() {
            let kind = self.rows[cell.row];
            if kind == RowIoKind::None {
                None
            } else {
                Some(get_iob_data_e(self.kind, kind))
            }
        } else if cell.row == self.row_s() {
            let kind = self.columns[cell.col].io;
            if kind == ColumnIoKind::None {
                None
            } else {
                Some(get_iob_data_s(self.kind, kind))
            }
        } else if cell.row == self.row_n() {
            let kind = self.columns[cell.col].io;
            if kind == ColumnIoKind::None {
                None
            } else {
                Some(get_iob_data_n(self.kind, kind))
            }
        } else {
            unreachable!()
        }
    }

    pub fn tile_global(&self) -> TileCoord {
        self.corner(DirHV::SW).tile(tslots::GLOBAL)
    }

    pub fn bel_global(&self) -> BelCoord {
        self.tile_global().bel(bslots::GLOBAL)
    }
}

impl Chip {
    pub fn dump(&self, o: &mut dyn std::io::Write, db: &IntDb) -> std::io::Result<()> {
        writeln!(o, "\tkind {};", self.kind)?;
        writeln!(o, "\tcolumns {{")?;
        for (col, cd) in &self.columns {
            if let Some(cols_clkv) = self.cols_clkv {
                if col == cols_clkv[DirH::W] {
                    writeln!(o, "\t\t// clock left spine")?;
                }
                if col == cols_clkv[DirH::E] {
                    writeln!(o, "\t\t// clock right spine")?;
                }
            }
            if col == self.col_clk {
                writeln!(o, "\t\t// clock spine")?;
            }
            write!(o, "\t\t")?;
            match cd.kind {
                ColumnKind::Io => write!(o, "io")?,
                ColumnKind::Clb => write!(o, "clb")?,
                ColumnKind::Bram => write!(o, "bram")?,
                ColumnKind::BramCont(i) => write!(o, "bram {i}")?,
                ColumnKind::Dsp => write!(o, "dsp")?,
            }
            match cd.io {
                ColumnIoKind::None => (),
                ColumnIoKind::Single => write!(o, " + io single")?,
                ColumnIoKind::Double(i) => write!(o, " + io double {i}")?,
                ColumnIoKind::Triple(i) => write!(o, " + io triple {i}")?,
                ColumnIoKind::Quad(i) => write!(o, " + io quad {i}")?,
                ColumnIoKind::SingleW => write!(o, " + io single w")?,
                ColumnIoKind::SingleE => write!(o, " + io single e")?,
                ColumnIoKind::SingleWAlt => write!(o, " + io single w alt")?,
                ColumnIoKind::SingleEAlt => write!(o, " + io single e alt")?,
                ColumnIoKind::DoubleW(i) => write!(o, " + io double w {i}")?,
                ColumnIoKind::DoubleE(i) => write!(o, " + io double e {i}")?,
                ColumnIoKind::DoubleEClk(i) => write!(o, " + io double e clock {i}")?,
            }
            if let Some(&cols_gt) = self.cols_gt.get(&col) {
                write!(
                    o,
                    " + gt [{bb}, {bt}]",
                    bb = cols_gt[DirV::S],
                    bt = cols_gt[DirV::N],
                )?;
            }
            writeln!(o, ", // {col}")?;
        }
        writeln!(o, "\t}}")?;
        writeln!(o, "\tcol_clk {};", self.col_clk)?;
        if let Some(cols_clkv) = self.cols_clkv {
            writeln!(
                o,
                "\tcols_clkv {cl}, {cr};",
                cl = cols_clkv[DirH::W],
                cr = cols_clkv[DirH::E]
            )?;
        }
        writeln!(o, "\trows {{")?;
        for (row, rd) in &self.rows {
            if row == self.rows_hclk[row] {
                writeln!(o, "\t\t// clock row")?;
            }
            if row == self.rows_hclk.range(row).start() && row != self.row_s() {
                writeln!(o, "\t\t// clock break")?;
            }
            if Some(row) == self.row_pci {
                writeln!(o, "\t\t// PCI row")?;
            }
            if row == self.row_mid() {
                writeln!(o, "\t\t// spine row")?;
            }
            write!(o, "\t\t")?;
            match rd {
                RowIoKind::None => write!(o, "null")?,
                RowIoKind::Single => write!(o, "io single")?,
                RowIoKind::Double(i) => write!(o, "io double {i}")?,
                RowIoKind::Triple(i) => write!(o, "io triple {i}")?,
                RowIoKind::Quad(i) => write!(o, "io quad {i}")?,
                RowIoKind::DoubleS(i) => write!(o, "io double s {i}")?,
                RowIoKind::DoubleN(i) => write!(o, "io double n {i}")?,
            }
            write!(o, ", // {row}")?;
            if let Some(rows_ram) = self.rows_ram {
                if row == rows_ram.start() - 1 {
                    write!(o, " BRAM S TERM")?;
                }
                if row == rows_ram.end() {
                    write!(o, " BRAM N TERM")?;
                }
            }
            writeln!(o)?;
        }
        writeln!(o, "\t}}")?;
        if let Some(row) = self.row_pci {
            writeln!(o, "\trow_pci {row};")?;
        }
        if let Some(rows_ram) = self.rows_ram {
            writeln!(o, "\trows_ram {rows_ram};")?;
        }
        for (rows, &row_hclk) in &self.rows_hclk {
            writeln!(o, "\trow_hclk {row_hclk} = {rows};")?;
        }
        for &(col, row) in &self.holes_ppc {
            writeln!(
                o,
                "\tppc {col_l}:{col_r}, {row_b}:{row_t};",
                col_l = col,
                col_r = col + 10,
                row_b = row,
                row_t = row + 16
            )?;
        }
        if let Some(dcms) = self.dcms {
            match dcms {
                Dcms::Two => writeln!(o, "\tdcms 2;")?,
                Dcms::Four => writeln!(o, "\tdcms 4;")?,
                Dcms::Eight => writeln!(o, "\tdcms 8;")?,
            }
        }
        if self.has_ll {
            writeln!(o, "\thas_ll;")?;
        }
        if self.has_small_int {
            writeln!(o, "\thas_small_int;")?;
        }
        for (k, pad) in &self.cfg_io {
            writeln!(o, "\tcfg_io {k} = {pad};", pad = pad.to_string(db))?;
        }
        for k in 0..8 {
            if let Some(&vr) = self.dci_io.get(&k) {
                writeln!(
                    o,
                    "\tdci {k} = vp {vp}, vn {vn};",
                    vp = vr[Polarity::P].to_string(db),
                    vn = vr[Polarity::N].to_string(db)
                )?;
            }
            if let Some(&vr) = self.dci_io_alt.get(&k) {
                writeln!(
                    o,
                    "\tdci alt {k} = vp {vp}, vn {vn};",
                    vp = vr[Polarity::P].to_string(db),
                    vn = vr[Polarity::N].to_string(db)
                )?;
            }
        }
        Ok(())
    }
}
