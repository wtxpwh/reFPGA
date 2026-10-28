use crate::{DocgenContext, interconnect::gen_intdb};

pub fn gen_ecp(ctx: &mut DocgenContext) {
    for kind in [
        "scm",
        "ecp",
        "xp",
        "machxo",
        "ecp2",
        "ecp2m",
        "xp2",
        "ecp3",
        "machxo2",
        "ecp4",
        "ecp5",
        "crosslink",
        "nexus",
    ] {
        let db = prjcombine_ecp::db::Database::from_file(
            ctx.ctx.root.join(format!("../databases/{kind}.zstd")),
        )
        .unwrap();
        gen_intdb(ctx, kind, &db.int);
    }
}
