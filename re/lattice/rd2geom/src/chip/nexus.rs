use std::collections::{BTreeMap, BTreeSet, HashMap, btree_map};

use prjcombine_ecp::{
    chip::{ChipKind, IoGroupKind, NexusKind, PllLoc, PllPad, SpecialIoKey, SpecialLocKey},
    defs::bslots,
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    dir::{Dir, DirV},
    grid::{BelCoord, ColId, ColIdExt, RowIdExt},
};
use prjcombine_re_lattice_naming::WireName;
use prjcombine_re_lattice_rawdump::PinDir;

use crate::chip::{ChipBuilder, ChipExt};

impl ChipBuilder<'_> {
    pub(super) fn fill_kind_nexus(&mut self) {
        let mut has_mipi = false;
        let mut has_pcs = false;
        let mut has_lmid = false;
        let mut has_pcie = false;
        for &wn in self.nodes.values() {
            if self.naming.strings[wn.suffix] == "JCLKHSBYTE0_TMID_CORE_TMIDMUX" {
                has_mipi = true;
            }
            if self.naming.strings[wn.suffix] == "JPCSCLKTX0_TMID_CORE_TMIDMUX" {
                has_pcs = true;
            }
            if self.naming.strings[wn.suffix] == "JHPFE0_LMID_CORE_LMIDMUX" {
                has_lmid = true;
            }
            if self.naming.strings[wn.suffix] == "JLMMIREADY_PCIE_CORE_PCIE0" {
                has_pcie = true;
            }
        }
        self.chip.kind = ChipKind::Nexus(if has_mipi {
            if has_pcie {
                NexusKind::SmallMipiPcie
            } else {
                NexusKind::SmallMipi
            }
        } else if has_pcs {
            NexusKind::LargePcs
        } else if !has_lmid {
            NexusKind::SmallHalfIo
        } else {
            NexusKind::SmallDenseIo
        });
    }

    pub(super) fn fill_clk_nexus(&mut self) {
        for &wn in self.nodes.values() {
            let suffix = self.naming.strings[wn.suffix].as_str();
            if matches!(suffix, "JDCSOUT_DCS_DCSIP" | "JDCSOUT_DCS_DCSIP0") {
                let cell = self.chip.xlat_rc_wire(wn).delta(1, 0);
                self.chip.col_clk = cell.col;
                self.chip.row_clk = cell.row;
                return;
            }
        }
        panic!("ummm where clocks");
    }

    pub(super) fn fill_pclk_nexus(&mut self) {
        let jclk0 = self.naming.strings.get("JCLK0").unwrap();
        let hpbx0000 = self.naming.strings.get("HPBX0000").unwrap();
        let vpsx0000 = self.naming.strings.get("VPSX0000").unwrap();
        let mut clks: BTreeMap<u8, Vec<ColId>> = BTreeMap::new();
        let mut side_e = BTreeSet::new();
        for &(nf, nt) in self.grid.pips.keys() {
            let wfn = self.nodes[nf];
            let wtn = self.nodes[nt];
            if wfn.suffix == hpbx0000 && wtn.suffix == jclk0 {
                let cell = self.chip.xlat_rc_wire(wtn);
                if cell.row == self.chip.row_clk {
                    clks.entry(wfn.c).or_default().push(cell.col);
                }
            }
            if wtn.suffix == hpbx0000 && wfn.suffix == vpsx0000 {
                side_e.insert(wtn.c);
            }
        }

        let mut clks_by_drv: BTreeMap<ColId, (Option<ColId>, Option<ColId>)> = BTreeMap::new();

        for (c, mut cols) in clks {
            cols.sort();
            let col_start = cols[0];
            for (i, &col) in cols.iter().enumerate() {
                assert_eq!(col, col_start + i);
            }
            let col_end = col_start + cols.len();
            if side_e.contains(&c) {
                clks_by_drv.entry(col_start).or_default().1 = Some(col_end);
            } else {
                clks_by_drv.entry(col_end).or_default().0 = Some(col_start);
            };
        }

        for (col_drv, (col_w, col_e)) in clks_by_drv {
            let col_w = col_w.unwrap_or(col_drv);
            let col_e = col_e.unwrap_or(col_drv);
            assert_eq!(self.chip.cols_pclk.next_id(), col_w);
            self.chip.cols_pclk.push(
                (col_e - col_w) as usize,
                if col_drv < self.chip.col_clk {
                    col_drv - 1
                } else {
                    col_drv
                },
            );
        }
        assert_eq!(self.chip.cols_pclk.len(), self.chip.columns.len());
    }

    pub(super) fn fill_special_loc_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let xlat_clk = |bank, idx| {
            let (edge, idx) = Self::xlat_nexus_clock(kind, bank, idx);
            SpecialLocKey::DllDel(edge, idx)
        };

        let mut mipi_wires = BTreeSet::new();
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.nodes[wf];
            let wtn = self.nodes[wt];
            let suffix = self.naming.strings[wfn.suffix].as_str();
            match suffix {
                "JLMMIREADY_DPHY_CORE_DPHY0" => {
                    mipi_wires.insert(wfn);
                }
                "JCLKOUT_DLLDEL_CORE_I0" => {
                    let cell = self.chip.xlat_rc_wire(wfn);
                    let key = match self.naming.strings[wtn.suffix].as_str() {
                        "JPCLKT10_TMID_CORE_TMIDMUX" => xlat_clk(1, 0),
                        "JPCLKT12_TMID_CORE_TMIDMUX" => xlat_clk(1, 2),
                        "JPCLKT10_RMID_CORE_RMIDMUX" => xlat_clk(1, 0),
                        "JPCLKT20_RMID_CORE_RMIDMUX" => xlat_clk(2, 0),
                        "JPCLKT30_RMID_CORE_RMIDMUX" => xlat_clk(3, 0),
                        "JPCLKT31_RMID_CORE_RMIDMUX" => xlat_clk(3, 1),
                        "JPCLKT60_LMID_CORE_LMIDMUX" => xlat_clk(6, 0),
                        "JPCLKT70_LMID_CORE_LMIDMUX" => xlat_clk(7, 0),
                        "JPCLKT80_LMID_CORE_LMIDMUX" => xlat_clk(8, 0),
                        "JPCLKT81_LMID_CORE_LMIDMUX" => xlat_clk(8, 1),
                        "JPCLKT20_BMID_CORE_BMIDMUX" => xlat_clk(2, 0),
                        "JPCLKT22_BMID_CORE_BMIDMUX" => xlat_clk(2, 2),
                        "JPCLKT30_BMID_CORE_BMIDMUX" => xlat_clk(3, 0),
                        "JPCLKT32_BMID_CORE_BMIDMUX" => xlat_clk(3, 2),
                        "JPCLKT40_BMID_CORE_BMIDMUX" => xlat_clk(4, 0),
                        "JPCLKT42_BMID_CORE_BMIDMUX" => xlat_clk(4, 2),
                        "JPCLKT50_BMID_CORE_BMIDMUX" => xlat_clk(5, 0),
                        "JPCLKT52_BMID_CORE_BMIDMUX" => xlat_clk(5, 2),
                        "JPCLKT60_BMID_CORE_BMIDMUX" => xlat_clk(6, 0),
                        "JPCLKT62_BMID_CORE_BMIDMUX" => xlat_clk(6, 2),
                        _ => continue,
                    };
                    self.chip.special_loc.insert(key, cell);
                }
                "JLMMIREADY_SGMIICDR_CORE" => {
                    if self.naming.strings[wtn.suffix] == "JF3" {
                        let mut cell = self.chip.xlat_rc_wire(wfn);
                        match kind {
                            NexusKind::SmallMipi => {
                                cell.col -= 1;
                            }
                            NexusKind::SmallMipiPcie
                            | NexusKind::LargePcs
                            | NexusKind::SmallDenseIo => {
                                cell.row -= 1;
                            }
                            _ => unreachable!(),
                        }
                        self.chip.special_loc.insert(SpecialLocKey::Cdr, cell);
                    }
                }
                "JLOCK_DDRDLL_CORE_I1" => {
                    let cell = self.chip.xlat_rc_wire(wfn);
                    let h = cell.col.side_of(self.chip.col_clk);
                    self.chip
                        .special_loc
                        .insert(SpecialLocKey::DdrDll(h & DirV::S), cell);
                }
                "JLMMIREADY_PLL_CORE_PLL_LMMI" => {
                    let cell = self.chip.xlat_rc_wire(wfn);
                    let h = cell.col.side_of(self.chip.col_clk);
                    let v = cell.row.side_of(self.chip.row_clk);
                    self.chip
                        .special_loc
                        .insert(SpecialLocKey::Pll(PllLoc::new(h & v, 0)), cell);
                }
                "JCALRDY_ADC_CORE_ADC0" => {
                    let cell = self.chip.xlat_rc_wire(wfn);
                    self.chip.special_loc.insert(SpecialLocKey::Adc, cell);
                }
                _ => (),
            }
        }
        for (i, wn) in mipi_wires.into_iter().enumerate() {
            let cell = self.chip.xlat_rc_wire(wn);
            self.chip.columns[cell.col].io_n = IoGroupKind::Mipi;
            self.chip.columns[cell.col].bank_n = Some(60 + i as u32);
        }
    }

    pub(super) fn fill_io_nexus(&mut self) {
        let tiles = BTreeMap::from_iter([
            ("SYSIO_B0_0", (false, 0)),
            ("SYSIO_B0_0_15K", (false, 0)),
            ("SYSIO_B1_0", (false, 1)),
            ("SYSIO_B1_0_C", (false, 1)),
            ("SYSIO_B1_0_15K", (false, 1)),
            ("SYSIO_B1_DED", (true, 1)),
            ("SYSIO_B1_DED_15K", (true, 1)),
            ("SYSIO_B2_0", (false, 2)),
            ("SYSIO_B2_0_C", (false, 2)),
            ("SYSIO_B3_0", (false, 3)),
            ("SYSIO_B3_0_DQS1", (false, 3)),
            ("SYSIO_B3_0_DQS3", (false, 3)),
            ("SYSIO_B3_0_DLY30_V18", (false, 3)),
            ("SYSIO_B3_0_15K_DQS32", (false, 3)),
            ("SYSIO_B3_0_ECLK_L", (false, 3)),
            ("SYSIO_B4_0", (false, 4)),
            ("SYSIO_B4_0_DQS1", (false, 4)),
            ("SYSIO_B4_0_DQS3", (false, 4)),
            ("SYSIO_B4_0_DQS1_MID", (false, 4)),
            ("SYSIO_B4_0_DQS3_MID", (false, 4)),
            ("SYSIO_B4_0_DLY50", (false, 4)),
            ("SYSIO_B4_0_DLY42", (false, 4)),
            ("SYSIO_B4_0_15K_DQS42", (false, 4)),
            ("SYSIO_B4_0_15K_V31", (false, 4)),
            ("SYSIO_B4_0_15K_BK4_V42", (false, 4)),
            ("PIC_B4_0_15K", (false, 4)),
            ("PIC_B4_0_15K_DLY32", (false, 4)),
            ("SYSIO_B5_0", (false, 5)),
            ("SYSIO_B5_0_DQS1", (false, 5)),
            ("SYSIO_B5_0_DQS3", (false, 5)),
            ("SYSIO_B5_0_15K_DQS52", (false, 5)),
            ("SYSIO_B6_0", (false, 6)),
            ("SYSIO_B6_0_C", (false, 6)),
            ("SYSIO_B6_0_DQS1", (false, 6)),
            ("SYSIO_B6_0_DQS3", (false, 6)),
            ("SYSIO_B6_0_ECLK_L", (false, 6)),
            ("SYSIO_B7_0", (false, 7)),
            ("SYSIO_B7_0_C", (false, 7)),
            // LFMXO5-25 / LIFCL-33
            ("SYSIO_B0", (false, 0)),
            ("SYSIO_B1", (false, 1)),
            ("SYSIO_B2", (false, 2)),
            ("SYSIO_B2_C", (false, 2)),
            ("SYSIO_B2_DED", (true, 2)),
            ("SYSIO_B3", (false, 3)),
            ("SYSIO_B3_C", (false, 3)),
            ("SYSIO_B4", (false, 4)),
            ("SYSIO_B4_C", (false, 4)),
            ("SYSIO_B5", (false, 5)),
            ("SYSIO_B7", (false, 7)),
            ("SYSIO_B7_C", (false, 7)),
            ("SYSIO_B8", (false, 8)),
            ("SYSIO_B8_C", (false, 8)),
            ("SYSIO_B9", (false, 9)),
            ("SYSIO_B9_C", (false, 9)),
        ]);
        for tile in &self.grid.tiles {
            let Some(&(is_jtag_en, bank)) = tiles.get(&tile.kind.as_str()) else {
                continue;
            };
            if let Some(rc) = tile.name.strip_prefix("CIB_R")
                && let Some((r, c)) = rc.split_once('C')
                && let Ok(r) = r.parse::<u8>()
                && let Ok(c) = c.parse::<u8>()
            {
                let cell = self.chip.xlat_rc(r, c);
                if is_jtag_en {
                    self.chip
                        .special_io
                        .insert(SpecialIoKey::JtagEn, cell.bel(bslots::IO[0]));
                }
                if c == 0 {
                    self.chip.rows[cell.row].io_w = IoGroupKind::Double;
                    self.chip.rows[cell.row].bank_w = Some(bank);
                } else if c == self.chip.columns.len() as u8 + 1 {
                    self.chip.rows[cell.row].io_e = IoGroupKind::Double;
                    self.chip.rows[cell.row].bank_e = Some(bank);
                } else if r == self.chip.rows.len() as u8 + 1 {
                    self.chip.columns[cell.col].io_s = IoGroupKind::Double;
                    self.chip.columns[cell.col].bank_s = Some(bank);
                } else if r == 0 {
                    self.chip.columns[cell.col].io_n = IoGroupKind::Double;
                    self.chip.columns[cell.col].bank_n = Some(bank);
                } else {
                    panic!("umm weird IO tile {}", tile.name);
                }
            } else {
                panic!("umm weird IO tile {}", tile.name);
            }
        }
        if self.chip.kind == ChipKind::Nexus(NexusKind::SmallDenseIo)
            && self.chip.columns.len() == 62
            && self.chip.rows.len() == 55
        {
            // uhhhhh. what. why. it's just.... missing from bittile map?
            for x in [44, 45, 47] {
                let col = ColId::from_idx(x);
                self.chip.columns[col].io_n = IoGroupKind::Double;
                self.chip.columns[col].bank_n = Some(1);
            }
        }
        for &(wf, wt) in self.grid.pips.keys() {
            let wfn = self.nodes[wf];
            let wtn = self.nodes[wt];
            if self.naming.strings[wtn.suffix] == "JTSDATA0_SIOLOGIC_CORE_IBASE_PIC_B" {
                let cell = self.chip.xlat_rc_wire(wtn);
                let kind = match self.naming.strings[wfn.suffix].as_str() {
                    "JCIBMUXOUTB4" => IoGroupKind::DoubleA,
                    "JCIBMUXOUTC0" => IoGroupKind::DoubleB,
                    "JCIBMUXOUTA0" => IoGroupKind::DoubleC,
                    s => {
                        println!("UMMMMMMMM {s}?");
                        continue;
                    }
                };
                match self.chip.get_io_edge(cell) {
                    Dir::S => self.chip.columns[cell.col].io_s = kind,
                    Dir::N => self.chip.columns[cell.col].io_n = kind,
                    Dir::W => self.chip.rows[cell.row].io_w = kind,
                    Dir::E => self.chip.rows[cell.row].io_e = kind,
                }
            }
        }
    }

    pub(super) fn gather_special_io_nexus(&mut self) -> Vec<(WireName, BelCoord, PinDir)> {
        let jpaddia = self.naming.strings.get("JPADDI_SEIO33_CORE_IOA").unwrap();
        let jpaddib = self.naming.strings.get("JPADDI_SEIO33_CORE_IOB").unwrap();
        let jpaddia_hp = self
            .naming
            .strings
            .get("JPADDI_DIFFIO18_CORE_IOA")
            .or(self.naming.strings.get("JPADDI_DIFFIO18A_CORE_IOA"))
            .unwrap();
        let jpaddib_hp = self
            .naming
            .strings
            .get("JPADDI_SEIO18_CORE_IOB")
            .or(self.naming.strings.get("JPADDI_SEIO18A_CORE_IOB"))
            .unwrap();
        let jpaddoa = self.naming.strings.get("JPADDO_SEIO33_CORE_IOA").unwrap();
        let jpaddob = self.naming.strings.get("JPADDO_SEIO33_CORE_IOB").unwrap();
        let jinadca_hp = self
            .naming
            .strings
            .get("JINADC_DIFFIO18_CORE_IOA")
            .or(self.naming.strings.get("JINADC_DIFFIO18A_CORE_IOA"))
            .unwrap();
        let mut pad_nodes = HashMap::new();
        for (node, &wn) in self.nodes {
            let cell = self.chip.xlat_rc_wire(wn);
            let (bel, dir) = if wn.suffix == jpaddia || wn.suffix == jpaddia_hp {
                (bslots::IO[0], PinDir::Input)
            } else if wn.suffix == jpaddib || wn.suffix == jpaddib_hp {
                (bslots::IO[1], PinDir::Input)
            } else if wn.suffix == jpaddoa {
                (bslots::IO[0], PinDir::Output)
            } else if wn.suffix == jpaddob {
                (bslots::IO[1], PinDir::Output)
            } else if wn.suffix == jinadca_hp {
                (bslots::IO[0], PinDir::Bidirectional)
            } else {
                continue;
            };
            pad_nodes.insert(node, (cell.bel(bel), dir));
        }
        let mut result = vec![];
        for &(wf, wt) in self.grid.pips.keys() {
            if let Some(&io) = pad_nodes.get(&wf) {
                let wn = self.nodes[wt];
                let suffix = self.naming.strings[wn.suffix].as_str();
                if matches!(
                    suffix,
                    "JDI_SIOLOGIC_CORE_IBASE_PIC_A"
                        | "JDI_SIOLOGIC_CORE_IBASE_PIC_B"
                        | "JDI_IOLOGIC_CORE_I_GEARING_PIC_TOP_A"
                        | "JDI_IOLOGIC_CORE_I_GEARING_PIC_TOP_B"
                        | "JF5"
                        | "JQ1"
                        | "JQ4"
                        | "JQ5"
                        | "JQ6"
                ) {
                    continue;
                }
                result.push((wn, io.0, io.1));
            } else if let Some(&io) = pad_nodes.get(&wt) {
                let wn = self.nodes[wf];
                let suffix = self.naming.strings[wn.suffix].as_str();
                if matches!(
                    suffix,
                    "JDOUT_SIOLOGIC_CORE_IBASE_PIC_A"
                        | "JDOUT_SIOLOGIC_CORE_IBASE_PIC_B"
                        | "JDOUT_IOLOGIC_CORE_I_GEARING_PIC_TOP_A"
                        | "JDOUT_IOLOGIC_CORE_I_GEARING_PIC_TOP_B"
                        | "JCIBMUXOUTA0"
                        | "JCIBMUXOUTA1"
                        | "JCIBMUXOUTB4"
                        | "JCIBMUXOUTC0"
                ) {
                    continue;
                }
                result.push((wn, io.0, io.1));
            }
        }
        result
    }

    fn xlat_nexus_clock(kind: NexusKind, bank: u32, idx: u8) -> (Dir, u8) {
        match kind {
            NexusKind::SmallMipi | NexusKind::SmallMipiPcie | NexusKind::LargePcs => match bank {
                0 => (Dir::N, idx),
                1 => (Dir::E, 3 + idx),
                2 => (Dir::E, idx),
                3 => (Dir::S, idx),
                4 => (Dir::S, 4 + idx),
                5 => (Dir::S, 8 + idx),
                6 => (Dir::W, idx),
                7 => (Dir::W, 3 + idx),
                _ => unreachable!(),
            },
            NexusKind::SmallHalfIo => match bank {
                1 => (Dir::N, idx),
                2 => (Dir::S, idx),
                3 => (Dir::S, 4 + idx),
                4 => (Dir::S, 8 + idx),
                _ => unreachable!(),
            },
            NexusKind::SmallDenseIo => match bank {
                0 => (Dir::N, idx),
                1 => (Dir::N, 2 + idx),
                2 => (Dir::E, 4 + idx),
                3 => (Dir::E, 2 + idx),
                4 => (Dir::E, idx),
                5 => (Dir::S, idx),
                6 => (Dir::S, 4 + idx),
                7 => (Dir::W, idx),
                8 => (Dir::W, 2 + idx),
                9 => (Dir::W, 4 + idx),
                _ => unreachable!(),
            },
        }
    }

    pub(super) fn fill_special_io_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let xlat_clk = |bank, idx| {
            let (edge, idx) = Self::xlat_nexus_clock(kind, bank, idx);
            SpecialIoKey::Clock(edge, idx)
        };
        let mut dlldel_xlat = BTreeMap::new();
        for (&loc, &cell) in &self.chip.special_loc {
            if let SpecialLocKey::DllDel(edge, idx) = loc {
                dlldel_xlat.insert(cell, SpecialIoKey::Clock(edge, idx));
            }
        }
        for (wn, io, dir) in self.gather_special_io_nexus() {
            let suffix = self.naming.strings[wn.suffix].as_str();
            let key = match suffix {
                "JEXTPINTN_PMU_CORE" => SpecialIoKey::PmuWakeupN,
                "JEXTCLK_PMU_CORE" | "JTCK_CONFIG_JTAG_CORE_CONFIG_JTAG" => SpecialIoKey::Cclk,
                "JTMS_CONFIG_JTAG_CORE_CONFIG_JTAG" => SpecialIoKey::SpiPCsB,
                "JTDI_CONFIG_JTAG_CORE_CONFIG_JTAG" => SpecialIoKey::D(0),
                "JTDO_CONFIG_JTAG_CORE_CONFIG_JTAG" => SpecialIoKey::D(1),
                "JALTSCLIN_I2CFIFO_CORE" | "JALTSCLOUT_I2CFIFO_CORE" => SpecialIoKey::D(2),
                "JALTSDAIN_I2CFIFO_CORE" | "JALTSDAOUT_I2CFIFO_CORE" => SpecialIoKey::D(3),
                "JDQSI_DQSBUF_CORE_I_DQS_TOP"
                | "JDQSI_DQSBUFA_CORE_I_DQS_TOP"
                | "JDQSI_DQSBUFB_CORE_I_DQS_TOP" => {
                    assert_eq!(io.row, self.chip.row_s());
                    assert_eq!(self.chip.columns[io.col].io_s, IoGroupKind::Double);
                    self.chip.columns[io.col].io_s = IoGroupKind::DoubleDqs;
                    continue;
                }
                "JREFCLK14_REFMUX_CORE_REFMUX" => {
                    let cell = self.chip.xlat_rc_wire(wn);
                    let h = cell.col.side_of(self.chip.col_clk);
                    let v = cell.row.side_of(self.chip.row_clk);
                    SpecialIoKey::Pll(PllPad::PllIn0, PllLoc::new(h & v, 0))
                }
                "JPCLKT00_TMID_CORE_TMIDMUX" => xlat_clk(0, 0),
                "JPCLKT01_TMID_CORE_TMIDMUX" => xlat_clk(0, 1),
                "JPCLKT10_TMID_CORE_TMIDMUX" => xlat_clk(1, 0),
                "JPCLKT11_TMID_CORE_TMIDMUX" => xlat_clk(1, 1),
                "JPCLKT13_TMID_CORE_TMIDMUX" => xlat_clk(1, 3),
                "JPCLKT21_BMID_CORE_BMIDMUX" | "JPCLKT1_ECLKBANK_CORE_ECLKBANK2" => xlat_clk(2, 1),
                "JPCLKT23_BMID_CORE_BMIDMUX" | "JPCLKT3_ECLKBANK_CORE_ECLKBANK2" => xlat_clk(2, 3),
                "JPCLKT31_BMID_CORE_BMIDMUX" | "JPCLKT1_ECLKBANK_CORE_ECLKBANK3" => xlat_clk(3, 1),
                "JPCLKT33_BMID_CORE_BMIDMUX" | "JPCLKT3_ECLKBANK_CORE_ECLKBANK3" => xlat_clk(3, 3),
                "JPCLKT41_BMID_CORE_BMIDMUX" | "JPCLKT1_ECLKBANK_CORE_ECLKBANK4" => xlat_clk(4, 1),
                "JPCLKT43_BMID_CORE_BMIDMUX" | "JPCLKT3_ECLKBANK_CORE_ECLKBANK4" => xlat_clk(4, 3),
                "JPCLKT51_BMID_CORE_BMIDMUX" | "JPCLKT1_ECLKBANK_CORE_ECLKBANK5" => xlat_clk(5, 1),
                "JPCLKT53_BMID_CORE_BMIDMUX" | "JPCLKT3_ECLKBANK_CORE_ECLKBANK5" => xlat_clk(5, 3),
                "JPCLKT61_BMID_CORE_BMIDMUX" | "JPCLKT1_ECLKBANK_CORE_ECLKBANK6" => xlat_clk(6, 1),
                "JPCLKT63_BMID_CORE_BMIDMUX" | "JPCLKT3_ECLKBANK_CORE_ECLKBANK6" => xlat_clk(6, 3),
                "JPCLKT61_LMID_CORE_LMIDMUX" => xlat_clk(6, 1),
                "JPCLKT62_LMID_CORE_LMIDMUX" => xlat_clk(6, 2),
                "JPCLKT70_LMID_CORE_LMIDMUX" => xlat_clk(7, 0),
                "JPCLKT71_LMID_CORE_LMIDMUX" => xlat_clk(7, 1),
                "JPCLKT72_LMID_CORE_LMIDMUX" => xlat_clk(7, 2),
                "JPCLKT90_LMID_CORE_LMIDMUX" => xlat_clk(9, 0),
                "JPCLKT91_LMID_CORE_LMIDMUX" => xlat_clk(9, 1),
                "JPCLKT92_LMID_CORE_LMIDMUX" => xlat_clk(9, 2),
                "JPCLKT11_RMID_CORE_RMIDMUX" => xlat_clk(1, 1),
                "JPCLKT12_RMID_CORE_RMIDMUX" => xlat_clk(1, 2),
                "JPCLKT20_RMID_CORE_RMIDMUX" => xlat_clk(2, 0),
                "JPCLKT21_RMID_CORE_RMIDMUX" => xlat_clk(2, 1),
                "JPCLKT22_RMID_CORE_RMIDMUX" => xlat_clk(2, 2),
                "JPCLKT40_RMID_CORE_RMIDMUX" => xlat_clk(4, 0),
                "JPCLKT41_RMID_CORE_RMIDMUX" => xlat_clk(4, 1),
                "JCLKIN_DLLDEL_CORE_I0" => {
                    // uhhh.
                    let key = dlldel_xlat[&self.chip.xlat_rc_wire(wn)];
                    if key == SpecialIoKey::Clock(Dir::N, 2)
                        && kind == NexusKind::SmallHalfIo
                        && io.slot != bslots::IO[1]
                    {
                        continue;
                    }
                    key
                }
                "JSGMIIIN_SGMIICDR_CORE" => {
                    let cell = self.chip.xlat_rc_wire(wn);
                    let cdr = self.chip.special_loc[&SpecialLocKey::Cdr];
                    let cdr0 = match kind {
                        NexusKind::SmallMipi => cdr.delta(1, 0),
                        NexusKind::SmallMipiPcie | NexusKind::LargePcs => cdr.delta(0, 2),
                        NexusKind::SmallHalfIo => unreachable!(),
                        NexusKind::SmallDenseIo => cdr.delta(0, 3),
                    };
                    SpecialIoKey::CdrRx(if cell == cdr0 { 0 } else { 1 })
                }
                // redundant
                "JREFCLK13_REFMUX_CORE_REFMUX"
                | "JREFCLK23_REFMUX_CORE_REFMUX"
                | "JREFCLK15_REFMUX_CORE_REFMUX"
                | "JREFCLK25_REFMUX_CORE_REFMUX"
                | "JREFCLK16_REFMUX_CORE_REFMUX"
                | "JREFCLK26_REFMUX_CORE_REFMUX"
                | "JCLKIN1_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0"
                | "JCLKIN2_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0"
                | "JCLKIN3_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0"
                | "JCLKIN4_DPHY_REFCLOCK_MUX_CORE_DPHY_REFCLOCK_MUX0" => continue,
                _ if let Some(rest) = suffix.strip_suffix("_ADC_CORE_ADC0")
                    && let Some(idx) = rest.strip_prefix("JGPIOP")
                    && let Ok(idx) = idx.parse::<u8>() =>
                {
                    SpecialIoKey::AdcCh(idx)
                }
                _ if let Some(rest) = suffix.strip_suffix("IP_ADC_CORE_ADC0")
                    && let Some(idx) = rest.strip_prefix("JCOMP")
                    && let Ok(idx) = idx.parse::<u8>() =>
                {
                    SpecialIoKey::AdcCompI(idx)
                }
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
