use bincode::{Decode, Encode};
use itertools::Itertools;
use prjcombine_entity::{EntityId, EntityRange, EntityVec, entity_id_u8};
use prjcombine_interconnect::{
    dir::{DirH, DirV},
    grid::{BelCoord, CellCoord, ColId, DieId, RowId},
};
use prjcombine_types::enumerable::EnumMap;
use std::collections::BTreeSet;

use crate::defs::bslots;

entity_id_u8!(pub RegId arith, RegTag tag, "REG");
entity_id_u8!(pub HRegId arith, HRegTag tag, "HREG");

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Chip {
    pub columns: EntityVec<ColId, Column>,
    pub cols_vbrk: BTreeSet<ColId>,
    pub cols_cpipe: BTreeSet<ColId>,
    pub cols_hard: Vec<HardColumn>,
    pub regs: usize,
    pub regs_gt: EnumMap<DirH, EntityVec<RegId, GtRowKind>>,
    pub fabric: FabricKind,
    pub ps: PsKind,
    pub hsr_s: HsrSKind,
    pub hsr_n: HsrNKind,
}

/// The version of the general fabric (mostly clock interconnect).
///
/// There is *some* correlation of this with versions of other blocks or the NOC,
/// but it's far from strict.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum FabricKind {
    /// The first-generation Versal parts.
    ///
    /// Applies to: vc1902, vc1702, vm1402.
    Orig,
    /// A minor revision with different scan clear clock [?].
    ///
    /// Applies to: vp1402.
    Rev2,
    /// Second-generation clock interconnect, with `GCLK_DELAY_SSIT` primitive.
    ///
    /// Applies to: vp1052, ve2102, ve2302, vc2802, vp1202 (and all its hats), vm2152, vn3716.
    Ssit,
    /// Third-generation clock interconnect, with `GCLK_DELAY_SSIT2` primitive.
    ///
    /// Applies to: vp1902.
    Ssit2,
    /// Fourth-generation clock interconnect (with 8 special point-to-point HDISTR lines),
    /// using a bunch of _VR variants of all tiles.  Still uses `GCLK_DELAY_SSIT2`.
    ///
    /// Applies to: vr1602-es1, 2ve3358, 2ve3558, 2ve3858, 2vp2602.
    Ulvt,
}

