use prjcombine_ecp::defs::{self, bslots, tcls, wires};
use prjcombine_entity::EntityId;
use prjcombine_interconnect::db::{CellSlotId, WireSlotIdExt};

use crate::{ChipContext, sb::EdgeKind};

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
}
