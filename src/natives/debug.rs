use crate::runtime::frame::Slots;

na!(DebugReg, "test/Debug", "registerNatives", "()V", th, f, {
    reg!(th.registry, N1, N2, N3, N4, N5, N6, N7, N8);
});

macro_rules! db {
    ($id: ident, $desc: expr, $f: ident, $v: expr) => {
        na!($id, "test/Debug", "print", $desc, th, $f, {
            let s = $v;
            print!("{}", s);
        });
    };
}

db!(N1, "(Ljava/lang/String;)V", f, f.this().jstring());
db!(N2, "(J)V", f, f.local_vars().get_i64(0));
db!(N3, "(F)V", f, f.local_vars().get_f32(0));
db!(N4, "(D)V", f, f.local_vars().get_f64(0));
db!(N5, "(Z)V", f, f.local_vars().get_u32(0) != 0);
db!(N6, "(B)V", f, f.local_vars().get_u32(0) as i8);
db!(N7, "(S)V", f, f.local_vars().get_u32(0) as i16);
db!(N8, "(I)V", f, f.local_vars().get_i32(0));