impl std::fmt::Display for FabricKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FabricKind::Orig => write!(f, "orig"),
            FabricKind::Rev2 => write!(f, "rev2"),
            FabricKind::Ssit => write!(f, "ssit"),
            FabricKind::Ssit2 => write!(f, "ssit2"),
            FabricKind::Ulvt => write!(f, "ulvt"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum InterposerKind {
    Single,
    Column,
    MirrorSquare,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Interposer {
    pub kind: InterposerKind,
    pub sll_columns: EntityVec<DieId, Vec<ColId>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Column {
    pub kind: ColumnKind,
    pub has_bli: EnumMap<DirV, bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum CleKind {
    Plain,
    Sll,
    Sll2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum BramKind {
    Plain,
    SplitterPd,
    Splitter,
    OptSplitter,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum ColumnKind {
    Cle(CleKind),
    Bram(BramKind),
    Uram,
    Dsp,
    Hard,
    Gt,
    HNicX,
    Term,
    TermSsit,
    Cfrm,
    VNoc,
    VNoc2,
    VNoc4,
    ContDsp,
    ContVNoc,
    ContHard,
    None,
}

impl ColumnKind {
    pub fn is_vnoc(self) -> bool {
        matches!(
            self,
            ColumnKind::VNoc | ColumnKind::VNoc2 | ColumnKind::VNoc4
        )
    }
}

impl std::fmt::Display for ColumnKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColumnKind::Cle(CleKind::Plain) => write!(f, "cle"),
            ColumnKind::Cle(CleKind::Sll) => write!(f, "cle + sll"),
            ColumnKind::Cle(CleKind::Sll2) => write!(f, "cle + sll2"),
            ColumnKind::Bram(BramKind::Plain) => write!(f, "bram"),
            ColumnKind::Bram(BramKind::SplitterPd) => write!(f, "bram + splitter_pd"),
            ColumnKind::Bram(BramKind::Splitter) => write!(f, "bram + splitter"),
            ColumnKind::Bram(BramKind::OptSplitter) => write!(f, "bram + opt_splitter"),
            ColumnKind::Uram => write!(f, "uram"),
            ColumnKind::Dsp => write!(f, "dsp"),
            ColumnKind::Hard => write!(f, "hard"),
            ColumnKind::Gt => write!(f, "gt"),
            ColumnKind::HNicX => write!(f, "hnicx"),
            ColumnKind::Term => write!(f, "term"),
            ColumnKind::TermSsit => write!(f, "term + ssit"),
            ColumnKind::Cfrm => write!(f, "cfrm"),
            ColumnKind::VNoc => write!(f, "vnoc"),
            ColumnKind::VNoc2 => write!(f, "vnoc2"),
            ColumnKind::VNoc4 => write!(f, "vnoc4"),
            ColumnKind::ContDsp => write!(f, "cont_dsp"),
            ColumnKind::ContVNoc => write!(f, "cont_vnoc"),
            ColumnKind::ContHard => write!(f, "cont_hard"),
            ColumnKind::None => write!(f, "none"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum PsKind {
    Ps9,
    Ps9Xram,
    Ps9Cpm4,
    Ps9Cpm5,
    Ps9Cpm6,
    PsXCpm5N,
    PsXcMmi,
    PsXt,
}

impl PsKind {
    pub fn height(self) -> usize {
        match self {
            PsKind::Ps9 => Chip::ROWS_PER_REG * 2,
            PsKind::Ps9Xram => Chip::ROWS_PER_REG * 2,
            PsKind::Ps9Cpm4 => Chip::ROWS_PER_REG * 3,
            PsKind::Ps9Cpm5 => Chip::ROWS_PER_REG * 6,
            PsKind::Ps9Cpm6 => Chip::ROWS_PER_REG * 6,
            PsKind::PsXCpm5N => Chip::ROWS_PER_REG * 9,
            PsKind::PsXcMmi => Chip::ROWS_PER_REG * 6,
            PsKind::PsXt => Chip::ROWS_PER_REG * 4,
        }
    }

    pub fn bufg_ps_height(self) -> usize {
        match self {
            PsKind::Ps9 => 1,
            PsKind::Ps9Xram => 1,
            PsKind::Ps9Cpm4 => 1,
            PsKind::Ps9Cpm5 => 3,
            PsKind::Ps9Cpm6 => 1,
            PsKind::PsXCpm5N => 4,
            PsKind::PsXcMmi => 2,
            PsKind::PsXt => 2,
        }
    }
}

impl std::fmt::Display for PsKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PsKind::Ps9 => write!(f, "ps9"),
            PsKind::Ps9Xram => write!(f, "ps9 + xram"),
            PsKind::Ps9Cpm4 => write!(f, "ps9 + cpm4"),
            PsKind::Ps9Cpm5 => write!(f, "ps9 + cpm5"),
            PsKind::Ps9Cpm6 => write!(f, "ps9 + cpm6"),
            PsKind::PsXCpm5N => write!(f, "psx + cpm5n"),
            PsKind::PsXcMmi => write!(f, "psxc + mmi"),
            PsKind::PsXt => write!(f, "psxt"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum HardRowKind {
    None,
    // single-height
    Hdio,
    Pcie4,
    Pcie5,
    Mrmac,
    Sdfec,
    DfeCfc,
    // double-height
    Ilkn,
    Dcmac,
    Hsc,
    // single-height, special (actually part of CPM4)
    CpmExt,
}

impl HardRowKind {
    pub fn is_double(self) -> bool {
        matches!(
            self,
            HardRowKind::Ilkn | HardRowKind::Dcmac | HardRowKind::Hsc
        )
    }
}

impl std::fmt::Display for HardRowKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HardRowKind::None => write!(f, "none"),
            HardRowKind::Hdio => write!(f, "hdio"),
            HardRowKind::Pcie4 => write!(f, "pcie4"),
            HardRowKind::Pcie5 => write!(f, "pcie5"),
            HardRowKind::Mrmac => write!(f, "mrmac"),
            HardRowKind::Sdfec => write!(f, "sdfec"),
            HardRowKind::DfeCfc => write!(f, "dfe_cfc"),
            HardRowKind::Dcmac => write!(f, "dcmac"),
            HardRowKind::Ilkn => write!(f, "ilkn"),
            HardRowKind::Hsc => write!(f, "hsc"),
            HardRowKind::CpmExt => write!(f, "cpm_ext"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct HardColumn {
    pub col: ColId,
    pub regs: EntityVec<RegId, HardRowKind>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum GtRowKind {
    None,
    // single-height
    Gty,
    Gtyp,
    Gtm,
    Gtm2,
    Xram,
    Vdu,
    BfrB,
    Isp2,
    // double-height
    Vcu2,
    RfAdc,
    RfDac,
}

impl GtRowKind {
    pub fn is_double(self) -> bool {
        matches!(self, GtRowKind::Vcu2 | GtRowKind::RfAdc | GtRowKind::RfDac)
    }

    pub fn has_noc(&self) -> bool {
        matches!(self, GtRowKind::Vcu2 | GtRowKind::Isp2)
    }
}

impl std::fmt::Display for GtRowKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GtRowKind::None => write!(f, "none"),
            GtRowKind::Gty => write!(f, "gty"),
            GtRowKind::Gtyp => write!(f, "gtyp"),
            GtRowKind::Gtm => write!(f, "gtm"),
            GtRowKind::Gtm2 => write!(f, "gtm2"),
            GtRowKind::RfAdc => write!(f, "rfadc"),
            GtRowKind::RfDac => write!(f, "rfdac"),
            GtRowKind::Xram => write!(f, "xram"),
            GtRowKind::Vdu => write!(f, "vdu"),
            GtRowKind::BfrB => write!(f, "bfr_b"),
            GtRowKind::Isp2 => write!(f, "isp2"),
            GtRowKind::Vcu2 => write!(f, "vcu2"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum HsrSKind {
    Xpio(XpioRow),
    X5io(X5ioRow),
    Ssit,
}

impl HsrSKind {
    pub fn has_bli_mux_noc(&self) -> bool {
        matches!(self, Self::X5io(..))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum HsrNKind {
    None,
    Xpio(XpioRow),
    Me(MeRow),
    Me2(Me2Row),
    Ssit,
    Ssit2,
    Hbm(HbmRow),
    Nic(NicRow),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct XpioRow {
    pub regions: EntityVec<HRegId, XpioRegion>,
}

impl XpioRow {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        for (hr, region) in &self.regions {
            writeln!(o, "\t\tregion {hr} {{")?;
            write!(o, "\t\t\txpio_bli ")?;
            let mut first = true;
            for col in region.xpio_bli {
                if !first {
                    write!(o, ", ")?;
                }
                first = false;
                if let Some(col) = col {
                    write!(o, "{col}")?;
                } else {
                    write!(o, "-")?;
                }
            }
            writeln!(o, ";")?;
            match region.clk_bli {
                XpioClkBli::Full(col) => writeln!(o, "\t\t\tclk_bli full {col};")?,
                XpioClkBli::Minimal(col) => writeln!(o, "\t\t\tclk_bli minimal {col};")?,
            }
            writeln!(o, "\t\t\tmmcm_bli {};", region.mmcm_bli)?;
            writeln!(o, "\t\t\tdpll_bli {};", region.dpll_bli)?;
            writeln!(o, "\t\t\txpll_bli {};", region.xpll_bli)?;
            match region.dci_bli {
                XpioTestBli::PrimaryA(col) => writeln!(o, "\t\t\tdci_bli primary_a {col};")?,
                XpioTestBli::PrimaryB(col) => writeln!(o, "\t\t\tdci_bli primary_b {col};")?,
                XpioTestBli::PrimaryC(col) => writeln!(o, "\t\t\tdci_bli primary_c {col};")?,
                XpioTestBli::Secondary(col) => writeln!(o, "\t\t\tdci_bli secondary {col};")?,
            }
            match region.scan_bli {
                XpioTestBli::PrimaryA(col) => writeln!(o, "\t\t\tscan_bli primary_a {col};")?,
                XpioTestBli::PrimaryB(col) => writeln!(o, "\t\t\tscan_bli primary_b {col};")?,
                XpioTestBli::PrimaryC(col) => writeln!(o, "\t\t\tscan_bli primary_c {col};")?,
                XpioTestBli::Secondary(col) => writeln!(o, "\t\t\tscan_bli secondary {col};")?,
            }
            if let Some(cols) = region.ddrmc_bli {
                writeln!(o, "\t\t\tddrmc_bli {}, {};", cols[0], cols[1])?;
            }
            if let Some(col) = region.sysmon_bli {
                writeln!(o, "\t\t\tsysmon_bli {};", col)?;
            }

            if let Some(col) = region.clk_route_col {
                writeln!(o, "\t\t\tclk_route_col {};", col)?;
            }
            if let Some(col) = region.clk_distr_col {
                writeln!(o, "\t\t\tclk_distr_col {};", col)?;
            }

            for (i, vnoc) in region.vnoc.into_iter().enumerate() {
                if let Some(vnoc) = vnoc {
                    writeln!(o, "\t\t\tvnoc {i} {vnoc};")?;
                }
            }

            writeln!(o, "\t\t}}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct XpioRegion {
    pub xpio_bli: [Option<ColId>; 9],
    pub clk_bli: XpioClkBli,
    pub mmcm_bli: ColId,
    pub dpll_bli: ColId,
    pub xpll_bli: ColId,
    pub dci_bli: XpioTestBli,
    pub scan_bli: XpioTestBli,
    pub ddrmc_bli: Option<[ColId; 2]>,
    pub sysmon_bli: Option<ColId>,
    pub clk_route_col: Option<ColId>,
    pub clk_distr_col: Option<ColId>,
    pub vnoc: [Option<VNocConn>; 2],
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum XpioClkBli {
    Minimal(ColId),
    Full(ColId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum XpioTestBli {
    PrimaryA(ColId),
    PrimaryB(ColId),
    PrimaryC(ColId),
    Secondary(ColId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct VNocConn(pub ColId, pub usize);

impl std::fmt::Display for VNocConn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct X5ioRow {
    pub regions: EntityVec<HRegId, X5ioRegion>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode, Default)]
pub struct X5ioRegion {
    pub clk: Option<X5ioClk>,
    pub hnoc: X5ioHNoc,
    pub io: Option<X5io>,
    pub sysmon_bli: Option<ColId>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct X5ioClk {
    pub clk_bli: X5ioClkBli,
    pub mmcm_bli: ColId,
    pub dpll_bli: Option<ColId>,
    pub clk_route_col: Option<ColId>,
    pub clk_distr_col: Option<ColId>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode, Default)]
pub enum X5ioHNoc {
    #[default]
    None,
    Ps9A,
    Ps9B,
    Ps9C {
        scan_bli: ColId,
    },
    PsXA,
    PsXB {
        flip: bool,
    },
    PsXC {
        scan_bli: ColId,
    },
    PsXTA,
    PsXTB {
        scan_bli: ColId,
    },
    PsXTC {
        scan_bli: ColId,
        vnoc: ColId,
    },
    Cross {
        alt: bool,
        flip: bool,
        scan_bli: ColId,
        vnoc: Option<ColId>,
    },
    Ddrmc {
        flip: bool,
        swap: bool,
        scan_bli: ColId,
        nmu_bli: [Option<[ColId; 3]>; 2],
        vnoc: [Option<VNocConn>; 2],
    },
    Cap {
        scan_bli: ColId,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct X5io {
    pub side: DirH,
    pub octad_bli: [Option<ColId>; 4],
    pub x5pll_bli: X5ioPllBli,
    pub bank_bli: Option<ColId>,
    pub scan_bli: X5ioScanBli,
    pub ddrmc: Option<X5ioDdrmc>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum X5ioScanBli {
    A(ColId),
    B(ColId),
    C(ColId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum X5ioPllBli {
    None,
    PrimaryA(ColId),
    PrimaryB(ColId),
    Secondary(ColId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum X5ioClkBli {
    MinimalA(ColId),
    MinimalB(ColId),
    Full(ColId),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct X5ioDdrmc {
    pub kind: X5ioDdrmcKind,
    pub bli: ColId,
    pub test_clk: X5ioDdrmcTestClk,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum X5ioDdrmcKind {
    Ddrmc5,
    Ddrmc5C,
    Ddrmc5E,
    Ddrmc5EAlt,
    Ddrmc5E2,
    Ddrmc5E2Cut,
    Ddrmc5X,
}

impl X5ioDdrmcKind {
    pub fn is_2hb(self) -> bool {
        self == Self::Ddrmc5X
    }
}

impl std::fmt::Display for X5ioDdrmcKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X5ioDdrmcKind::Ddrmc5 => write!(f, "ddrmc5"),
            X5ioDdrmcKind::Ddrmc5C => write!(f, "ddrmc5c"),
            X5ioDdrmcKind::Ddrmc5E => write!(f, "ddrmc5e"),
            X5ioDdrmcKind::Ddrmc5EAlt => write!(f, "ddrmc5e_alt"),
            X5ioDdrmcKind::Ddrmc5E2 => write!(f, "ddrmc5e2"),
            X5ioDdrmcKind::Ddrmc5E2Cut => write!(f, "ddrmc5e2_cut"),
            X5ioDdrmcKind::Ddrmc5X => write!(f, "ddrmc5x"),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum X5ioDdrmcTestClk {
    A(HRegId),
    B(HRegId),
}

impl std::fmt::Display for X5ioDdrmcTestClk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            X5ioDdrmcTestClk::A(reg) => write!(f, "a {reg}"),
            X5ioDdrmcTestClk::B(reg) => write!(f, "b {reg}"),
        }
    }
}

impl X5ioRow {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        for (hr, region) in &self.regions {
            writeln!(o, "\t\tregion {hr} {{")?;

            if let Some(clk) = region.clk {
                match clk.clk_bli {
                    X5ioClkBli::Full(col) => writeln!(o, "\t\t\tclk_bli full {col};")?,
                    X5ioClkBli::MinimalA(col) => writeln!(o, "\t\t\tclk_bli minimal_a {col};")?,
                    X5ioClkBli::MinimalB(col) => writeln!(o, "\t\t\tclk_bli minimal_b {col};")?,
                }
                writeln!(o, "\t\t\tmmcm_bli {};", clk.mmcm_bli)?;
                if let Some(dpll_bli) = clk.dpll_bli {
                    writeln!(o, "\t\t\tdpll_bli {dpll_bli};")?;
                }

                if let Some(col) = clk.clk_route_col {
                    writeln!(o, "\t\t\tclk_route_col {};", col)?;
                }
                if let Some(col) = clk.clk_distr_col {
                    writeln!(o, "\t\t\tclk_distr_col {};", col)?;
                }
            }

            match region.hnoc {
                X5ioHNoc::None => (),
                X5ioHNoc::Ps9A => writeln!(o, "\t\t\thnoc ps9_a;")?,
                X5ioHNoc::Ps9B => writeln!(o, "\t\t\thnoc ps9_b;")?,
                X5ioHNoc::Ps9C { scan_bli } => {
                    writeln!(o, "\t\t\thnoc ps9_c scan_bli {scan_bli};")?
                }
                X5ioHNoc::PsXA => writeln!(o, "\t\t\thnoc psx_a;")?,
                X5ioHNoc::PsXB { flip } => writeln!(
                    o,
                    "\t\t\thnoc psx_b{flip};",
                    flip = if flip { " flip" } else { "" }
                )?,
                X5ioHNoc::PsXC { scan_bli } => {
                    writeln!(o, "\t\t\thnoc psx_c scan_bli {scan_bli};")?
                }
                X5ioHNoc::PsXTA => writeln!(o, "\t\t\thnoc psxt_a;")?,
                X5ioHNoc::PsXTB { scan_bli } => {
                    writeln!(o, "\t\t\thnoc psxt_b scan_bli {scan_bli};")?
                }
                X5ioHNoc::PsXTC { scan_bli, vnoc } => {
                    writeln!(o, "\t\t\thnoc psxt_c scan_bli {scan_bli} vnoc {vnoc};")?
                }
                X5ioHNoc::Cross {
                    alt,
                    flip,
                    scan_bli,
                    vnoc,
                } => {
                    write!(
                        o,
                        "\t\t\thnoc cross{alt}{flip} scan_bli {scan_bli}",
                        alt = if alt { " alt" } else { "" },
                        flip = if flip { " flip" } else { "" },
                    )?;
                    if let Some(vnoc) = vnoc {
                        write!(o, " vnoc {vnoc}")?;
                    }
                    writeln!(o, ";")?;
                }
                X5ioHNoc::Ddrmc {
                    flip,
                    swap,
                    scan_bli,
                    nmu_bli,
                    vnoc,
                } => {
                    write!(
                        o,
                        "\t\t\thnoc ddrmc{flip}{swap} scan_bli {scan_bli} nmu_bli [",
                        flip = if flip { " flip" } else { "" },
                        swap = if swap { " swap" } else { "" },
                    )?;
                    if let Some([bli0, bli1, bli2]) = nmu_bli[0] {
                        write!(o, "({bli0}, {bli1}, {bli2})")?;
                    } else {
                        write!(o, "-")?;
                    }
                    write!(o, ", ")?;
                    if let Some([bli0, bli1, bli2]) = nmu_bli[1] {
                        write!(o, "({bli0}, {bli1}, {bli2})")?;
                    } else {
                        write!(o, "-")?;
                    }
                    write!(o, "]")?;
                    if vnoc[0].is_some() || vnoc[1].is_some() {
                        write!(o, " vnoc [")?;
                        if let Some(vnoc) = vnoc[0] {
                            write!(o, "{vnoc}")?;
                        } else {
                            write!(o, "-")?;
                        }
                        write!(o, ", ")?;
                        if let Some(vnoc) = vnoc[1] {
                            write!(o, "{vnoc}")?;
                        } else {
                            write!(o, "-")?;
                        }
                        write!(o, "]")?;
                    }
                    writeln!(o, ";")?;
                }
                X5ioHNoc::Cap { scan_bli } => writeln!(o, "\t\t\thnoc cap scan_bli {scan_bli};")?,
            }

            if let Some(io) = region.io {
                writeln!(o, "\t\t\tio {side}", side = io.side)?;
                write!(o, "\t\t\toctad_bli ")?;
                let mut first = true;
                for col in io.octad_bli {
                    if !first {
                        write!(o, ", ")?;
                    }
                    first = false;
                    if let Some(col) = col {
                        write!(o, "{col}")?;
                    } else {
                        write!(o, "-")?;
                    }
                }
                writeln!(o, ";")?;
                match io.x5pll_bli {
                    X5ioPllBli::None => (),
                    X5ioPllBli::Secondary(col) => writeln!(o, "\t\t\tx5pll_bli secondary {col};")?,
                    X5ioPllBli::PrimaryA(col) => writeln!(o, "\t\t\tx5pll_bli primary_a {col};")?,
                    X5ioPllBli::PrimaryB(col) => writeln!(o, "\t\t\tx5pll_bli primary_b {col};")?,
                }
                if let Some(col) = io.bank_bli {
                    writeln!(o, "\t\t\tbank_bli {col};")?;
                }
                match io.scan_bli {
                    X5ioScanBli::A(col) => writeln!(o, "\t\t\tscan_bli a {col};")?,
                    X5ioScanBli::B(col) => writeln!(o, "\t\t\tscan_bli b {col};")?,
                    X5ioScanBli::C(col) => writeln!(o, "\t\t\tscan_bli c {col};")?,
                }
                if let Some(ddrmc) = io.ddrmc {
                    writeln!(
                        o,
                        "\t\t\tddrmc {} bli {} test_clk {};",
                        ddrmc.kind, ddrmc.bli, ddrmc.test_clk
                    )?;
                }
            }

            if let Some(sysmon_bli) = region.sysmon_bli {
                writeln!(o, "\t\t\tsysmon_bli {sysmon_bli};")?;
            }

            writeln!(o, "\t\t}}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct HbmRow {
    pub intf_bli: EntityVec<ColId, [ColId; 3]>,
    pub ms_bli: [ColId; 2],
}

impl HbmRow {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(o, "\t\tintf_bli {{")?;
        for (col, cols_bli) in &self.intf_bli {
            writeln!(
                o,
                "\t\t\t[{c0}, {c1}, {c2}], // {col}",
                c0 = cols_bli[0],
                c1 = cols_bli[1],
                c2 = cols_bli[2],
            )?;
        }
        writeln!(o, "\t\t}}")?;
        writeln!(
            o,
            "\t\tms_bli {c0}, {c1};",
            c0 = self.ms_bli[0],
            c1 = self.ms_bli[1]
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct MeRow {
    pub regions: EntityVec<HRegId, MeRegion>,
    pub ai: Option<AiArray>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct MeRegion {
    pub sysmon_bli: Option<[ColId; 2]>,
    pub clk_col: Option<ColId>,
    pub vnoc: MeVNocConn,
    pub connect_hsm: bool,
    pub pll_test: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum MeVNocConn {
    None,
    Port1([VNocConn; 2]),
    Port3([VNocConn; 2]),
}

impl MeRow {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        for (hr, region) in &self.regions {
            writeln!(o, "\t\tregion {hr} {{")?;

            if let Some(cols) = region.sysmon_bli {
                writeln!(o, "\t\t\tsysmon_bli {}, {};", cols[0], cols[1])?;
            }

            if let Some(col) = region.clk_col {
                writeln!(o, "\t\t\tclk_col {};", col)?;
            }

            match region.vnoc {
                MeVNocConn::None => (),
                MeVNocConn::Port1(conns) => writeln!(
                    o,
                    "\t\t\tvnoc 1 {vnoc0}, {vnoc1};",
                    vnoc0 = conns[0],
                    vnoc1 = conns[1],
                )?,
                MeVNocConn::Port3(conns) => writeln!(
                    o,
                    "\t\t\tvnoc 3 {vnoc0}, {vnoc1};",
                    vnoc0 = conns[0],
                    vnoc1 = conns[1],
                )?,
            }

            if region.connect_hsm {
                writeln!(o, "\t\t\tconnect_hsm;")?;
            }

            if region.pll_test {
                writeln!(o, "\t\t\tpll_test;")?;
            }

            writeln!(o, "\t\t}}")?;
        }
        if let Some(ref ai) = self.ai {
            ai.dump(o)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Me2Row {
    pub regions: EntityVec<HRegId, Me2Region>,
    pub scan_bli_w: ColId,
    pub scan_bli_e: ColId,
    pub ai: AiArray,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct Me2Region {
    pub sysmon_bli: [ColId; 2],
    pub clk_col: Option<ColId>,
    pub scan_bli: ColId,
    pub hnoc: Me2HNoc,
    pub connect_hsm: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum Me2HNoc {
    Plain,
    Cross,
    VNoc(ColId),
    VNocNcrb(ColId),
    VNoc4(ColId),
}

impl Me2Row {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(o, "\t\tscan_bli_w {};", self.scan_bli_w)?;
        writeln!(o, "\t\tscan_bli_e {};", self.scan_bli_e)?;
        for (hr, region) in &self.regions {
            writeln!(o, "\t\tregion {hr} {{")?;

            writeln!(
                o,
                "\t\t\tsysmon_bli {}, {};",
                region.sysmon_bli[0], region.sysmon_bli[1]
            )?;

            if let Some(col) = region.clk_col {
                writeln!(o, "\t\t\tclk_col {};", col)?;
            }

            if region.connect_hsm {
                writeln!(o, "\t\t\tconnect_hsm;")?;
            }

            writeln!(o, "\t\t\tscan_bli {};", region.scan_bli)?;

            match region.hnoc {
                Me2HNoc::Plain => writeln!(o, "\t\t\thnoc plain;")?,
                Me2HNoc::Cross => writeln!(o, "\t\t\thnoc cross;")?,
                Me2HNoc::VNoc(col) => writeln!(o, "\t\t\thnoc vnoc {col};")?,
                Me2HNoc::VNocNcrb(col) => writeln!(o, "\t\t\thnoc vnoc {col} + ncrb;")?,
                Me2HNoc::VNoc4(col) => writeln!(o, "\t\t\thnoc vnoc4 {col};")?,
            }

            writeln!(o, "\t\t}}")?;
        }
        self.ai.dump(o)?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum AiKind {
    Ai,
    AiMl,
    AiMl2,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct AiArray {
    pub kind: AiKind,
    pub columns: EntityVec<ColId, AiColumn>,
    pub rows: usize,
    pub rows_mem: usize,
    pub pll_bli: ColId,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct AiColumn {
    pub axis_bli: [Option<ColId>; 2],
    pub test_bli: Option<[ColId; 2]>,
    pub axi_clk_test: Option<HRegId>,
}

impl AiArray {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self.kind {
            AiKind::Ai => writeln!(o, "\t\tai {{")?,
            AiKind::AiMl => writeln!(o, "\t\taiml {{")?,
            AiKind::AiMl2 => writeln!(o, "\t\taiml2 {{")?,
        }
        writeln!(o, "\t\t\tcolumns {{")?;
        for (col, cd) in &self.columns {
            write!(o, "\t\t\t\t[")?;
            if let Some(col_bli) = cd.axis_bli[0] {
                write!(o, "{col_bli}")?;
            } else {
                write!(o, "-")?;
            }
            write!(o, ", ")?;
            if let Some(col_bli) = cd.axis_bli[1] {
                write!(o, "{col_bli}")?;
            } else {
                write!(o, "-")?;
            }
            write!(o, "]")?;
            if let Some([c0, c1]) = cd.test_bli {
                write!(o, " + test_bli [{c0}, {c1}]")?;
            }
            if let Some(reg) = cd.axi_clk_test {
                write!(o, " + axi_clk_test {reg}")?;
            }
            writeln!(o, ", // {col}")?;
        }
        writeln!(o, "\t\t\t}}")?;
        writeln!(o, "\t\t\trows {};", self.rows)?;
        if self.rows_mem != 0 {
            writeln!(o, "\t\t\trows_mem {};", self.rows_mem)?;
        }
        writeln!(o, "\t\t\tpll_bli {};", self.pll_bli)?;

        writeln!(o, "\t\t}}")?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub struct NicRow {
    pub blocks: EntityVec<ColId, NicBlock>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Encode, Decode)]
pub enum NicBlock {
    Cpm5NCdx,
    Cpm5NDpu {
        scan_bli: ColId,
    },
    Bli {
        nsu_bli: [ColId; 4],
        nmu_bli: [ColId; 3],
        scan_bli: ColId,
    },
    VNoc {
        scan_bli: ColId,
        ncrb_scan_bli: ColId,
        swap: bool,
    },
    HNicX {
        scan_bli: ColId,
    },
}

impl NicRow {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        for (col, &block) in &self.blocks {
            match block {
                NicBlock::Cpm5NCdx => writeln!(o, "\t\tcdx; // {col}")?,
                NicBlock::Cpm5NDpu { scan_bli } => {
                    writeln!(o, "\t\tdpu scan {scan_bli}; // {col}")?
                }
                NicBlock::Bli {
                    nsu_bli: [s0, s1, s2, s3],
                    nmu_bli: [m0, m1, m2],
                    scan_bli,
                } => writeln!(
                    o,
                    "\t\tbli nsu [{s0}, {s1}, {s2}, {s3}], nmu [{m0}, {m1}, {m2}], scan {scan_bli}; // {col}"
                )?,
                NicBlock::VNoc {
                    scan_bli,
                    ncrb_scan_bli,
                    swap,
                } => writeln!(
                    o,
                    "\t\tvnoc scan {scan_bli} ncrb_scan {ncrb_scan_bli}{swap}; // {col}",
                    swap = if swap { " swap" } else { "" }
                )?,
                NicBlock::HNicX { scan_bli } => writeln!(o, "\t\thnicx scan {scan_bli}; // {col}")?,
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode)]
pub enum DisabledPart {
    HardIp(DieId, ColId, RegId),
    HardIpSite(DieId, ColId, RegId),
    HdioDpll(DieId, ColId, RegId),
    Column(DieId, ColId),
    GtE(DieId, RegId),
    Region(DieId, RegId),
}

impl std::fmt::Display for DisabledPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisabledPart::HardIp(die, col, reg) => write!(f, "HARD_IP:{die}:{col}:{reg}"),
            DisabledPart::HardIpSite(die, col, reg) => write!(f, "HARD_IP_SITE:{die}:{col}:{reg}"),
            DisabledPart::HdioDpll(die, col, reg) => write!(f, "HDIO_DPLL:{die}:{col}:{reg}"),
            DisabledPart::Column(die, col) => write!(f, "COLUMN:{die}:{col}"),
            DisabledPart::GtE(die, reg) => write!(f, "GT_RIGHT:{die}:{reg}"),
            DisabledPart::Region(die, reg) => write!(f, "REGION:{die}:{reg}"),
        }
    }
}

impl Chip {
    pub const ROWS_PER_REG: usize = 48;

    pub fn row_to_reg(&self, row: RowId) -> RegId {
        RegId::from_idx(row.to_idx() / Self::ROWS_PER_REG)
    }

    pub fn row_reg_bot(&self, reg: RegId) -> RowId {
        RowId::from_idx(reg.to_idx() * Self::ROWS_PER_REG)
    }

    pub fn row_reg_hclk(&self, reg: RegId) -> RowId {
        let reg = if self.is_reg_n(reg) { reg } else { reg + 1 };
        self.row_reg_bot(reg)
    }

    pub fn is_reg_n(&self, reg: RegId) -> bool {
        reg.to_idx() == self.regs - 1 || reg.to_idx() % 2 == 1
    }

    pub fn is_reg_half(&self, reg: RegId) -> bool {
        reg.to_idx().is_multiple_of(2) && reg.to_idx() == self.regs - 1
    }

    pub fn regs(&self) -> EntityRange<RegId> {
        EntityRange::new(0, self.regs)
    }

    pub fn rows_pl(&self) -> EntityRange<RowId> {
        EntityRange::new(0, self.regs * Self::ROWS_PER_REG)
    }

    pub fn rows_ai(&self) -> EntityRange<RowId> {
        let start = self.regs * Self::ROWS_PER_REG + 2;
        let rows = if let Some(ai) = self.ai() {
            ai.rows + ai.rows_mem + 1
        } else {
            0
        };
        EntityRange::new(start, start + rows)
    }

    pub fn ai(&self) -> Option<&AiArray> {
        match self.hsr_n {
            HsrNKind::Me(ref me) => me.ai.as_ref(),
            HsrNKind::Me2(ref me) => Some(&me.ai),
            _ => None,
        }
    }

    pub fn row_hsr(&self, edge: DirV) -> RowId {
        match edge {
            DirV::S => RowId::from_idx(self.regs * Self::ROWS_PER_REG),
            DirV::N => RowId::from_idx(self.regs * Self::ROWS_PER_REG + 1),
        }
    }

    pub fn get_col_hard(&self, col: ColId) -> Option<&HardColumn> {
        self.cols_hard.iter().find(|x| x.col == col)
    }

    pub fn col_side(&self, col: ColId) -> DirH {
        if col.to_idx().is_multiple_of(2) {
            DirH::W
        } else {
            DirH::E
        }
    }

    pub fn col_edge(&self, edge: DirH) -> ColId {
        match edge {
            DirH::W => self.columns.first_id().unwrap(),
            DirH::E => self.columns.last_id().unwrap(),
        }
    }

    pub fn col_w(&self) -> ColId {
        self.col_edge(DirH::W)
    }

    pub fn col_e(&self) -> ColId {
        self.col_edge(DirH::E)
    }

    pub fn row_edge(&self, edge: DirV) -> RowId {
        match edge {
            DirV::S => self.rows_pl().first().unwrap(),
            DirV::N => self.rows_pl().last().unwrap(),
        }
    }

    pub fn row_s(&self) -> RowId {
        self.row_edge(DirV::S)
    }

    pub fn row_n(&self) -> RowId {
        self.row_edge(DirV::N)
    }

    pub fn x5io(&self, cell: CellCoord) -> Option<X5io> {
        if cell.row != self.row_hsr(DirV::S) {
            return None;
        }
        let HsrSKind::X5io(ref hsr) = self.hsr_s else {
            return None;
        };
        let reg = HRegId::from_idx(cell.col.to_idx() / 4);
        hsr.regions.get(reg)?.io
    }

    pub fn x5io_ddrmc(&self, cell: CellCoord) -> Option<X5ioDdrmc> {
        self.x5io(cell)?.ddrmc
    }

    pub fn x5io_to_w(&self, cell: CellCoord) -> Option<CellCoord> {
        match self.x5io(cell)?.side {
            DirH::W => Some(cell),
            DirH::E => Some(cell.delta(-4, 0)),
        }
    }

    pub fn x5io_to_e(&self, cell: CellCoord) -> Option<CellCoord> {
        if self.x5io(cell)?.side == DirH::E {
            Some(cell)
        } else if let cell_e = cell.delta(4, 0)
            && self.x5io(cell_e)?.side == DirH::E
        {
            Some(cell_e)
        } else {
            None
        }
    }

    pub fn x5io_to_ddrmc(&self, cell: CellCoord) -> Option<BelCoord> {
        let io = self.x5io(cell)?;
        let cell_w = self.x5io_to_w(cell)?;
        let cell_e = self.x5io_to_e(cell);

        // case 1: standing on it
        if self.x5io_ddrmc(cell_w).is_some() {
            return Some(cell_w.bel(bslots::DDRMC5));
        } else if let Some(cell_e) = cell_e
            && self.x5io_ddrmc(cell_e).is_some()
        {
            return Some(cell_e.bel(bslots::DDRMC5));
        }

        // case 2: non-DDRMC5X in neighbouring region
        if io.side == DirH::W
            && cell.col.to_idx() != 0
            && let cell_prev = cell.delta(-4, 0)
            && let Some(io_prev) = self.x5io(cell_prev)
            && io_prev.side == DirH::E
            && io_prev.ddrmc.is_some()
        {
            return Some(cell_prev.bel(bslots::DDRMC5));
        } else if io.side == DirH::E
            && let cell_next = cell.delta(4, 0)
            && let Some(io_next) = self.x5io(cell_next)
            && io_next.side == DirH::W
            && io_next.ddrmc.is_some()
        {
            return Some(cell_next.bel(bslots::DDRMC5));
        }

        None
    }
}

impl Chip {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(o, "\tfabric {};", self.fabric)?;
        writeln!(o, "\tps {};", self.ps)?;
        match self.hsr_s {
            HsrSKind::Xpio(ref xrow) => {
                writeln!(o, "\thsr_s xpio {{")?;
                xrow.dump(o)?;
                writeln!(o, "\t}}")?;
            }
            HsrSKind::X5io(ref xrow) => {
                writeln!(o, "\thsr_s x5io {{")?;
                xrow.dump(o)?;
                writeln!(o, "\t}}")?;
            }
            HsrSKind::Ssit => writeln!(o, "\thsr_s ssit;")?,
        }
        match self.hsr_n {
            HsrNKind::Xpio(ref xrow) => {
                writeln!(o, "\thsr_n xpio {{")?;
                xrow.dump(o)?;
                writeln!(o, "\t}}")?;
            }
            HsrNKind::Ssit => writeln!(o, "\thsr_n ssit;")?,
            HsrNKind::Ssit2 => writeln!(o, "\thsr_n ssit2;")?,
            HsrNKind::None => (),
            HsrNKind::Me(ref me) => {
                writeln!(o, "\thsr_n me {{")?;
                me.dump(o)?;
                writeln!(o, "\t}}")?;
            }
            HsrNKind::Me2(ref me2) => {
                writeln!(o, "\thsr_n me2 {{")?;
                me2.dump(o)?;
                writeln!(o, "\t}}")?;
            }
            HsrNKind::Hbm(ref hbm) => {
                writeln!(o, "\thsr_n hbm {{")?;
                hbm.dump(o)?;
                writeln!(o, "\t}}")?;
            }
            HsrNKind::Nic(ref nic) => {
                writeln!(o, "\thsr_n nic {{")?;
                nic.dump(o)?;
                writeln!(o, "\t}}")?;
            }
        }
        writeln!(o, "\tcolumns {{")?;
        for (col, cd) in &self.columns {
            if self.cols_vbrk.contains(&col) {
                writeln!(o, "\t\t// break")?;
            }
            if self.cols_cpipe.contains(&col) {
                writeln!(o, "\t\t// CPIPE")?;
            }
            if matches!(
                cd.kind,
                ColumnKind::ContDsp | ColumnKind::ContVNoc | ColumnKind::ContHard
            ) {
                continue;
            }
            if cd.kind == ColumnKind::Gt {
                let side = self.col_side(col);
                writeln!(o, "\t\tgt [")?;
                for (reg, kind) in &self.regs_gt[side] {
                    writeln!(o, "\t\t\t{kind}, // {y}", y = self.row_reg_bot(reg))?;
                }
                write!(o, "\t\t]")?;
            } else if cd.kind == ColumnKind::Hard {
                writeln!(o, "\t\thard [")?;
                for hc in &self.cols_hard {
                    if hc.col == col {
                        for (reg, kind) in &hc.regs {
                            writeln!(o, "\t\t\t{kind}, // {y}", y = self.row_reg_bot(reg))?;
                        }
                    }
                }
                write!(o, "\t\t]")?;
            } else {
                write!(o, "\t\t{}", cd.kind)?;
                if cd.has_bli[DirV::S] {
                    write!(o, " + bli_s")?;
                }
                if cd.has_bli[DirV::N] {
                    write!(o, " + bli_n")?;
                }
            }
            if matches!(
                cd.kind,
                ColumnKind::Dsp
                    | ColumnKind::Hard
                    | ColumnKind::VNoc
                    | ColumnKind::VNoc2
                    | ColumnKind::VNoc4
            ) {
                writeln!(o, ", // {col}-{cr}", cr = col + 1)?;
            } else {
                writeln!(o, ", // {col}")?;
            }
        }
        writeln!(o, "\t}}")?;
        if !self.cols_vbrk.is_empty() {
            writeln!(
                o,
                "\tcols_vbrk {};",
                self.cols_vbrk.iter().map(|x| x.to_string()).join(", ")
            )?;
        }
        if !self.cols_cpipe.is_empty() {
            writeln!(
                o,
                "\tcols_cpipe {};",
                self.cols_cpipe.iter().map(|x| x.to_string()).join(", ")
            )?;
        }

        writeln!(o, "\tregs {r};", r = self.regs)?;
        Ok(())
    }
}

impl Interposer {
    pub fn dump(&self, o: &mut dyn std::io::Write) -> std::io::Result<()> {
        match self.kind {
            InterposerKind::Single => writeln!(o, "\tsingle;")?,
            InterposerKind::Column => writeln!(o, "\tcolumn;")?,
            InterposerKind::MirrorSquare => writeln!(o, "\tmirror_square;")?,
        }
        for (die, die_sll_columns) in &self.sll_columns {
            if !die_sll_columns.is_empty() {
                writeln!(o, "\tsll_columns {die} {{")?;
                for &col in die_sll_columns {
                    writeln!(o, "\t\t{col},")?;
                }
                writeln!(o, "\t}}")?;
            }
        }
        Ok(())
    }
}
