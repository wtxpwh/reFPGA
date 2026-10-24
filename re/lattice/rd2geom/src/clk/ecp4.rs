use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, SpecialLocKey},
    defs::{self, bcls, bslots, rslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::{CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirHV, DirMap},
};

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn process_pclk_ecp4(&mut self) {
        let mut vptx = BTreeMap::new();
        for i in 0..20 {
            let pclk = wires::PCLK[i];
            for cell in self.edev.die_cells(Chip::DIE) {
                if !self.edev.has_bel(cell.bel(bslots::INT)) {
                    continue;
                }
                let pclk = self.naming.interconnect[&cell.wire(pclk)];
                let cur_vptx = self.claim_single_in(pclk);
                let cell_tag = self.edev[cell].region_root[rslots::PCLK_LEAF];
                match vptx.entry((cell_tag, i)) {
                    btree_map::Entry::Vacant(e) => {
                        e.insert(cur_vptx);
                    }
                    btree_map::Entry::Occupied(e) => {
                        assert_eq!(*e.get(), cur_vptx);
                    }
                }
            }
        }
        for &tcrd in &self.edev.tile_index[tcls::PCLK_DRIVER] {
            let bcrd = tcrd.bel(bslots::PCLK_DRIVER);
            let cslot_s = defs::ecp4::tcls::PCLK_DRIVER::S;
            let cslot_n = defs::ecp4::tcls::PCLK_DRIVER::N;
            let cell_s = self.edev[tcrd].cells[cslot_s];
            let cell_n = self.edev[tcrd].cells[cslot_n];
            let mut ex = self.extract_switchbox(bcrd);
            for i in 0..20 {
                ex.progbuf_output_bind(vptx[&(cell_s, i)], cslot_s.wire(wires::PCLK[i]))
                    .progbuf_output_bind(vptx[&(cell_n, i)], cslot_n.wire(wires::PCLK[i]))
                    .input_int(wires::PCLK_ROW[i].cell(0));
            }
        }

        for tcid in [tcls::PCLK_ROW_DRIVER_W, tcls::PCLK_ROW_DRIVER_E] {
            for &tcrd in &self.edev.tile_index[tcid] {
                let bcrd = tcrd.bel(bslots::PCLK_ROW_DRIVER);
                let mut ex = self.extract_switchbox(bcrd);
                for i in 0..20 {
                    ex.progbuf_output_int(wires::PCLK_ROW[i].cell(0))
                        .input_int(wires::PCLK_ROOT[i].cell(0));
                }
            }
        }
    }

    pub(super) fn process_clk_edge_ecp4(&mut self) {
        for edge in Dir::DIRS {
            let cell_tile = self.chip.clk_emid(edge);
            let cell = match edge {
                Dir::H(_) => cell_tile,
                Dir::V(_) => cell_tile.step(Dir::W, 1),
            };
            let lrbt = match edge {
                Dir::W => 'L',
                Dir::E => 'R',
                Dir::S => 'B',
                Dir::N => 'T',
            };
            let num_dcc = match edge {
                Dir::H(_) => 14,
                Dir::V(_) => 16,
            };
            let bcrd_int = cell_tile.bel(bslots::CLK_INT);
            let tcid = self.edev[self.edev.bel_tile(bcrd_int)].class;
            let ci_base = match tcid {
                tcls::CLK_N_L => 9,
                tcls::CLK_S_S | tcls::CLK_S_M | tcls::CLK_S_L => 1,
                _ => 5,
            };

            {
                let bcrd = cell_tile.bel(bslots::CLKTEST);
                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("CLKTEST_{lrbt}MID"));
                for i in 0..24 {
                    if i < 4 {
                        bel.input_xlat_rc(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
                    } else {
                        bel.add_wire_rc(format!("TESTIN{i}"), format!("JTESTIN{i}_CLKTEST"));
                    }
                }
            }

            let num_serdes_quads = if self
                .chip
                .special_loc
                .contains_key(&SpecialLocKey::SerdesSingle)
            {
                1
            } else if self
                .chip
                .special_loc
                .contains_key(&SpecialLocKey::SerdesDouble)
            {
                2
            } else if self
                .chip
                .special_loc
                .contains_key(&SpecialLocKey::SerdesTriple)
            {
                3
            } else {
                unreachable!()
            };

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
                    let corner = match hv {
                        DirHV::SW => "LL",
                        DirHV::SE => "LR",
                        DirHV::NW => "UL",
                        DirHV::NE => "UR",
                    };
                    for i in 0..2 {
                        for pin in ["CLKOP", "CLKOS", "CLKOS2", "CLKOS3"] {
                            ex.input_xlat_rc(format!("J{corner}CPLL{i}{pin}"));
                        }
                    }
                }
                if edge == Dir::S {
                    for i in 0..4 {
                        for pin in ["CDIVX", "CDIV1"] {
                            ex.input_xlat_rc(format!("PCS{pin}{i}"));
                        }
                    }
                    for quad in 0..num_serdes_quads {
                        let abcd = ['A', 'B', 'C', 'D'][quad];
                        for ch in 0..4 {
                            for pin in ["RXCLK", "TXCLK"] {
                                ex.input_xlat_rc(format!("JPCS{abcd}{pin}{ch}"));
                            }
                        }
                    }
                } else {
                    for i in 0..4 {
                        ex.input_xlat_rc(format!("J{lrbt}CDIVX{i}"));
                    }
                    if edge == Dir::E && 2 < num_serdes_quads {
                        for ch in 0..4 {
                            for pin in ["TXCLK", "RXCLK"] {
                                ex.input_xlat_rc(format!("JPCS{pin}{ch}"));
                            }
                        }
                    }
                }
                for i in 0..8 {
                    if !ex.ctx.edev.has_bel(bcrd_int.bel(bslots::DLLDEL[i])) {
                        continue;
                    }
                    let name = match edge {
                        Dir::N => ["00", "01", "10", "11", "20", "21", "30", "31"][i],
                        Dir::W => ["60", "61", "70", "71"][i],
                        Dir::E => ["50", "51", "40", "41"][i],
                        Dir::S => unreachable!(),
                    };
                    ex.input_xlat_rc(format!("JPCLKT{name}"));
                }
                let int_names = match edge {
                    Dir::W => [
                        ("JLLQPCLKCIB0", 1),
                        ("JLLQPCLKCIB1", 1),
                        ("JULQPCLKCIB0", 1),
                        ("JULQPCLKCIB1", 1),
                        ("JLLMPCLKCIB0", 2),
                        ("JULMPCLKCIB0", 2),
                        ("JLLMPCLKCIB2", 3),
                        ("JULMPCLKCIB2", 3),
                    ],
                    Dir::E => [
                        ("JLRQPCLKCIB0", 1),
                        ("JLRQPCLKCIB1", 1),
                        ("JURQPCLKCIB0", 1),
                        ("JURQPCLKCIB1", 1),
                        ("JLRMPCLKCIB0", 2),
                        ("JURMPCLKCIB0", 2),
                        ("JLRMPCLKCIB2", 3),
                        ("JURMPCLKCIB2", 3),
                    ],
                    Dir::S => [
                        ("JBLQPCLKCIB0", 1),
                        ("JBLQPCLKCIB1", 1),
                        ("JBRQPCLKCIB0", 1),
                        ("JBRQPCLKCIB1", 1),
                        ("JLLMPCLKCIB1", 2),
                        ("JLRMPCLKCIB1", 2),
                        ("JLLMPCLKCIB3", 3),
                        ("JLRMPCLKCIB3", 3),
                    ],
                    Dir::N => [
                        ("JTLQPCLKCIB0", 1),
                        ("JTLQPCLKCIB1", 1),
                        ("JTRQPCLKCIB0", 1),
                        ("JTRQPCLKCIB1", 1),
                        ("JULMPCLKCIB1", 2),
                        ("JURMPCLKCIB1", 2),
                        ("JULMPCLKCIB3", 3),
                        ("JURMPCLKCIB3", 3),
                    ],
                };
                for (name, min_quads) in int_names {
                    if num_serdes_quads < min_quads {
                        continue;
                    }
                    ex.input_xlat_rc(name);
                }
                match edge {
                    Dir::H(_) => {
                        let (num_s, num_n) = match tcid {
                            tcls::CLK_W_S | tcls::CLK_E_S => (3, 3),
                            _ => (6, 5),
                        };
                        for i in 0..num_s {
                            ex.input_xlat_rc(format!("JL{lrbt}CDR{i}"));
                        }
                        for i in 0..num_n {
                            ex.input_xlat_rc(format!("JU{lrbt}CDR{i}"));
                        }
                    }
                    Dir::N => {
                        let num = match tcid {
                            tcls::CLK_N_S => 3,
                            tcls::CLK_N_M => 5,
                            tcls::CLK_N_L => 7,
                            _ => unreachable!(),
                        };
                        for i in 0..num {
                            ex.input_xlat_rc(format!("JTLCDR{i}"))
                                .input_xlat_rc(format!("JTRCDR{i}"));
                        }
                    }
                    Dir::S => (),
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

    pub(super) fn process_clk_root_ecp4(&mut self) {
        let cell_tile = self.chip.clk_root();
        let cell = cell_tile.delta(-1, 0);

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
            let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
            ex.cell(cell)
                .permabuf_output_xlat_rc("JBRGECLK0")
                .permabuf_output_xlat_rc("JBRGECLK1")
                .permabuf_output_xlat_rc("JBRGECLK2")
                .permabuf_output_xlat_rc("JBRGECLK3")
                .input_xlat_rc("JRECLK0")
                .input_xlat_rc("JRECLK1")
                .input_xlat_rc("JLECLK2")
                .input_xlat_rc("JLECLK3")
                .output_bind_rc("DCS0CLK0", wires::IMUX_DCS_CLKA[0].cell(4))
                .output_bind_rc("DCS0CLK1", wires::IMUX_DCS_CLKB[0].cell(4))
                .output_bind_rc("DCS0CLK2", wires::IMUX_DCS_CLKC[0].cell(4))
                .output_bind_rc("DCS0CLK3", wires::IMUX_DCS_CLKD[0].cell(4))
                .output_bind_rc("DCS1CLK0", wires::IMUX_DCS_CLKA[1].cell(4))
                .output_bind_rc("DCS1CLK1", wires::IMUX_DCS_CLKB[1].cell(4))
                .output_bind_rc("DCS1CLK2", wires::IMUX_DCS_CLKC[1].cell(4))
                .output_bind_rc("DCS1CLK3", wires::IMUX_DCS_CLKD[1].cell(4))
                .input_bind_rc("DCS0", wires::OUT_DCS_DCSOUT[0].cell(4))
                .input_bind_rc("DCS1", wires::OUT_DCS_DCSOUT[1].cell(4))
                .input_bind_rc("LLCPCLKCIB0", wires::OUT_DCC_CLKO[0].cell(0))
                .input_bind_rc("LRCPCLKCIB0", wires::OUT_DCC_CLKO[0].cell(1))
                .input_bind_rc("ULCPCLKCIB0", wires::OUT_DCC_CLKO[0].cell(2))
                .input_bind_rc("URCPCLKCIB0", wires::OUT_DCC_CLKO[0].cell(3));
            for (edge, num_dcc) in [(Dir::W, 14), (Dir::E, 14), (Dir::S, 16), (Dir::N, 16)] {
                for j in 0..num_dcc {
                    let bcrd_dcc = cell_edge[edge].bel(bslots::DCC[j]);
                    let wire_dcc = ex.ctx.naming.bel_wire(bcrd_dcc, "CLKO_OUT");
                    ex.input_xlat(wire_dcc);
                }
            }
            for ci in 0..4 {
                for i in 0..20 {
                    let wire = wires::PCLK_ROOT[i].cell(ci);
                    ex.output_int(wire);
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
            for j in 0..4 {
                bel.input_xlat_rc(bcls::DCS::SEL[j], format!("JSEL{j}_DCS{i}"))
                    .input_xlat_rc(bcls::DCS::CLK[j], format!("CLK{j}_DCS{i}"))
                    .claim_pip("DCSOUT", format!("CLK{j}"));
            }
        }
    }
}
