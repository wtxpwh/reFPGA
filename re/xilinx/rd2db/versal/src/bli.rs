use std::collections::HashMap;

use prjcombine_re_xilinx_rawdump::{Coord, Part, WireId};

pub type BliConns = HashMap<(Coord, WireId), (Coord, WireId)>;

pub fn get_bli(rd: &Part) -> BliConns {
    let mut res = HashMap::new();
    for (&crd, tile) in &rd.tiles {
        let tk = &rd.tile_kinds[tile.kind];
        let tkn = rd.tile_kinds.key(tile.kind);
        if !tkn.starts_with("BLI_LS_CORE") && !tkn.starts_with("BLI_LS_MUX_CORE") {
            continue;
        }
        for (wi, ni) in tk
            .conn_wires
            .iter()
            .map(|(i, &wi)| (wi, tile.conn_wires.get(i).copied()))
        {
            let Some(ni) = ni else {
                continue;
            };
            for (tc, w) in rd.iter_node(ni) {
                let otile = &rd.tiles[&tc];
                let otkn = rd.tile_kinds.key(otile.kind);
                if otkn.starts_with("BLI_")
                    || otkn.starts_with("CLE_BC_")
                    || otkn.starts_with("INTF_")
                {
                    continue;
                }
                res.insert((tc, w), (crd, wi));
            }
        }
    }
    res
}
