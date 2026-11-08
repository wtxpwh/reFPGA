use std::collections::{BTreeMap, HashMap, btree_map};

use prjcombine_ecp::{
    chip::{
        Chip, ChipKind, IoGroupKind, MachXo2Kind, PllLoc, PllPad, RowKind, SpecialIoKey,
        SpecialLocKey,
    },
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    coord::{BelCoord, ColId, ColIdExt, DieIdExt, RowId},
    db::PinDir,
    dir::{Dir, DirH, DirV},
};
use prjcombine_re_lattice_naming::WireName;

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_kind_machxo2(&mut self) {
        let mut has_riologic = false;
        let mut has_slewrate = false;
        let mut has_icc = false;
        let mut has_esb = false;
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix] == "JCLKA_RIOLOGIC" {
                has_riologic = true;
            }
            if self.grid.strings[wn.suffix] == "JSLEWRATEA_PIO" {
                has_slewrate = true;
            }
            if self.grid.strings[wn.suffix] == "JASFCLKI_ESB" {
                has_esb = true;
            }
            if self.grid.strings[wn.suffix] == "JRXCLK_ICC" {
                has_icc = true;
            }
        }
        if has_icc {
            self.chip.kind = ChipKind::MachXo2(MachXo2Kind::MachNx);
        } else if has_esb {
            self.chip.kind = ChipKind::MachXo2(MachXo2Kind::MachXo3D);
        } else if has_slewrate {
            self.chip.kind = ChipKind::MachXo2(MachXo2Kind::MachXo3Lfp);
        } else if !has_riologic && self.chip.rows.values().any(|rd| rd.kind == RowKind::Ebr) {
            self.chip.kind = ChipKind::MachXo2(MachXo2Kind::MachXo3L);
        }
    }

    pub(super) fn fill_clk_machxo2(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix] == "DCMOUT6_DCM" {
                let cell = self.chip.xlat_rc_wire(wn).delta(1, 0);
                self.chip.col_clk = cell.col;
                self.chip.row_clk = cell.row;
                return;
            }
        }
        panic!("ummm where clocks");
    }

    pub(super) fn fill_ebr_machxo2(&mut self) {
        for &wn in self.grid.nodes.values() {
            if self.grid.strings[wn.suffix].ends_with("_EBR") {
                let cell = self.chip.xlat_rc_wire(wn);
                if cell.row == self.chip.row_n() {
                    self.chip.columns[cell.col].io[DirV::N] = IoGroupKind::Ebr;
                }
            }
        }
    }

    pub(super) fn fill_io_machxo2(&mut self) {
        self.fill_io(&[
            ("PIC_L0", IoGroupKind::Quad),
            ("PIC_L0_I3C", IoGroupKind::QuadI3c),
            ("PIC_LS0", IoGroupKind::Double),
            ("PIC_L0_VREF3", IoGroupKind::Quad),
            ("PIC_L0_VREF4", IoGroupKind::Quad),
            ("PIC_L0_VREF5", IoGroupKind::Quad),
            ("PIC_L1", IoGroupKind::Quad),
            ("PIC_L1_I3C", IoGroupKind::QuadI3c),
            ("PIC_L1_VREF3", IoGroupKind::Quad),
            ("PIC_L1_VREF4", IoGroupKind::Quad),
            ("PIC_L1_VREF5", IoGroupKind::Quad),
            ("PIC_L2", IoGroupKind::Quad),
            ("PIC_L2_VREF4", IoGroupKind::Quad),
            ("PIC_L2_VREF5", IoGroupKind::Quad),
            ("PIC_L3", IoGroupKind::Quad),
            ("PIC_L3_VREF4", IoGroupKind::Quad),
            ("PIC_L3_VREF5", IoGroupKind::Quad),
            ("LLC0PIC", IoGroupKind::Double),
            ("LLC0PIC_VREF3", IoGroupKind::Quad),
            ("LLC0PIC_I3C_VREF3", IoGroupKind::QuadI3c),
            ("LLC1PIC", IoGroupKind::Quad),
            ("LLC3PIC_VREF3", IoGroupKind::Quad),
            ("ULC3PIC", IoGroupKind::Quad),
            ("PIC_R0", IoGroupKind::Quad),
            ("PIC_R0_256", IoGroupKind::Quad),
            ("PIC_RS0", IoGroupKind::Double),
            ("PIC_RS0_256", IoGroupKind::Double),
            ("PIC_R1", IoGroupKind::Quad),
            ("PIC_R1_640", IoGroupKind::Quad),
            ("LRC0PIC", IoGroupKind::Quad),
            ("LRC1PIC1", IoGroupKind::Quad),
            ("LRC1PIC2", IoGroupKind::Quad),
            ("URC1PIC", IoGroupKind::Quad),
            ("PIC_B0", IoGroupKind::Quad),
            ("PIC_B0_256", IoGroupKind::Quad),
            ("PIC_BS0_256", IoGroupKind::Double),
            ("PIC_T0", IoGroupKind::Quad),
            ("PIC_T0_256", IoGroupKind::Quad),
            ("PIC_TS0", IoGroupKind::Double),
        ]);
        if self.chip.rows.len() == 11 {
            self.chip.columns[ColId::from_idx(8)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(10)].io[DirV::S] = IoGroupKind::QuadReverse;
        }
        if self.chip.rows.len() == 14 {
            self.chip.columns[ColId::from_idx(17)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(19)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(23)].io[DirV::N] = IoGroupKind::QuadReverse;
        }
        if self.chip.rows.len() == 21 {
            self.chip.columns[ColId::from_idx(5)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(11)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(17)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(25)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(11)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(18)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(19)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(21)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(22)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(25)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(27)].io[DirV::N] = IoGroupKind::QuadReverse;
        }
        if self.chip.rows.len() == 26 {
            self.chip.columns[ColId::from_idx(6)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(9)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(14)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(20)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(27)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(33)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(11)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(15)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(20)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(21)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(25)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(26)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(31)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(33)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(35)].io[DirV::N] = IoGroupKind::QuadReverse;
        }
        if self.chip.rows.len() == 30 {
            self.chip.columns[ColId::from_idx(7)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(12)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(20)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(26)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(31)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(40)].io[DirV::S] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(11)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(20)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(23)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(26)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(28)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(30)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(38)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(41)].io[DirV::N] = IoGroupKind::QuadReverse;
            self.chip.columns[ColId::from_idx(43)].io[DirV::N] = IoGroupKind::QuadReverse;
        }
    }

    pub(super) fn fill_io_banks_machxo2(&mut self) {
        let (r4, r5) = match self.chip.rows.len() {
            6 => (6, 6),
            7 => (7, 7),
            11 => (11, 11),
            14 => (4, 9),
            21 => (7, 13),
            26 => (9, 17),
            30 => (11, 18),
            _ => unreachable!(),
        };
        let row4 = RowId::from_idx(r4);
        let row5 = RowId::from_idx(r5);
        for (row, rd) in &mut self.chip.rows {
            if rd.io[DirH::W] != IoGroupKind::None {
                if row < row4 {
                    rd.bank.insert(DirH::W, 3);
                } else if row < row5 {
                    rd.bank.insert(DirH::W, 4);
                } else {
                    rd.bank.insert(DirH::W, 5);
                }
            }
            if rd.io[DirH::E] != IoGroupKind::None {
                rd.bank.insert(DirH::E, 1);
            }
        }
        for (_, cd) in &mut self.chip.columns {
            if cd.io[DirV::S] != IoGroupKind::None {
                cd.bank.insert(DirV::S, 2);
            }
            if cd.io[DirV::N] != IoGroupKind::None {
                cd.bank.insert(DirV::N, 0);
            }
        }
    }

    pub(super) fn fill_bc_machxo2(&mut self) {
        let mut xlat = BTreeMap::new();
        let suffix = self.grid.strings.get("JPGENI_BCPG").unwrap();
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.grid.nodes[wf];
            let wtn = self.grid.nodes[wt];
            if wtn.suffix == suffix {
                let cell_to = self.chip.xlat_rc_wire(wtn);
                let cell_from = self.chip.xlat_rc_wire(wfn);
                xlat.insert(cell_to, cell_from);
            }
        }
        let has_bank4 = self
            .chip
            .rows
            .values()
            .any(|rd| rd.bank.get(DirH::W) == Some(&4));
        let bcs = if has_bank4 {
            vec![
                (0, self.chip.col_clk - 1, self.chip.row_n()),
                (1, self.chip.col_e(), self.chip.row_clk),
                (2, self.chip.col_clk - 1, self.chip.row_s()),
                (3, self.chip.col_w(), self.chip.row_s()),
                (4, self.chip.col_w(), self.chip.row_clk),
                (5, self.chip.col_w(), self.chip.row_n()),
            ]
        } else {
            vec![
                (0, self.chip.col_clk - 1, self.chip.row_n()),
                (1, self.chip.col_e(), self.chip.row_clk),
                (2, self.chip.col_clk - 1, self.chip.row_s()),
                (3, self.chip.col_w(), self.chip.row_clk),
            ]
        };
        for (bank, col, row) in bcs {
            self.chip
                .special_loc
                .insert(SpecialLocKey::Bc(bank), xlat[&Chip::DIE.cell(col, row)]);
        }
    }

    pub(super) fn fill_dqsdll_machxo2(&mut self) {
        let Some(suffix) = self.grid.strings.get("JLOCK_DQSDLL") else {
            return;
        };

        for &wn in self.grid.nodes.values() {
            if wn.suffix == suffix {
                let cell = self.chip.xlat_rc_wire(wn);
                let key = if cell.row == self.chip.row_s() {
                    SpecialLocKey::DqsDll(Dir::S)
                } else {
                    SpecialLocKey::DqsDll(Dir::N)
                };
                self.chip.special_loc.insert(key, cell);
            }
        }
    }

    pub(super) fn gather_special_io_machxo2(&mut self) -> BTreeMap<WireName, (BelCoord, PinDir)> {
        let jpaddia_pio = self.grid.strings.get("JPADDIA_PIO").unwrap();
        let jpaddib_pio = self.grid.strings.get("JPADDIB_PIO").unwrap();
        let jpaddic_pio = self.grid.strings.get("JPADDIC_PIO").unwrap();
        let jpaddid_pio = self.grid.strings.get("JPADDID_PIO").unwrap();
        let jdia = self.grid.strings.get("JDIA").unwrap();
        let jdib = self.grid.strings.get("JDIB").unwrap();
        let jdic = self.grid.strings.get("JDIC").unwrap();
        let jdid = self.grid.strings.get("JDID").unwrap();
        let jpaddoa = self.grid.strings.get("JPADDOA").unwrap();
        let jpaddob = self.grid.strings.get("JPADDOB").unwrap();
        let jpaddoc = self.grid.strings.get("JPADDOC").unwrap();
        let jpaddod = self.grid.strings.get("JPADDOD").unwrap();
        let mut pad_nodes = HashMap::new();
        for (node, &wn) in &self.grid.nodes {
            let cell = self.chip.xlat_rc_wire(wn);
            let (bel, dir) = if wn.suffix == jpaddia_pio || wn.suffix == jdia {
                (bslots::IO[0], PinDir::Input)
            } else if wn.suffix == jpaddib_pio || wn.suffix == jdib {
                (bslots::IO[1], PinDir::Input)
            } else if wn.suffix == jpaddic_pio || wn.suffix == jdic {
                (bslots::IO[2], PinDir::Input)
            } else if wn.suffix == jpaddid_pio || wn.suffix == jdid {
                (bslots::IO[3], PinDir::Input)
            } else if wn.suffix == jpaddoa {
                (bslots::IO[0], PinDir::Output)
            } else if wn.suffix == jpaddob {
                (bslots::IO[1], PinDir::Output)
            } else if wn.suffix == jpaddoc {
                (bslots::IO[2], PinDir::Output)
            } else if wn.suffix == jpaddod {
                (bslots::IO[3], PinDir::Output)
            } else {
                continue;
            };
            pad_nodes.insert(node, (cell.bel(bel), dir));
        }
        let mut result = BTreeMap::new();
        for &(wf, wt) in self.grid.pips.keys() {
            if let Some(&io) = pad_nodes.get(&wf) {
                let wn = self.grid.nodes[wt];
                let suffix = self.grid.strings[wn.suffix].as_str();
                if matches!(
                    suffix,
                    "PADDIA_IOLOGIC"
                        | "PADDIB_IOLOGIC"
                        | "PADDIC_IOLOGIC"
                        | "PADDID_IOLOGIC"
                        | "PADDIA_RIOLOGIC"
                        | "PADDIB_RIOLOGIC"
                        | "PADDIC_RIOLOGIC"
                        | "PADDID_RIOLOGIC"
                        | "PADDIA_BIOLOGIC"
                        | "PADDIC_BSIOLOGIC"
                        | "PADDIA_TIOLOGIC"
                        | "PADDIC_TSIOLOGIC"
                        | "DIA_IOLOGIC"
                        | "DIB_IOLOGIC"
                        | "DIC_IOLOGIC"
                        | "DID_IOLOGIC"
                        | "DIA_RIOLOGIC"
                        | "DIB_RIOLOGIC"
                        | "DIC_RIOLOGIC"
                        | "DID_RIOLOGIC"
                        | "DIA_BIOLOGIC"
                        | "DIC_BSIOLOGIC"
                        | "DIA_TIOLOGIC"
                        | "DIC_TSIOLOGIC"
                        | "JDIA"
                        | "JDIB"
                        | "JDIC"
                        | "JDID"
                        | "JQ0"
                        | "JQ1"
                        | "JQ2"
                        | "JQ3"
                        | "PADDOA_PIO"
                        | "PADDOB_PIO"
                        | "PADDOC_PIO"
                        | "PADDOD_PIO"
                        | "JPADDOA_PIO"
                        | "JPADDOB_PIO"
                        | "JPADDOC_PIO"
                        | "JPADDOD_PIO"
                ) {
                    continue;
                }
                result.insert(wn, io);
            } else if let Some(&io) = pad_nodes.get(&wt) {
                let wn = self.grid.nodes[wf];
                let suffix = self.grid.strings[wn.suffix].as_str();
                if matches!(
                    suffix,
                    "JA0"
                        | "JA1"
                        | "JA2"
                        | "JA3"
                        | "INDDA_IOLOGIC"
                        | "INDDB_IOLOGIC"
                        | "INDDC_IOLOGIC"
                        | "INDDD_IOLOGIC"
                        | "INDDA_RIOLOGIC"
                        | "INDDB_RIOLOGIC"
                        | "INDDC_RIOLOGIC"
                        | "INDDD_RIOLOGIC"
                        | "INDDA_BIOLOGIC"
                        | "INDDC_BSIOLOGIC"
                        | "INDDA_TIOLOGIC"
                        | "INDDC_TSIOLOGIC"
                ) {
                    continue;
                }
                result.insert(wn, io);
            }
        }
        result
    }

    pub(super) fn fill_special_io_machxo2(&mut self) {
        fn get_edge(chip: &Chip, wn: WireName) -> Dir {
            let cell = chip.xlat_rc_wire(wn);
            chip.get_io_edge(cell)
        }
        fn get_pll(chip: &Chip, wn: WireName) -> PllLoc {
            let cell = chip.xlat_rc_wire(wn);
            PllLoc::new(DirV::N & cell.col.side_of(chip.col_clk), 0)
        }
        for (wn, (io, dir)) in self.gather_special_io_machxo2() {
            let suffix = self.grid.strings[wn.suffix].as_str();
            let key = match suffix {
                "JTCK_JTAG" => SpecialIoKey::Tck,
                "JTMS_JTAG" => SpecialIoKey::Tms,
                "JTDI_JTAG" => SpecialIoKey::Tdi,
                "JTDO_JTAG" => SpecialIoKey::Tdo,
                "JUFMSN_EFB" => SpecialIoKey::SpiPCsB,
                "JSPIMCSN0_EFB" => SpecialIoKey::SpiCCsB,
                "JSPIMOSII_EFB" => SpecialIoKey::D(0),
                "JSPIMOSIO_EFB" => SpecialIoKey::D(0),
                "JSPIMISOI_EFB" => SpecialIoKey::D(1),
                "JSPIMISOO_EFB" => SpecialIoKey::D(1),
                "JSPISCKI_EFB" => SpecialIoKey::Cclk,
                "JSPISCKO_EFB" => SpecialIoKey::Cclk,
                "JI2C1SCLI_EFB" => SpecialIoKey::D(2),
                "JI2C1SCLO_EFB" => SpecialIoKey::D(2),
                "JI2C1SDAI_EFB" => SpecialIoKey::D(3),
                "JI2C1SDAO_EFB" => SpecialIoKey::D(3),
                "JPCLKT00" => SpecialIoKey::Clock(Dir::N, 0),
                "JPCLKT01" => SpecialIoKey::Clock(Dir::N, 1),
                "JPCLKT10" => SpecialIoKey::Clock(Dir::E, 0),
                "JPCLKT20" => SpecialIoKey::Clock(Dir::S, 0),
                "JPCLKT21" => SpecialIoKey::Clock(Dir::S, 1),
                "JPCLKT30" => SpecialIoKey::Clock(Dir::W, 0),
                "JPCLKT31" => SpecialIoKey::Clock(Dir::W, 1),
                "JPCLKT32" => SpecialIoKey::Clock(Dir::W, 2),
                "JPADDI0" | "JCLKI0_DLLDEL" => SpecialIoKey::Clock(get_edge(&self.chip, wn), 0),
                "JPADDI1" | "JCLKI1_DLLDEL" => SpecialIoKey::Clock(get_edge(&self.chip, wn), 1),
                "JPADDI2" | "JCLKI2_DLLDEL" => SpecialIoKey::Clock(get_edge(&self.chip, wn), 2),
                "JREFCLK3" => SpecialIoKey::Pll(PllPad::PllIn0, get_pll(&self.chip, wn)),
                "JCLKFB0" => SpecialIoKey::Pll(PllPad::PllFb, get_pll(&self.chip, wn)),
                "JDQSI0_DQS" => SpecialIoKey::DqsE(0),
                "JDQSI1_DQS" => SpecialIoKey::DqsE(1),
                // redundant
                "JREFCLK4" | "JREFCLK5" | "JREFCLK6" | "JREFCLK7" => continue,
                _ => {
                    println!(
                        "{name}: WEIRD SPECIO: R{r}C{c}_{suffix} {io} {dir:?}",
                        name = self.name,
                        r = wn.r,
                        c = wn.c,
                        io = io.to_string(self.intdb)
                    );
                    continue;
                }
            };
            match self.chip.special_io.entry(key) {
                btree_map::Entry::Vacant(e) => {
                    e.insert(io);
                }
                btree_map::Entry::Occupied(e) => {
                    assert_eq!(*e.get(), io);
                }
            }
        }
    }
}
