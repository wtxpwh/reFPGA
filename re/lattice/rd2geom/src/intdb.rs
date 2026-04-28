use prjcombine_ecp::{
    chip::ChipKind,
    defs::{bslots, tslots, wires},
};
use prjcombine_interconnect::db::{
    BelInfo, BelPin, BelSlotId, IntDb, LegacyBel, PermaBuf, ProgBuf, SwitchBox, SwitchBoxItem,
    TileClass, TileSlotId, WireSlotId, WireSlotIdExt,
};
use prjcombine_types::bsdata::PolTileBit;

struct TileClassBuilder<'db> {
    db: &'db mut IntDbBuilder,
    name: String,
    class: TileClass,
}

impl<'db> TileClassBuilder<'db> {
    fn bel<'tcls>(&'tcls mut self, slot: BelSlotId) -> BelBuilder<'db, 'tcls> {
        BelBuilder {
            tcls: self,
            slot,
            bel: LegacyBel::default(),
        }
    }
}

impl Drop for TileClassBuilder<'_> {
    fn drop(&mut self) {
        self.db.db.tile_classes.insert(
            std::mem::take(&mut self.name),
            std::mem::replace(&mut self.class, TileClass::new(tslots::INT, 1)),
        );
    }
}

struct BelBuilder<'db, 'tcls> {
    tcls: &'tcls mut TileClassBuilder<'db>,
    slot: BelSlotId,
    bel: LegacyBel,
}

impl BelBuilder<'_, '_> {
    fn add_input(&mut self, name: &str, cell: usize, wire: WireSlotId) {
        self.bel
            .pins
            .insert(name.into(), BelPin::new_in(wire.cell(cell)));
    }

    fn add_output(&mut self, name: &str, cell: usize, wire: WireSlotId) {
        self.bel
            .pins
            .insert(name.into(), BelPin::new_out(wire.cell(cell)));
    }
}

impl Drop for BelBuilder<'_, '_> {
    fn drop(&mut self) {
        self.tcls
            .class
            .bels
            .insert(self.slot, BelInfo::Legacy(std::mem::take(&mut self.bel)));
    }
}

struct IntDbBuilder {
    kind: ChipKind,
    db: IntDb,
}

