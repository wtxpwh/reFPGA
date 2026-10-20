use prjcombine_ecp::{
    chip::{Chip, ChipKind, MachXo2Kind, RowKind, SpecialLocKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::WireSlotIdExt,
    dir::{Dir, DirHV},
    grid::DieIdExt,
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub(super) fn process_hsdclk_root_machxo2(&mut self) {
        for &tcrd in &self.edev.tile_index[tcls::SCLK_H_ROOT] {
            let bcrd = tcrd.bel(bslots::SCLK_H_SPLITTER);
            let mut ex = self.extract_switchbox(bcrd);
            for i in 0..8 {
                ex.progbuf_output_int(wires::SCLK_H[i].cell(0))
                    .progbuf_output_int(wires::SCLK_H[i].cell(1))
                    .input_int(wires::SCLK_ROOT[i].cell(1));
            }
        }
    }

    pub(super) fn process_dlldel_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        if is_smol {
            return;
        }
        for (edge, idx) in [
            (Dir::W, 0),
            (Dir::W, 1),
            (Dir::W, 2),
            (Dir::E, 0),
            (Dir::S, 0),
            (Dir::S, 1),
            (Dir::N, 0),
            (Dir::N, 1),
        ] {
            let cell = match edge {
                Dir::H(edge) => Chip::DIE.cell(self.chip.col_edge(edge), self.chip.row_clk),
                Dir::V(edge) => Chip::DIE.cell(self.chip.col_clk - 1, self.chip.row_edge(edge)),
            };
            let ci = if edge == Dir::E && self.chip.kind == ChipKind::MachXo2(MachXo2Kind::MachXo2)
            {
                2
            } else {
                0
            };
            {
                self.extract_switchbox(cell.bel(bslots::CLK_INT))
                    .output_xlat_rc_io(format!("JINCK{idx}"))
                    .input_io_rc_io(
                        format!("JPADDI{idx}"),
                        match edge {
                            Dir::W => wires::IO_GCLK_W[idx].cell(ci),
                            Dir::E => wires::IO_GCLK_E[idx].cell(ci),
                            Dir::S => wires::IO_GCLK_S[idx].cell(ci),
                            Dir::N => wires::IO_GCLK_N[idx].cell(ci),
                        },
                    )
                    .input_bind_rc_io(format!("DLLDEL{idx}"), wires::OUT_DLLDEL_CLKO[idx].cell(ci));
            }
            let bcrd = cell.bel(bslots::DLLDEL[idx]);
            let lrbt = match edge {
                Dir::W => 'L',
                Dir::E => 'R',
                Dir::S => 'B',
                Dir::N => 'T',
            };
            let mut bel = self.extract_bel(bcrd);
            bel.name(format!("{lrbt}DLLDEL{idx}"))
                .input_io_rc_io(
                    bcls::DLLDEL::CLKI,
                    format!("JCLKI{idx}_DLLDEL"),
                    match edge {
                        Dir::W => wires::IO_GCLK_W[idx].cell(ci),
                        Dir::E => wires::IO_GCLK_E[idx].cell(ci),
                        Dir::S => wires::IO_GCLK_S[idx].cell(ci),
                        Dir::N => wires::IO_GCLK_N[idx].cell(ci),
                    },
                )
                .output_xlat_rc_io(bcls::DLLDEL::CLKO, format!("CLKO{idx}_DLLDEL"));

            let cell_dqsdll = bel.ctx.chip.special_loc[&SpecialLocKey::DqsDll(match edge {
                Dir::W => Dir::S,
                Dir::E => Dir::N,
                Dir::S => Dir::S,
                Dir::N => Dir::N,
            })];
            bel.add_wire_rc_io("DQSDEL", format!("JDQSDEL{idx}_DLLDEL"));
            let wire_dqsdll = bel.ctx.rc_wire(cell_dqsdll, "JDQSDEL_DQSDLL");
            bel.claim_pip_from("DQSDEL", wire_dqsdll);
        }
    }

    pub(super) fn process_clk_machxo2(&mut self) {
        let is_smol = self.chip.rows[self.chip.row_clk].kind != RowKind::Ebr;
        let has_bank4 = self.chip.special_loc.contains_key(&SpecialLocKey::Bc(4));
        let has_2ebr = self
            .chip
            .rows
            .values()
            .filter(|rd| rd.kind == RowKind::Ebr)
            .count()
            >= 2;

        let bcrd = self.chip.bel_clk_root();
        let cell = bcrd.cell.delta(-1, 0);

        {
            let mut ex = self.extract_switchbox(bcrd.bel(bslots::CLK_INT));
            ex.cell(cell)
                .input_xlat_rc("JSNETCIBL0")
                .input_xlat_rc("JSNETCIBL1")
                .input_xlat_rc("JSNETCIBR0")
                .input_xlat_rc("JSNETCIBR1")
                .input_xlat_rc("JSNETCIBB0")
                .input_xlat_rc("JSNETCIBB1")
                .input_xlat_rc("JSNETCIBT0")
                .input_xlat_rc("JSNETCIBT1")
                .input_xlat_rc("JSNETCIBMID0")
                .input_xlat_rc("JSNETCIBMID1")
                .input_xlat_rc("JSNETCIBMID2")
                .input_xlat_rc("JSNETCIBMID3")
                .input_xlat_rc("JSNETCIBMID4")
                .input_xlat_rc("JSNETCIBMID5")
                .input_xlat_rc("JSNETCIBMID6")
                .input_xlat_rc("JSNETCIBMID7")
                .output_bind_rc("VPRXCLKI0", wires::IMUX_DCC_CLKI[0].cell(0))
                .output_bind_rc("VPRXCLKI1", wires::IMUX_DCC_CLKI[1].cell(0))
                .output_bind_rc("VPRXCLKI2", wires::IMUX_DCC_CLKI[2].cell(0))
                .output_bind_rc("VPRXCLKI3", wires::IMUX_DCC_CLKI[3].cell(0))
                .output_bind_rc("VPRXCLKI4", wires::IMUX_DCC_CLKI[4].cell(0))
                .output_bind_rc("VPRXCLKI5", wires::IMUX_DCC_CLKI[5].cell(0))
                .output_bind_rc("VPRXCLKI60", wires::IMUX_DCM_CLK0[0].cell(0))
                .output_bind_rc("VPRXCLKI61", wires::IMUX_DCM_CLK1[0].cell(0))
                .output_bind_rc("VPRXCLKI70", wires::IMUX_DCM_CLK0[1].cell(0))
                .output_bind_rc("VPRXCLKI71", wires::IMUX_DCM_CLK1[1].cell(0))
                .input_xlat_rc("JPCLKCIBLLQ0")
                .input_xlat_rc("JPCLKCIBLLQ1")
                .input_xlat_rc("JPCLKCIBLRQ0")
                .input_xlat_rc("JPCLKCIBLRQ1")
                .input_xlat_rc("JPCLKCIBVIQB0")
                .input_xlat_rc("JPCLKCIBVIQB1")
                .input_xlat_rc("JPCLKCIBVIQT0")
                .input_xlat_rc("JPCLKCIBVIQT1")
                .input_xlat_rc("PCLKCIBLLQ0")
                .input_xlat_rc("PCLKCIBLLQ1")
                .input_xlat_rc("PCLKCIBLRQ0")
                .input_xlat_rc("PCLKCIBLRQ1")
                .input_xlat_rc("PCLKCIBVIQB0")
                .input_xlat_rc("PCLKCIBVIQB1")
                .input_xlat_rc("PCLKCIBVIQT0")
                .input_xlat_rc("PCLKCIBVIQT1");

            for i in 0..8 {
                ex.output_int(wires::SCLK_ROOT[i].cell(0));
            }

            if is_smol {
                ex.input_io_rc("JPCLKT30", wires::IO_GCLK_W[0].cell(0))
                    .input_io_rc("JPCLKT31", wires::IO_GCLK_W[1].cell(0))
                    .input_io_rc("JPCLKT32", wires::IO_GCLK_W[2].cell(0))
                    .input_io_rc("JPCLKT10", wires::IO_GCLK_E[0].cell(0))
                    .input_io_rc("JPCLKT20", wires::IO_GCLK_S[0].cell(0))
                    .input_io_rc("JPCLKT21", wires::IO_GCLK_S[1].cell(0))
                    .input_io_rc("JPCLKT00", wires::IO_GCLK_N[0].cell(0))
                    .input_io_rc("JPCLKT01", wires::IO_GCLK_N[1].cell(0));
            } else {
                ex.input_xlat_rc("JPCLKT30")
                    .input_xlat_rc(if has_bank4 { "JPCLKT40" } else { "JPCLKT31" })
                    .input_xlat_rc(if has_bank4 { "JPCLKT50" } else { "JPCLKT32" })
                    .input_xlat_rc("JPCLKT10")
                    .input_xlat_rc("JPCLKT20")
                    .input_xlat_rc("JPCLKT21")
                    .input_xlat_rc("JPCLKT00")
                    .input_xlat_rc("JPCLKT01")
                    .input_xlat_rc("JPCLKCIBMID2")
                    .input_xlat_rc("JPCLKCIBMID3")
                    .input_xlat_rc("PCLKCIBMID2")
                    .input_xlat_rc("PCLKCIBMID3")
                    .input_xlat_rc("JBCDIV10")
                    .input_xlat_rc("JBCDIV11")
                    .input_xlat_rc("JBCDIVX0")
                    .input_xlat_rc("JBCDIVX1")
                    .input_xlat_rc("JTCDIV10")
                    .input_xlat_rc("JTCDIV11")
                    .input_xlat_rc("JTCDIVX0")
                    .input_xlat_rc("JTCDIVX1")
                    .output_bind_rc("EBRG0CLK0", wires::IMUX_ECLKBRIDGECS_CLK0[0].cell(0))
                    .output_bind_rc("EBRG0CLK1", wires::IMUX_ECLKBRIDGECS_CLK1[0].cell(0))
                    .output_bind_rc("EBRG1CLK0", wires::IMUX_ECLKBRIDGECS_CLK0[1].cell(0))
                    .output_bind_rc("EBRG1CLK1", wires::IMUX_ECLKBRIDGECS_CLK1[1].cell(0))
                    .input_xlat_rc("JECLKCIBB0")
                    .input_xlat_rc("JECLKCIBB1")
                    .input_xlat_rc("JECLKCIBT0")
                    .input_xlat_rc("JECLKCIBT1")
                    .input_xlat_rc("ECLKCIBB0")
                    .input_xlat_rc("ECLKCIBB1")
                    .input_xlat_rc("ECLKCIBT0")
                    .input_xlat_rc("ECLKCIBT1");
                if has_2ebr {
                    ex.input_xlat_rc("JPCLKCIBULQ0")
                        .input_xlat_rc("JPCLKCIBULQ1")
                        .input_xlat_rc("JPCLKCIBURQ0")
                        .input_xlat_rc("JPCLKCIBURQ1")
                        .input_xlat_rc("JPCLKCIBMID0")
                        .input_xlat_rc("JPCLKCIBMID1")
                        .input_xlat_rc("PCLKCIBULQ0")
                        .input_xlat_rc("PCLKCIBULQ1")
                        .input_xlat_rc("PCLKCIBURQ0")
                        .input_xlat_rc("PCLKCIBURQ1")
                        .input_xlat_rc("PCLKCIBMID0")
                        .input_xlat_rc("PCLKCIBMID1");
                }
                if ex.ctx.chip.try_bel_pll(DirHV::NW, 0).is_some() {
                    ex.input_xlat_rc("JLPLLCLK0")
                        .input_xlat_rc("JLPLLCLK1")
                        .input_xlat_rc("JLPLLCLK2")
                        .input_xlat_rc("JLPLLCLK3");
                }
                if ex.ctx.chip.try_bel_pll(DirHV::NE, 0).is_some() {
                    ex.input_xlat_rc("JRPLLCLK0")
                        .input_xlat_rc("JRPLLCLK1")
                        .input_xlat_rc("JRPLLCLK2")
                        .input_xlat_rc("JRPLLCLK3");
                }
            }
        }

        for i in 0..2 {
            let ii = i + 6;
            self.extract_bel_cell(bcrd.bel(bslots::DCM[i]), cell)
                .name(format!("DCM{ii}"))
                .input_xlat_rc(bcls::DCM::SEL, format!("JSEL{ii}_DCM"))
                .input_xlat_rc(bcls::DCM::CLK[0], format!("CLK0_{ii}_DCM"))
                .input_xlat_rc(bcls::DCM::CLK[1], format!("CLK1_{ii}_DCM"))
                .output_bind_rc(
                    bcls::DCM::DCMOUT,
                    format!("DCMOUT{ii}_DCM"),
                    wires::OUT_DCM_DCMOUT[i].cell(0),
                )
                .claim_pip("DCMOUT", "CLK0")
                .claim_pip("DCMOUT", "CLK1");
        }

        for i in 0..8 {
            self.extract_bel_cell(bcrd.bel(bslots::DCC[i]), cell)
                .name(format!("DCC{i}"))
                .input_xlat_rc(bcls::DCC::CLKI, format!("CLKI{i}_DCC"))
                .input_xlat_rc(bcls::DCC::CE, format!("JCE{i}_DCC"))
                .output_xlat_rc(bcls::DCC::CLKO, format!("CLKO{i}_DCC"))
                .claim_pip("CLKO", "CLKI");
        }

        self.extract_bel_cell(bcrd.bel(bslots::CLKTEST), cell)
            .name("CENTEST")
            .suffix("CENTEST");

        if !is_smol {
            for i in 0..2 {
                let bcrd = bcrd.bel(bslots::ECLKBRIDGECS[i]);
                self.extract_bel_cell(bcrd, cell)
                    .name(format!("ECLKBRIDGECS{i}"))
                    .input_xlat_rc(bcls::ECLKBRIDGECS::SEL, format!("JSEL{i}_ECLKBRIDGECS"))
                    .input_xlat_rc(bcls::ECLKBRIDGECS::CLK[0], format!("CLK0_{i}_ECLKBRIDGECS"))
                    .input_xlat_rc(bcls::ECLKBRIDGECS::CLK[1], format!("CLK1_{i}_ECLKBRIDGECS"))
                    .output_xlat_rc(
                        bcls::ECLKBRIDGECS::ECSOUT,
                        format!("JECSOUT{i}_ECLKBRIDGECS"),
                    );
            }
        }
    }
}
