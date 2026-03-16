use crate::{DocgenContext, interconnect::gen_intdb};

pub fn gen_ultrascale(ctx: &mut DocgenContext) {
    for kind in ["ultrascale", "ultrascaleplus"] {
        let db = prjcombine_ultrascale::db::Database::from_file(
            ctx.ctx.root.join(format!("../databases/{kind}.zstd")),
        )
        .unwrap();
        gen_intdb(ctx, kind, &db.int);
    }
}
