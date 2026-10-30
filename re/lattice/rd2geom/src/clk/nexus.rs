use std::collections::{BTreeMap, btree_map};

use prjcombine_ecp::{
    chip::{Chip, ChipKind, NexusKind, SpecialIoKey},
    defs::{self, bcls, bslots, tcls, tslots, wires},
};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::{
    db::{CellSlotId, CellSlotIdExt, WireSlotIdExt},
    dir::{Dir, DirH, DirV},
    grid::{ColIdExt, DieIdExt, RowIdExt},
};

use crate::{ChipContext, RcMode, sb::EdgeKind};

impl ChipContext<'_> {
    pub(super) fn process_pclk_nexus(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::PCLK_DRIVER] {
            let bcrd = tcrd.bel(bslots::PCLK_DRIVER);
            let has_w = self.edev[tcrd]
                .cells
                .contains_id(defs::nexus::tcls::PCLK_DRIVER::W);
            let has_e = self.edev[tcrd]
                .cells
                .contains_id(defs::nexus::tcls::PCLK_DRIVER::E);
            let mut ex = self.extract_switchbox(bcrd);
            for i in 0..16 {
                ex.input_int(wires::PCLK_COL[i].cell(2));
            }
            for i in 0..8 {
                if has_w {
                    ex.output_int(wires::PCLK[i].cell(0));
                    let gwire_w = ex
                        .ctx
                        .edev
                        .resolve_bel_wire(bcrd, wires::PCLK[i].cell(0))
                        .unwrap();
                    let wname_w = ex.ctx.naming.interconnect[&gwire_w];
                    let wname_e = ex.ctx.claim_single_in(wname_w);
                    if has_e {
                        let gwire_e = ex
                            .ctx
                            .edev
                            .resolve_bel_wire(bcrd, wires::PCLK[i].cell(1))
                            .unwrap();
                        assert_eq!(wname_e, ex.ctx.naming.interconnect[&gwire_e]);
                        ex.output_int(wires::PCLK[i].cell(1));
                    } else {
                        ex.output_raw(wname_e, wires::PCLK[i].cell(1), EdgeKind::Mux);
                    }
                } else {
                    ex.output_int(wires::PCLK[i].cell(1));
                }
            }
            ex.flush_edges();
            let old_edges = core::mem::take(&mut ex.edges);
            for (wt, wf, _) in old_edges {
                assert_eq!(wt.cell, CellSlotId::from_idx(1));
                let idx = wires::PCLK.index_of(wt.wire).unwrap();
                ex.edge(wires::PCLK_MUX[idx].cell(2), wf);
            }
            for i in 0..8 {
                ex.edge_progbuf(wires::PCLK[i].cell(0), wires::PCLK_MUX[i].cell(2));
                ex.edge_progbuf(wires::PCLK[i].cell(1), wires::PCLK_MUX[i].cell(2));
            }
        }
        for &tcrd in &self.edev.tile_index[tcls::PCLK_COL_DRIVER] {
            let bcrd = tcrd.bel(bslots::PCLK_COL_DRIVER);
            let mut ex = self.extract_switchbox(bcrd);
            for i in 0..16 {
                ex.progbuf_output_int(wires::PCLK_COL[i].cell(0))
                    .input_int(wires::PCLK_ROOT[i].cell(0));
            }
        }
    }

    pub(super) fn process_clk_edge_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        for edge in Dir::DIRS {
            if matches!(edge, Dir::H(_)) && matches!(kind, NexusKind::SmallHalfIo) {
                continue;
            }
            let tcrd = self.chip.clk_emid(edge).tile(tslots::CLK);
            let cell = match edge {
                Dir::H(_) => tcrd.cell,
                Dir::V(_) => tcrd.step(Dir::W, 1),
            };
            let mode = if edge == Dir::E
                || (edge == Dir::S
                    && matches!(
                        kind,
                        NexusKind::LargePcs(_) | NexusKind::SmallHalfIo | NexusKind::SmallDenseIo
                    )) {
                RcMode::Base
            } else {
                RcMode::Io
            };
            let num_cib_we = match kind {
                NexusKind::SmallMipi | NexusKind::SmallMipiPcie => 10,
                NexusKind::LargePcs(_) | NexusKind::SmallDenseIo => 9,
                NexusKind::SmallHalfIo => 0,
            };
            let (lrbt, dcc_in, ci_mid, num_dcc, num_cib) = match edge {
                Dir::W => ('L', "HPFE", 2, 12, num_cib_we),
                Dir::E => ('R', "HPFW", 2, 12, num_cib_we),
                Dir::S => ('B', "VPFN", 6, 18, 6),
                Dir::N => ('T', "VPFS", 3, 16, 6),
            };
            let suffix = format!("{lrbt}MID_CORE_{lrbt}MIDMUX");

            {
                let mut ex = self.extract_switchbox(tcrd.bel(bslots::CLK_INT));
                ex.cell(cell);
                for i in 0..num_dcc {
                    ex.output_bind_rc_mode(
                        format!("J{dcc_in}{i}_{suffix}"),
                        mode,
                        wires::IMUX_DCC_CLKI[i].cell(ci_mid),
                    );
                }
                for i in 0..num_cib {
                    ex.input_xlat_rc_mode(format!("JPCLKCIB{lrbt}{i}_{suffix}"), mode);
                }
                match edge {
                    Dir::H(edge) => {
                        if kind == NexusKind::SmallDenseIo {
                            let io = defs::nexus::tcls::CLK_W_JD5F::IO;
                            let dlldel = defs::nexus::tcls::CLK_W_JD5F::DLLDEL;
                            let banks = match edge {
                                DirH::W => [7, 8, 9],
                                DirH::E => [4, 3, 2],
                            };
                            for i in 0..2 {
                                ex.input_xlat_rc_mode_nudge(
                                    format!("JPCLKT{bank}{i}_{suffix}", bank = banks[1]),
                                    mode,
                                    dlldel[i],
                                );
                            }
                            for (i, (bank, bi)) in [
                                (banks[0], 0),
                                (banks[0], 1),
                                (banks[2], 0),
                                (banks[2], 1),
                                (banks[2], 2),
                            ]
                            .into_iter()
                            .enumerate()
                            {
                                ex.input_io_rc_mode(
                                    format!("JPCLKT{bank}{bi}_{suffix}"),
                                    mode,
                                    io[i].wire(wires::OUT_IO_INCK[0]),
                                );
                            }
                            let corner = match edge {
                                DirH::W => "UL",
                                DirH::E => "LR",
                            };
                            ex.input_xlat_rc_mode(format!("J{corner}CLKOP_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("J{corner}CLKOS_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("J{corner}CLKOS2_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("J{corner}CLKOS3_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("J{corner}CLKOS4_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("J{corner}CLKOS5_{suffix}"), mode);
                        } else {
                            let io = defs::nexus::tcls::CLK_W_JD5D::IO;
                            let dlldel = defs::nexus::tcls::CLK_W_JD5D::DLLDEL;
                            let pll = defs::nexus::tcls::CLK_W_JD5D::PLL;
                            let banks = match edge {
                                DirH::W => [6, 7],
                                DirH::E => [2, 1],
                            };
                            for i in 0..6 {
                                let bank = banks[i / 3];
                                let bi = i % 3;
                                let name = format!("JPCLKT{bank}{bi}_{suffix}");
                                if let Some(&bcrd_io) = ex
                                    .ctx
                                    .chip
                                    .special_io
                                    .get(&SpecialIoKey::Clock(edge.into(), i as u8))
                                {
                                    if bi == 0 {
                                        ex.input_xlat_rc_mode_nudge(name, mode, dlldel[i / 3]);
                                    } else {
                                        let io_idx = bslots::IO.index_of(bcrd_io.slot).unwrap();
                                        ex.input_io_rc_mode(
                                            name,
                                            mode,
                                            io[i / 3 * 2 + bi - 1].wire(wires::OUT_IO_INCK[io_idx]),
                                        );
                                    }
                                } else {
                                    ex.input_raw_rc_mode(
                                        name,
                                        mode,
                                        if bi == 0 {
                                            dlldel[i / 3].wire(wires::OUT_DLLDEL_CLKO[0])
                                        } else {
                                            io[i / 3 * 2 + bi - 1].wire(wires::OUT_IO_INCK[0])
                                        },
                                    );
                                }
                            }
                            ex.input_xlat_rc_mode(format!("JL{lrbt}CLKOP_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JL{lrbt}CLKOS_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JL{lrbt}CLKOS2_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JL{lrbt}CLKOS3_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JL{lrbt}CLKOS4_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JL{lrbt}CLKOS5_{suffix}"), mode);
                            if (edge == DirH::W && kind != NexusKind::SmallMipi)
                                || (edge == DirH::E && matches!(kind, NexusKind::LargePcs(_)))
                            {
                                ex.input_xlat_rc_mode(format!("JU{lrbt}CLKOP_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JU{lrbt}CLKOS_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JU{lrbt}CLKOS2_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JU{lrbt}CLKOS3_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JU{lrbt}CLKOS4_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JU{lrbt}CLKOS5_{suffix}"), mode);
                            } else if edge == DirH::W {
                                ex.input_raw_rc_mode(
                                    format!("JU{lrbt}CLKOP_{suffix}"),
                                    mode,
                                    pll[1].wire(wires::OUT_PLL_CLKOP[0]),
                                )
                                .input_raw_rc_mode(
                                    format!("JU{lrbt}CLKOS_{suffix}"),
                                    mode,
                                    pll[1].wire(wires::OUT_PLL_CLKOS[0]),
                                )
                                .input_raw_rc_mode(
                                    format!("JU{lrbt}CLKOS2_{suffix}"),
                                    mode,
                                    pll[1].wire(wires::OUT_PLL_CLKOS2[0]),
                                )
                                .input_raw_rc_mode(
                                    format!("JU{lrbt}CLKOS3_{suffix}"),
                                    mode,
                                    pll[1].wire(wires::OUT_PLL_CLKOS3[0]),
                                )
                                .input_raw_rc_mode(
                                    format!("JU{lrbt}CLKOS4_{suffix}"),
                                    mode,
                                    pll[1].wire(wires::OUT_PLL_CLKOS4[0]),
                                )
                                .input_raw_rc_mode(
                                    format!("JU{lrbt}CLKOS5_{suffix}"),
                                    mode,
                                    pll[1].wire(wires::OUT_PLL_CLKOS5[0]),
                                );
                            }
                        }
                    }
                    Dir::S => {
                        let num_banks = if kind == NexusKind::SmallDenseIo {
                            2
                        } else {
                            3
                        };
                        for i in 0..(num_banks * 4) {
                            ex.input_xlat_rc_mode(format!("JECLKDIV{i}_{suffix}"), mode);
                        }
                        if kind != NexusKind::SmallDenseIo {
                            ex.input_xlat_rc_mode(format!("JLLCLKOP_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLLCLKOS_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLLCLKOS2_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLLCLKOS3_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLLCLKOS4_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLLCLKOS5_{suffix}"), mode);
                        }
                        if kind != NexusKind::SmallHalfIo {
                            ex.input_xlat_rc_mode(format!("JLRCLKOP_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLRCLKOS_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLRCLKOS2_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLRCLKOS3_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLRCLKOS4_{suffix}"), mode)
                                .input_xlat_rc_mode(format!("JLRCLKOS5_{suffix}"), mode);
                        }
                        let io = defs::nexus::tcls::CLK_S_JE5D15::IO;
                        let dlldel = defs::nexus::tcls::CLK_S_JE5D15::DLLDEL;
                        match kind {
                            NexusKind::SmallMipi
                            | NexusKind::SmallMipiPcie
                            | NexusKind::LargePcs(_) => {
                                ex.input_xlat_rc_mode_nudge(
                                    format!("JPCLKT30_{suffix}"),
                                    mode,
                                    dlldel[0],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT32_{suffix}"),
                                    mode,
                                    dlldel[1],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT40_{suffix}"),
                                    mode,
                                    dlldel[2],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT42_{suffix}"),
                                    mode,
                                    dlldel[3],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT50_{suffix}"),
                                    mode,
                                    dlldel[4],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT52_{suffix}"),
                                    mode,
                                    dlldel[5],
                                );
                                for i in 0..6 {
                                    ex.input_io_rc_mode(
                                        format!(
                                            "JPCLKT{a}{b}_{suffix}",
                                            a = 3 + i / 2,
                                            b = i % 2 * 2 + 1
                                        ),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                            NexusKind::SmallHalfIo => {
                                ex.input_xlat_rc_mode_nudge(
                                    format!("JPCLKT20_{suffix}"),
                                    mode,
                                    dlldel[0],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT22_{suffix}"),
                                    mode,
                                    dlldel[1],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT30_{suffix}"),
                                    mode,
                                    dlldel[2],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT32_{suffix}"),
                                    mode,
                                    dlldel[3],
                                );
                                for i in 0..5 {
                                    ex.input_io_rc_mode(
                                        format!(
                                            "JPCLKT{a}{b}_{suffix}",
                                            a = 2 + i / 2,
                                            b = i % 2 * 2 + 1
                                        ),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                            NexusKind::SmallDenseIo => {
                                ex.input_xlat_rc_mode_nudge(
                                    format!("JPCLKT50_{suffix}"),
                                    mode,
                                    dlldel[0],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT52_{suffix}"),
                                    mode,
                                    dlldel[1],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT60_{suffix}"),
                                    mode,
                                    dlldel[2],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT62_{suffix}"),
                                    mode,
                                    dlldel[3],
                                );
                                for i in 0..4 {
                                    ex.input_io_rc_mode(
                                        format!(
                                            "JPCLKT{a}{b}_{suffix}",
                                            a = 5 + i / 2,
                                            b = i % 2 * 2 + 1
                                        ),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                        }
                    }
                    Dir::N => {
                        ex.input_xlat_rc_mode(format!("JJTCLK_{suffix}"), mode);
                        match kind {
                            NexusKind::SmallMipi | NexusKind::SmallMipiPcie => {
                                let mipi = defs::nexus::tcls::CLK_N_JE5D15::MIPI;
                                let pll = defs::nexus::tcls::CLK_N_JE5D15::PLL;
                                let pcie = defs::nexus::tcls::CLK_N_JE5D15::PCIE;
                                let io = defs::nexus::tcls::CLK_N_JE5D15::IO;
                                for i in 0..2 {
                                    if ex.ctx.skip_mipi {
                                        ex.input_raw_rc_mode(
                                            format!("JMIPIDPHY{i}RX_{suffix}"),
                                            mode,
                                            mipi.wire(wires::OUT_MIPI_URWDCKHS[i]),
                                        )
                                        .input_raw_rc_mode(
                                            format!("JMIPIDPHY{i}TX_{suffix}"),
                                            mode,
                                            mipi.wire(wires::OUT_MIPI_UTWDCKHS[i]),
                                        )
                                        .input_raw_rc_mode(
                                            format!("JCLKHSBYTE{i}_{suffix}"),
                                            mode,
                                            mipi.wire(wires::OUT_MIPI_CLKHSBYTE[i]),
                                        );
                                    } else {
                                        ex.input_xlat_rc_mode(
                                            format!("JMIPIDPHY{i}RX_{suffix}"),
                                            mode,
                                        )
                                        .input_xlat_rc_mode(
                                            format!("JMIPIDPHY{i}TX_{suffix}"),
                                            mode,
                                        )
                                        .input_xlat_rc_mode(
                                            format!("JCLKHSBYTE{i}_{suffix}"),
                                            mode,
                                        );
                                    }
                                }
                                if kind == NexusKind::SmallMipi {
                                    ex.input_raw_rc_mode(
                                        format!("JULCLKOP_{suffix}"),
                                        mode,
                                        pll.wire(wires::OUT_PLL_CLKOP[0]),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JULCLKOS_{suffix}"),
                                        mode,
                                        pll.wire(wires::OUT_PLL_CLKOS[0]),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JULCLKOS2_{suffix}"),
                                        mode,
                                        pll.wire(wires::OUT_PLL_CLKOS2[0]),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JULCLKOS3_{suffix}"),
                                        mode,
                                        pll.wire(wires::OUT_PLL_CLKOS3[0]),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JPCIELINKCLK0_{suffix}"),
                                        mode,
                                        pcie.wire(wires::OUT_PCIE_LINKCLK),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JPCIEUSRCLK0_{suffix}"),
                                        mode,
                                        pcie.wire(wires::OUT_PCIE_USRCLK[0]),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JPCIECLKTX0_{suffix}"),
                                        mode,
                                        pcie.wire(wires::OUT_PCS_CLKTX[0]),
                                    )
                                    .input_raw_rc_mode(
                                        format!("JPCIECLKRX0_{suffix}"),
                                        mode,
                                        pcie.wire(wires::OUT_PCS_CLKRX[0]),
                                    );
                                } else {
                                    ex.input_xlat_rc_mode(format!("JULCLKOP_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JULCLKOS_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JULCLKOS2_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JULCLKOS3_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JPCIELINKCLK0_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JPCIEUSRCLK0_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JPCIECLKTX0_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JPCIECLKRX0_{suffix}"), mode);
                                }
                                for i in 0..2 {
                                    ex.input_io_rc_mode(
                                        format!("JPCLKT0{i}_{suffix}"),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                            NexusKind::LargePcs(_) => {
                                let io = defs::nexus::tcls::CLK_N_JD5D::IO;
                                ex.input_xlat_rc_mode(format!("JULCLKOP_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS2_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS3_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JURCLKOP_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JURCLKOS_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JURCLKOS2_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JURCLKOS3_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JPCIEUSRCLK0_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JPCIEUSRCLK1_{suffix}"), mode);
                                for i in 0..8 {
                                    ex.input_xlat_rc_mode(format!("JPCSCLKRX{i}_{suffix}"), mode)
                                        .input_xlat_rc_mode(format!("JPCSCLKTX{i}_{suffix}"), mode);
                                }
                                for i in 0..2 {
                                    ex.input_io_rc_mode(
                                        format!("JPCLKT0{i}_{suffix}"),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                            NexusKind::SmallHalfIo => {
                                let io = defs::nexus::tcls::CLK_N_JD5U::IO;
                                let dlldel = defs::nexus::tcls::CLK_N_JD5U::DLLDEL;
                                ex.input_xlat_rc_mode_nudge(
                                    format!("JPCLKT10_{suffix}"),
                                    mode,
                                    dlldel[0],
                                )
                                .input_xlat_rc_mode_nudge(
                                    format!("JPCLKT12_{suffix}"),
                                    mode,
                                    dlldel[1],
                                );
                                for i in 0..2 {
                                    ex.input_io_rc_mode(
                                        format!("JPCLKT1{ii}_{suffix}", ii = i * 2 + 1),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                            NexusKind::SmallDenseIo => {
                                let io = defs::nexus::tcls::CLK_N_JD5F::IO;
                                ex.input_xlat_rc_mode(format!("JULCLKOP_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS2_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS3_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS4_{suffix}"), mode)
                                    .input_xlat_rc_mode(format!("JULCLKOS5_{suffix}"), mode);
                                for i in 0..4 {
                                    ex.input_io_rc_mode(
                                        format!("JPCLKT{a}{b}_{suffix}", a = i / 2, b = i % 2),
                                        mode,
                                        io[i].wire(wires::OUT_IO_INCK[0]),
                                    );
                                }
                            }
                        }
                    }
                }
                if edge == Dir::N
                    || (edge == Dir::E
                        && matches!(kind, NexusKind::LargePcs(_) | NexusKind::SmallDenseIo))
                {
                    ex.input_xlat_rc_mode(format!("JOSCHF_{suffix}"), mode)
                        .input_xlat_rc_mode(format!("JOSCLF_{suffix}"), mode);
                }
                if edge == Dir::W {
                    ex.input_xlat_rc_mode(format!("JSGMIICLK0_{suffix}"), mode)
                        .input_xlat_rc_mode(format!("JSGMIICLK1_{suffix}"), mode);
                }
            }
            {
                let mut bel = self.extract_bel_cell(tcrd.bel(bslots::EMIDMUX_TEST), cell);
                bel.name(format!("{lrbt}MID_CORE"));
                for i in 0..4 {
                    bel.input_xlat_rc_mode(
                        bcls::EMIDMUX_TEST::TESTINP[i],
                        format!("JTESTINP{i}_{suffix}"),
                        mode,
                    );
                }
            }
            for i in 0..num_dcc {
                self.extract_bel_cell(tcrd.bel(bslots::DCC[i]), cell)
                    .name(format!("DCC_{lrbt}{i}"))
                    .input_xlat_rc_mode(bcls::DCC::CLKI, format!("JCLKI_DCC_DCC{i}"), mode)
                    .input_xlat_rc_mode(bcls::DCC::CE, format!("JCE_DCC_DCC{i}"), mode)
                    .output_bind_rc_mode(
                        bcls::DCC::CLKO,
                        format!("JCLKO_DCC_DCC{i}"),
                        mode,
                        wires::OUT_DCC_CLKO[i].cell(ci_mid),
                    )
                    .claim_pip("CLKO", "CLKI");
            }
        }
    }

    pub(super) fn process_clk_root_nexus(&mut self) {
        let ChipKind::Nexus(kind) = self.chip.kind else {
            unreachable!()
        };
        let tcrd = self.chip.clk_root().tile(tslots::CLK);
        let cell = tcrd.step(Dir::W, 1);
        for i in 0..4 {
            self.extract_bel_cell(tcrd.bel(bslots::DCC[i]), cell)
                .name(format!("DCC_C{i}"))
                .input_xlat_rc(bcls::DCC::CLKI, format!("JCLKI_DCC_DCC{i}"))
                .input_xlat_rc(bcls::DCC::CE, format!("JCE_DCC_DCC{i}"))
                .output_bind_rc(
                    bcls::DCC::CLKO,
                    format!("JCLKO_DCC_DCC{i}"),
                    wires::OUT_DCC_CLKO[i].cell(3),
                )
                .claim_pip("CLKO", "CLKI");
        }
        {
            self.extract_bel_cell(tcrd.bel(bslots::GSR), cell)
                .name_rc("GSR_CORE")
                .suffix("GSR_CORE_GSR_CENTER")
                .add_wire_rc("GSROUT", "JGSROUT_GSR_CORE_GSR_CENTER");
        }
        let (num_dcs, num_mux) = if kind.is_large() { (2, 4) } else { (1, 2) };
        {
            let jz = self.rc_wire(cell, "JZ_VHI");
            let vcc = self.rc_wire(cell, "VCC");
            let nc = self.rc_wire(cell, "JNC_CMUXCLKTEST_CORE_ICLKTEST");
            self.claim_pip(vcc, jz);
            let mut ex = self
                .extract_switchbox_names(tcrd.bel(bslots::CLK_INT), ["CMUXCLKTEST_CORE", "VHI"]);
            ex.cell(cell)
                .add_wire("Z", jz)
                .add_wire("VCC", vcc)
                .add_wire("NC", nc);
            for i in 0..num_mux {
                let suffix = format!("DCSMUX_CORE_DCSMUX{i}");
                ex.output_bind_rc(format!("JDCSMUXOUT_{suffix}"), wires::IMUX_DCS[i].cell(3))
                    .input_xlat_rc(format!("JJCLKLL_{suffix}"))
                    .input_xlat_rc(format!("JJCLKLR_{suffix}"))
                    .input_xlat_rc(format!("JJCLKUL_{suffix}"))
                    .input_xlat_rc(format!("JJCLKUR_{suffix}"));
                for j in 0..18 {
                    ex.input_xlat_rc(format!("JVPFN{j}_{suffix}"));
                }
                for j in 0..16 {
                    ex.input_xlat_rc(format!("JVPFS{j}_{suffix}"));
                }
                for j in 0..12 {
                    if kind == NexusKind::SmallHalfIo {
                        let clk_w = defs::nexus::tcls::CLK_ROOT_JD5U::CLK_W;
                        let clk_e = defs::nexus::tcls::CLK_ROOT_JD5U::CLK_E;
                        ex.input_raw_rc(
                            format!("JHPFE{j}_{suffix}"),
                            clk_w.wire(wires::OUT_DCC_CLKO[j]),
                        )
                        .input_raw_rc(
                            format!("JHPFW{j}_{suffix}"),
                            clk_e.wire(wires::OUT_DCC_CLKO[j]),
                        );
                    } else {
                        ex.input_xlat_rc(format!("JHPFE{j}_{suffix}"))
                            .input_xlat_rc(format!("JHPFW{j}_{suffix}"));
                    }
                }
            }
            let quads = if kind.is_large() {
                [("BL", 0), ("BR", 1), ("TL", 2), ("TR", 3)].as_slice()
            } else {
                [("L", 2), ("R", 3)].as_slice()
            };
            for &(qname, ci) in quads {
                for j in 0..16 {
                    let wire = wires::PCLK_ROOT[j].cell(ci);
                    let wire = ex.ctx.edev.resolve_tile_wire(tcrd, wire).unwrap();
                    let wname_out = ex.ctx.naming.interconnect[&wire];
                    let wname = ex.ctx.claim_single_in(wname_out);
                    ex.ctx.bind_wire(wname, wire);
                    ex.add_wire(format!("{qname}HPRX{j}"), wname);
                }
            }
            let mut vcchprx = BTreeMap::new();
            for &col in ex.ctx.chip.cols_pclk.values() {
                let h = col.side_of(ex.ctx.chip.col_clk);
                for row in [ex.ctx.chip.row_s(), ex.ctx.chip.row_n()] {
                    let v = row.side_of(ex.ctx.chip.row_clk);
                    if v == DirV::S && !kind.is_large() {
                        continue;
                    }
                    let cell = Chip::DIE.cell(col, row);
                    let wname_out = ex.ctx.naming.interconnect[&cell.wire(wires::TIE1)];
                    let wname = ex.ctx.claim_single_in(wname_out);
                    let hv = h & v;
                    match vcchprx.entry(hv) {
                        btree_map::Entry::Vacant(e) => {
                            e.insert(wname);
                            ex.add_wire(format!("VCCHPRX_{hv}"), wname);
                            ex.ctx.claim_pip(wname, vcc);
                        }
                        btree_map::Entry::Occupied(e) => {
                            assert_eq!(*e.get(), wname);
                        }
                    }
                }
            }
        }
        for i in 0..num_mux {
            let mut bel = self.extract_bel_cell(tcrd.bel(bslots::DCSMUX_TEST[i]), cell);
            let abcd = ['A', 'B', 'C', 'D'][i];
            bel.name(format!("DCSMUX_CORE{abcd}"));
            for j in 0..7 {
                bel.input_xlat_rc(
                    bcls::DCSMUX_TEST::TESTINP[j],
                    format!("JTESTINP{j}_DCSMUX_CORE_DCSMUX{i}"),
                );
            }
        }
        for i in 0..num_dcs {
            let suffix = if kind.is_large() {
                format!("PCLKDIV_PCLKDIV{i}")
            } else {
                "PCLKDIV_PCLKDIV".to_string()
            };
            let mut bel = self.extract_bel_cell(tcrd.bel(bslots::PCLKDIV[i]), cell);
            bel.name(if kind.is_large() {
                ["PCLKDIVA", "PCLKDIVB"][i]
            } else {
                "PCLKDIV"
            })
            .input_xlat_rc(bcls::PCLKDIV::CLKIN, format!("JCLKIN_{suffix}"))
            .input_xlat_rc(bcls::PCLKDIV::LSRPDIV, format!("JLSRPDIV_{suffix}"))
            .output_bind_rc(
                bcls::PCLKDIV::CLKOUT,
                format!("JCLKOUT_{suffix}"),
                wires::OUT_PCLKDIV_CLKOUT[i].cell(3),
            )
            .claim_pip("CLKOUT", "CLKIN");
            for j in 0..3 {
                bel.input_xlat_rc(
                    bcls::PCLKDIV::PCLKDIVTESTINP[j],
                    format!("JPCLKDIVTESTINP{j}_{suffix}"),
                );
            }
        }
        for i in 0..num_dcs {
            let suffix = if kind.is_large() {
                format!("DCS_DCSIP{i}")
            } else {
                "DCS_DCSIP".to_string()
            };
            let mut bel = self.extract_bel_cell(tcrd.bel(bslots::DCS[i]), cell);
            bel.name(if kind.is_large() {
                ["DCSA", "DCSB"][i]
            } else {
                "DCS0"
            })
            .input_xlat_rc(bcls::DCS::CLK[0], format!("JCLK0_{suffix}"))
            .input_xlat_rc(bcls::DCS::CLK[1], format!("JCLK1_{suffix}"))
            .input_xlat_rc(bcls::DCS::SEL[0], format!("JSEL_{suffix}"))
            .input_xlat_rc(bcls::DCS::SELFORCE, format!("JSELFORCE_{suffix}"))
            .output_bind_rc(
                bcls::DCS::DCSOUT,
                format!("JDCSOUT_{suffix}"),
                wires::OUT_DCS_DCSOUT[i].cell(3),
            )
            .claim_pip("DCSOUT", "CLK0")
            .claim_pip("DCSOUT", "CLK1");
        }
        {
            let mut ex = self.extract_switchbox(tcrd.bel(bslots::CLK_INT));
            ex.cell(cell);
            for i in 0..num_mux {
                let suffix = format!("CMUX_CORE_CMUX{i}");
                ex.input_xlat_rc(format!("JJCLKLL_{suffix}"))
                    .input_xlat_rc(format!("JJCLKLR_{suffix}"))
                    .input_xlat_rc(format!("JJCLKUL_{suffix}"))
                    .input_xlat_rc(format!("JJCLKUR_{suffix}"));
                for j in 0..16 {
                    ex.output_xlat_rc(format!("JHPRX{j}_{suffix}"));
                }
                for j in 0..num_dcs {
                    ex.input_xlat_rc(format!("JDCS{j}_{suffix}"));
                }
                for j in 0..18 {
                    ex.input_xlat_rc(format!("JVPFN{j}_{suffix}"));
                }
                for j in 0..16 {
                    ex.input_xlat_rc(format!("JVPFS{j}_{suffix}"));
                }
                for j in 0..12 {
                    if kind == NexusKind::SmallHalfIo {
                        let clk_w = defs::nexus::tcls::CLK_ROOT_JD5U::CLK_W;
                        let clk_e = defs::nexus::tcls::CLK_ROOT_JD5U::CLK_E;
                        ex.input_raw_rc(
                            format!("JHPFE{j}_{suffix}"),
                            clk_w.wire(wires::OUT_DCC_CLKO[j]),
                        )
                        .input_raw_rc(
                            format!("JHPFW{j}_{suffix}"),
                            clk_e.wire(wires::OUT_DCC_CLKO[j]),
                        );
                    } else {
                        ex.input_xlat_rc(format!("JHPFE{j}_{suffix}"))
                            .input_xlat_rc(format!("JHPFW{j}_{suffix}"));
                    }
                }
            }
        }
        for i in 0..num_mux {
            let mut bel = self.extract_bel_cell(tcrd.bel(bslots::CMUX_TEST[i]), cell);
            let abcd = ['A', 'B', 'C', 'D'][i];
            bel.name(format!("CMUX_CORE{abcd}"));
            for j in 0..5 {
                bel.input_xlat_rc(
                    bcls::CMUX_TEST::TESTINP[j],
                    format!("JTESTINP{j}_CMUX_CORE_CMUX{i}"),
                );
            }
        }
        //
    }
}
