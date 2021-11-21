na!(
    ClassReg,
    "java/lang/Class",
    "registerNatives",
    "()V",
    th,
    f,
    { reg!(th.registry, N0, N1, N2, N3) }
);

na!(
    N0,
    "java/lang/Class",
    "getPrimitiveClass",
    "(Ljava/lang/String;)Ljava/lang/Class;",
    th,
    f,
    {
        let s = f.this().as_utf8();
        let mut cl = f.heap.loader.load(&s);

        if cl.clinit(th) {
            th.revert_pc();
            return;
        }

        f.stack.push_obj(cl.j_class)
    }
);

na!(
    N1,
    "java/lang/Class",
    "desiredAssertionStatus0",
    "(Ljava/lang/Class;)Z",
    th,
    f,
    { f.stack.push_u32(0) }
);

na!(
    N2,
    "java/lang/Class",
    "getName0",
    "()Ljava/lang/String;",
    th,
    f,
    {
        let cl = f.this().class;
        let js = f.heap.new_jstr(&cl.name);
        f.stack.push_obj(js)
    }
);

na!(
    N3,
    "java/lang/Class",
    "forName0",
    "(Ljava/lang/String;ZLjava/lang/ClassLoader;Ljava/lang/Class;)Ljava/lang/Class;",
    th,
    f,
    {
        use crate::heap::class::Object;
        use crate::rp::Rp;
        let js: Rp<Object> = (f.local_vars[0] as usize).into();
        let n = js.as_utf8();
        let cl = f.heap.loader.load(&n);
        f.stack.push_obj(cl.j_class)
    }
);
