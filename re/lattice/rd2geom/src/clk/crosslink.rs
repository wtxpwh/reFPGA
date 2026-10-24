use prjcombine_ecp::defs::{bcls, bslots, wires};
use prjcombine_interconnect::{db::WireSlotIdExt, dir::DirV};

use crate::{ChipContext, RcMode};

impl ChipContext<'_> {
    pub(super) fn process_clk_edge_crosslink(&mut self) {
        for edge in [DirV::S, DirV::N] {
            let cell_tile = self.chip.clk_emid(edge);
            let cell = cell_tile.delta(-1, 0);
            let bt = match edge {
                DirV::S => 'B',
                DirV::N => 'T',
            };

            {
                let bcrd = cell_tile.bel(bslots::CLKTEST);
                let mut bel = self.extract_bel_cell(bcrd, cell);
                bel.name(format!("CLKTEST_{bt}MID"));
                for i in 0..3 {
                    bel.input_xlat_rc(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
                }
                for i in 3..6 {
                    bel.add_wire_rc(format!("TESTIN{i}"), format!("TESTIN{i}_CLKTEST"));
                }
            }

            {
                let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
                ex.cell(cell);
                match edge {
                    DirV::S => {
                        for i in 0..8 {
                            ex.output_bind_rc(
                                format!("BDCC{i}CLKI"),
                                wires::IMUX_DCC_CLKI[i].cell(2),
                            );
                        }
                        ex.input_xlat_rc("JPCLKCIBB0")
                            .input_xlat_rc("JPCLKCIBB1")
                            .input_xlat_rc("JPCLK20")
                            .input_xlat_rc("JPCLK21")
                            .input_xlat_rc("JPCLK10")
                            .input_xlat_rc("JPCLK11")
                            .input_xlat_rc("JCLKOP")
                            .input_xlat_rc("JCLKOS")
                            .input_xlat_rc("JCLKOS2")
                            .input_xlat_rc("JCLKOS3")
                            .input_xlat_rc("JCLKDIVX0")
                            .input_xlat_rc("JCLKDIVX1")
                            .input_xlat_rc("JOSCLOW")
                            .input_xlat_rc("JOSCHIGH");
                    }
                    DirV::N => {
                        for i in 0..6 {
                            ex.output_bind_rc(
                                format!("TDCC{i}CLKI"),
                                wires::IMUX_DCC_CLKI[i].cell(1),
                            );
                        }
                        ex.input_xlat_rc("JPCLKCIBT0")
                            .input_xlat_rc("JPCLKCIBT1")
                            .input_xlat_rc("JPCLK00")
                            .input_xlat_rc("JPCLK01")
                            .input_xlat_rc("JCLKDIVX2")
                            .input_xlat_rc("JCLKDIVX3")
                            .input_xlat_rc("JCLKHSBYTE0")
                            .input_xlat_rc("JCLKHSBYTE1")
                            .input_xlat_rc("JMIPIDPHY0TX")
                            .input_xlat_rc("JMIPIDPHY1TX")
                            .input_xlat_rc("JMIPIDPHY0RX")
                            .input_xlat_rc("JMIPIDPHY1RX");
                    }
                }
            }

            let num_dcc = match edge {
                DirV::S => 8,
                DirV::N => 6,
            };
            let ci_base = match edge {
                DirV::S => 2,
                DirV::N => 1,
            };

            for i in 0..num_dcc {
                let bcrd_dcc = cell_tile.bel(bslots::DCC[i]);
                self.extract_bel_cell(bcrd_dcc, cell)
                    .name(format!("DCC_{bt}{i}"))
                    .input_xlat_rc(bcls::DCC::CE, format!("JCE_{bt}DCC{i}"))
                    .input_xlat_rc(bcls::DCC::CLKI, format!("CLKI_{bt}DCC{i}"))
                    .output_bind_step_rc_mode(
                        bcls::DCC::CLKO,
                        format!("CLKO_{bt}DCC{i}"),
                        RcMode::Base,
                        wires::OUT_DCC_CLKO[i].cell(ci_base),
                    )
                    .claim_pip("CLKO", "CLKI");
            }
        }
    }

    pub(super) fn process_clk_root_crosslink(&mut self) {
        let cell_tile = self.chip.clk_root();
        let cell = cell_tile.delta(-1, 0);

        {
            let mut bel = self.extract_bel_cell(cell_tile.bel(bslots::CLKTEST), cell);
            bel.name("CLKTEST_CEN");
            for i in 0..4 {
                bel.input_xlat_rc(bcls::CLKTEST::TESTIN[i], format!("JTESTIN{i}_CLKTEST"));
            }
            for i in 4..6 {
                bel.add_wire_rc(format!("TESTIN{i}"), format!("TESTIN{i}_CLKTEST"));
            }
        }

        {
            let mut ex = self.extract_switchbox(cell_tile.bel(bslots::CLK_INT));
            ex.cell(cell)
                .output_bind_rc("DCSCLK0", wires::IMUX_DCS_CLKA[0].cell(1))
                .output_bind_rc("DCSCLK1", wires::IMUX_DCS_CLKB[0].cell(1))
                .input_bind_rc("DCSOUT", wires::OUT_DCS_DCSOUT[0].cell(1));
            for i in 0..8 {
                ex.output_bind_rc(format!("PCLK{i}"), wires::PCLK_ROOT[i].cell(1))
                    .progbuf_output_int(wires::PCLK_HALF[i].cell(0))
                    .progbuf_output_int(wires::PCLK_HALF[i].cell(1));
            }
            for (edge, num_dcc) in [(DirV::S, 8), (DirV::N, 6)] {
                let cell_edge = cell_tile.with_row(ex.ctx.chip.row_edge(edge));
                for j in 0..num_dcc {
                    let bcrd_dcc = cell_edge.bel(bslots::DCC[j]);
                    let wire_dcc = ex.ctx.naming.bel_wire(bcrd_dcc, "CLKO_OUT");
                    ex.input_xlat(wire_dcc);
                }
            }
        }

        self.extract_bel_cell(cell_tile.bel(bslots::DCS[0]), cell)
            .name("DCS")
            .suffix("DCS")
            .input_xlat_rc(bcls::DCS::CLK[0], "CLK0_DCS")
            .input_xlat_rc(bcls::DCS::CLK[1], "CLK1_DCS")
            .output_xlat_rc(bcls::DCS::DCSOUT, "DCSOUT_DCS")
            .claim_pip("DCSOUT", "CLK0")
            .claim_pip("DCSOUT", "CLK1");
    }
}