impl IntDbBuilder {
    fn tile_class(
        &mut self,
        name: impl Into<String>,
        slot: TileSlotId,
        num_cells: usize,
    ) -> TileClassBuilder<'_> {
        TileClassBuilder {
            db: self,
            name: name.into(),
            class: TileClass::new(slot, num_cells),
        }
    }

    fn fill_int_tiles(&mut self) {
        let int_tiles = match self.kind {
            ChipKind::Scm => ["INT_PLC", "INT_EBR", "INT_IO"].as_slice(),
            ChipKind::MachXo => [
                "INT_PLC",
                "INT_SIO_S_W",
                "INT_SIO_S_W_CLK",
                "INT_SIO_S_E",
                "INT_SIO_S_E_CFG",
                "INT_SIO_S_S4",
                "INT_SIO_S_S6",
                "INT_SIO_S_N4",
                "INT_SIO_S_N6",
                "INT_SIO_L_W",
                "INT_SIO_L_E",
                "INT_SIO_L_E_CFG",
                "INT_SIO_L_S4",
                "INT_SIO_L_S6",
                "INT_SIO_L_N4",
                "INT_SIO_L_N6",
            ]
            .as_slice(),
            ChipKind::Ecp | ChipKind::Xp => {
                ["INT_PLC", "INT_IO_WE", "INT_IO_SN", "INT_EBR", "INT_PLL"].as_slice()
            }
            ChipKind::Ecp2 | ChipKind::Xp2 => [
                "INT_PLC",
                "INT_IO_WE",
                "INT_IO_S",
                "INT_IO_N",
                "INT_EBR",
                "INT_EBR_IO",
            ]
            .as_slice(),
            ChipKind::Ecp2M => [
                "INT_PLC",
                "INT_IO_WE",
                "INT_IO_S",
                "INT_IO_N",
                "INT_EBR",
                "INT_EBR_IO",
                "INT_SERDES_N",
            ]
            .as_slice(),
            ChipKind::Ecp3 | ChipKind::Ecp3A => [
                "INT_PLC",
                "INT_IO_WE",
                "INT_IO_S",
                "INT_IO_N",
                "INT_EBR",
                "INT_EBR_IO",
                "INT_EBR_SCLK",
                "INT_EBR_IO_SCLK",
            ]
            .as_slice(),
            ChipKind::MachXo2(_) => {
                ["INT_PLC", "INT_IO_WE", "INT_IO_S", "INT_IO_N", "INT_EBR"].as_slice()
            }
            ChipKind::Ecp4 => {
                ["INT_PLC", "INT_IO_WE", "INT_IO_S", "INT_IO_N", "INT_EBR"].as_slice()
            }
            ChipKind::Ecp5 => {
                ["INT_PLC", "INT_IO_WE", "INT_IO_S", "INT_IO_N", "INT_EBR"].as_slice()
            }
            ChipKind::Crosslink => ["INT_PLC", "INT_IO_S", "INT_IO_N", "INT_EBR"].as_slice(),
        };
        for &name in int_tiles {
            let mut tcls = self.tile_class(name, tslots::INT, 1);
            tcls.class
                .bels
                .insert(bslots::INT, BelInfo::SwitchBox(SwitchBox::default()));
        }

        if self.kind == ChipKind::MachXo {
            self.tile_class("CIBTEST_SEL", tslots::BEL, 1)
                .bel(bslots::CIBTEST_SEL);
        }
    }

    fn fill_pclk_tiles_ecp3(&mut self) {
        for i in 0..4 {
            {
                let mut tcls = self.tile_class(format!("PCLK{i}_SOURCE"), tslots::PCLK_SOURCE, 2);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if !(i == 2 && matches!(self.kind, ChipKind::Ecp3 | ChipKind::Ecp3A)) {
                let mut tcls = self.tile_class(format!("PCLK{i}_SOURCE_W"), tslots::PCLK_SOURCE, 2);
                tcls.bel(bslots::PCLK_SOURCE_W);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if !(i == 1 && matches!(self.kind, ChipKind::Ecp3 | ChipKind::Ecp3A)) {
                let mut tcls = self.tile_class(format!("PCLK{i}_SOURCE_E"), tslots::PCLK_SOURCE, 2);
                tcls.bel(bslots::PCLK_SOURCE_E);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }
        }
    }

    fn fill_pclk_tiles_machxo2(&mut self) {
        for i in 0..4 {
            for name in [format!("PCLK{i}_SOURCE_IO_N"), format!("PCLK{i}_SOURCE_N")] {
                let mut tcls = self.tile_class(name, tslots::PCLK_SOURCE, 1);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if matches!(i, 0 | 3) {
                let mut tcls =
                    self.tile_class(format!("PCLK{i}_SOURCE_IO_N_W"), tslots::PCLK_SOURCE, 1);
                tcls.bel(bslots::PCLK_SOURCE_W);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if i == 1 {
                let mut tcls =
                    self.tile_class(format!("PCLK{i}_SOURCE_N_W"), tslots::PCLK_SOURCE, 1);
                tcls.bel(bslots::PCLK_SOURCE_W);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if matches!(i, 0 | 1) {
                let mut tcls =
                    self.tile_class(format!("PCLK{i}_SOURCE_IO_N_E"), tslots::PCLK_SOURCE, 1);
                tcls.bel(bslots::PCLK_SOURCE_E);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if i == 1 {
                let mut tcls =
                    self.tile_class(format!("PCLK{i}_SOURCE_N_E"), tslots::PCLK_SOURCE, 1);
                tcls.bel(bslots::PCLK_SOURCE_E);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            {
                let mut tcls = self.tile_class(format!("PCLK{i}_SOURCE"), tslots::PCLK_SOURCE, 2);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            {
                let mut tcls = self.tile_class(format!("PCLK{i}_SOURCE_W"), tslots::PCLK_SOURCE, 2);
                tcls.bel(bslots::PCLK_SOURCE_W);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }

            if i != 2 {
                let mut tcls = self.tile_class(format!("PCLK{i}_SOURCE_E"), tslots::PCLK_SOURCE, 2);
                tcls.bel(bslots::PCLK_SOURCE_E);
                tcls.bel(bslots::PCLK_DCC[0]);
                tcls.bel(bslots::PCLK_DCC[1]);
            }
        }
    }

    fn fill_pclk_tiles(&mut self) {
        match self.kind {
            ChipKind::Scm
            | ChipKind::Ecp
            | ChipKind::Xp
            | ChipKind::MachXo
            | ChipKind::Ecp2
            | ChipKind::Ecp2M
            | ChipKind::Xp2
            | ChipKind::Ecp4
            | ChipKind::Ecp5
            | ChipKind::Crosslink => (),
            ChipKind::Ecp3 | ChipKind::Ecp3A => self.fill_pclk_tiles_ecp3(),
            ChipKind::MachXo2(_) => self.fill_pclk_tiles_machxo2(),
        }
    }

    fn fill_sclk_tiles(&mut self) {
        if !self.kind.has_distributed_sclk() {
            return;
        }

        for i in 0..4 {
            let mut tiles = vec![(format!("SCLK{i}_SOURCE"), vec![(i, 0), (i + 4, 1)])];
            if self.kind.has_distributed_sclk_ecp3() {
                if !(i == 2 && matches!(self.kind, ChipKind::Ecp3 | ChipKind::Ecp3A)) {
                    tiles.push((
                        format!("SCLK{i}_SOURCE_W"),
                        vec![
                            (i, 0),
                            (i + 4, 1),
                            ((i + 3) % 4, 6),
                            ((i + 3) % 4 + 4, 7),
                            ((i + 2) % 4, 4),
                            ((i + 2) % 4 + 4, 5),
                        ],
                    ));
                }
                if !(i == 1 && matches!(self.kind, ChipKind::Ecp3 | ChipKind::Ecp3A)) {
                    tiles.push((
                        format!("SCLK{i}_SOURCE_E"),
                        vec![(i, 0), (i + 4, 1), ((i + 1) % 4, 2), ((i + 1) % 4 + 4, 3)],
                    ));
                }
            }

            for (name, clocks) in tiles {
                let mut sb = SwitchBox::default();
                for (si, vsdi) in clocks {
                    sb.items.push(SwitchBoxItem::PermaBuf(PermaBuf {
                        dst: wires::RSCLK[si].cell(0),
                        src: wires::RSCLK_V[vsdi].cell(0).pos(),
                    }));
                }
                let mut tcls = self.tile_class(name, tslots::SCLK_SOURCE, 1);
                tcls.class
                    .bels
                    .insert(bslots::SCLK_SOURCE, BelInfo::SwitchBox(sb));
            }
        }

        for name in ["HSDCLK_SPLITTER", "HSDCLK_ROOT"] {
            if name == "HSDCLK_SPLITTER" && matches!(self.kind, ChipKind::MachXo2(_)) {
                continue;
            }
            let mut sb = SwitchBox::default();
            for i in 0..2 {
                let wire = wires::RSCLK_H[i + 4];
                for j in 0..4 {
                    sb.items.push(SwitchBoxItem::ProgBuf(ProgBuf {
                        dst: wire.cell(j),
                        src: wire.cell(4 + j).pos(),
                        bit: PolTileBit::DUMMY,
                    }));
                    sb.items.push(SwitchBoxItem::ProgBuf(ProgBuf {
                        dst: wire.cell(4 + j),
                        src: wire.cell(j).pos(),
                        bit: PolTileBit::DUMMY,
                    }));
                }
            }
            let mut tcls = self.tile_class(name, tslots::HSDCLK_SPLITTER, 8);
            tcls.class
                .bels
                .insert(bslots::HSDCLK_SPLITTER, BelInfo::SwitchBox(sb));
            if name == "HSDCLK_ROOT" {
                let mut bel = tcls.bel(bslots::HSDCLK_ROOT);
                for i in 0..8 {
                    bel.add_output(&format!("OUT_W{i}"), i % 4, wires::RSCLK_H[i / 4 * 4]);
                    bel.add_output(&format!("OUT_E{i}"), 4 + i % 4, wires::RSCLK_H[i / 4 * 4]);
                }
            }
        }
    }

    fn fill_clk_tiles_scm(&mut self) {
        for slot in bslots::CLKDIV {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for name in ["CLK_W", "CLK_E", "CLK_N"] {
            let mut tcls = self.tile_class(name, tslots::CLK, 2);
            tcls.bel(bslots::ECLK_ROOT);
            for i in 0..4 {
                tcls.bel(bslots::CLKDIV[i]);
            }
            for i in 0..2 {
                tcls.bel(bslots::DCS[i]);
            }
            tcls.bel(bslots::CLK_EDGE);
        }
        {
            let mut tcls = self.tile_class("CLK_S", tslots::CLK, 2);
            for i in 0..2 {
                tcls.bel(bslots::DCS[i]);
            }
            tcls.bel(bslots::CLK_EDGE);
        }
        for name in ["CLK_SW", "CLK_SE"] {
            let mut tcls = self.tile_class(name, tslots::CLK, 2);
            tcls.bel(bslots::ECLK_ROOT);
            for i in 0..4 {
                tcls.bel(bslots::CLKDIV[i]);
            }
        }
        {
            let mut tcls = self.tile_class("CLK_ROOT", tslots::CLK, 4);
            tcls.bel(bslots::CLK_ROOT);
            tcls.bel(bslots::CLKTEST);
        }
    }

    fn fill_clk_tiles_machxo(&mut self) {
        for name in ["CLK_ROOT_0PLL", "CLK_ROOT_1PLL", "CLK_ROOT_2PLL"] {
            self.tile_class(name, tslots::CLK, 6).bel(bslots::CLK_ROOT);
        }
    }

    fn fill_clk_tiles_ecp(&mut self) {
        let tile_classes = match self.kind {
            ChipKind::Ecp => [("CLK_ROOT_2PLL", 22), ("CLK_ROOT_4PLL", 32)].as_slice(),
            ChipKind::Xp => [
                ("CLK_ROOT_2PLL_A", 28),
                ("CLK_ROOT_2PLL_B", 28),
                ("CLK_ROOT_4PLL", 32),
            ]
            .as_slice(),
            ChipKind::Ecp2 => [
                ("CLK_ROOT_2PLL", 30),
                ("CLK_ROOT_4PLL", 30),
                ("CLK_ROOT_6PLL", 30),
            ]
            .as_slice(),
            ChipKind::Ecp2M => [("CLK_ROOT_8PLL", 30)].as_slice(),
            ChipKind::Xp2 => [("CLK_ROOT_2PLL", 30), ("CLK_ROOT_4PLL", 30)].as_slice(),
            _ => unreachable!(),
        };
        for &(name, num_cells) in tile_classes {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            tcls.bel(bslots::CLK_ROOT);
            for i in 0..2 {
                tcls.bel(bslots::DCS_SW[i]);
                tcls.bel(bslots::DCS_SE[i]);
                tcls.bel(bslots::DCS_NW[i]);
                tcls.bel(bslots::DCS_NE[i]);
            }
        }
    }

    fn fill_clk_tiles_ecp3(&mut self) {
        let mut tcls = self.tile_class("CLK_ROOT", tslots::CLK, 30);
        tcls.bel(bslots::CLK_ROOT);
        for i in 0..6 {
            tcls.bel(bslots::DCC_SW[i]);
            tcls.bel(bslots::DCC_SE[i]);
            tcls.bel(bslots::DCC_NW[i]);
            tcls.bel(bslots::DCC_NE[i]);
        }
        for i in 0..2 {
            tcls.bel(bslots::DCS_SW[i]);
            tcls.bel(bslots::DCS_SE[i]);
            tcls.bel(bslots::DCS_NW[i]);
            tcls.bel(bslots::DCS_NE[i]);
        }
    }

    fn fill_clk_tiles_machxo2(&mut self) {
        for slot in bslots::CLKDIV {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for slot in bslots::DLLDEL {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for slot in bslots::DQS {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for (name, num_cells) in [
            ("CLK_ROOT_0EBR", 8),
            ("CLK_ROOT_1EBR", 8),
            ("CLK_ROOT_2EBR", 13),
            ("CLK_ROOT_3EBR", 14),
        ] {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            for i in 0..8 {
                tcls.bel(bslots::DCC[i]);
            }
            for bel in bslots::DCM {
                tcls.bel(bel);
            }
            if name != "CLK_ROOT_0EBR" {
                for bel in bslots::ECLKBRIDGECS {
                    tcls.bel(bel);
                }
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_ROOT);
        }
        for name in ["CLK_S", "CLK_N"] {
            let mut tcls = self.tile_class(name, tslots::CLK, 1);
            tcls.bel(bslots::ECLKSYNC[0]);
            tcls.bel(bslots::ECLKSYNC[1]);
            tcls.bel(bslots::DLLDEL[0]);
            tcls.bel(bslots::DLLDEL[1]);
            tcls.bel(bslots::CLKDIV[0]);
            tcls.bel(bslots::CLKDIV[1]);
            if name == "CLK_S" {
                tcls.bel(bslots::CLKFBBUF[0]);
                tcls.bel(bslots::CLKFBBUF[1]);
            }
        }
        for name in ["DQSDLL_S", "DQSDLL_N"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 1);
            tcls.bel(bslots::DQSDLL);
            tcls.bel(bslots::DQSDLLTEST);
        }
        {
            let mut tcls = self.tile_class("CLK_W", tslots::CLK, 1);
            tcls.bel(bslots::DLLDEL[0]);
            tcls.bel(bslots::DLLDEL[1]);
            tcls.bel(bslots::DLLDEL[2]);
        }
        {
            let mut tcls = self.tile_class("CLK_E", tslots::CLK, 1);
            tcls.bel(bslots::DLLDEL[0]);
        }
        {
            let mut tcls = self.tile_class("CLK_E_DQS", tslots::CLK, 5);
            tcls.bel(bslots::DLLDEL[0]);
            tcls.bel(bslots::DQS[0]);
            tcls.bel(bslots::DQS[1]);
        }
    }

    fn fill_clk_tiles_ecp4(&mut self) {
        for slot in bslots::CLKDIV {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for slot in bslots::DLLDEL {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for (name, num_cells, bank_range, num_dcc) in [
            ("CLK_N_S", 8, 1..3, 16),
            ("CLK_N_M", 10, 1..3, 16),
            ("CLK_N_L", 22, 0..4, 16),
            ("CLK_W_S", 8, 0..2, 14),
            ("CLK_W_M", 10, 0..2, 14),
            ("CLK_W_L", 12, 0..2, 14),
            ("CLK_E_S", 8, 0..2, 14),
            ("CLK_E_M", 10, 0..2, 14),
            ("CLK_E_L", 12, 0..2, 14),
        ] {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            for i in 0..num_dcc {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
            for i in bank_range.clone() {
                for j in 0..4 {
                    tcls.bel(bslots::ECLKSYNC[i * 4 + j]);
                }
            }
            tcls.bel(bslots::CLKTEST_ECLK);
            for i in bank_range.clone() {
                for j in 0..2 {
                    tcls.bel(bslots::DLLDEL[i * 2 + j]);
                }
            }
            for i in 0..4 {
                tcls.bel(bslots::CLKDIV[i]);
            }
            if !name.starts_with("CLK_N") {
                for i in 0..2 {
                    tcls.bel(bslots::BRGECLKSYNC[i]);
                    tcls.bel(bslots::ECLKBRIDGECS[i]);
                }
            }
        }
        for (name, num_cells, num_quads) in
            [("CLK_S_S", 2, 1), ("CLK_S_M", 4, 2), ("CLK_S_L", 6, 3)]
        {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            for i in 0..16 {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
            for i in 0..num_quads {
                tcls.bel(bslots::PCSCLKDIV[i]);
            }
        }

        {
            let mut tcls = self.tile_class("CLK_ROOT", tslots::CLK, 4);
            tcls.bel(bslots::DCC_SW[0]);
            tcls.bel(bslots::DCC_SE[0]);
            tcls.bel(bslots::DCC_NW[0]);
            tcls.bel(bslots::DCC_NE[0]);
            tcls.bel(bslots::DCS[0]);
            tcls.bel(bslots::DCS[1]);
            tcls.bel(bslots::CLK_ROOT);
            tcls.bel(bslots::CLKTEST);
        }
    }

    fn fill_clk_tiles_ecp5(&mut self) {
        for slot in bslots::CLKDIV {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for slot in bslots::DLLDEL {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for (name, num_cells) in [
            ("CLK_W_S", 12),
            ("CLK_W_L", 14),
            ("CLK_E_S", 12),
            ("CLK_E_L", 14),
        ] {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            for i in 0..14 {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
            for i in 0..2 {
                for j in 0..2 {
                    tcls.bel(bslots::ECLKSYNC[i * 2 + j]);
                }
            }
            tcls.bel(bslots::CLKTEST_ECLK);
            for i in 0..2 {
                for j in 0..2 {
                    tcls.bel(bslots::DLLDEL[i * 2 + j]);
                }
            }
            for i in 0..2 {
                tcls.bel(bslots::CLKDIV[i]);
            }
            tcls.bel(bslots::BRGECLKSYNC[0]);
            tcls.bel(bslots::ECLKBRIDGECS[0]);
        }
        for (name, num_cells) in [("CLK_N_S", 10), ("CLK_N_L", 12)] {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            for i in 0..12 {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
            for i in 0..2 {
                for j in 0..2 {
                    tcls.bel(bslots::DLLDEL[i * 2 + j]);
                }
            }
        }
        for (name, num_cells) in [("CLK_S_S", 8), ("CLK_S_L", 10)] {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            for i in 0..16 {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
            for i in 0..2 {
                tcls.bel(bslots::PCSCLKDIV[i]);
            }
        }

        for (name, num_cells) in [("CLK_ROOT_S", 4), ("CLK_ROOT_L", 8)] {
            let mut tcls = self.tile_class(name, tslots::CLK, num_cells);
            tcls.bel(bslots::DCC_SW[0]);
            tcls.bel(bslots::DCC_SE[0]);
            tcls.bel(bslots::DCC_NW[0]);
            tcls.bel(bslots::DCC_NE[0]);
            tcls.bel(bslots::DCS[0]);
            tcls.bel(bslots::DCS[1]);
            tcls.bel(bslots::CLK_ROOT);
            tcls.bel(bslots::CLKTEST);
        }
    }

    fn fill_clk_tiles_crosslink(&mut self) {
        for slot in bslots::CLKDIV {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        for slot in bslots::DLLDEL {
            self.db.bel_slots[slot].tile_slot = tslots::CLK;
        }
        {
            let mut tcls = self.tile_class("CLK_S", tslots::CLK, 6);
            for i in 0..8 {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
            for i in 0..2 {
                for j in 0..2 {
                    tcls.bel(bslots::ECLKSYNC[i * 2 + j]);
                }
            }
            tcls.bel(bslots::CLKTEST_ECLK);
            for i in 0..2 {
                for j in 0..2 {
                    tcls.bel(bslots::DLLDEL[i * 2 + j]);
                }
            }
            for i in 0..4 {
                tcls.bel(bslots::CLKDIV[i]);
            }
        }
        {
            let mut tcls = self.tile_class("CLK_N", tslots::CLK, 2);
            for i in 0..6 {
                tcls.bel(bslots::DCC[i]);
            }
            tcls.bel(bslots::CLKTEST);
            tcls.bel(bslots::CLK_EDGE);
        }
        {
            let mut tcls = self.tile_class("CLK_ROOT", tslots::CLK, 1);
            tcls.bel(bslots::DCS[0]);
            tcls.bel(bslots::CLK_ROOT);
            tcls.bel(bslots::CLKTEST);
        }
    }

    fn fill_clk_tiles(&mut self) {
        match self.kind {
            ChipKind::Scm => self.fill_clk_tiles_scm(),
            ChipKind::Ecp | ChipKind::Xp | ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                self.fill_clk_tiles_ecp()
            }
            ChipKind::MachXo => self.fill_clk_tiles_machxo(),
            ChipKind::Ecp3 | ChipKind::Ecp3A => self.fill_clk_tiles_ecp3(),
            ChipKind::MachXo2(_) => self.fill_clk_tiles_machxo2(),
            ChipKind::Ecp4 => self.fill_clk_tiles_ecp4(),
            ChipKind::Ecp5 => self.fill_clk_tiles_ecp5(),
            ChipKind::Crosslink => self.fill_clk_tiles_crosslink(),
        }
    }

    fn fill_plc_tiles(&mut self) {
        let kind = self.kind;
        for name in ["PLC", "FPLC"] {
            if name == "FPLC"
                && matches!(
                    self.kind,
                    ChipKind::Scm
                        | ChipKind::MachXo2(_)
                        | ChipKind::Ecp4
                        | ChipKind::Ecp5
                        | ChipKind::Crosslink
                )
            {
                continue;
            }
            let mut tcls = self.tile_class(name, tslots::BEL, 1);
            for i in 0..4 {
                let mut bel = tcls.bel(bslots::SLICE[i]);
                let i0 = 2 * i;
                let i1 = 2 * i + 1;
                for (w, l) in [
                    (wires::IMUX_A, 'A'),
                    (wires::IMUX_B, 'B'),
                    (wires::IMUX_C, 'C'),
                    (wires::IMUX_D, 'D'),
                    (wires::IMUX_M, 'M'),
                ] {
                    bel.add_input(&format!("{l}0"), 0, w[i0]);
                    bel.add_input(&format!("{l}1"), 0, w[i1]);
                }
                if i < 3 || !kind.has_ecp2_plc() {
                    if matches!(kind, ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Crosslink) {
                        bel.add_input("CLK", 0, wires::IMUX_MUXCLK[i]);
                        bel.add_input("LSR", 0, wires::IMUX_MUXLSR[i]);
                    } else {
                        bel.add_input("CLK", 0, wires::IMUX_CLK[i]);
                        bel.add_input("LSR", 0, wires::IMUX_LSR[i]);
                    }
                    bel.add_input("CE", 0, wires::IMUX_CE[i]);
                    bel.add_output("Q0", 0, wires::OUT_Q[i0]);
                    bel.add_output("Q1", 0, wires::OUT_Q[i1]);
                }
                for (w, l) in [(wires::OUT_F, "F"), (wires::OUT_OFX, "OFX")] {
                    if matches!(kind, ChipKind::Ecp5 | ChipKind::Crosslink) && l == "OFX" {
                        continue;
                    }
                    bel.add_output(&format!("{l}0"), 0, w[i0]);
                    bel.add_output(&format!("{l}1"), 0, w[i1]);
                }
                if i == 3 && kind.has_ecp_plc() {
                    bel.add_input("FXB", 0, wires::OUT_OFX_W[3]);
                }
                if i == 3 && matches!(kind, ChipKind::MachXo2(_) | ChipKind::Ecp4) {
                    bel.add_input("FXA", 0, wires::OUT_OFX_W[3]);
                }
                if i == 3 && matches!(kind, ChipKind::Ecp5 | ChipKind::Crosslink) {
                    bel.add_input("FXA", 0, wires::OUT_F_W[3]);
                }
                if i == 2 && matches!(kind, ChipKind::Ecp5 | ChipKind::Crosslink) {
                    bel.add_input("WCK", 0, wires::IMUX_CLK[1]);
                    bel.add_input("WRE", 0, wires::IMUX_LSR[1]);
                }
            }
        }
    }

    fn fill_ebr_tiles(&mut self) {
        let (num_cells, num_bels) = match self.kind {
            ChipKind::Scm | ChipKind::Ecp | ChipKind::Xp => (2, 1),
            ChipKind::MachXo => (4, 1),
            ChipKind::Ecp2
            | ChipKind::Ecp2M
            | ChipKind::Xp2
            | ChipKind::Ecp3
            | ChipKind::Ecp3A
            | ChipKind::MachXo2(_) => (3, 1),
            ChipKind::Ecp4 | ChipKind::Ecp5 | ChipKind::Crosslink => (9, 4),
        };
        let tiles = match self.kind {
            ChipKind::Scm => ["EBR_W", "EBR_E"].as_slice(),
            ChipKind::MachXo2(_) => ["EBR", "EBR_N"].as_slice(),
            _ => ["EBR"].as_slice(),
        };
        let kind = self.kind;
        for &tcname in tiles {
            let mut tcls = self.tile_class(tcname, tslots::BEL, num_cells);
            for i in 0..num_bels {
                tcls.bel(bslots::EBR[i]);
            }
            if kind == ChipKind::Scm {
                tcls.bel(bslots::EBR_INT);
            }
        }
    }

    fn fill_dsp_tiles(&mut self) {
        let (num_cells, num_bels) = match self.kind {
            ChipKind::Scm
            | ChipKind::Xp
            | ChipKind::MachXo
            | ChipKind::MachXo2(_)
            | ChipKind::Crosslink => return,
            ChipKind::Ecp => (8, 1),
            ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => (9, 1),
            ChipKind::Ecp3 | ChipKind::Ecp3A | ChipKind::Ecp4 | ChipKind::Ecp5 => (9, 2),
        };
        let mut tcls = self.tile_class("DSP", tslots::BEL, num_cells);
        for i in 0..num_bels {
            tcls.bel(bslots::DSP[i]);
        }
    }

    fn fill_config_tiles_scm(&mut self) {
        let mut tcls = self.tile_class("CONFIG", tslots::BEL, 13);
        tcls.bel(bslots::JTAG);
        tcls.bel(bslots::RDBK);
        tcls.bel(bslots::GSR);
        tcls.bel(bslots::OSC);
        tcls.bel(bslots::START);
        tcls.bel(bslots::SYSBUS);
        tcls.bel(bslots::SERDES_CENTER);
    }

    fn fill_config_tiles_ecp(&mut self) {
        for (name, num_cells) in [("CONFIG_S", 4), ("CONFIG_L", 5)] {
            let mut tcls = self.tile_class(name, tslots::BEL, num_cells);
            tcls.bel(bslots::START);
            tcls.bel(bslots::OSC);
            tcls.bel(bslots::JTAG);
            // RDBK exists as stub?
            tcls.bel(bslots::GSR);
        }
    }

    fn fill_config_tiles_xp(&mut self) {
        let mut tcls = self.tile_class("CONFIG", tslots::BEL, 1);
        tcls.bel(bslots::START);
        tcls.bel(bslots::JTAG);
        tcls.bel(bslots::GSR);
        // OSC and RDBK exist as stubs?
    }

    fn fill_config_tiles_machxo(&mut self) {
        for name in ["OSC", "OSC_X"] {
            self.tile_class(name, tslots::BEL, 1).bel(bslots::OSC);
        }

        let mut tcls = self.tile_class("CONFIG", tslots::BEL, 5);
        tcls.bel(bslots::GSR);
        tcls.bel(bslots::JTAG);
    }

    fn fill_config_tiles_ecp2(&mut self) {
        let mut tcls = self.tile_class("CONFIG", tslots::BEL, 3);
        tcls.bel(bslots::START);
        tcls.bel(bslots::OSC);
        tcls.bel(bslots::JTAG);
        tcls.bel(bslots::GSR);
        tcls.bel(bslots::SED);
        tcls.bel(bslots::SPIM);
    }

    fn fill_config_tiles_xp2(&mut self) {
        {
            let mut tcls = self.tile_class("CONFIG", tslots::BEL, 1);
            tcls.bel(bslots::JTAG);
            tcls.bel(bslots::SED);
        }

        self.tile_class("OSC", tslots::BEL, 1).bel(bslots::OSC);
    }

    fn fill_config_tiles_ecp3(&mut self) {
        {
            let mut tcls = self.tile_class("CONFIG", tslots::BEL, 14);
            tcls.bel(bslots::START);
            tcls.bel(bslots::JTAG);
            tcls.bel(bslots::OSC);
            tcls.bel(bslots::GSR);
            tcls.bel(bslots::SED);
            tcls.bel(bslots::AMBOOT);
            tcls.bel(bslots::PERREG);
        }

        for (name, num_cells) in [
            ("TEST_SW", 3),
            ("TEST_SE", 3),
            ("TEST_NW", 2),
            ("TEST_NE", 2),
        ] {
            let mut tcls = self.tile_class(name, tslots::BEL, num_cells);
            tcls.bel(bslots::TESTIN);
            tcls.bel(bslots::TESTOUT);
            if name == "TEST_SE" {
                tcls.bel(bslots::DTS);
            }
        }
    }

    fn fill_config_tiles_machxo2(&mut self) {
        for (name, num_cells) in [("CONFIG", 4), ("CONFIG_XO3D", 10)] {
            let mut tcls = self.tile_class(name, tslots::BEL, num_cells);
            tcls.bel(bslots::START);
            tcls.bel(bslots::JTAG);
            tcls.bel(bslots::OSC);
            tcls.bel(bslots::GSR);
            tcls.bel(bslots::SED);
            tcls.bel(bslots::PCNTR);
            tcls.bel(bslots::TSALL);
            tcls.bel(bslots::EFB);
            if name == "CONFIG_XO3D" {
                tcls.bel(bslots::ESB);
            }
        }
    }

    fn fill_config_tiles_ecp4(&mut self) {
        let mut tcls = self.tile_class("CONFIG", tslots::BEL, 11);
        tcls.bel(bslots::START);
        tcls.bel(bslots::JTAG);
        tcls.bel(bslots::OSC);
        tcls.bel(bslots::GSR);
        tcls.bel(bslots::SED);
        tcls.bel(bslots::PCNTR);
        tcls.bel(bslots::EFB);
    }

    fn fill_config_tiles_ecp5(&mut self) {
        let mut tcls = self.tile_class("CONFIG", tslots::BEL, 4);
        tcls.bel(bslots::START);
        tcls.bel(bslots::JTAG);
        tcls.bel(bslots::OSC);
        tcls.bel(bslots::GSR);
        tcls.bel(bslots::SED);
        tcls.bel(bslots::CCLK);
    }

    fn fill_config_tiles_crosslink(&mut self) {
        for name in ["I2C_W", "I2C_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 2);
            tcls.bel(bslots::I2C);
            if name == "I2C_E" {
                tcls.bel(bslots::NVCMTEST);
            }
        }
        self.tile_class("OSC", tslots::BEL, 1).bel(bslots::OSC);
        {
            let mut tcls = self.tile_class("CONFIG", tslots::BEL, 4);
            tcls.bel(bslots::GSR);
            tcls.bel(bslots::CFGTEST);
        }
        {
            let mut tcls = self.tile_class("PMU", tslots::BEL, 1);
            tcls.bel(bslots::PMU);
            tcls.bel(bslots::PMUTEST);
        }
    }

    fn fill_config_tiles(&mut self) {
        match self.kind {
            ChipKind::Scm => self.fill_config_tiles_scm(),
            ChipKind::Ecp => self.fill_config_tiles_ecp(),
            ChipKind::Xp => self.fill_config_tiles_xp(),
            ChipKind::MachXo => self.fill_config_tiles_machxo(),
            ChipKind::Ecp2 | ChipKind::Ecp2M => self.fill_config_tiles_ecp2(),
            ChipKind::Xp2 => self.fill_config_tiles_xp2(),
            ChipKind::Ecp3 | ChipKind::Ecp3A => self.fill_config_tiles_ecp3(),
            ChipKind::MachXo2(_) => self.fill_config_tiles_machxo2(),
            ChipKind::Ecp4 => self.fill_config_tiles_ecp4(),
            ChipKind::Ecp5 => self.fill_config_tiles_ecp5(),
            ChipKind::Crosslink => self.fill_config_tiles_crosslink(),
        }
    }

    fn fill_io_tiles_scm(&mut self) {
        for (name, num_io, num_cells) in [
            ("IO_W4", 4, 2),
            ("IO_W12", 12, 4),
            ("IO_E4", 4, 2),
            ("IO_E12", 12, 4),
            ("IO_S4", 4, 2),
            ("IO_S12", 12, 4),
            ("IO_N4", 4, 2),
            ("IO_N8", 8, 3),
            ("IO_N12", 12, 4),
        ] {
            let mut tcls = self.tile_class(name, tslots::IO, num_cells);
            for i in 0..num_io {
                tcls.bel(bslots::IO[i]);
            }
            for i in 0..num_io / 4 {
                tcls.bel(bslots::PICTEST[i]);
            }
        }
        for name in ["IO_INT_W", "IO_INT_E", "IO_INT_S"] {
            self.tile_class(name, tslots::BEL, 1).bel(bslots::IO_INT);
        }
    }

    fn fill_io_tiles_ecp(&mut self) {
        for name in ["IO_W", "IO_E", "IO_S", "IO_N"] {
            let mut tcls = self.tile_class(name, tslots::IO, 1);
            tcls.bel(bslots::IO[0]);
            tcls.bel(bslots::IO[1]);
        }
        for name in ["DQS_W", "DQS_E", "DQS_S", "DQS_N"] {
            if matches!(self.kind, ChipKind::Ecp2 | ChipKind::Ecp2M) && name == "DQS_N" {
                continue;
            }
            self.tile_class(name, tslots::BEL, 1).bel(bslots::DQS[0]);
        }
        if matches!(self.kind, ChipKind::Ecp | ChipKind::Xp) {
            for name in ["DQSDLL_S", "DQSDLL_N"] {
                self.tile_class(name, tslots::BEL, 1).bel(bslots::DQSDLL);
            }
        }
        if self.kind == ChipKind::Xp2 {
            for name in ["CLK_W", "CLK_E"] {
                let mut tcls =
                    self.tile_class(name, tslots::BEL, if name == "CLK_E" { 2 } else { 1 });
                tcls.bel(bslots::DQSDLL);
                tcls.bel(bslots::CLKDIV[0]);
                if name == "CLK_E" {
                    tcls.bel(bslots::SSPI);
                    tcls.bel(bslots::STF);
                    tcls.bel(bslots::WAKEUP);
                    tcls.bel(bslots::START);
                    tcls.bel(bslots::GSR);
                }
            }
        }
        if matches!(self.kind, ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2) {
            for (name, num_cells) in [
                ("ECLK_ROOT_W", 1),
                ("ECLK_ROOT_E", 1),
                ("ECLK_ROOT_S", 2),
                ("ECLK_ROOT_N", 2),
            ] {
                self.tile_class(name, tslots::CLK, num_cells)
                    .bel(bslots::ECLK_ROOT);
            }
            {
                let mut tcls = self.tile_class("ECLK_TAP", tslots::ECLK_TAP, 1);
                let mut bel = tcls.bel(bslots::ECLK_TAP);
                bel.add_output("ECLK0", 0, wires::OUT_F[6]);
                bel.add_output("ECLK1", 0, wires::OUT_F[7]);
            }
        }
    }

    fn fill_io_tiles_machxo(&mut self) {
        for (name, num) in [
            ("SIO_S_W2", 4),
            ("SIO_S_W4", 4),
            ("SIO_S_E2", 4),
            ("SIO_S_E4", 4),
            ("SIO_S_S4", 4),
            ("SIO_S_S6", 6),
            ("SIO_S_N4", 4),
            ("SIO_S_N6", 6),
            ("SIO_L_W2", 4),
            ("SIO_L_W4", 4),
            ("SIO_L_E2", 4),
            ("SIO_L_E4", 4),
            ("SIO_L_S4", 4),
            ("SIO_L_S6", 6),
            ("SIO_L_N4", 4),
            ("SIO_L_N6", 6),
        ] {
            let mut tcls = self.tile_class(name, tslots::IO, 1);
            for i in 0..num {
                tcls.bel(bslots::IO[i]);
            }
        }
    }

    fn fill_io_tiles_ecp3(&mut self) {
        for name in [
            "IO_W",
            "IO_DQS_W",
            "IO_DQS_DUMMY_W",
            "IO_E",
            "IO_DQS_E",
            "XSIO_E",
            "XSIO_S",
            "SIO_N",
            "SIO_A_N",
            "SIO_DQS_N",
            "XSIO_N",
        ] {
            let mut tcls = self.tile_class(name, tslots::IO, 3);
            for i in 0..4 {
                tcls.bel(bslots::IO[i]);
            }
            if name.contains("DQS") {}
        }
        for name in ["DQS_N", "DQS_W", "DQS_E", "DQS_A_W", "DQS_A_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, if name == "DQS_N" { 6 } else { 3 });
            tcls.bel(bslots::DQS[0]);
            if name.starts_with("DQS_A") {
                tcls.bel(bslots::DQSTEST);
            }
        }
        for name in ["IO_PLL_W", "IO_PLL_E"] {
            let mut tcls = self.tile_class(name, tslots::IO, 2);
            for i in 0..4 {
                tcls.bel(bslots::IO[i]);
            }
        }
        for name in ["ECLK_ROOT_W", "ECLK_ROOT_E", "ECLK_ROOT_N"] {
            let mut tcls = self.tile_class(name, tslots::CLK, 1);
            tcls.bel(bslots::ECLKSYNC[0]);
            tcls.bel(bslots::ECLKSYNC[1]);
        }
        {
            let mut tcls = self.tile_class("ECLK_TAP", tslots::ECLK_TAP, 1);
            let mut bel = tcls.bel(bslots::ECLK_TAP);
            bel.add_output("ECLK0", 0, wires::OUT_F[6]);
            bel.add_output("ECLK1", 0, wires::OUT_F[7]);
        }
    }

    fn fill_io_tiles_machxo2(&mut self) {
        for (name, num_cells) in [
            ("BC", 1),
            ("BC_N", 1),
            ("BCSR_W", 1),
            ("BCSR_E", 2),
            ("BCSR_S", 1),
            ("BCSR_N", 1),
        ] {
            let mut tcls = self.tile_class(name, tslots::BC, num_cells);
            tcls.bel(bslots::BCINRD);
            tcls.bel(bslots::BCPG);
            if name.ends_with("_N") {
                tcls.bel(bslots::BCLVDSO);
            }
            if name.starts_with("BCSR") {
                tcls.bel(bslots::BCSLEWRATE);
            }
        }
        for (name, num) in [
            ("IO_W2", 2),
            ("IO_W4", 4),
            ("IO_W4_I3C", 4),
            ("IO_E2", 2),
            ("IO_E4", 4),
            ("IO_S4", 4),
            ("SIO_S2", 2),
            ("SIO_S4", 4),
            ("IO_N4", 4),
            ("SIO_N2", 2),
            ("SIO_N4", 4),
        ] {
            let mut tcls = self.tile_class(name, tslots::IO, 1);
            for i in 0..num {
                tcls.bel(bslots::IO[i]);
            }
        }
    }

    fn fill_io_tiles_ecp4(&mut self) {
        for name in ["BC_W", "BC_E", "BC_N"] {
            let mut tcls = self.tile_class(name, tslots::BC, 1);
            tcls.bel(bslots::BCINRD);
            tcls.bel(bslots::BCPG);
            tcls.bel(bslots::BCLVDSO);
            tcls.bel(bslots::BCPUSL);
            tcls.bel(bslots::BREFTEST);
        }
        {
            let mut tcls = self.tile_class("PVTTEST", tslots::BEL, 1);
            tcls.bel(bslots::PVTTEST);
            tcls.bel(bslots::PVTCAL);
        }
        for name in ["DDRDLL_S", "DDRDLL_N"] {
            self.tile_class(name, tslots::BEL, 1).bel(bslots::DDRDLL);
        }
        for name in ["DTR_S", "DTR_N"] {
            self.tile_class(name, tslots::BEL, 1).bel(bslots::DTR);
        }
        for name in [
            "IO_W",
            "IO_W_EBR_S",
            "IO_W_EBR_N",
            "IO_W_DSP_S",
            "IO_W_DSP_N",
            "IO_E",
            "IO_E_EBR_S",
            "IO_E_EBR_N",
            "IO_E_DSP_S",
            "IO_E_DSP_N",
            "IO_N",
        ] {
            let mut tcls = self.tile_class(name, tslots::IO, 4);
            for i in 0..4 {
                tcls.bel(bslots::IO[i]);
            }
        }
        for name in [
            "DQS_W",
            "DQS_W_BELOW_DSP_N",
            "DQS_W_BELOW_EBR_N",
            "DQS_W_BELOW_EBR_S",
            "DQS_W_EBR_S",
            "DQS_W_EBR_N",
            "DQS_W_DSP_N",
            "DQS_E",
            "DQS_E_BELOW_DSP_N",
            "DQS_E_BELOW_EBR_N",
            "DQS_E_BELOW_EBR_S",
            "DQS_E_EBR_S",
            "DQS_E_EBR_N",
            "DQS_E_DSP_N",
            "DQS_N",
        ] {
            self.tile_class(name, tslots::BEL, 5).bel(bslots::DQS[0]);
        }
    }

    fn fill_io_tiles_ecp5(&mut self) {
        for (name, is_we) in [
            ("BC0", false),
            ("BC1", false),
            ("BC2", true),
            ("BC3", true),
            ("BC4", false),
            ("BC8", false),
            ("BC6", true),
            ("BC7", true),
        ] {
            let mut tcls = self.tile_class(name, tslots::BC, 1);
            if is_we {
                tcls.bel(bslots::BCINRD);
                tcls.bel(bslots::BCLVDSO);
            }
            tcls.bel(bslots::BREFTEST);
        }
        {
            self.tile_class("DTR", tslots::BEL, 1).bel(bslots::DTR);
        }
        self.tile_class("DDRDLL", tslots::BEL, 1)
            .bel(bslots::DDRDLL);
        for (name, num_io, num_cells) in [
            ("IO_W4", 4, 3),
            ("IO_E4", 4, 3),
            ("IO_N2", 2, 2),
            ("IO_S1", 1, 1),
            ("IO_S2", 2, 2),
        ] {
            let mut tcls = self.tile_class(name, tslots::IO, num_cells);
            for i in 0..num_io {
                tcls.bel(bslots::IO[i]);
            }
        }
        for name in ["DQS_W", "DQS_E"] {
            self.tile_class(name, tslots::BEL, 6).bel(bslots::DQS[0]);
        }
    }

    fn fill_io_tiles_crosslink(&mut self) {
        for slot in [bslots::BCINRD, bslots::BCLVDSO] {
            self.db.bel_slots[slot].tile_slot = tslots::BEL;
        }
        {
            let mut tcls = self.tile_class("BC", tslots::BEL, 1);
            tcls.bel(bslots::BCINRD);
            tcls.bel(bslots::BCLVDSO);
            tcls.bel(bslots::DDRDLL);
        }
        for (name, num_cells) in [("IO_S4", 4), ("IO_S1A", 1), ("IO_S1B", 1)] {
            let mut tcls = self.tile_class(name, tslots::IO, num_cells);
            for i in 0..num_cells {
                tcls.bel(bslots::IO[i]);
            }
        }
    }

    fn fill_io_tiles(&mut self) {
        match self.kind {
            ChipKind::Scm => self.fill_io_tiles_scm(),
            ChipKind::Ecp | ChipKind::Xp | ChipKind::Ecp2 | ChipKind::Ecp2M | ChipKind::Xp2 => {
                self.fill_io_tiles_ecp()
            }
            ChipKind::MachXo => self.fill_io_tiles_machxo(),
            ChipKind::Ecp3 | ChipKind::Ecp3A => self.fill_io_tiles_ecp3(),
            ChipKind::MachXo2(_) => self.fill_io_tiles_machxo2(),
            ChipKind::Ecp4 => self.fill_io_tiles_ecp4(),
            ChipKind::Ecp5 => self.fill_io_tiles_ecp5(),
            ChipKind::Crosslink => self.fill_io_tiles_crosslink(),
        }
    }

    fn fill_serdes_tiles_scm(&mut self) {
        for name in ["SERDES_W", "SERDES_E"] {
            self.tile_class(name, tslots::BEL, 7).bel(bslots::SERDES);
        }
        for name in ["MACO_W", "MACO_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 16);
            tcls.bel(bslots::MACO);
            tcls.bel(bslots::MACO_INT);
        }
    }

    fn fill_serdes_tiles_ecp2(&mut self) {
        for name in ["SERDES_S", "SERDES_N"] {
            self.tile_class(name, tslots::BEL, 27).bel(bslots::SERDES);
        }
    }

    fn fill_serdes_tiles_ecp3(&mut self) {
        self.tile_class("SERDES", tslots::BEL, 36)
            .bel(bslots::SERDES);
    }

    fn fill_serdes_tiles_ecp4(&mut self) {
        for (name, num_cells) in [("SERDES1", 62), ("SERDES2", 120), ("SERDES3", 177)] {
            self.tile_class(name, tslots::BEL, num_cells)
                .bel(bslots::SERDES);
        }
    }

    fn fill_serdes_tiles_ecp5(&mut self) {
        self.tile_class("SERDES", tslots::BEL, 12)
            .bel(bslots::SERDES);
    }

    fn fill_mipi_tiles_crosslink(&mut self) {
        for name in ["MIPI_W", "MIPI_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 24);
            tcls.bel(bslots::MIPI);
            tcls.bel(bslots::CLKTEST_MIPI);
        }
    }

    fn fill_serdes_tiles(&mut self) {
        match self.kind {
            ChipKind::Scm => self.fill_serdes_tiles_scm(),
            ChipKind::Ecp
            | ChipKind::Xp
            | ChipKind::Ecp2
            | ChipKind::Xp2
            | ChipKind::MachXo
            | ChipKind::MachXo2(_) => (),
            ChipKind::Ecp2M => self.fill_serdes_tiles_ecp2(),
            ChipKind::Ecp3 | ChipKind::Ecp3A => self.fill_serdes_tiles_ecp3(),
            ChipKind::Ecp4 => self.fill_serdes_tiles_ecp4(),
            ChipKind::Ecp5 => self.fill_serdes_tiles_ecp5(),
            ChipKind::Crosslink => self.fill_mipi_tiles_crosslink(),
        }
    }

    fn fill_pll_tiles_scm(&mut self) {
        for name in ["PLL_SW", "PLL_SE"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 6);
            for i in 0..2 {
                tcls.bel(bslots::PLL[i]);
            }
            for i in 0..4 {
                tcls.bel(bslots::DLL[i]);
            }
            for i in 0..2 {
                tcls.bel(bslots::DLL_DCNTL[i]);
            }
            tcls.bel(bslots::PLL_SMI);
            tcls.bel(bslots::PROMON);
            if name == "PLL_SW" {
                tcls.bel(bslots::RNET);
            }
        }
        for name in ["PLL_NW", "PLL_NE"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 3);
            for i in 0..2 {
                tcls.bel(bslots::PLL[i]);
            }
            for i in 0..2 {
                tcls.bel(bslots::DLL[i]);
            }
            for i in 0..2 {
                tcls.bel(bslots::DLL_DCNTL[i]);
            }
            tcls.bel(bslots::PLL_SMI);
            tcls.bel(bslots::SERDES_CORNER);
            if name == "PLL_NW" {
                tcls.bel(bslots::M0);
                tcls.bel(bslots::M1);
                tcls.bel(bslots::M2);
                tcls.bel(bslots::M3);
                tcls.bel(bslots::RESETN);
                tcls.bel(bslots::RDCFGN);
            }
            if name == "PLL_NE" {
                tcls.bel(bslots::CCLK);
                tcls.bel(bslots::TCK);
                tcls.bel(bslots::TMS);
                tcls.bel(bslots::TDI);
            }
        }
    }

    fn fill_pll_tiles_ecp(&mut self) {
        for name in ["PLL_W", "PLL_E"] {
            self.tile_class(name, tslots::BEL, 1).bel(bslots::PLL[0]);
        }
    }

    fn fill_pll_tiles_machxo(&mut self) {
        for name in ["PLL_S", "PLL_N"] {
            self.tile_class(name, tslots::BEL, 1).bel(bslots::PLL[0]);
        }
    }

    fn fill_pll_tiles_ecp2(&mut self) {
        for name in ["SPLL_W", "SPLL_E"] {
            self.tile_class(name, tslots::BEL, 2).bel(bslots::SPLL);
        }
        for name in ["PLL_W", "PLL_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 4);
            tcls.bel(bslots::PLL[0]);
            tcls.bel(bslots::DLL[0]);
            tcls.bel(bslots::DLLDEL[0]);
            tcls.bel(bslots::CLKDIV[0]);
            tcls.bel(bslots::ECLK_ALT_ROOT);
            tcls.bel(bslots::DQSDLL);
        }
    }

    fn fill_pll_tiles_xp2(&mut self) {
        for name in ["PLL_S", "PLL_N"] {
            self.tile_class(name, tslots::BEL, 2).bel(bslots::PLL[0]);
        }
    }

    fn fill_pll_tiles_ecp3(&mut self) {
        for name in ["PLL_DLL_W", "PLL_DLL_E", "PLL_DLL_A_W", "PLL_DLL_A_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 18);
            tcls.bel(bslots::PLL[0]);
            tcls.bel(bslots::DLL[0]);
            tcls.bel(bslots::DLLDEL[0]);
            tcls.bel(bslots::DQSDLL);
            tcls.bel(bslots::DQSDLLTEST);
            tcls.bel(bslots::ECLK_ALT_ROOT);
            tcls.bel(bslots::CLKDIV[0]);
        }
        for name in ["PLL_W", "PLL_E", "PLL_A_W", "PLL_A_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 13);
            tcls.bel(bslots::PLL[0]);
        }
    }

    fn fill_pll_tiles_machxo2(&mut self) {
        for name in ["PLL_W", "PLL_E"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 2);
            tcls.bel(bslots::PLL[0]);
            tcls.bel(bslots::PLLREFCS[0]);
        }
    }

    fn fill_pll_tiles_ecp4(&mut self) {
        for name in ["PLL_SW", "PLL_SE", "PLL_NW", "PLL_NE"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 4);
            for i in 0..2 {
                tcls.bel(bslots::PLL[i]);
                tcls.bel(bslots::PLLREFCS[i]);
            }
        }
    }

    fn fill_pll_tiles_ecp5(&mut self) {
        for name in ["PLL_SW", "PLL_SE", "PLL_NW", "PLL_NE"] {
            let mut tcls = self.tile_class(name, tslots::BEL, 2);
            tcls.bel(bslots::PLL[0]);
            tcls.bel(bslots::PLLREFCS[0]);
        }
    }

    fn fill_pll_tiles_crosslink(&mut self) {
        let mut tcls = self.tile_class("PLL", tslots::BEL, 2);
        tcls.bel(bslots::PLL[0]);
        tcls.bel(bslots::PLLREFCS[0]);
    }

    fn fill_pll_tiles(&mut self) {
        match self.kind {
            ChipKind::Scm => self.fill_pll_tiles_scm(),
            ChipKind::Ecp | ChipKind::Xp => self.fill_pll_tiles_ecp(),
            ChipKind::MachXo => self.fill_pll_tiles_machxo(),
            ChipKind::Ecp2 | ChipKind::Ecp2M => self.fill_pll_tiles_ecp2(),
            ChipKind::Xp2 => self.fill_pll_tiles_xp2(),
            ChipKind::Ecp3 | ChipKind::Ecp3A => self.fill_pll_tiles_ecp3(),
            ChipKind::MachXo2(_) => self.fill_pll_tiles_machxo2(),
            ChipKind::Ecp4 => self.fill_pll_tiles_ecp4(),
            ChipKind::Ecp5 => self.fill_pll_tiles_ecp5(),
            ChipKind::Crosslink => self.fill_pll_tiles_crosslink(),
        }
    }

    fn build(mut self) -> IntDb {
        self.fill_int_tiles();
        self.fill_pclk_tiles();
        self.fill_sclk_tiles();
        self.fill_clk_tiles();
        self.fill_plc_tiles();
        self.fill_ebr_tiles();
        self.fill_dsp_tiles();
        self.fill_config_tiles();
        self.fill_io_tiles();
        self.fill_serdes_tiles();
        self.fill_pll_tiles();
        self.db
    }
}

pub fn init_intdb(kind: ChipKind) -> IntDb {
    let db: IntDb = bincode::decode_from_slice(
        match kind {
            ChipKind::Scm => prjcombine_ecp::defs::scm::INIT,
            ChipKind::Ecp => prjcombine_ecp::defs::ecp::INIT,
            ChipKind::Xp => prjcombine_ecp::defs::xp::INIT,
            ChipKind::MachXo => prjcombine_ecp::defs::machxo::INIT,
            ChipKind::Ecp2 | ChipKind::Ecp2M => prjcombine_ecp::defs::ecp2::INIT,
            ChipKind::Xp2 => prjcombine_ecp::defs::xp2::INIT,
            ChipKind::Ecp3 | ChipKind::Ecp3A => prjcombine_ecp::defs::ecp3::INIT,
            ChipKind::MachXo2(_) => prjcombine_ecp::defs::machxo2::INIT,
            ChipKind::Ecp4 => prjcombine_ecp::defs::ecp4::INIT,
            ChipKind::Ecp5 => prjcombine_ecp::defs::ecp5::INIT,
            ChipKind::Crosslink => prjcombine_ecp::defs::crosslink::INIT,
        },
        bincode::config::standard(),
    )
    .unwrap()
    .0;

    let builder = IntDbBuilder { kind, db };

    builder.build()
}
