use crate::heap::class::Class;

na!(
    ThreadReg,
    "java/lang/Thread",
    "registerNatives",
    "()V",
    th,
    f,
    {
        reg!(th.registry, N0);
    }
);

na!(
    N0,
    "java/lang/Thread",
    "currentThread",
    "()Ljava/lang/Thread;",
    th,
    f,
    {
        let c = th.heap.loader.load("java/lang/Thread");
        let mut o = Class::new_obj(c);
        let tgc = th.heap.loader.load("java/lang/ThreadGroup");
        let to = Class::new_obj(tgc);

        o.set_field_ref("group", to);
        o.set_field("priority", 1);

        f.stack.push_obj(o)
    }
);
