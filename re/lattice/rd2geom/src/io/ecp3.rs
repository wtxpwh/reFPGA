use prjcombine_ecp::{
    chip::{Chip, ChipKind, IoGroupKind, IoKind, SpecialIoKey},
    defs::{bcls, bslots, tcls, wires},
};
use prjcombine_interconnect::{
    db::{LegacyBel, WireSlotIdExt},
    dir::{Dir, DirH, DirHV, DirV},
    grid::{BelCoord, CellCoord},
};

use crate::ChipContext;

impl ChipContext<'_> {
    pub fn xlat_io_loc_ecp3(&self, bcrd: BelCoord) -> (CellCoord, &'static str) {
        let edge = self.chip.get_io_edge(bcrd);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        match edge {
            Dir::H(h) => {
                if bcrd.col != self.chip.col_edge(h) {
                    let abcd = ["EA", "EB", "EC", "ED"][idx];
                    (bcrd.with_col(self.chip.col_edge(h)), abcd)
                } else {
                    let cell = if idx < 2 {
                        bcrd.cell.delta(0, 2)
                    } else {
                        bcrd.cell
                    };
                    let abcd = ["A", "B"][idx % 2];
                    (cell, abcd)
                }
            }
            Dir::V(_) => {
                let cell = if idx < 2 {
                    bcrd.cell
                } else {
                    bcrd.cell.delta(2, 0)
                };
                let abcd = ["A", "B"][idx % 2];
                (cell, abcd)
            }
        }
    }

    fn prep_single_io_ecp3(&mut self, bcrd: BelCoord) {
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let wire = bcrd.wire(wires::OUT_IO_PADDI[idx]);
        let wname = self.get_paddi_wire(wire);
        self.bind_wire(wname, wire);
    }

    pub(super) fn prep_io_ecp3(&mut self) {
        for cell in self.edev.column(Chip::DIE, self.chip.col_w()) {
            if matches!(
                self.chip.rows[cell.row].io_w,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                for iob in 0..4 {
                    self.prep_single_io_ecp3(cell.bel(bslots::IO[iob]));
                }
            }
        }
        for cell in self.edev.column(Chip::DIE, self.chip.col_e()) {
            if matches!(
                self.chip.rows[cell.row].io_e,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                for iob in 0..4 {
                    self.prep_single_io_ecp3(cell.bel(bslots::IO[iob]));
                }
            }
        }
        for cell in self.edev.row(Chip::DIE, self.chip.row_s()) {
            if matches!(
                self.chip.columns[cell.col].io_s,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                for iob in 0..4 {
                    self.prep_single_io_ecp3(cell.bel(bslots::IO[iob]));
                }
            }
        }
        for cell in self.edev.row(Chip::DIE, self.chip.row_n()) {
            if matches!(
                self.chip.columns[cell.col].io_n,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                for iob in 0..4 {
                    self.prep_single_io_ecp3(cell.bel(bslots::IO[iob]));
                }
            }
        }
        for (_loc, bel_pll) in self.chip.plls() {
            for iob in 0..4 {
                self.prep_single_io_ecp3(bel_pll.bel(bslots::IO[iob]));
            }
        }
    }

    fn process_single_io_ecp3(&mut self, bcrd: BelCoord) {
        let edge = self.chip.get_io_edge(bcrd);
        let idx = bslots::IO.index_of(bcrd.slot).unwrap();
        let (cell, abcd) = self.xlat_io_loc_ecp3(bcrd);
        let (name_pio, name_iol) = match edge {
            Dir::H(h) => {
                let lr = match h {
                    DirH::W => 'L',
                    DirH::E => 'R',
                };
                let (r, _) = self.rc(cell);
                if bcrd.col != self.chip.col_edge(h) {
                    let e_abcd = ["E_A", "E_B", "E_C", "E_D"][idx];
                    (format!("P{lr}{r}{e_abcd}"), format!("IOL_{lr}{r}{abcd}"))
                } else {
                    (format!("P{lr}{r}{abcd}"), format!("IOL_{lr}{r}{abcd}"))
                }
            }
            Dir::V(v) => {
                let bt = match v {
                    DirV::S => 'B',
                    DirV::N => 'T',
                };
                let (_, c) = self.rc(cell);
                (format!("P{bt}{c}{abcd}"), format!("IOL_{bt}{c}{abcd}"))
            }
        };
        self.name_bel(bcrd, [name_pio, name_iol]);
        let kind = self.chip.get_io_kind(bcrd);
        let (iol, iol_alt) = match kind {
            IoKind::Dummy => unreachable!(),
            IoKind::Io => ("IOLOGIC", "IOLOGIC"),
            IoKind::Sio => ("SIOLOGIC", "SIOLOGIC"),
            IoKind::Xsio => ("XSIOLOGIC", "XSIOLOGIC"),
            IoKind::IoPll => ("IOLOGIC", "XSIOLOGIC"),
            IoKind::Dqs => ("DQSIOL", "DQSIOL"),
            IoKind::SDqs => ("SDQSIOL", "SDQSIOL"),
        };

        let mut bel = self.extract_bel_cell(bcrd, cell);

        let paddi = bel.add_wire_rc_io_get("PADDI", format!("JPADDI{abcd}_PIO"));
        let paddo = bel.add_wire_rc_io_get("PADDO", format!("JPADDO{abcd}_PIO"));
        let paddt = bel.add_wire_rc_io_get("PADDT", format!("JPADDT{abcd}_PIO"));
        let di = bel.add_wire_rc_io_get("DI", format!("DI{abcd}_{iol_alt}"));
        bel.ctx.claim_pip(di, paddi);

        let ioldo_pio = bel.add_wire_rc_io_get("IOLDO_PIO", format!("IOLDO{abcd}_PIO"));
        let ioldo_iol = bel.add_wire_rc_io_get("IOLDO_IOLOGIC", format!("IOLDO{abcd}_{iol_alt}"));
        bel.ctx.claim_pip(ioldo_pio, ioldo_iol);

        let iolto_pio = bel.add_wire_rc_io_get("IOLTO_PIO", format!("IOLTO{abcd}_PIO"));
        let iolto_iol = bel.add_wire_rc_io_get("IOLTO_IOLOGIC", format!("IOLTO{abcd}_{iol_alt}"));
        bel.ctx.claim_pip(iolto_pio, iolto_iol);

        bel.input_xlat_rc_io(bcls::IO::CLK, format!("JCLK{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::LSR, format!("JLSR{abcd}_{iol}"))
            .input_xlat_rc_io(bcls::IO::CE, format!("JCE{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::INB, format!("JINB{abcd}_{iol}"))
            .output_xlat_rc_io(bcls::IO::INDD, format!("JINDD{abcd}_{iol}"));
        for i in 0..4 {
            bel.input_xlat_rc_io(bcls::IO::DEL[i], format!("JDEL{i}{abcd}_{iol}"));
        }

        if kind != IoKind::IoPll {
            bel.input_xlat_rc_io(bcls::IO::TS, format!("JTS{abcd}_{iol}"))
                .input_xlat_rc_io(bcls::IO::OPOSA, format!("JOPOSA{abcd}_{iol}"));
        }

        if !matches!(kind, IoKind::IoPll | IoKind::Xsio) {
            bel.input_xlat_rc_io(bcls::IO::ONEGB, format!("JONEGB{abcd}_{iol}"));
        }

        if matches!(kind, IoKind::Io | IoKind::Dqs) {
            bel.input_xlat_rc_io(bcls::IO::OPOSB, format!("JOPOSB{abcd}_{iol}"));
        }
        if kind == IoKind::Io {
            bel.input_xlat_rc_io(bcls::IO::ONEGA, format!("JONEGA{abcd}_{iol}"));
        }

        if kind != IoKind::Xsio {
            bel.output_xlat_rc_io(bcls::IO::IPA, format!("JIPA{abcd}_{iol}"))
                .output_xlat_rc_io(bcls::IO::IPB, format!("JIPB{abcd}_{iol}"))
                .output_xlat_rc_io(bcls::IO::INA, format!("JINA{abcd}_{iol}"));
        }

        {
            let bpin = bel.ctx.xlat_output(bcrd, paddi);
            assert_eq!(bpin, bel.bel.outputs[bcls::IO::INDD]);
        }

        if kind == IoKind::IoPll {
            for pin in ["OPOSA", "OPOSB", "ONEGA", "ONEGB", "TS"] {
                bel.add_wire_rc_io(pin, format!("J{pin}{abcd}_{iol}"));
            }
        } else {
            let bpin = bel.ctx.xlat_input(bcrd, paddo);
            assert_eq!(bpin, bel.bel.inputs[bcls::IO::OPOSA]);
            let bpin = bel.ctx.xlat_input(bcrd, paddt);
            assert_eq!(bpin, bel.bel.inputs[bcls::IO::TS]);
        }

        if matches!(kind, IoKind::Dqs | IoKind::SDqs) {
            let dqsw = bel.add_wire_rc_io_get("DQSW", format!("JDQSW{abcd}_{iol}"));
            let bcrd_dqs = bcrd.bel(bslots::DQS[0]);
            let dqsw_dqs = bel.ctx.naming.bel_wire(bcrd_dqs, "DQSW");
            bel.ctx.claim_pip(dqsw, dqsw_dqs);

            let dqstclko = bel.add_wire_rc_io_get("DQSTCLKO", format!("DQSTCLKO{abcd}_{iol}"));
            let dqstclki = bel.add_wire_rc_io_get("DQSTCLKI", format!("DQSTCLKI{abcd}_{iol}"));
            bel.ctx.claim_pip(dqstclki, dqstclko);
        }

        if kind != IoKind::Xsio {
            bel.input_xlat_rc_io(bcls::IO::ECLK, format!("ECLK{abcd}_{iol}"));
            bel.input_xlat_rc_io(bcls::IO::ECLKDQSR, format!("ECLKDQSR{abcd}_{iol}"));

            let bcrd_dqs = bel.ctx.edev.bel_io_dqs(bcrd).unwrap();
            let mut pins = vec!["DQCLK1", "DDRCLKPOL", "DDRLAT"];
            if matches!(kind, IoKind::Io | IoKind::Dqs | IoKind::IoPll) {
                pins.push("DQCLK0");
            }
            for pin in pins {
                let wire = bel.add_wire_rc_io_get(pin, format!("{pin}{abcd}_{iol}"));
                let wire_dqs = bel.ctx.naming.bel_wire(bcrd_dqs, &format!("{pin}_OUT"));
                bel.ctx.claim_pip(wire, wire_dqs);
                if pin == "DQCLK1"
                    && matches!(kind, IoKind::Sio | IoKind::SDqs)
                    && bel.ctx.chip.kind == ChipKind::Ecp3
                {
                    let dqclk1_in = bel.add_wire_rc_io_get("DQCLK1_IN", format!("JDQCLK1{abcd}"));
                    bel.ctx.claim_pip(wire, dqclk1_in);
                    let bpin = bel.ctx.xlat_input(bcrd, dqclk1_in);
                    assert_eq!(bel.bel.inputs[bcls::IO::CLK], bpin);
                }
            }
        }
    }

    fn process_io_cell_ecp3(&mut self, cell: CellCoord) {
        let chip = self.chip;
        let io0 = cell.bel(bslots::IO[0]);
        let edge = chip.get_io_edge(io0);
        if chip.get_io_kind(io0) != IoKind::Xsio {
            let eclk0 = self
                .naming
                .bel_wire(chip.bel_eclksync(edge, 0), "ECLKO_OUT");
            let eclk1 = self
                .naming
                .bel_wire(chip.bel_eclksync(edge, 1), "ECLKO_OUT");
            let bcrd_dqs = self.edev.bel_io_dqs(io0).unwrap();
            let wire_dqs = self.naming.bel_wire(bcrd_dqs, "ECLKDQSR_OUT");
            let mut ex = self.extract_switchbox(cell.bel(bslots::IO_INT));
            ex.input_xlat(eclk0).input_xlat(eclk1).input_xlat(wire_dqs);
            for iob in 0..4 {
                let bcrd = cell.bel(bslots::IO[iob]);
                if chip.get_io_kind(bcrd) != IoKind::Xsio {
                    let (cell_io, abcd) = ex.ctx.xlat_io_loc_ecp3(bcrd);
                    ex.cell(cell_io)
                        .output_bind_rc_io(format!("ECLK{abcd}"), wires::IMUX_IO_ECLK[iob].cell(0));
                    if chip.kind != ChipKind::Ecp3A {
                        ex.output_bind_rc_io(
                            format!("ECLKDQSR{abcd}"),
                            wires::IMUX_IO_ECLKDQSR[iob].cell(0),
                        );
                    }
                }
            }
        }
        for iob in 0..4 {
            self.process_single_io_ecp3(cell.bel(bslots::IO[iob]));
        }
    }

    fn process_dqs_ecp3(&mut self, bcrd_io: BelCoord) {
        let chip = self.chip;
        let edge = chip.get_io_edge(bcrd_io);
        let bcrd_dqs = bcrd_io.bel(bslots::DQS[0]);
        let kind = chip.get_io_kind(bcrd_io);
        let cell = match edge {
            Dir::H(_) => bcrd_dqs.cell.delta(0, 1),
            Dir::V(_) => bcrd_dqs.cell.delta(1, 0),
        };

        let eclk0 = self
            .naming
            .bel_wire(chip.bel_eclksync(edge, 0), "ECLKO_OUT");
        let eclk1 = self
            .naming
            .bel_wire(chip.bel_eclksync(edge, 1), "ECLKO_OUT");
        self.extract_switchbox(bcrd_dqs.bel(bslots::BEL_INT))
            .cell(cell)
            .output_bind_rc_io("ECLK", wires::IMUX_DQS_ECLK.cell(0))
            .output_bind_rc_io("ECLKW", wires::IMUX_DQS_ECLKW.cell(0))
            .input_xlat(eclk0)
            .input_xlat(eclk1);

        let (r, c) = self.rc(cell);
        let name = match edge {
            Dir::W => format!("DQS_L{r}"),
            Dir::E => format!("DQS_R{r}"),
            Dir::N => format!("DQS_T{c}"),
            _ => unreachable!(),
        };
        let suffix = match kind {
            IoKind::Io | IoKind::Dqs => "DQS",
            IoKind::SDqs => "SDQS",
            _ => unreachable!(),
        };
        let mut pins = vec![
            ("DQCLK1", "DQCLK1"),
            (
                "DDRLAT",
                if chip.kind == ChipKind::Ecp3A && kind != IoKind::SDqs {
                    "DDRLAT"
                } else {
                    "JDDRLAT"
                },
            ),
            ("DDRCLKPOL", "JDDRCLKPOL"),
        ];
        if kind != IoKind::SDqs {
            pins.extend([("DQCLK0", "DQCLK0")]);
        }

        let eclkdqsr = self.rc_io_wire(cell, &format!("JECLKDQSR_{suffix}"));
        let eclkdqsr_out = self.pips_fwd[&eclkdqsr]
            .iter()
            .copied()
            .find(|w| !self.int_wires.contains_key(w))
            .unwrap();
        self.bind_wire(eclkdqsr_out, bcrd_dqs.wire(wires::OUT_DQS_ECLKDQSR));

        let mut bel = self.extract_bel_cell(bcrd_dqs, cell);

        if matches!(edge, Dir::H(_)) && chip.kind == ChipKind::Ecp3A {
            let name_test = match edge {
                Dir::W => format!("DQSTEST_L{r}"),
                Dir::E => format!("DQSTEST_R{r}"),
                _ => unreachable!(),
            };
            bel.names([name, name_test])
                .output_xlat_rc_io(bcls::DQS::TEST_DQSCNT[0], "JDQSCNT0_DQSTEST")
                .output_xlat_rc_io(bcls::DQS::TEST_DQSCNT[1], "JDQSCNT1_DQSTEST")
                .output_xlat_rc_io(bcls::DQS::TEST_ECLKCNT, "JECLKCNT_DQSTEST");
        } else {
            bel.name(name);
        }

        bel.suffix(suffix)
            .add_wire("ECLKDQSR_OUT", eclkdqsr_out)
            .input_xlat_rc_io(bcls::DQS::ECLK, format!("ECLK_{suffix}"))
            .input_xlat_rc_io(bcls::DQS::ECLKW, format!("ECLKW_{suffix}"));

        for (pin, name) in pins {
            let wire = bel.ctx.rc_io_wire(cell, &format!("{name}_{suffix}"));
            if !name.starts_with('J') {
                bel.add_wire(pin, wire);
            }
            let wire_out = bel.ctx.pips_fwd[&wire]
                .iter()
                .copied()
                .find(|w| !bel.ctx.int_wires.contains_key(w))
                .unwrap();
            bel.ctx.claim_pip(wire_out, wire);
            bel.add_wire(format!("{pin}_OUT"), wire_out);
        }

        let dqsi = bel.add_wire_rc_io_get("DQSI", format!("JDQSI_{suffix}"));
        if kind != IoKind::Io {
            let wire_io = bel.ctx.get_io_wire_in(bcrd_io);
            bel.ctx.claim_pip(dqsi, wire_io);
        }

        let dqsdel = bel.add_wire_rc_io_get("DQSDEL", format!("JDQSDEL_{suffix}"));
        let bcrd_dqsdll = chip.bel_dqsdll(cell);
        let dqsdel_dqsdll = bel.ctx.naming.bel_wire(bcrd_dqsdll, "DQSDEL");
        bel.ctx.claim_pip(dqsdel, dqsdel_dqsdll);
    }

    pub(super) fn process_io_ecp3(&mut self) {
        // DQS
        for cell in self.edev.column(Chip::DIE, self.chip.col_w()) {
            if matches!(
                self.chip.rows[cell.row].io_w,
                IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_dqs_ecp3(cell.bel(bslots::IO[2]));
            }
        }
        for cell in self.edev.column(Chip::DIE, self.chip.col_e()) {
            if matches!(
                self.chip.rows[cell.row].io_e,
                IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_dqs_ecp3(cell.bel(bslots::IO[2]));
            }
        }
        for cell in self.edev.row(Chip::DIE, self.chip.row_n()) {
            if matches!(
                self.chip.columns[cell.col].io_n,
                IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_dqs_ecp3(cell.bel(bslots::IO[2]));
            }
        }
        // actual IO
        for cell in self.edev.column(Chip::DIE, self.chip.col_w()) {
            if matches!(
                self.chip.rows[cell.row].io_w,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_io_cell_ecp3(cell);
            }
        }
        for cell in self.edev.column(Chip::DIE, self.chip.col_e()) {
            if matches!(
                self.chip.rows[cell.row].io_e,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_io_cell_ecp3(cell);
            }
        }
        for cell in self.edev.row(Chip::DIE, self.chip.row_s()) {
            if matches!(
                self.chip.columns[cell.col].io_s,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_io_cell_ecp3(cell);
            }
        }
        for cell in self.edev.row(Chip::DIE, self.chip.row_n()) {
            if matches!(
                self.chip.columns[cell.col].io_n,
                IoGroupKind::Quad | IoGroupKind::QuadDqs | IoGroupKind::QuadDqsDummy
            ) {
                self.process_io_cell_ecp3(cell);
            }
        }
        for (_loc, bel_pll) in self.chip.plls() {
            self.process_io_cell_ecp3(bel_pll.cell);
        }
    }

    pub(super) fn process_eclk_ecp3(&mut self) {
        for (edge, lrt) in [(Dir::W, 'L'), (Dir::E, 'R'), (Dir::N, 'T')] {
            for idx in 0..2 {
                let idxp1 = idx + 1;
                let bcrd = self.chip.bel_eclksync(edge, idx);
                let cell = bcrd.cell;
                self.name_bel(bcrd, [format!("{lrt}ECLKSYNC{idxp1}")]);
                let mut bel = LegacyBel::default();

                let stop = self.rc_wire(cell, &format!("JSTOP{idxp1}_ECLKSYNC"));
                self.add_bel_wire(bcrd, "STOP", stop);
                bel.pins
                    .insert("STOP".into(), self.xlat_int_wire(bcrd, stop));

                let wire_eclk = cell.wire(wires::ECLK[idx]);
                let eclko = self.rc_wire(cell, &format!("JECLKO{idxp1}_ECLKSYNC"));
                self.add_bel_wire(bcrd, "ECLKO", eclko);
                self.bind_wire(eclko, wire_eclk);

                let eclk_name = format!("{lrt}ECLK{idxp1}");
                let eclko_out = self.pips_fwd[&eclko]
                    .iter()
                    .copied()
                    .find(|wn| self.naming.strings[wn.suffix].starts_with(&eclk_name))
                    .unwrap();
                self.claim_pip(eclko_out, eclko);
                self.add_bel_wire(bcrd, "ECLKO_OUT", eclko_out);
                self.bind_wire(eclko_out, wire_eclk);

                let eclki = self.rc_wire(cell, &format!("ECLKI{idxp1}_ECLKSYNC"));
                self.add_bel_wire(bcrd, "ECLKI", eclki);
                self.claim_pip(eclko, eclki);

                let eclki_in = self.claim_single_in(eclki);
                self.add_bel_wire(bcrd, "ECLKI_IN", eclki_in);

                let eclki_io = self.rc_wire(cell, &format!("JPIO{idxp1}"));
                self.add_bel_wire(bcrd, "ECLKI_IO", eclki_io);
                self.claim_pip(eclki_in, eclki_io);
                let wire_io = self.get_special_io_wire_in(SpecialIoKey::Clock(edge, idx as u8));
                self.claim_pip(eclki_io, wire_io);

                let eclki_int = self.rc_wire(cell, &format!("JCIBCLK{idxp1}"));
                self.add_bel_wire(bcrd, "ECLKI_INT", eclki_int);
                self.claim_pip(eclki_in, eclki_int);
                bel.pins
                    .insert("ECLKI".into(), self.xlat_int_wire(bcrd, eclki_int));

                if self.chip.kind == ChipKind::Ecp3A {
                    let eclki_center = self.rc_wire(cell, &format!("JBRGECLK{idxp1}"));
                    self.add_bel_wire(bcrd, "ECLKI_CENTER", eclki_center);
                    self.claim_pip(eclki_in, eclki_center);
                    let wire_center = self.rc_wire(
                        self.chip.bel_clk_root().delta(-1, 0),
                        &format!("JBRGECLK{idxp1}"),
                    );
                    self.claim_pip(eclki_center, wire_center);
                }

                if edge == Dir::N {
                    let cell_dll_w = self.chip.bel_pll(DirHV::NW, 0).cell;
                    let cell_dll_e = self.chip.bel_pll(DirHV::NE, 0).cell;
                    let cell_pll_w = self
                        .chip
                        .try_bel_pll(DirHV::NW, 1)
                        .map(|bcrd| bcrd.cell)
                        .unwrap_or(cell_dll_w)
                        .delta(3, 0);
                    let cell_pll_e = self
                        .chip
                        .try_bel_pll(DirHV::NE, 1)
                        .map(|bcrd| bcrd.cell)
                        .unwrap_or(cell_dll_e)
                        .delta(-3, 0);
                    let cell_dll_w = cell_dll_w.delta(13, 0);
                    let cell_dll_e = cell_dll_e.delta(-13, 0);
                    for (tgt, pin, name, cell_src, name_src) in [
                        (
                            0,
                            "ECLKI_PLL_W_CLKOP",
                            "JPLLCLKOP1",
                            cell_pll_w,
                            "JCLKOP_PLL",
                        ),
                        (
                            1,
                            "ECLKI_PLL_E_CLKOP",
                            "JPLLCLKOP2",
                            cell_pll_e,
                            "JCLKOP_PLL",
                        ),
                        (
                            1,
                            "ECLKI_PLL_W_CLKOS",
                            "JPLLCLKOS2",
                            cell_pll_w,
                            "JCLKOS_PLL",
                        ),
                        (
                            0,
                            "ECLKI_PLL_E_CLKOS",
                            "JPLLCLKOS1",
                            cell_pll_e,
                            "JCLKOS_PLL",
                        ),
                        (
                            1,
                            "ECLKI_DLL_W_CLKOS",
                            "JDLLCLKOS2",
                            cell_dll_w,
                            "JCLKOS_DLL",
                        ),
                        (
                            0,
                            "ECLKI_DLL_E_CLKOS",
                            "JDLLCLKOS1",
                            cell_dll_e,
                            "JCLKOS_DLL",
                        ),
                        (0, "ECLKI_DLLDEL_W", "JDLLDEL1", cell_dll_w, "JCLKO_DLLDEL"),
                        (1, "ECLKI_DLLDEL_E", "JDLLDEL2", cell_dll_e, "JCLKO_DLLDEL"),
                    ] {
                        if tgt != idx {
                            continue;
                        }
                        let eclki_pll = self.rc_wire(cell, name);
                        self.add_bel_wire(bcrd, pin, eclki_pll);
                        self.claim_pip(eclki_in, eclki_pll);
                        let wire = self.rc_wire(cell_src, name_src);
                        self.claim_pip(eclki_pll, wire);
                    }
                } else {
                    let Dir::H(edge) = edge else {
                        unreachable!();
                    };
                    let eclki_dll = self.rc_wire(cell, &format!("JDLLECLK{idxp1}"));
                    self.add_bel_wire(bcrd, "ECLKI_DLL", eclki_dll);
                    self.claim_pip(eclki_in, eclki_dll);
                    let cell_dll = match edge {
                        DirH::W => cell.delta(14, 0),
                        DirH::E => cell.delta(-14, 0),
                    };
                    let wire_dll = self.rc_wire(cell_dll, &format!("JDLLECLK{idxp1}"));
                    self.claim_pip(eclki_dll, wire_dll);

                    if self.chip.kind == ChipKind::Ecp3A
                        && let Some(bel_pll) = self.chip.try_bel_pll(edge & DirV::S, 0)
                    {
                        let cell_pll = match edge {
                            DirH::W => bel_pll.delta(3, 0),
                            DirH::E => bel_pll.delta(-3, 0),
                        };
                        let pin = ["CLKOS", "CLKOP"][idx];
                        let eclki_pll = self.rc_wire(cell, &format!("JPLL{pin}"));
                        self.add_bel_wire(bcrd, format!("ECLKI_PLL_{pin}"), eclki_pll);
                        self.claim_pip(eclki_in, eclki_pll);
                        let wire_pll = self.rc_wire(cell_pll, &format!("J{pin}_PLL"));
                        self.claim_pip(eclki_pll, wire_pll);
                    }
                }

                self.insert_legacy_bel(bcrd, bel);
            }
        }
    }

    pub(super) fn process_dqsdll_ecp3(&mut self) {
        for edge in [DirH::W, DirH::E] {
            let bcrd = self.chip.bel_dqsdll_ecp2(edge);
            let cell = match edge {
                DirH::W => bcrd.cell.delta(2, 0),
                DirH::E => bcrd.cell.delta(-2, 0),
            };
            self.name_bel(
                bcrd,
                [match edge {
                    DirH::W => "LDQSDLL",
                    DirH::E => "RDQSDLL",
                }],
            );
            let mut bel = self.extract_simple_legacy_bel(bcrd, cell, "DQSDLL");
            let dqsdel = self.rc_wire(cell, "JDQSDEL_DQSDLL");
            self.add_bel_wire(bcrd, "DQSDEL", dqsdel);
            if self.chip.kind == ChipKind::Ecp3A {
                let clk = self.rc_wire(cell, "CLK_DQSDLL");
                self.add_bel_wire(bcrd, "CLK", clk);

                let clk_in = self.rc_wire(cell, "DQSDLLCLK");
                self.add_bel_wire(bcrd, "CLK_IN", clk_in);
                self.claim_pip(clk, clk_in);

                let clk_int = self.rc_wire(cell, "JCIBCLK");
                self.add_bel_wire(bcrd, "CLK_INT", clk_int);
                self.claim_pip(clk_in, clk_int);
                bel.pins
                    .insert("CLK".into(), self.xlat_int_wire(bcrd, clk_int));

                let clk_eclk = self.rc_wire(cell, "JECLK");
                self.add_bel_wire(bcrd, "CLK_ECLK", clk_eclk);
                self.claim_pip(clk_in, clk_eclk);
                let bcrd_eclk = self.chip.bel_eclksync(Dir::H(edge), 0);
                let wire_eclk = self.naming.bel_wire(bcrd_eclk, "ECLKO");
                self.claim_pip(clk_eclk, wire_eclk);
            }
            self.insert_legacy_bel(bcrd, bel);

            let bcrd = bcrd.bel(bslots::DQSDLLTEST);
            self.name_bel(
                bcrd,
                [match edge {
                    DirH::W => "LDQSDLLTEST",
                    DirH::E => "RDQSDLLTEST",
                }],
            );
            self.insert_simple_legacy_bel(bcrd, cell, "DQSDLLTEST");
        }
    }

    pub(super) fn process_eclk_tap_ecp3(&mut self) {
        let tcid = tcls::ECLK_TAP;
        for &tcrd in &self.edev.tile_index[tcid] {
            let edge = if tcrd.col == self.chip.col_w() {
                Dir::W
            } else if tcrd.col == self.chip.col_e() {
                Dir::E
            } else if tcrd.row == self.chip.row_n() {
                Dir::N
            } else {
                unreachable!()
            };
            for i in 0..2 {
                let bel_eclksync = self.chip.bel_eclksync(edge, i);
                let eclk_out = tcrd.wire(wires::OUT_F[6 + i]);
                let eclk = self.naming.bel_wire(bel_eclksync, "ECLKO_OUT");
                self.claim_pip_int_out(eclk_out, eclk);
            }
        }
    }
}
