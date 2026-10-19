use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, IoGroupKind, SpecialLocKey},
    defs::{self, bcls, bslots, tcls, wires},
};
use prjcombine_entity::EntityIdArith;
use prjcombine_interconnect::{
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirH, DirHV, DirMap, DirV},
    grid::DieIdExt,
};
use prjcombine_re_lattice_naming::WireName;

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn process_pclk_ecp5(&mut self) -> BTreeMap<(DirHV, usize), WireName> {
        let mut hprx = BTreeMap::new();
        for i in 0..16 {
            let pclk = wires::PCLK[i];
            for (col, cd) in &self.chip.columns {
                if !cd.pclk_drive {
                    continue;
                }
                let h = if col < self.chip.col_clk {
                    DirH::W
                } else {
                    DirH::E
                };
                for (v, row_s, row_n) in [
                    (DirV::S, self.chip.row_s(), self.chip.row_clk),
                    (DirV::N, self.chip.row_clk, self.chip.row_n() + 1),
                ] {
                    let mut vptx = None;
                    for col_tgt in [col - 1, col] {
                        for row in row_s.range(row_n) {
                            let cell = Chip::DIE.cell(col_tgt, row);
                            let pclk = self.naming.interconnect[&cell.wire(pclk)];
                            let cur_vptx = self.claim_single_in(pclk);
                            if vptx.is_none() {
                                self.claim_node(cur_vptx);
                                vptx = Some(cur_vptx);
                            } else {
                                assert_eq!(vptx, Some(cur_vptx));
                            }
                            if col == col_tgt {
                                self.add_bel_wire_no_claim(
                                    cell.bel(bslots::INT),
                                    format!("PCLK{i}_IN"),
                                    cur_vptx,
                                );
                            }
                        }
                    }
                    let cur_hprx = self.claim_single_in(vptx.unwrap());
                    match hprx.entry((DirHV { h, v }, i)) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(cur_hprx);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), cur_hprx);
                        }
                    }
                }
            }
        }
        hprx
    }

    pub(super) fn process_clk_edge_ecp5(&mut self) {
        for (edge, col, row) in [
            (Dir::W, self.chip.col_w(), self.chip.row_clk),
            (Dir::E, self.chip.col_e(), self.chip.row_clk),
            (Dir::S, self.chip.col_clk, self.chip.row_s()),
            (Dir::N, self.chip.col_clk, self.chip.row_n()),
        ] {
            let cell_tile = Chip::DIE.cell(col, row);
            let cell = match edge {
                Dir::H(_) => cell_tile,
                Dir::V(_) => cell_tile.delta(-1, 0),
            };
            let lrbt = match edge {
                Dir::W => 'L',
                Dir::E => 'R',
                Dir::S => 'B',
                Dir::N => 'T',
            };
            let num_dcc = match edge {
                Dir::H(_) => 14,
                Dir::S => 16,
                Dir::N => 12,
            };
            let bcrd_int = cell_tile.bel(bslots::CLK_INT);
            let ci_base = match edge {
                Dir::H(_) => 6,
                Dir::V(_) => 8,
            };

            {
                let num_pins = if edge == Dir::S { 4 } else { 3 };
                let bcrd = cell_tile.bel(bslots::CLKTEST);
                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("CLKTEST_{lrbt}MID"));
                for i in 0..24 {
                    if i < num_pins {
                        bel.input_xlat_rc(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
                    } else {
                        bel.add_wire_rc(format!("TESTIN{i}"), format!("JTESTIN{i}_CLKTEST"));
                    }
                }
            }

            {
                let mut ex = self.extract_switchbox(bcrd_int);
                ex.cell(cell);
                for i in 0..num_dcc {
                    ex.output_bind_rc(
                        format!("{lrbt}DCC{i}CLKI"),
                        wires::IMUX_DCC_CLKI[i].cell(ci_base),
                    );
                }

                let plls = match edge {
                    Dir::W => [DirHV::SW, DirHV::NW].as_slice(),
                    Dir::E => [DirHV::SE, DirHV::NE].as_slice(),
                    Dir::N => [DirHV::NW, DirHV::NE].as_slice(),
                    Dir::S => [].as_slice(),
                };
                for &hv in plls {
                    if ex.ctx.chip.try_bel_pll(hv, 0).is_some() {
                        let corner = match hv {
                            DirHV::SW => "LL",
                            DirHV::SE => "LR",
                            DirHV::NW => "UL",
                            DirHV::NE => "UR",
                        };
                        for pin in ["CLKOP", "CLKOS", "CLKOS2", "CLKOS3"] {
                            ex.input_xlat_rc(format!("J{corner}CPLL0{pin}"));
                        }
                    }
                }

                match edge {
                    Dir::N => {
                        ex.input_xlat_rc("JPCLKT00")
                            .input_xlat_rc("JPCLKT01")
                            .input_xlat_rc("JPCLKT10")
                            .input_xlat_rc("JPCLKT11");
                    }
                    Dir::W => {
                        ex.input_xlat_rc("JPCLKT60")
                            .input_xlat_rc("JPCLKT61")
                            .input_xlat_rc("JPCLKT70")
                            .input_xlat_rc("JPCLKT71")
                            .input_xlat_rc("JLCDIVX0")
                            .input_xlat_rc("JLCDIVX1")
                            .input_xlat_rc("JOSC");
                    }
                    Dir::E => {
                        ex.input_xlat_rc("JPCLKT30")
                            .input_xlat_rc("JPCLKT31")
                            .input_xlat_rc("JPCLKT20")
                            .input_xlat_rc("JPCLKT21")
                            .input_xlat_rc("JRCDIVX0")
                            .input_xlat_rc("JRCDIVX1");
                    }
                    Dir::S => {
                        ex.input_xlat_rc("JSEDCLKOUT").input_xlat_rc("JJTCK");
                        if ex.ctx.skip_serdes {
                            ex.missing_input(wires::OUT_CLKDIV_CDIVX[0].cell(ci_base))
                                .missing_input(wires::OUT_CLKDIV_CDIVX[1].cell(ci_base))
                                .missing_input(wires::OUT_CLKDIV_CDIV1[0].cell(ci_base))
                                .missing_input(wires::OUT_CLKDIV_CDIV1[1].cell(ci_base));
                            for cslot in defs::ecp5::tcls::CLK_S::SERDES {
                                ex.missing_input(cslot.wire(wires::OUT_SERDES_FF_RX_PCLK[0]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_FF_RX_PCLK[1]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_FF_TX_PCLK[0]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_FF_TX_PCLK[1]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_REFCLKO));
                            }
                        } else {
                            ex.input_xlat_rc("PCSCDIVX0")
                                .input_xlat_rc("PCSCDIVX1")
                                .input_xlat_rc("PCSCDIV10")
                                .input_xlat_rc("PCSCDIV11");
                            let serdes_cnt = ex
                                .ctx
                                .chip
                                .columns
                                .values()
                                .filter(|cd| cd.io_s == IoGroupKind::Serdes)
                                .count();
                            for dual in 0..serdes_cnt {
                                let cslot = defs::ecp5::tcls::CLK_S::SERDES[dual];
                                let abcd = ['A', 'B'][dual];
                                for ch in 0..2 {
                                    for pin in ["RXCLK", "TXCLK"] {
                                        ex.input_xlat_rc_nudge(
                                            format!("JPCS{abcd}{pin}{ch}"),
                                            cslot,
                                        );
                                    }
                                }
                                ex.input_xlat_rc_nudge(format!("JSERDESREFCLK{dual}"), cslot);
                            }
                            for dual in serdes_cnt..2 {
                                let cslot = defs::ecp5::tcls::CLK_S::SERDES[dual];
                                ex.missing_input(cslot.wire(wires::OUT_SERDES_FF_RX_PCLK[0]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_FF_RX_PCLK[1]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_FF_TX_PCLK[0]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_FF_TX_PCLK[1]))
                                    .missing_input(cslot.wire(wires::OUT_SERDES_REFCLKO));
                            }
                        }
                    }
                }

                let int_names = match edge {
                    Dir::W => [
                        "JLLQPCLKCIB0",
                        "JLLQPCLKCIB1",
                        "JULQPCLKCIB0",
                        "JULQPCLKCIB1",
                        "JLLMPCLKCIB0",
                        "JLLMPCLKCIB2",
                        "JULMPCLKCIB0",
                        "JULMPCLKCIB2",
                    ],
                    Dir::E => [
                        "JLRQPCLKCIB0",
                        "JLRQPCLKCIB1",
                        "JURQPCLKCIB0",
                        "JURQPCLKCIB1",
                        "JLRMPCLKCIB0",
                        "JLRMPCLKCIB2",
                        "JURMPCLKCIB0",
                        "JURMPCLKCIB2",
                    ],
                    Dir::S => [
                        "JBLQPCLKCIB0",
                        "JBLQPCLKCIB1",
                        "JBRQPCLKCIB0",
                        "JBRQPCLKCIB1",
                        "JLLMPCLKCIB1",
                        "JLLMPCLKCIB3",
                        "JLRMPCLKCIB1",
                        "JLRMPCLKCIB3",
                    ],
                    Dir::N => [
                        "JTLQPCLKCIB0",
                        "JTLQPCLKCIB1",
                        "JTRQPCLKCIB0",
                        "JTRQPCLKCIB1",
                        "JULMPCLKCIB1",
                        "JULMPCLKCIB3",
                        "JURMPCLKCIB1",
                        "JURMPCLKCIB3",
                    ],
                };
                for (i, name) in int_names.into_iter().enumerate() {
                    if ex
                        .ctx
                        .chip
                        .special_loc
                        .contains_key(&SpecialLocKey::PclkIn(edge, i as u8))
                    {
                        ex.input_xlat_rc(name);
                    }
                }
            }

            for i in 0..num_dcc {
                let bcrd_dcc = cell_tile.bel(bslots::DCC[i]);
                self.extract_bel_cell(bcrd_dcc, cell)
                    .name(format!("DCC_{lrbt}{i}"))
                    .input_xlat_rc(bcls::DCC::CE, format!("JCE_{lrbt}DCC{i}"))
                    .input_xlat_rc(bcls::DCC::CLKI, format!("CLKI_{lrbt}DCC{i}"))
                    .output_bind_step_rc_mode(
                        bcls::DCC::CLKO,
                        format!("CLKO_{lrbt}DCC{i}"),
                        RcMode::Base,
                        wires::OUT_DCC_CLKO[i].cell(ci_base),
                    )
                    .claim_pip("CLKO", "CLKI");
            }
        }
    }

    pub(super) fn process_clk_root_ecp5(&mut self, hprx: BTreeMap<(DirHV, usize), WireName>) {
        let tcrd = self.edev.bel_tile(self.chip.bel_clk_root());
        let cell_tile = self.chip.bel_clk_root().cell;
        let cell = cell_tile.delta(-1, 0);

        let (cslot, cslot_sw, cslot_se, cslot_nw, cslot_ne) = match self.edev[tcrd].class {
            tcls::CLK_ROOT_S => (
                defs::ecp5::tcls::CLK_ROOT_S::CENTER,
                defs::ecp5::tcls::CLK_ROOT_S::SW,
                defs::ecp5::tcls::CLK_ROOT_S::SE,
                defs::ecp5::tcls::CLK_ROOT_S::NW,
                defs::ecp5::tcls::CLK_ROOT_S::NE,
            ),
            tcls::CLK_ROOT_L => (
                defs::ecp5::tcls::CLK_ROOT_L::CENTER,
                defs::ecp5::tcls::CLK_ROOT_L::SW,
                defs::ecp5::tcls::CLK_ROOT_L::SE,
                defs::ecp5::tcls::CLK_ROOT_L::NW,
                defs::ecp5::tcls::CLK_ROOT_L::NE,
            ),
            _ => unreachable!(),
        };

        let cell_edge = DirMap::from_fn(|edge| match edge {
            Dir::H(edge) => cell_tile.with_col(self.chip.col_edge(edge)),
            Dir::V(edge) => cell_tile.with_row(self.chip.row_edge(edge)),
        });

        {
            let bcrd = cell_tile.bel(bslots::CLKTEST);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name("CLKTEST_CEN");
            for i in 0..24 {
                bel.input_xlat_rc(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
            }
        }

        {
            let bcrd = cell_tile.bel(bslots::CLK_INT);
            let mut ex = self.extract_switchbox(bcrd);
            ex.cell(cell)
                .permabuf_output_xlat_rc("JBRGECLK0")
                .permabuf_output_xlat_rc("JBRGECLK1")
                .input_xlat_rc("JRECLK0")
                .input_xlat_rc("JLECLK1")
                .output_bind_rc("DCS0CLK0", cslot.wire(wires::IMUX_DCS_CLKA[0]))
                .output_bind_rc("DCS0CLK1", cslot.wire(wires::IMUX_DCS_CLKB[0]))
                .output_bind_rc("DCS1CLK0", cslot.wire(wires::IMUX_DCS_CLKA[1]))
                .output_bind_rc("DCS1CLK1", cslot.wire(wires::IMUX_DCS_CLKB[1]))
                .input_bind_rc("DCS0", cslot.wire(wires::OUT_DCS_DCSOUT[0]))
                .input_bind_rc("DCS1", cslot.wire(wires::OUT_DCS_DCSOUT[1]))
                .input_bind_rc("LLCPCLKCIB0", cslot_sw.wire(wires::OUT_DCC_CLKO[0]))
                .input_bind_rc("LRCPCLKCIB0", cslot_se.wire(wires::OUT_DCC_CLKO[0]))
                .input_bind_rc("ULCPCLKCIB0", cslot_nw.wire(wires::OUT_DCC_CLKO[0]))
                .input_bind_rc("URCPCLKCIB0", cslot_ne.wire(wires::OUT_DCC_CLKO[0]));
            for (edge, num_dcc) in [(Dir::W, 14), (Dir::E, 14), (Dir::S, 16), (Dir::N, 12)] {
                for j in 0..num_dcc {
                    let bcrd_dcc = cell_edge[edge].bel(bslots::DCC[j]);
                    let wire_dcc = ex.ctx.naming.bel_wire(bcrd_dcc, "CLKO_OUT");
                    ex.input_xlat(wire_dcc);
                }
            }
            for (cslot, hv) in [
                (cslot_sw, DirHV::SW),
                (cslot_se, DirHV::SE),
                (cslot_nw, DirHV::NW),
                (cslot_ne, DirHV::NE),
            ] {
                for i in 0..16 {
                    let wire = cslot.wire(wires::PCLK[i]);
                    let pclk_hprx = hprx[&(hv, i)];
                    ex.ctx
                        .add_bel_wire(bcrd, format!("PCLK{i}_{hv}_OUT"), pclk_hprx);
                    let pclk = ex.ctx.claim_single_in(pclk_hprx);
                    ex.output_bind(pclk, wire);
                }
            }
        }

        for (quad, slot) in [
            ("BL", bslots::DCC_SW[0]),
            ("BR", bslots::DCC_SE[0]),
            ("TL", bslots::DCC_NW[0]),
            ("TR", bslots::DCC_NE[0]),
        ] {
            let bcrd = cell_tile.bel(slot);
            self.extract_bel_cell(bcrd, cell)
                .name(format!("DCC_{quad}"))
                .input_xlat_rc(bcls::DCC::CE, format!("JCE_DCC{quad}"))
                .input_xlat_rc(bcls::DCC::CLKI, format!("JCLKI_DCC{quad}"))
                .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO_DCC{quad}"))
                .claim_pip("CLKO", "CLKI");
        }

        for i in 0..2 {
            let bcrd = cell_tile.bel(bslots::DCS[i]);
            let mut bel = self.extract_bel_cell(bcrd, cell);
            bel.name(format!("DCS{i}"))
                .input_xlat_rc(bcls::DCS::MODESEL, format!("JMODESEL_DCS{i}"))
                .output_xlat_rc(bcls::DCS::DCSOUT, format!("DCSOUT_DCS{i}"));
            for j in 0..2 {
                bel.input_xlat_rc(bcls::DCS::SEL[j], format!("JSEL{j}_DCS{i}"))
                    .input_xlat_rc(bcls::DCS::CLK[j], format!("CLK{j}_DCS{i}"))
                    .claim_pip("DCSOUT", format!("CLK{j}"));
            }
        }
    }
}
